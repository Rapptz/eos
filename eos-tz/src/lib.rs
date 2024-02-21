#![allow(clippy::manual_range_contains)]

pub(crate) mod error;
mod local;
mod posix;
pub(crate) mod reader;
pub(crate) mod sys;
pub(crate) mod timestamp;
mod timezone;
mod transitions;

pub use error::{Error, ParseError};
pub use posix::PosixTimeZone;
pub use timezone::TimeZone;

#[cfg(feature = "localtime")]
pub use local::Local;

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
