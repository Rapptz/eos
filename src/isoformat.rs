//! Traits and types that help with parsing to and from the ISO-8601 standard.
//!
//! Note that ISO-8601 output is the default [`std::fmt::Display`] output for
//! all types in the library. Despite the fact that this module depends on the `format`
//! feature, the `Display` implementations are still enabled without that feature.
//!
//! The library does not aim for strict ISO-8601 compliance, for example ISO-8601 does not
//! support concepts such as negative duration and have a lot of esoteric formats that aren't
//! supported. The support in this library is similar to the [`java.time`] library, where things
//! are supported in a way that make the most sense for the given domain and restriction of the
//! library.
//!
//! [`java.time`]: https://docs.oracle.com/javase/8/docs/api/java/time/package-summary.html

use crate::{utils::divmod, Date, Time};
use core::{fmt::Write, iter::Peekable, str::Bytes};

#[cfg(feature = "parsing")]
use crate::error::ParseError;

/// Converts a value from an ISO-8601-1:2019 formatted string.
///
/// This trait is similar to [`std::str::FromStr`] except it deals with a subset of
/// string formats.
///
/// Note that this library does not aim for strict ISO-8601 compliance and a lot of esoteric
/// formats are not supported. The documentation of the individual implementations should
/// mention what formats are supported.
#[cfg(feature = "parsing")]
pub trait FromIsoFormat: Sized {
    /// Parses a string `s` to return a valid value of this type.
    ///
    /// If parsing fails then [`ParseError`] is returned in the [`Err`] variant.
    fn from_iso_format(s: &str) -> Result<Self, ParseError>;
}

/// An enum that specifies how the [`ToIsoFormat`] trait should handle precision of the components.
///
/// If a given precision would omit certain values from displaying, these values are *omitted*
/// rather than rounded to that value. For example, if the [`IsoFormatPrecision::Hour`] precision
/// is given at a time representing `12:59` then `12:00` will be returned not `13:00`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[cfg(feature = "formatting")]
pub enum IsoFormatPrecision {
    /// Display up to the hour, leaving remaining values either as 0 or omitted if possible.
    Hour,
    /// Display up to the minute, leaving remaining values either as 0 or omitted if possible.
    Minute,
    /// Display up to the second, leaving the fractional seconds omitted.
    Second,
    /// Display fractional seconds up to millisecond precision.
    Millisecond,
    /// Display fractional seconds up to microsecond precision.
    Microsecond,
    /// Display fractional seconds up to nanosecond precision.
    Nanosecond,
}

/// Converts a value to an ISO-8601 extended format string. Note that does not
/// aim for strict ISO-8601 compliance, for example ISO-8601 does not support concepts
/// such as negative duration and have a lot of esoteric formats that aren't supported.
///
/// This conversion should be infallible (other than allocation errors...).
#[cfg(feature = "formatting")]
pub trait ToIsoFormat {
    /// Converts to an appropriate ISO-8601 extended formatted string with the given precision.
    ///
    /// Certain types do not make use of this precision and will be ignored. A much simpler
    /// alternative is provided under [`Self::to_iso_format`].
    fn to_iso_format_with_precision(&self, precision: IsoFormatPrecision) -> String;

    /// Converts to an appropriate ISO-8601 extended formatted string.
    ///
    /// This function attempts to convert with appropriate precision for the given type.
    /// This means that certain values (typically fractional seconds) will be omitted if they
    /// can be.
    fn to_iso_format(&self) -> String;
}

#[cfg(feature = "formatting")]
impl ToIsoFormat for core::time::Duration {
    fn to_iso_format_with_precision(&self, _precision: IsoFormatPrecision) -> String {
        self.to_iso_format()
    }

    fn to_iso_format(&self) -> String {
        let mut buffer = String::new();
        let total_secs = self.as_secs_f64();
        if total_secs < 60.0 {
            // Simple case with (just) fractional seconds
            write!(&mut buffer, "PT{}S", self.as_secs_f64()).expect("unexpected error when writing to string");
        } else {
            let (hours, seconds) = divmod!(total_secs, 3600.0);
            let (minutes, seconds) = divmod!(seconds, 60.0);
            buffer.push('P');
            buffer.push('T');
            if hours > 0.0 {
                buffer
                    .write_fmt(format_args!("{}H", hours))
                    .expect("unexpected error when writing to string");
            }

            if minutes > 0.0 {
                buffer
                    .write_fmt(format_args!("{}M", minutes))
                    .expect("unexpected error when writing to string");
            }

            if seconds > 0.0 {
                buffer
                    .write_fmt(format_args!("{}S", hours))
                    .expect("unexpected error when writing to string");
            }
        }
        buffer
    }
}

/// A parser to parse ISO-8601 like strings.
///
/// This assumes that the string is entirely ASCII (which it should be).
/// Anything outside of the ASCII range would return an appropriate error anyway.
#[cfg(feature = "parsing")]
pub(crate) struct IsoParser<'a> {
    bytes: Peekable<Bytes<'a>>,
}

