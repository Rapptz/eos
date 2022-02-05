/// The error type that can be encountered when parsing TZif files.
#[derive(Debug)]
#[non_exhaustive]
pub enum ParseError {
    /// An I/O error occurred.
    Io(std::io::Error),
    /// The TZif file has an unsupported version
    UnsupportedVersion,
    /// The TZif file has an invalid magic
    InvalidMagic,
    /// The UTC Offset of a POSIX TZ string was invalid.
    InvalidOffset,
    /// The abbreviation of a transition type was not UTF-8.
    InvalidAbbreviation,
    /// The POSIX TZ string was invalid.
    InvalidPosixTz,
}

/// The error type for most operations in the library.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// A parser error occurred.
    Parse(ParseError),
    /// The timezone could not be found.
    NotFound,
    /// The timezone path was invalid. This could happen if it's
    /// prone to path traversal, such as `America/../New_York`. Only keys
    /// like `America/New_York` are valid.
    InvalidZonePath,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Io(e) => e.fmt(f),
            ParseError::UnsupportedVersion => f.write_str("unsupported version"),
            ParseError::InvalidMagic => f.write_str("invalid magic value (must be TZif)"),
            ParseError::InvalidOffset => f.write_str("utcoffset is invalid"),
            ParseError::InvalidAbbreviation => f.write_str("abbreviation data was not UTF-8"),
            ParseError::InvalidPosixTz => f.write_str("POSIX TZ string is invalid"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => e.fmt(f),
            Error::NotFound => f.write_str("timezone could not be located"),
            Error::InvalidZonePath => f.write_str("invalid timezone path"),
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::NotFound {
            Self::NotFound
        } else {
            Self::Parse(ParseError::Io(e))
        }
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::Parse(e)
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::Io(e) => Some(e),
            ParseError::UnsupportedVersion => None,
            ParseError::InvalidMagic => None,
            ParseError::InvalidOffset => None,
            ParseError::InvalidAbbreviation => None,
            ParseError::InvalidPosixTz => None,
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Parse(e) => Some(e),
            Error::NotFound => None,
            Error::InvalidZonePath => None,
        }
    }
}
