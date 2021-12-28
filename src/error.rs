/// Represents all types of errors that can be encountered when using the library.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Error {
    /// Construction or modification of a date or time was out of range.
    OutOfRange,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::OutOfRange => f.write_str("value out of range"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