/// Represents either a month or an ordinal date
#[cfg(feature = "parsing")]
enum OrdinalMonthResult {
    Month(u8),
    Ordinal(u16),
}

#[cfg(feature = "parsing")]
const POW10: [u32; 9] = [1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000];

#[cfg(feature = "parsing")]
impl<'a> IsoParser<'a> {
    pub(crate) fn new(s: &'a str) -> Self {
        Self {
            bytes: s.bytes().peekable(),
        }
    }

    /// Peeks the next character in the stream
    #[inline]
    pub(crate) fn peek(&mut self) -> Option<u8> {
        self.bytes.peek().copied()
    }

    /// Advances the stream by 1 byte.
    #[inline]
    pub(crate) fn advance(&mut self) -> Option<u8> {
        self.bytes.next()
    }

    /// Advances the stream if the predicate is met.
    pub(crate) fn advance_if<F>(&mut self, predicate: F) -> Option<u8>
    where
        F: FnOnce(&u8) -> bool,
    {
        self.bytes.next_if(predicate)
    }

    /// Advanced the stream by 1 byte if the next
    /// byte matches the expected one.
    #[inline]
    pub(crate) fn advance_if_equal(&mut self, expected: u8) -> Option<u8> {
        self.advance_if(|&ch| ch == expected)
    }

    /// Parses up to N digits, returning an N-sized array with a count of how many digits were found.
    ///
    /// This does not error on end of string or if a non-digit is found.
    pub(crate) fn parse_up_to_n_digits<const N: usize>(&mut self) -> ([u8; N], usize) {
        let mut digits = [0u8; N];
        let mut index = 0;
        while index < N {
            match self.bytes.next_if(u8::is_ascii_digit) {
                Some(b) => digits[index] = b - b'0',
                None => break,
            }
            index += 1;
        }

        (digits, index)
    }

