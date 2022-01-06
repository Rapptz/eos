/// Represents all types of errors that can be encountered when using the library.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Error {
    /// Construction or modification of a date or time was out of range.
    OutOfRange,
    /// Could not get the local time or timezone information
    NoLocalTime,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::OutOfRange => f.write_str("value out of range"),
            Error::NoLocalTime => f.write_str("could not fetch local time or timezone"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
