use std::io::{Read, Seek};

use crate::{
    error::ParseError,
    reader::parse_tzif,
    timestamp::NaiveTimestamp,
    transitions::{Transition, Transitions},
};

/// Represents an IANA database backed timezone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeZone {
    id: String,
    transitions: Transitions,
}

/// This is essentially (current, previous_transition)
/// The previous transition could not be found!
pub(crate) type TransitionPair<'a> = (&'a Transition, Option<&'a Transition>);

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
    /// If a parser error happens then [`ParserError`] is returned.
    ///
    /// Note that the time zone identifier *must* be valid, for example `America/New_York`.
    pub fn load<R: Read + Seek>(reader: R, id: String) -> Result<Self, ParseError> {
        let transitions = parse_tzif(reader)?;
        Ok(Self { id, transitions })
    }

    // todo: TimeZone::locate(...)
    // fetches from OS store, Linux only.

    /// Returns the identifier name.
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub(crate) fn lookup_transition<'a>(&'a self, ts: &NaiveTimestamp, utc: bool) -> Option<TransitionPair<'a>> {
        let key: fn(&Transition) -> NaiveTimestamp = if utc { |trans| trans.at } else { |trans| trans.to };
        let idx = match self.transitions.data.binary_search_by_key(ts, key) {
            Ok(idx) => {
                // We need to get the latest prior to the given timestamp
                // That means if we're exactly at the boundary then the transition we're looking for
                // is the next one
                idx + 1
            }
            Err(idx) => idx,
        };

        if idx == 0 {
            self.transitions.data.get(idx).map(|f| (f, None))
        } else {
            self.transitions
                .data
                .get(idx)
                .map(|f| (f, self.transitions.data.get(idx - 1)))
        }
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
        match self.lookup_transition(&ts, false) {
            None => match &self.transitions.posix {
                None => None, // this shouldn't happen but...
                Some(posix) => posix.name(date, time),
            },
            Some((trans, prev)) => {
                match prev {
                    Some(p) if ts < trans.local => {
                        // This could use some ambiguous handling..
                        let ttype = &self.transitions.types[p.type_idx];
                        Some(ttype.abbr.clone())
                    }
                    _ => {
                        let ttype = &self.transitions.types[trans.type_idx];
                        Some(ttype.abbr.clone())
                    }
                }
            }
        }
    }

    fn offset(&self, date: &eos::Date, time: &eos::Time) -> eos::UtcOffset {
        let ts = NaiveTimestamp::new(date, time);
        match self.lookup_transition(&ts, false) {
            None => match &self.transitions.posix {
                None => eos::UtcOffset::UTC, // this shouldn't happen but...
                Some(posix) => posix.offset(date, time),
            },
            Some((trans, prev)) => {
                match prev {
                    Some(p) if ts < trans.local => {
                        // This could use some ambiguous handling..
                        p.offset
                    }
                    _ => trans.offset,
                }
            }
        }
    }

    fn at(self, mut utc: eos::DateTime<eos::Utc>) -> eos::DateTime<Self>
    where
        Self: Sized,
    {
        let mut ts = NaiveTimestamp::new(utc.date(), utc.time());
        match self.lookup_transition(&ts, true) {
            None => match &self.transitions.posix {
                None => utc.with_timezone(self),
                Some(posix) => {
                    posix.shift_utc(&mut utc);
                    utc.with_timezone(self)
                }
            },
            Some((trans, prev)) => {
                match prev {
                    Some(p) if ts < trans.at => {
                        let ttype = &self.transitions.types[p.type_idx];
                        ts.0 += ttype.offset as i64;
                    }
                    _ => {
                        let ttype = &self.transitions.types[trans.type_idx];
                        ts.0 += ttype.offset as i64;
                    }
                }
                ts.to_utc().with_timezone(self)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use eos::datetime;

    use super::*;

    #[test]
    fn test_name() {
        let dt = eos::Local::now().unwrap();
        println!("{:?}", &dt);
        let utc = datetime!(2007-3-11 8:00);
        let tz = zone!("America/New_York");
        println!("{:?}", utc.at(tz));
    }
}
