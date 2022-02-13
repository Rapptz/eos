use std::{
    io::{Read, Seek},
    sync::Arc,
};

use crate::{
    error::{Error, ParseError},
    posix::PosixTimeZone,
    reader::parse_tzif,
    timestamp::NaiveTimestamp,
    transitions::{Transition, TransitionType},
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct TimeZoneData {
    id: String,
    transitions: Vec<Transition>,
    ttypes: Vec<TransitionType>,
    posix: Option<PosixTimeZone>,
    fixed: bool,
}

/// An IANA database backed timezone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeZone(Arc<TimeZoneData>);

#[cfg(target_family = "unix")]
const TZ_SEARCH_PATHS: [&'static str; 4] = [
    "/usr/share/zoneinfo",
    "/usr/lib/zoneinfo",
    "/usr/share/lib/zoneinfo",
    "/etc/zoneinfo",
];

#[cfg(target_family = "unix")]
#[inline]
fn is_valid_path<P: AsRef<std::path::Path>>(path: P) -> bool {
    // Components does its own micro form of normalisation,
    // but for our use-case it's mostly ok.
    path.as_ref()
        .components()
        .all(|x| matches!(x, std::path::Component::Normal(_)))
}

// Make this an internal macro to always inline the function call
#[cfg(all(not(feature = "bundled"), target_family = "windows"))]
macro_rules! __get_impl {
    ($zone:ident) => {
        panic!("windows does not have a data source to retrieve from, consider using the `bundled` feature")
    };
}

#[cfg(all(not(feature = "bundled"), target_family = "unix"))]
macro_rules! __get_impl {
    ($zone:ident) => {
        TimeZone::locate($zone)
    };
}

#[cfg(feature = "bundled")]
macro_rules! __get_impl {
    ($zone:ident) => {
        TimeZone::bundled($zone)
    };
}

impl TimeZone {
    /// Loads a `TimeZone` from a reader that points to a TZif file and the
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
        // A fixed transition is one that has no transition information at all.
        // There are a few assumptions here:
        // 1. No more than 1 transition in the list.
        //    The first element of the transition list is typically a non-DST transition and can be ignored.
        // 2. The POSIX timezone embedded is also fixed, i.e. no DST information.
        //    It may be possible to construct a POSIX timezone with DST information
        //    but no actual transitions (something silly over a 365 day range...).
        //    I'm not sure if these exist in practice.
        // Nevertheless, this should be mostly fine.
        // If a type doesn't have a POSIX transition but has 1 transition in the file then it's still fixed.
        let fixed = transitions.len() <= 1 && posix.as_ref().map(eos::TimeZone::is_fixed).unwrap_or(true);
        let data = TimeZoneData {
            id,
            transitions,
            ttypes,
            posix,
            fixed,
        };
        Ok(Self(Arc::new(data)))
    }

    /// Loads a `TimeZone` from the internal bundled copy of the TZif files.
    ///
    /// Unlike the [`zone`] macro, this allows querying with a runtime string.
    #[cfg(feature = "bundled")]
    pub fn bundled(zone: &str) -> Result<Self, Error> {
        match eos_tzdata::locate_tzif(zone) {
            Some(bytes) => Ok(Self::load(std::io::Cursor::new(bytes), zone.to_owned())?),
            None => Err(Error::NotFound),
        }
    }

    /// Loads a `TimeZone` from the system provided timezone database.
    ///
    /// This is only available on non-Windows systems. If you want to load a
    /// file that's either bundled or from the system, consider using
    /// [`TimeZone::get`] for cross-platform code.
    ///
    /// If the timezone could not be located, [`Error`] is returned.
    ///
    /// # OS-specific behavior
    ///
    /// This searches through the following paths in order until it finds a match:
    ///
    /// - `/usr/share/zoneinfo`
    /// - `/usr/lib/zoneinfo`
    /// - `/usr/share/lib/zoneinfo`
    /// - `/etc/zoneinfo`
    ///
    /// This should have a wide range of compatibility with most operating systems
    /// and distributions.
    #[cfg(target_family = "unix")]
    pub fn locate(zone: &str) -> Result<Self, Error> {
        if !is_valid_path(zone) {
            return Err(Error::InvalidZonePath);
        }

        for p in TZ_SEARCH_PATHS {
            let mut path = std::path::PathBuf::from(p);
            path.push(zone);
            match std::fs::File::open(path) {
                Ok(file) => {
                    let buf = std::io::BufReader::new(file);
                    return Ok(Self::load(buf, zone.to_owned())?);
                }
                Err(_) => continue,
            }
        }

        Err(Error::NotFound)
    }

    /// Load a `TimeZone` from either the bundled data source or
    /// the system provided timezone database. The bundled data source
    /// takes priority over the system provided timezone.
    ///
    /// This allows you to use the bundled data source on Windows
    /// while using the system provided time on Linux using a single
    /// constructor for ease of cross-platform use.
    ///
    /// If the timezone could not be located, [`Error`] is returned.
    ///
    /// # See also
    ///
    /// Check the [`TimeZone::locate`] documentation for search paths.
    pub fn get(zone: &str) -> Result<Self, Error> {
        __get_impl!(zone)
    }

    /// Returns the identifier name.
    pub fn id(&self) -> &str {
        self.0.id.as_str()
    }

    pub(crate) fn get_transition(&self, ts: NaiveTimestamp) -> Option<&Transition> {
        let idx = match self.0.transitions.binary_search_by_key(&ts, |trans| trans.utc_start) {
            Ok(idx) => idx,
            Err(idx) => {
                if idx != self.0.transitions.len() {
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
        self.0.transitions.get(idx)
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
    fn name(&self, ts: eos::Timestamp) -> Option<&str> {
        match self.get_transition(ts.into()) {
            None => match &self.0.posix {
                // See below
                None => None,
                Some(posix) => posix.name(ts),
            },
            Some(trans) => self.0.ttypes.get(trans.name_idx).map(|ttype| ttype.abbr.as_str()),
        }
    }

    fn offset(&self, ts: eos::Timestamp) -> eos::UtcOffset {
        match self.get_transition(ts.into()) {
            None => match &self.0.posix {
                // According to RFC 8536 having no transition *and* no POSIX
                // string at the end means the time is unspecified. Since this
                // is undefined behaviour territory, just return a plausible value,
                // which in this case is the *last* transition's offset value.
                None => self.0.transitions.last().map(|t| t.offset).unwrap_or_default(),
                Some(posix) => posix.offset(ts),
            },
            Some(trans) => trans.offset,
        }
    }

    fn convert_utc(self, mut utc: eos::DateTime<eos::Utc>) -> eos::DateTime<Self>
    where
        Self: Sized,
    {
        let ts = utc.timestamp();
        match self.get_transition(ts.into()) {
            None => match &self.0.posix {
                None => utc.with_timezone(self.clone()),
                Some(posix) => {
                    posix.shift_utc(&mut utc);
                    utc.with_timezone(self.clone())
                }
            },
            Some(trans) => {
                utc.shift(trans.offset);
                utc.with_timezone(self.clone())
            }
        }
    }

    fn resolve(self, date: eos::Date, time: eos::Time) -> eos::DateTimeResolution<Self>
    where
        Self: Sized,
    {
        let ts = NaiveTimestamp::new(&date, &time);
        // Manually check transitions since we need to get the surrounding ones
        let (prev, trans, next) = match self.0.transitions.binary_search_by_key(&ts, |t| t.start) {
            Ok(idx) => (
                self.0.transitions.get(idx.wrapping_sub(1)),
                &self.0.transitions[idx],
                self.0.transitions.get(idx + 1),
            ),
            Err(idx) if idx != self.0.transitions.len() => (
                self.0.transitions.get(idx.wrapping_sub(2)),
                &self.0.transitions[idx - 1],
                Some(&self.0.transitions[idx]),
            ),
            Err(idx) => {
                // There's a specific case where we're past the last transition and into the TZStr
                // Yet simultaneously be within a gap because we're not actually done with the prior
                // transition. So we need to check for that case, for example in Africa/Abidjan there are
                // only two transitions:
                // [start-of-time, 1912-01-01 00:00) UTC -00:16:08
                // [1912-01-01 00:00, end-of-time) UTC +00:00:00
                // Since 1912-01-01 00:01 is a gap period lingering from the +00:16:08 that needs to
                // be accounted for, there needs to be a check for that here
                if !self.0.transitions.is_empty() {
                    let trans = &self.0.transitions[idx - 1];
                    if trans.is_missing(ts) {
                        let earlier = self.0.transitions[idx - 2].offset;
                        return eos::DateTimeResolution::missing(date, time, earlier, trans.offset, self.clone());
                    }
                }
                // If this transition is in the future then we fall back to the POSIX timezone
                match &self.0.posix {
                    Some(posix) => {
                        let (kind, earlier, later) = posix.partial_resolution(&date, &time);
                        return match kind {
                            eos::DateTimeResolutionKind::Missing => {
                                eos::DateTimeResolution::missing(date, time, earlier, later, self.clone())
                            }
                            eos::DateTimeResolutionKind::Unambiguous => {
                                eos::DateTimeResolution::unambiguous(date, time, earlier, self.clone())
                            }
                            eos::DateTimeResolutionKind::Ambiguous => {
                                eos::DateTimeResolution::ambiguous(date, time, earlier, later, self.clone())
                            }
                        };
                    }
                    None => {
                        // This is unspecified (as said above)
                        // Return a garbage unambiguous time
                        let offset = self.0.transitions.get(idx - 1).map(|t| t.offset).unwrap_or_default();
                        return eos::DateTimeResolution::unambiguous(date, time, offset, self.clone());
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
                return eos::DateTimeResolution::ambiguous(date, time, trans.offset, next.offset, self.clone());
            }
        }
        if trans.is_missing(ts) {
            if let Some(prev) = prev {
                return eos::DateTimeResolution::missing(date, time, prev.offset, trans.offset, self.clone());
            }
        }

        // Assume remaining cases are unambiguous
        // Hopefully this holds.
        eos::DateTimeResolution::unambiguous(date, time, trans.offset, self.clone())
    }

    fn is_fixed(&self) -> bool {
        self.0.fixed
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
        let name = tz.name(dt.timestamp());
        assert_eq!(name, Some("LMT"));

        let tz = zone!("America/Santiago");
        let dt = datetime!(2040-04-06 00:00);
        assert_eq!(tz.name(dt.timestamp()), Some("-03"));
    }
}
