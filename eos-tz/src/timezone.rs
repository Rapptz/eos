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
            |trans| trans.start
        } else {
            |trans| trans.start_at_local()
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
                    idx
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
    fn name(&self, date: &eos::Date, time: &eos::Time) -> Option<String> {
        let ts = NaiveTimestamp::new(date, time);
        match self.get_transition(ts, false) {
            None => match &self.posix {
                // See below
                None => None,
                Some(posix) => posix.name(date, time),
            },
            Some(trans) => self.ttypes.get(trans.name_idx).map(|ttype| ttype.abbr.clone()),
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

    fn at(self, mut utc: eos::DateTime<eos::Utc>) -> eos::DateTime<Self>
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
}

#[cfg(test)]
mod tests {
    use eos::datetime;

    use super::*;

    #[test]
    #[cfg(feature = "bundled")]
    fn test_name() {
        let dt = eos::Local::now().unwrap();
        println!("{:?}", &dt);
        let utc = datetime!(2007-3-11 8:00);
        let tz = zone!("America/New_York");
        println!("{:?}", utc.at(tz));
    }
}
