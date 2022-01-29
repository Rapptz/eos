/// Represents parsing errors that can be encountered when parsing TZif files.
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

impl From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
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
