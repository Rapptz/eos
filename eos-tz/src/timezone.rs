use std::io::{Read, Seek};

use crate::{
    error::ParseError,
    posix::PosixTimeZone,
    reader::parse_tzif,
    timestamp::NaiveTimestamp,
    transitions::{Transition, TransitionType},
};

/// Represents an IANA database backed timezone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeZone {
    id: String,
    transitions: Vec<Transition>,
    ttypes: Vec<TransitionType>,
    posix: Option<PosixTimeZone>,
}

impl TimeZone {
    /// Loads a [`TimeZone`] from a reader that points to a TZif file and the
    /// given Zone identifier.
    ///
    /// When reading from a source against which short reads are not
    /// efficient, such as a [`std::fs::File`], you will want to apply your own buffering
    /// such as [`std::io::BufReader`] since the library will not buffer reads.
    ///
    /// When using an in-memory stream such as raw bytes, you will need to
    /// wrap the type into an [`std::io::Cursor`] to allow it to be seekable.
    ///
    /// If a parser error happens then [`ParseError`] is returned.
    ///
    /// Note that the time zone identifier *must* be valid, for example `America/New_York`.
    pub fn load<R: Read + Seek>(reader: R, id: String) -> Result<Self, ParseError> {
        let (transitions, ttypes, posix) = parse_tzif(reader)?;
        Ok(Self {
            id,
            transitions,
            ttypes,
            posix,
        })
    }

    /// Loads a [`TimeZone`] from the internal bundled copy of the TZif files.
    ///
    /// Unlike the [`zone`] macro, this allows querying with a runtime string.
    #[cfg(feature = "bundled")]
    pub fn bundled(zone: &str) -> Result<Self, ParseError> {
        match eos_tzdata::locate_tzif(zone) {
            Some(bytes) => Self::load(std::io::Cursor::new(bytes), zone.to_owned()),
            None => Err(ParseError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "could not locate time zone",
            ))),
        }
    }

    // todo: TimeZone::locate(...)
    // fetches from OS store, Linux only.

    /// Returns the identifier name.
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub(crate) fn get_transition(&self, ts: NaiveTimestamp, utc: bool) -> Option<&Transition> {
        let key: fn(&Transition) -> NaiveTimestamp = if utc {
            |trans| trans.utc_start
        } else {
            |trans| trans.start
        };
        let idx = match self.transitions.binary_search_by_key(&ts, key) {
            Ok(idx) => idx,
            Err(idx) => {
                if idx != self.transitions.len() {
                    // The first entry in the transition list is always extended until
                    // the beginning of time. The remaining ones start based off of the previous
                    // end value. This offset by 1 means that we take the one that we actually care about.
                    idx - 1
                } else {
                    // If we're reaching the end of time transition then fall back to the POSIX
                    // timezone
                    return None;
                }
            }
        };
        self.transitions.get(idx)
    }
}

/// A macro to return a [`TimeZone`] for the given zone identifier.
///
/// This requires that the `bundled` feature is enabled, since that's
/// where it gets the backing data from.
///
/// # Examples
///
/// ```no_run
/// use eos_tz::zone;
///
/// let tz = zone!("America/New_York");
/// ```
///
/// # Panics
///
/// Panics if the backing TZif data could not be parsed. This should be unlikely or impossible
/// and denotes a bug with the library.
#[macro_export]
#[cfg(feature = "bundled")]
macro_rules! zone {
    ($zone_id:literal) => {{
        const DATA: &'static [u8] = eos_tzdata::tzif!($zone_id);
        $crate::TimeZone::load(std::io::Cursor::new(DATA), std::string::String::from($zone_id)).unwrap()
    }};
}

#[cfg(feature = "bundled")]
pub use zone;

impl eos::TimeZone for TimeZone {
    fn name(&self, date: &eos::Date, time: &eos::Time) -> Option<&str> {
        let ts = NaiveTimestamp::new(date, time);
        match self.get_transition(ts, false) {
            None => match &self.posix {
                // See below
                None => None,
                Some(posix) => posix.name(date, time),
            },
            Some(trans) => self.ttypes.get(trans.name_idx).map(|ttype| ttype.abbr.as_str()),
        }
    }

    fn offset(&self, date: &eos::Date, time: &eos::Time) -> eos::UtcOffset {
        let ts = NaiveTimestamp::new(date, time);
        match self.get_transition(ts, false) {
            None => match &self.posix {
                // According to RFC 8536 having no transition *and* no POSIX
                // string at the end means the time is unspecified. Since this
                // is undefined behaviour territory, just return a plausible value,
                // which in this case is the *last* transition's offset value.
                None => self.transitions.last().map(|t| t.offset).unwrap_or_default(),
                Some(posix) => posix.offset(date, time),
            },
            Some(trans) => trans.offset,
        }
    }