    /// Expects the stream to have the following byte
    #[inline]
    pub(crate) fn expect(&mut self, expected: u8) -> Result<u8, ParseError> {
        match self.advance() {
            Some(b) if b == expected => Ok(b),
            Some(b) => Err(ParseError::UnexpectedChar {
                expected: expected as char,
                found: b as char,
            }),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    /// Parses an optional ± and returns whether the value is negative.
    pub(crate) fn parse_sign(&mut self) -> bool {
        match self.peek() {
            Some(b'+') => {
                self.bytes.next();
                false
            }
            Some(b'-') => {
                self.bytes.next();
                true
            }
            _ => false,
        }
    }

    /// Parses a year matching the syntax `±?YYYYY?`. Years must be zero-padded.
    pub(crate) fn parse_year(&mut self) -> Result<i16, ParseError> {
        let negative = self.parse_sign();
        let mut digits = [0u8; 4];
        for digit in digits.iter_mut() {
            match self.advance() {
                Some(b) if b.is_ascii_digit() => *digit = b - b'0',
                Some(_) => return Err(ParseError::UnexpectedNonDigit),
                None => return Err(ParseError::UnexpectedEnd),
            }
        }

        let year = digits[0] as i32 * 1000 + digits[1] as i32 * 100 + digits[2] as i32 * 10 + digits[3] as i32;
        let year = if let Some(b) = self.advance_if(u8::is_ascii_digit) {
            let new_year = year * 10 + (b - b'0') as i32;
            i16::try_from(new_year)?
        } else {
            year as i16
        };

        Ok(if negative { -year } else { year })
    }

    /// Parses a two digit unit (e.g. `02`) into their integer representation.
    /// This must match the syntax `NN`. No signs permitted.
    ///
    /// Must be zero padded. Note this does not do bound checking.
    pub(crate) fn parse_two_digits(&mut self) -> Result<u8, ParseError> {
        let mut digits = [0u8; 2];
        for digit in digits.iter_mut() {
            match self.advance() {
                Some(b) if b.is_ascii_digit() => *digit = b - b'0',
                Some(_) => return Err(ParseError::UnexpectedNonDigit),
                None => return Err(ParseError::UnexpectedEnd),
            }
        }

        Ok(digits[0] * 10 + digits[1])
    }

    /// Parses a single digit
    pub(crate) fn parse_digit(&mut self) -> Result<u8, ParseError> {
        match self.advance() {
            Some(b) if b.is_ascii_digit() => Ok(b - b'0'),
            Some(_) => Err(ParseError::UnexpectedNonDigit),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    /// Parses a month matching the `MM` syntax. This performs bounds checking between
    /// 1 and 12.
    pub(crate) fn parse_month(&mut self) -> Result<u8, ParseError> {
        let digits = self.parse_two_digits()?;
        if digits == 0 || digits > 12 {
            Err(ParseError::OutOfBounds)
        } else {
            Ok(digits)
        }
    }

    /// Parses either a month in `NN` syntax or an ordinal in `NNN` syntax.
    ///
    /// This bound checks the month but *not* the ordinal date.
    fn parse_month_or_ordinal(&mut self) -> Result<OrdinalMonthResult, ParseError> {
        let digits = self.parse_two_digits()?;
        if let Some(b) = self.advance_if(u8::is_ascii_digit) {
            let ordinal = digits as u16 * 10 + (b - b'0') as u16;
            Ok(OrdinalMonthResult::Ordinal(ordinal))
        } else if digits == 0 || digits > 12 {
            Err(ParseError::OutOfBounds)
        } else {
            Ok(OrdinalMonthResult::Month(digits))
        }
    }

    /// Parses the supported date formats.
    ///
    /// Right now these are:
    ///
    /// - `±YYYYY-MM-DD` (e.g. `2012-02-13` or `-9999-10-12`)
    /// - `±YYYYY-MM` (e.g. `2012-02`)
    /// - `±YYYYY-Www` (e.g. `2012-W10`)
    /// - `±YYYYY-Www-D` (e.g. `2012-W10-1`)
    /// - `±YYYYY-DDD` (e.g. `2021-048`)
    pub(crate) fn parse_date(&mut self) -> Result<Date, ParseError> {
        let year = self.parse_year()?;
        self.expect(b'-')?;
        match self.advance_if_equal(b'W') {
            Some(_) => {
                // week date parsing, i.e. 2012-W10-1
                let week = self.parse_two_digits()?;
                if week == 0 || week > crate::gregorian::iso_weeks_in_year(year) {
                    return Err(ParseError::OutOfBounds);
                }
                let weekday = match self.advance_if_equal(b'-') {
                    Some(_) => match self.parse_digit()? {
                        n @ 1..=7 => n - 1,
                        _ => return Err(ParseError::OutOfBounds),
                    },
                    None => 0,
                };
                let epoch =
                    crate::gregorian::iso_week_start_epoch_from_year(year) + (week as i32 - 1) * 7 + weekday as i32;
                let (year, month, day) = crate::gregorian::date_from_epoch_days(epoch);
                Ok(Date { year, month, day })
            }
            None => {
                match self.parse_month_or_ordinal()? {
                    OrdinalMonthResult::Month(month) => {
                        let day = match self.advance_if_equal(b'-') {
                            Some(_) => {
                                // YYYY-MM-DD
                                let day = self.parse_two_digits()?;
                                if day > crate::gregorian::days_in_month(year, month) {
                                    return Err(ParseError::OutOfBounds);
                                }
                                day
                            }
                            None => 1,
                        };
                        // The parser ensures these values are bound checked
                        Ok(Date { year, month, day })
                    }
                    OrdinalMonthResult::Ordinal(ordinal) => {
                        Date::from_ordinal(year, ordinal).map_err(|_| ParseError::OutOfBounds)
                    }
                }
            }
        }
    }

    /// Parses the supported time formats.
    ///
    /// Right now these are:
    ///
    /// - `HH:MM` (e.g. `10:23`)
    /// - `HH:MM:SS` (e.g. `10:24:30`)
    /// - `HH:MM:SS.sssssssss`, up to 9 digits of precision (e.g. `10:24:30.999999999`)
    /// - `HH:MM:SS,sssssssss`, same as above
    ///
    pub(crate) fn parse_time(&mut self) -> Result<Time, ParseError> {
        let hour = self.parse_two_digits()?;
        self.expect(b':')?;
        let minute = self.parse_two_digits()?;
        let (mut second, mut nanosecond) = match self.advance_if_equal(b':') {
            Some(_) => {
                let seconds = self.parse_two_digits()?;
                let nanoseconds = match self.advance_if(|&c| c == b'.' || c == b',') {
                    Some(_) => {
                        let (digits, count) = self.parse_up_to_n_digits::<9>();
                        if count == 0 {
                            // 12:34:56. is invalid (and incomplete...)
                            return Err(ParseError::UnexpectedNonDigit);
                        }
                        let mut ns = 0;
                        for (index, value) in digits.iter().enumerate() {
                            ns += *value as u32 * POW10[8 - index];
                        }
                        ns
                    }
                    None => 0,
                };
                (seconds, nanoseconds)
            }
            None => (0, 0),
        };

        if second == 60 {
            second -= 1;
            nanosecond += crate::interval::NANOS_PER_SEC as u32;
        }

        if hour > 24 || minute > 59 || second > 59 || nanosecond > 1_999_999_999 {
            Err(ParseError::OutOfBounds)
        } else {
            Ok(Time {
                hour,
                minute,
                second,
                nanosecond,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_year() {
        let mut parser = IsoParser::new("2012-02-13");
        assert_eq!(parser.parse_year(), Ok(2012));
        assert_eq!(parser.advance(), Some(b'-'));

        let mut parser = IsoParser::new("-2012-02-13");
        assert_eq!(parser.parse_year(), Ok(-2012));
        assert_eq!(parser.advance(), Some(b'-'));

        let mut parser = IsoParser::new("-45678-02-13");
        assert_eq!(parser.parse_year(), Err(ParseError::OutOfBounds));

        let mut parser = IsoParser::new("234");
        assert_eq!(parser.parse_year(), Err(ParseError::UnexpectedEnd));

        let mut parser = IsoParser::new("234a");
        assert_eq!(parser.parse_year(), Err(ParseError::UnexpectedNonDigit));
    }
}
