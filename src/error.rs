use crate::{Date, Time};

/// The error type for most errors that can be encountered when using the library.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Error {
    /// Construction or modification of a date or time was out of range.
    ///
    /// This is mainly returned when using the [`crate::Builder`] interface,
    /// most out of bound errors are done using [`Option`] instead.
    OutOfRange,
    /// Could not get the system time or timezone information
    NoSystemTime,
    /// The [`DateTime`] cannot be represented.
    ///
    /// [`DateTime`]: crate::DateTime
    SkippedDateTime(Date, Time),
    /// The [`DateTime`] is ambiguous.
    ///
    /// [`DateTime`]: crate::DateTime
    AmbiguousDateTime(Date, Time),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::OutOfRange => f.write_str("value out of range"),
            Error::NoSystemTime => f.write_str("could not fetch system time or timezone"),
            Error::SkippedDateTime(date, time) => write!(f, "{}T{} was skipped", date, time),
            Error::AmbiguousDateTime(date, time) => write!(f, "{}T{} is ambiguous", date, time),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