    fn convert_utc(self, mut utc: eos::DateTime<eos::Utc>) -> eos::DateTime<Self>
    where
        Self: Sized,
    {
        let ts = NaiveTimestamp::new(utc.date(), utc.time());
        match self.get_transition(ts, true) {
            None => match &self.posix {
                None => utc.with_timezone(self),
                Some(posix) => {
                    posix.shift_utc(&mut utc);
                    utc.with_timezone(self)
                }
            },
            Some(trans) => {
                utc.shift(trans.offset);
                utc.with_timezone(self)
            }
        }
    }

    fn resolve(self, date: eos::Date, time: eos::Time) -> eos::DateTimeResolution<Self>
    where
        Self: Sized,
    {
        let ts = NaiveTimestamp::new(&date, &time);
        // Manually check transitions since we need to get the surrounding ones
        let (prev, trans, next) = match self.transitions.binary_search_by_key(&ts, |t| t.start) {
            Ok(idx) => (
                self.transitions.get(idx - 1),
                &self.transitions[idx],
                self.transitions.get(idx + 1),
            ),
            Err(idx) if idx != self.transitions.len() => (
                self.transitions.get(idx - 2),
                &self.transitions[idx - 1],
                Some(&self.transitions[idx]),
            ),
            Err(idx) => {
                // If this transition is in the future then we fall back to the POSIX timezone
                match &self.posix {
                    Some(posix) => {
                        let (kind, earlier, later) = posix.partial_resolution(&date, &time);
                        return match kind {
                            eos::DateTimeResolutionKind::Missing => {
                                eos::DateTimeResolution::missing(date, time, earlier, later, self)
                            }
                            eos::DateTimeResolutionKind::Unambiguous => {
                                eos::DateTimeResolution::unambiguous(date, time, earlier, self)
                            }
                            eos::DateTimeResolutionKind::Ambiguous => {
                                eos::DateTimeResolution::ambiguous(date, time, earlier, later, self)
                            }
                        };
                    }
                    None => {
                        // This is unspecified (as said above)
                        // Return a garbage unambiguous time
                        let offset = self.transitions.get(idx - 1).map(|t| t.offset).unwrap_or_default();
                        return eos::DateTimeResolution::unambiguous(date, time, offset, self);
                    }
                };
            }
        };

        // Europe/Prague had an interesting transition period with a negative DST offset.
        // It can also serve as an example of a positive DST transition:
        // DST start: 1946-05-06 2AM UTC+1 -> +1 hour (CET -> CEST)
        // DST end: 1946-10-06 3AM UTC+2 -> -1 hour (CEST -> CET)
        // DST start: 1946-12-01 3AM UTC+1 -> -1 hour (CET -> GMT)
        // DST end: 1947-02-23 2AM UTC+0 -> +1 hour (GMT -> CET)

        // At 1946-12-01 2:30 AM the time is ambiguous because it can either be
        // before the transition (UTC+1) or after (UTC).
        // In this code, `trans` is UTC+1 and `next` is UTC.
        // At 1946-10-06 2:30 AM the time is ambiguous in the other direction
        // since it can be before transition (UTC+2) or after (UTC+1).
        // In that scenario, `trans` is UTC+2 and `next` is UTC+1
        // Notice how in both of these cases, the transition before and after match.
        // To check for ambiguity, we need to check whether the *next* transition has an ambiguity.

        // Next is missing times. In the negative transition offset case there's
        // 1947-02-23 2:30 AM where time goes from UTC -> UTC+1
        // In this code `trans` is UTC+1 and `prev` is UTC
        // There's also 1946-05-06 2:30 AM where it goes from UTC+1 -> UTC+2
        // In that case `trans` is UTC+2 and `prev` is UTC+1
        // To check for missing, we need to check whether the *current* transition is missing.

        if let Some(next) = next {
            if next.is_ambiguous(ts) {
                return eos::DateTimeResolution::ambiguous(date, time, trans.offset, next.offset, self);
            }
        }
        if trans.is_missing(ts) {
            if let Some(prev) = prev {
                return eos::DateTimeResolution::missing(date, time, prev.offset, trans.offset, self);
            }
        }

        // Assume remaining cases are unambiguous
        // Hopefully this holds.
        eos::DateTimeResolution::unambiguous(date, time, trans.offset, self)
    }
}

#[cfg(test)]
mod tests {
    use eos::datetime;

    use super::*;

    #[test]
    #[cfg(feature = "bundled")]
    fn test_bundled_loading() {
        use eos::TimeZone;

        let dt = datetime!(1911-12-30 00:00);
        let tz = zone!("Africa/Abidjan");
        let name = tz.name(dt.date(), dt.time());
        assert_eq!(name, Some("LMT"));

        let tz = zone!("America/Santiago");
        let dt = datetime!(2040-04-06 00:00);
        assert_eq!(tz.name(dt.date(), dt.time()), Some("-03"));
    }
}
