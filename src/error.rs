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

/// Represents an error that occurs during parsing a string to the specified
/// type. For example, this is given as a result of a failure in
/// the [`crate::isoformat::FromIsoFormat`] trait.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "format")]
pub enum ParseError {
    /// The parser expected a character but there were no more.
    UnexpectedEnd,
    /// The parser expected a character but it found something else.
    UnexpectedChar { expected: char, found: char },
    /// The parser expected a digit but did not find one
    UnexpectedNonDigit,
    /// A value was out of bounds (such as a year, month, day, etc.)
    ///
    /// To prevent the enum from bloating up these are all consolidated into one variant.
    OutOfBounds,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseError::UnexpectedEnd => f.write_str("unexpected end of string"),
            ParseError::UnexpectedChar { expected, found } => {
                write!(f, "expected `{}` but received `{}`", expected, found)
            }
            ParseError::UnexpectedNonDigit => f.write_str("expected a digit but did not find one"),
            ParseError::OutOfBounds => f.write_str("a unit was out of bounds"),
        }
    }
}

#[cfg(all(feature = "std", feature = "format"))]
impl std::error::Error for ParseError {}

impl From<core::num::TryFromIntError> for ParseError {
    fn from(_: core::num::TryFromIntError) -> Self {
        Self::OutOfBounds
    }
}
