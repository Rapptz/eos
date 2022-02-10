//! Utilities for formatting and parsing various types in the library.
//!
//! # Usage
//!
//! There are multiple types of formatting provided by this library: ISO-8601, RFC 3339,
//! and a [strftime] inspired syntax.
//!
//! # ISO-8601 and RFC 3339
//!
//! The library provides simple methods to format and parse into these two common formats through
//! the [`ToIsoFormat`] and [`FromIsoFormat`] trait. Most types in the library implement these traits along with
//! [`std::time::Duration`].
//!
//! Note that ISO-8601 output is the default [`std::fmt::Display`] output for
//! all types in the library. Despite the fact that this module depends on the `formatting`
//! feature, the `Display` implementations are still enabled without that feature.
//!
//! The library does not aim for strict ISO-8601 compliance, for example ISO-8601 does not
//! support concepts such as negative duration and have a lot of esoteric formats that aren't
//! supported. The support in this library is similar to the [`java.time`] library, where things
//! are supported in a way that make the most sense for the given domain and restriction of the
//! library.
//!
//! RFC 3339 formatting is only provided by [`DateTime`] using [`DateTime::to_rfc3339`]
//! and [`DateTime::from_rfc3339`].
//!
//! # Format
//!
//! Extended formatting is done through functions such as [`DateTime::format`]. These
//! accept a slice of [`FormatSpec`] which detail how the formatting should be done and returns
//! a wrapper type that implements [`std::fmt::Display`].
//!
//! There are two ways to get a slice of [`FormatSpec`].
//!
//! 1. Using the [`format_spec`] macro, which requires the `macros` feature.
//! 2. Parsing it using the [`parse_spec`] function.
//!
//! The first option is preferred if the format string is done at compile-time since it
//! validates the format string entirely at compile-time and also works in `#![no_std]`
//! contexts. While the second option should be reserved for cases where the format string
//! is received at runtime.
//!
//! ## Format Specifiers
//!
//! The format specifiers in this library were mainly modelled after [strftime] but
//! with certain formats either being added, changed, or removed due to being incompatible
//! with the library or legacy reasons. For example, the libc `%y` and `%C` don't make
//! sense since the range of data used in this library are larger than the ones in `<time.h>`.
//!
//! | Specifier | Meaning                                                         | Example                          |
//! |:---------:|:----------------------------------------------------------------|:---------------------------------|
//! |   `%a`    | Abbreviated weekday name.                                       | Sun, Mon, ..., Sat               |
//! |   `%A`    | Full weekday name.                                              | Sunday, Monday, ..., Saturday    |
//! |   `%w`    | Weekday as a number where 0 is Sunday and 6 is Saturday.        | 0, 1, ... 6                      |
//! |   `%u`    | Weekday as a number where 1 is Monday and 7 is Saturday.        | 1, 2, ... 7                      |
//! |   `%d`    | Day of the month as a zero-padded number.[^1]                   | 01, 02, ..., 31                  |
//! |   `%j`    | Ordinal day of the year as a zero-padded number.[^1][^6]        | 001, 002, ..., 365               |
//! |   `%b`    | Abbreviated month name.                                         | Jan, Feb, ..., Dec               |
//! |   `%B`    | Full month name.                                                | January, February, ..., December |
//! |   `%m`    | Month as a zero-padded number.[^1]                              | 01, 02, ..., 12                  |
//! |   `%Y`    | Year as a zero-padded number.[^1]                               | 0001, 0002, ..., 32767           |
//! |   `%y`    | Same as `%Y` but with explicit sign.[^1]                        | -0001, 0000, ..., +32767         |
//! |   `%G`    | ISO 8601 week calendar year as a zero-padded number.[^1][^5]    | 0001, 0002, ..., 32767           |
//! |   `%V`    | ISO 8601 week as a zero-padded number.[^1][^5]                  | 01, 02, ..., 53                  |
//! |   `%H`    | Hour (24-hour clock) as a zero-padded number.[^1]               | 00, 01, ..., 23                  |
//! |   `%I`    | Hour (12-hour clock) as a zero-padded number.[^1]               | 01, 02, ..., 12                  |
//! |   `%p`    | The time meridiem (am or pm).                                   | AM, PM                           |
//! |   `%M`    | Minute as a zero-padded number.[^1]                             | 00, 01, ..., 59                  |
//! |   `%S`    | Second as a zero-padded number.[^1][^2]                         | 00, 01, ..., 59                  |
//! |   `%f`    | Nanoseconds as a zero-padded number.[^1][^3]                    | 0000000, 0000001, ..., 999999999 |
//! |   `%z`    | UTC offset as `±HHMM[SS]` or empty.                             | +0000, -0500, +102340, ...       |
//! |   `%o`    | UTC offset as `±HH:MM[:SS]` or empty.                           | +00:00, -05:00, +10:23:40, ...   |
//! |   `%Z`    | Timezone name or empty.[^4]                                     | UTC, EST, ...                    |
//! |   `%%`    | The literal `%` character.                                      | %                                |
//!
//! ### Modifiers
//!
//! Directives that are zero-padded support so called modifiers that help modify the formatting behavior.
//! These help change the formatting from zero-padding to either space or no padding. They *must* follow the `%` sign.
//!
//! | Modifier | Meaning                                      | Example                       |
//! |:--------:|:---------------------------------------------|:------------------------------|
//! |   `#`    | Use no padding at all                        | `%#d` outputs 1, 2, ..., 31   |
//! |   `_`    | Use spaces for padding instead of zeroes     | `%_d` outputs ` 1`, ` 2`, ... |
//!
//! ### Parsing Behaviour
//!
//! When parsing, the default without specifiers requires that certain digits must be zero-padded.
//! For example, using `%y` to parse `1` would fail since it expects zero-padded digits to represent the year.
//! However, using `%#y` is fine. This applies to every specifier that can be modified.
//!
//! [^1]: Supports modifiers. During parsing, modifiers are ignored and zero-padding is optional.
//! [^2]: This is leap second aware so `60` is possible.
//! [^3]: This is since the last whole second. This means the value will never be higher than `999_999_999`.
//!       Anything above that value is rolled over to the seconds value.
//!
//! [^4]: Unsupported when parsing. Usage will return a [`ParseError`].
//! [^5]: This is only used in calculating during parsing if used together.
//! [^6]: If provided with a year then this will be used for calculations.
//!
//! [strftime]: https://en.cppreference.com/w/cpp/chrono/c/strftime
//! [`java.time`]: https://docs.oracle.com/javase/8/docs/api/java/time/package-summary.html

use crate::{utils::divmod, Date, DateTime, Time, TimeZone, Weekday};
use core::{fmt::Write, iter::Peekable, str::Bytes};

/// The error type that occurs during parsing a string.
///
/// For example, this is given as a result of a failure in the [`FromIsoFormat`] trait.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "parsing")]
pub enum ParseError {
    /// The parser expected a character but there were no more.
    UnexpectedEnd,
    /// The parser expected a character but it found something else.
    ///
    /// The inner character is the character found.
    UnexpectedChar(char),
    /// The parser expected a digit but did not find one
    UnexpectedNonDigit,
    /// The parser found an unsupported directive or modifier.
    UnsupportedSpecifier,
    /// A value was out of bounds (such as a year, month, day, etc.)
    ///
    /// To prevent the enum from bloating up these are all consolidated into one variant.
    OutOfBounds,
}

#[cfg(feature = "parsing")]
impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseError::UnexpectedEnd => f.write_str("unexpected end of string"),
            ParseError::UnexpectedChar(found) => {
                write!(f, "unexpected character found `{}`", found)
            }
            ParseError::UnexpectedNonDigit => f.write_str("expected a digit but did not find one"),
            ParseError::OutOfBounds => f.write_str("a unit was out of bounds"),
            ParseError::UnsupportedSpecifier => f.write_str("unsupported format or specifier found"),
        }
    }
}

#[cfg(all(feature = "std", feature = "parsing"))]
impl std::error::Error for ParseError {}

#[cfg(feature = "parsing")]
impl From<core::num::TryFromIntError> for ParseError {
    fn from(_: core::num::TryFromIntError) -> Self {
        Self::OutOfBounds
    }
}

/* ISO 8601 related functionality */

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
    #[must_use]
    fn to_iso_format_with_precision(&self, precision: IsoFormatPrecision) -> String;

    /// Converts to an appropriate ISO-8601 extended formatted string.
    ///
    /// This function attempts to convert with appropriate precision for the given type.
    /// This means that certain values (typically fractional seconds) will be omitted if they
    /// can be.
    #[must_use]
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

/// A parser to parse date time strings.
#[cfg(feature = "parsing")]
pub(crate) struct Parser<'a> {
    bytes: Peekable<Bytes<'a>>,
}

/// Either a month or an ordinal date
#[cfg(feature = "parsing")]
enum OrdinalMonthResult {
    Month(u8),
    Ordinal(u16),
}

#[cfg(feature = "parsing")]
const POW10: [u32; 9] = [1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000];

#[cfg(feature = "parsing")]
impl<'a> Parser<'a> {
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

    /// Parses up to 9 digits, returning the number being represented.
    ///
    /// This also handles the optional sign. If the number is too large to fit in an
    /// i32 then it errors out. If no numbers are given then this will also error.
    pub(crate) fn parse_i32(&mut self) -> Result<i32, ParseError> {
        let negative = self.parse_sign();
        let mut read_any: bool = false;
        let mut n: i32 = 0;
        for _ in 0..9 {
            match self.advance_if(u8::is_ascii_digit) {
                Some(c) => {
                    n = n * 10 + (c as u8 - b'0') as i32;
                    read_any = true;
                }
                None => break,
            }
        }

        if read_any {
            Ok(if negative { -n } else { n })
        } else {
            Err(ParseError::UnexpectedNonDigit)
        }
    }

    /// Parses up to 9 digits, returning the number being represented.
    ///
    /// If the number is too large to fit in an u32 then it errors out.
    /// If no numbers are given then this will also error.
    pub(crate) fn parse_u32(&mut self) -> Result<u32, ParseError> {
        let mut read_any: bool = false;
        let mut n: u32 = 0;
        for _ in 0..9 {
            match self.advance_if(u8::is_ascii_digit) {
                Some(c) => {
                    n = n * 10 + (c as u8 - b'0') as u32;
                    read_any = true;
                }
                None => break,
            }
        }

        if read_any {
            Ok(n)
        } else {
            Err(ParseError::UnexpectedNonDigit)
        }
    }

    /// Parses up to N digits, returning the number being represented.
    ///
    /// This is mainly used for parsing between 2-3 digits without leading zeroes.
    pub(crate) fn parse_u16<const N: usize>(&mut self) -> Result<u16, ParseError> {
        let mut read_any: bool = false;
        let mut n: u16 = 0;
        for _ in 0..N {
            match self.advance_if(u8::is_ascii_digit) {
                Some(c) => {
                    n = n * 10 + (c as u8 - b'0') as u16;
                    read_any = true;
                }
                None => break,
            }
        }

        if read_any {
            Ok(n)
        } else {
            Err(ParseError::UnexpectedNonDigit)
        }
    }

    /// Parses up to 9 digits, returning the number being represented.
    ///
    /// If the number is too large to fit in an u32 then it errors out.
    /// If no numbers are given then this will also error.
    pub(crate) fn parse_nanoseconds(&mut self) -> Result<u32, ParseError> {
        let (digits, count) = self.parse_up_to_n_digits::<9>();
        if count == 0 {
            Err(ParseError::UnexpectedNonDigit)
        } else {
            let mut result = 0;
            for (index, value) in digits.iter().enumerate() {
                result += *value as u32 * POW10[8 - index];
            }
            Ok(result)
        }
    }

    /// Expects the stream to have the following byte
    #[inline]
    pub(crate) fn expect(&mut self, expected: u8) -> Result<u8, ParseError> {
        match self.advance() {
            Some(b) if b == expected => Ok(b),
            Some(b) => Err(ParseError::UnexpectedChar(b as char)),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    #[inline]
    pub(crate) fn expect_str(&mut self, expected: &[u8]) -> Result<(), ParseError> {
        for byte in expected {
            match self.advance() {
                Some(b) if b == *byte => continue,
                Some(b) => return Err(ParseError::UnexpectedChar(b as char)),
                None => return Err(ParseError::UnexpectedEnd),
            }
        }
        Ok(())
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

    /// Parses a required ± and returns whether the value is negative.
    pub(crate) fn parse_required_sign(&mut self) -> Result<bool, ParseError> {
        match self.peek() {
            Some(b'+') => {
                self.bytes.next();
                Ok(false)
            }
            Some(b'-') => {
                self.bytes.next();
                Ok(true)
            }
            Some(c) => Err(ParseError::UnexpectedChar(c as char)),
            None => Err(ParseError::UnexpectedEnd),
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
                    Some(_) => self.parse_nanoseconds()?,
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

/// A handle for how a fragment should be formatted.
///
/// A slice of these is usually passed to the corresponding `format` function.
/// These slices can be received via either the [`format_spec`] macro or
/// using the [`parse_spec`] function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatSpec<'a> {
    kind: FormatSpecKind<'a>,
    padding: FormatSpecPadding,
}

impl<'a> FormatSpec<'a> {
    /// Creates a new [`FormatSpec`] from the given [`FormatSpecKind`].
    pub const fn new(kind: FormatSpecKind<'a>) -> Self {
        Self {
            kind,
            padding: FormatSpecPadding::Zero,
        }
    }

    /// Returns a new [`FormatSpec`] with no padding.
    ///
    /// This is equivalent to the `#` modifier.
    pub const fn with_no_padding(mut self) -> Self {
        self.padding = FormatSpecPadding::Empty;
        self
    }

    /// Returns a new [`FormatSpec`] with zero padding.
    ///
    /// This is the default modifier.
    pub const fn with_zero_padding(mut self) -> Self {
        self.padding = FormatSpecPadding::Zero;
        self
    }

    /// Returns a new [`FormatSpec`] with space padding.
    ///
    /// This is equivalent to the `_` modifier.
    pub const fn with_space_padding(mut self) -> Self {
        self.padding = FormatSpecPadding::Space;
        self
    }

    /// Returns a new [`FormatSpec`] with a raw string.
    pub const fn raw(s: &'a str) -> Self {
        Self {
            kind: FormatSpecKind::Raw(s),
            padding: FormatSpecPadding::Empty,
        }
    }

    #[cfg(feature = "parsing")]
    pub(crate) fn parse_into(
        &self,
        builder: &mut crate::Builder<crate::UtcOffset>,
        parser: &mut Parser,
    ) -> Result<(), ParseError> {
        match self.kind {
            FormatSpecKind::Raw(x) => {
                parser.expect_str(x.as_bytes())?;
            }
            FormatSpecKind::AbbreviatedWeekday => {
                // Mon, Tue, Wed, Thu, Fri, Sat, Sun
                match parser.advance() {
                    Some(b'M') => {
                        parser.expect_str(b"on")?;
                        builder.weekday(Weekday::Monday);
                    }
                    Some(b'T') => match parser.advance() {
                        Some(b'u') => {
                            parser.expect(b'e')?;
                            builder.weekday(Weekday::Tuesday);
                        }
                        Some(b'h') => {
                            parser.expect(b'u')?;
                            builder.weekday(Weekday::Thursday);
                        }
                        Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                        None => return Err(ParseError::UnexpectedEnd),
                    },
                    Some(b'W') => {
                        parser.expect_str(b"ed")?;
                        builder.weekday(Weekday::Wednesday);
                    }
                    Some(b'F') => {
                        parser.expect_str(b"ri")?;
                        builder.weekday(Weekday::Friday);
                    }
                    Some(b'S') => match parser.advance() {
                        Some(b'a') => {
                            parser.expect(b't')?;
                            builder.weekday(Weekday::Saturday);
                        }
                        Some(b'u') => {
                            parser.expect(b'n')?;
                            builder.weekday(Weekday::Sunday);
                        }
                        Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                        None => return Err(ParseError::UnexpectedEnd),
                    },
                    Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                    None => return Err(ParseError::UnexpectedEnd),
                }
            }
            FormatSpecKind::FullWeekday => {
                // Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday
                match parser.advance() {
                    Some(b'M') => {
                        parser.expect_str(b"onday")?;
                        builder.weekday(Weekday::Monday);
                    }
                    Some(b'T') => match parser.advance() {
                        Some(b'u') => {
                            parser.expect_str(b"esday")?;
                            builder.weekday(Weekday::Tuesday);
                        }
                        Some(b'h') => {
                            parser.expect_str(b"ursday")?;
                            builder.weekday(Weekday::Thursday);
                        }
                        Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                        None => return Err(ParseError::UnexpectedEnd),
                    },
                    Some(b'W') => {
                        parser.expect_str(b"ednesday")?;
                        builder.weekday(Weekday::Wednesday);
                    }
                    Some(b'F') => {
                        parser.expect_str(b"riday")?;
                        builder.weekday(Weekday::Friday);
                    }
                    Some(b'S') => match parser.advance() {
                        Some(b'a') => {
                            parser.expect_str(b"turday")?;
                            builder.weekday(Weekday::Saturday);
                        }
                        Some(b'u') => {
                            parser.expect_str(b"nday")?;
                            builder.weekday(Weekday::Sunday);
                        }
                        Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                        None => return Err(ParseError::UnexpectedEnd),
                    },
                    Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                    None => return Err(ParseError::UnexpectedEnd),
                }
            }
            FormatSpecKind::Weekday => {
                let digit = parser.parse_digit()?;
                let weekday = match digit {
                    0 => Weekday::Sunday,
                    1 => Weekday::Monday,
                    2 => Weekday::Tuesday,
                    3 => Weekday::Wednesday,
                    4 => Weekday::Thursday,
                    5 => Weekday::Friday,
                    6 => Weekday::Saturday,
                    _ => return Err(ParseError::OutOfBounds),
                };
                builder.weekday(weekday);
            }
            FormatSpecKind::IsoWeekday => {
                let digit = parser.parse_digit()?;
                let weekday = match digit {
                    1 => Weekday::Monday,
                    2 => Weekday::Tuesday,
                    3 => Weekday::Wednesday,
                    4 => Weekday::Thursday,
                    5 => Weekday::Friday,
                    6 => Weekday::Saturday,
                    7 => Weekday::Sunday,
                    _ => return Err(ParseError::OutOfBounds),
                };
                builder.weekday(weekday);
            }
            FormatSpecKind::Day => {
                let day = parser.parse_u16::<2>()? as u8;
                builder.day(day);
            }
            FormatSpecKind::Ordinal => {
                let ordinal = parser.parse_u16::<3>()?;
                builder.ordinal(ordinal);
            }
            FormatSpecKind::AbbreviatedMonth => {
                // Jan, Feb, Mar, Apr, May, Jun, Jul, Aug, Sep, Oct, Nov, Dec
                match parser.advance() {
                    Some(b'J') => match parser.advance() {
                        Some(b'a') => {
                            parser.expect(b'n')?;
                            builder.month(1);
                        }
                        Some(b'u') => {
                            match parser.advance() {
                                Some(b'n') => builder.month(6),
                                Some(b'l') => builder.month(7),
                                Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                                None => return Err(ParseError::UnexpectedEnd),
                            };
                        }
                        Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                        None => return Err(ParseError::UnexpectedEnd),
                    },
                    Some(b'F') => {
                        parser.expect_str(b"eb")?;
                        builder.month(2);
                    }
                    Some(b'M') => {
                        parser.expect(b'a')?;
                        match parser.advance() {
                            Some(b'r') => builder.month(3),
                            Some(b'y') => builder.month(5),
                            Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                            None => return Err(ParseError::UnexpectedEnd),
                        };
                    }
                    Some(b'A') => {
                        parser.expect_str(b"ug")?;
                        builder.month(8);
                    }
                    Some(b'S') => {
                        parser.expect_str(b"ep")?;
                        builder.month(9);
                    }
                    Some(b'O') => {
                        parser.expect_str(b"ct")?;
                        builder.month(10);
                    }
                    Some(b'N') => {
                        parser.expect_str(b"ov")?;
                        builder.month(11);
                    }
                    Some(b'D') => {
                        parser.expect_str(b"ec")?;
                        builder.month(12);
                    }
                    Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                    None => return Err(ParseError::UnexpectedEnd),
                }
            }
            FormatSpecKind::FullMonth => {
                // January, February, March, April, May, June, July, August, September, October, November, December
                match parser.advance() {
                    Some(b'J') => match parser.advance() {
                        Some(b'a') => {
                            parser.expect_str(b"nuary")?;
                            builder.month(1);
                        }
                        Some(b'u') => {
                            match parser.advance() {
                                Some(b'n') => {
                                    parser.expect(b'e')?;
                                    builder.month(6);
                                }
                                Some(b'l') => {
                                    parser.expect(b'y')?;
                                    builder.month(7);
                                }
                                Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                                None => return Err(ParseError::UnexpectedEnd),
                            };
                        }
                        Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                        None => return Err(ParseError::UnexpectedEnd),
                    },
                    Some(b'F') => {
                        parser.expect_str(b"ebruary")?;
                        builder.month(2);
                    }
                    Some(b'M') => {
                        parser.expect(b'a')?;
                        match parser.advance() {
                            Some(b'r') => {
                                parser.expect_str(b"ch")?;
                                builder.month(3);
                            }
                            Some(b'y') => {
                                builder.month(5);
                            }
                            Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                            None => return Err(ParseError::UnexpectedEnd),
                        };
                    }
                    Some(b'A') => {
                        parser.expect_str(b"ugust")?;
                        builder.month(8);
                    }
                    Some(b'S') => {
                        parser.expect_str(b"eptember")?;
                        builder.month(9);
                    }
                    Some(b'O') => {
                        parser.expect_str(b"ctober")?;
                        builder.month(10);
                    }
                    Some(b'N') => {
                        parser.expect_str(b"ovember")?;
                        builder.month(11);
                    }
                    Some(b'D') => {
                        parser.expect_str(b"ecember")?;
                        builder.month(12);
                    }
                    Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                    None => return Err(ParseError::UnexpectedEnd),
                }
            }
            FormatSpecKind::Month => {
                let month = parser.parse_u16::<2>()? as u8;
                builder.month(month);
            }
            FormatSpecKind::Year | FormatSpecKind::SignedYear | FormatSpecKind::IsoWeekYear => {
                let negative = parser.parse_sign();
                let year = i16::try_from(parser.parse_u16::<5>()?)?;
                builder.year(if negative { -year } else { year });
            }
            FormatSpecKind::IsoWeek => {
                let iso_week = parser.parse_u16::<2>()? as u8;
                builder.iso_week(iso_week);
            }
            FormatSpecKind::Hour | FormatSpecKind::Hour12 => {
                let hour = parser.parse_u16::<2>()? as u8;
                builder.hour(hour);
            }
            FormatSpecKind::Meridiem => match parser.advance() {
                Some(b'a' | b'A') => match parser.advance() {
                    Some(b'm' | b'M') => {
                        builder.am();
                    }
                    Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                    None => return Err(ParseError::UnexpectedEnd),
                },
                Some(b'p' | b'P') => match parser.advance() {
                    Some(b'm' | b'M') => {
                        builder.pm();
                    }
                    Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                    None => return Err(ParseError::UnexpectedEnd),
                },
                Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                None => return Err(ParseError::UnexpectedEnd),
            },
            FormatSpecKind::Minute => {
                let minute = parser.parse_u16::<2>()? as u8;
                builder.minute(minute);
            }
            FormatSpecKind::Second => {
                let second = parser.parse_u16::<2>()? as u8;
                builder.second(second);
            }
            FormatSpecKind::Nanosecond => {
                let nanos = parser.parse_nanoseconds()?;
                builder.nanosecond(nanos);
            }
            FormatSpecKind::UtcOffset => {
                // [+-]HH:MM[:SS]?
                let negative = parser.parse_required_sign()?;
                let hour = parser.parse_two_digits()? as i8;
                parser.expect(b':')?;
                let minute = parser.parse_two_digits()? as i8;
                let seconds = if parser.advance_if_equal(b':').is_some() {
                    parser.parse_two_digits()? as i8
                } else {
                    0
                };
                let offset = crate::UtcOffset::from_hms(hour, minute, seconds).map_err(|_| ParseError::OutOfBounds)?;
                if negative {
                    builder.timezone = -offset;
                } else {
                    builder.timezone = offset;
                }
            }
            FormatSpecKind::UtcOffsetBrief => {
                // [+-]HHMM[SS]?
                let negative = parser.parse_required_sign()?;
                let hour = parser.parse_two_digits()? as i8;
                let minute = parser.parse_two_digits()? as i8;
                let seconds = match parser.peek() {
                    Some(c) if c.is_ascii_digit() => parser.parse_two_digits()? as i8,
                    _ => 0,
                };
                let offset = crate::UtcOffset::from_hms(hour, minute, seconds).map_err(|_| ParseError::OutOfBounds)?;
                if negative {
                    builder.timezone = -offset;
                } else {
                    builder.timezone = offset;
                }
            }
            FormatSpecKind::ZoneName => return Err(ParseError::UnsupportedSpecifier),
            FormatSpecKind::Escape => {
                parser.expect(b'%')?;
            }
        }
        Ok(())
    }
}

/// The error that occurred during parsing in [`parse_spec`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Error {
    /// An unknown format specifier was found
    UnknownSpecifier(u8),
    /// A specifier was expected after a `%` or a modifier.
    SpecifierNotFound,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownSpecifier(c) => write!(f, "unknown specifier `{}`", *c as char),
            Error::SpecifierNotFound => write!(f, "expected specifier after `%`, `%_`, or `%#`"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum FormatSpecPadding {
    Zero,
    Empty,
    Space,
}

/// The kind of fragment in a format specification.
///
/// These are the internal data within a [`FormatSpec`] and are generally
/// not manually constructed.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatSpecKind<'a> {
    /// A raw string
    Raw(&'a str),
    /// An abbreviated weekday name (`%a`)
    AbbreviatedWeekday,
    /// A full weekday name (`%A`)
    FullWeekday,
    /// A numerical weekday (`%w`)
    Weekday,
    /// An ISO weekday (`%u`)
    IsoWeekday,
    /// A numerical day (`%d`)
    Day,
    /// An ordinal day (`%j`)
    Ordinal,
    /// An abbreviated month name (`%b`)
    AbbreviatedMonth,
    /// A full month name (`%B`)
    FullMonth,
    /// The numerical month (`%m`)
    Month,
    /// The numerical year (`%Y`)
    Year,
    /// An explicitly signed year (`%y`)
    SignedYear,
    /// An ISO week calendar year (`%G`)
    IsoWeekYear,
    /// An ISO week calendar week (`%V`)
    IsoWeek,
    /// The numerical 24-hour clock (`%H`)
    Hour,
    /// The numerical 12-hour clock (`%I`)
    Hour12,
    /// The time meridiem (`%p`)
    Meridiem,
    /// The numerical minute (`%M`)
    Minute,
    /// The numerical second (`%S`)
    Second,
    /// The numerical nanosecond (`%f`)
    Nanosecond,
    /// The UTC offset (`%o`)
    UtcOffset,
    /// A shortened UTC offset (`%z`)
    UtcOffsetBrief,
    /// The timezone name (`%Z`)
    ZoneName,
    /// A literal `%` character (`%%`)
    Escape,
}

fn parse_directive(directive: u8) -> Result<FormatSpecKind<'static>, Error> {
    match directive {
        b'a' => Ok(FormatSpecKind::AbbreviatedWeekday),
        b'A' => Ok(FormatSpecKind::FullWeekday),
        b'w' => Ok(FormatSpecKind::Weekday),
        b'u' => Ok(FormatSpecKind::IsoWeekday),
        b'd' => Ok(FormatSpecKind::Day),
        b'j' => Ok(FormatSpecKind::Ordinal),
        b'b' => Ok(FormatSpecKind::AbbreviatedMonth),
        b'B' => Ok(FormatSpecKind::FullMonth),
        b'm' => Ok(FormatSpecKind::Month),
        b'Y' => Ok(FormatSpecKind::Year),
        b'y' => Ok(FormatSpecKind::SignedYear),
        b'G' => Ok(FormatSpecKind::IsoWeekYear),
        b'V' => Ok(FormatSpecKind::IsoWeek),
        b'H' => Ok(FormatSpecKind::Hour),
        b'I' => Ok(FormatSpecKind::Hour12),
        b'p' => Ok(FormatSpecKind::Meridiem),
        b'M' => Ok(FormatSpecKind::Minute),
        b'S' => Ok(FormatSpecKind::Second),
        b'f' => Ok(FormatSpecKind::Nanosecond),
        b'o' => Ok(FormatSpecKind::UtcOffset),
        b'z' => Ok(FormatSpecKind::UtcOffsetBrief),
        b'Z' => Ok(FormatSpecKind::ZoneName),
        b'%' => Ok(FormatSpecKind::Escape),
        b'_' | b'#' => Err(Error::SpecifierNotFound),
        _ => Err(Error::UnknownSpecifier(directive)),
    }
}

struct FormatSpecParser<'a> {
    data: &'a [u8],
    inside_directive: bool,
}

impl<'a> FormatSpecParser<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            data: s.as_bytes(),
            inside_directive: s.as_bytes().first() == Some(&b'%'),
        }
    }
}

impl<'a> Iterator for FormatSpecParser<'a> {
    type Item = Result<FormatSpec<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inside_directive {
            self.inside_directive = false;
            match self.data {
                [b'%', b'#', directive, rest @ ..] => {
                    self.data = rest;
                    match parse_directive(*directive) {
                        Ok(kind) => Some(Ok(FormatSpec {
                            kind,
                            padding: FormatSpecPadding::Empty,
                        })),
                        Err(e) => Some(Err(e)),
                    }
                }
                [b'%', b'_', directive, rest @ ..] => {
                    self.data = rest;
                    match parse_directive(*directive) {
                        Ok(kind) => Some(Ok(FormatSpec {
                            kind,
                            padding: FormatSpecPadding::Space,
                        })),
                        Err(e) => Some(Err(e)),
                    }
                }
                [b'%', directive, rest @ ..] => {
                    self.data = rest;
                    match parse_directive(*directive) {
                        Ok(kind) => Some(Ok(FormatSpec {
                            kind,
                            padding: FormatSpecPadding::Zero,
                        })),
                        Err(e) => Some(Err(e)),
                    }
                }
                _ => Some(Err(Error::SpecifierNotFound)),
            }
        } else if self.data.is_empty() {
            None
        } else {
            match self.data.iter().position(|&c| c == b'%') {
                None => {
                    // SAFETY: The input data came from a UTF-8 encoded string
                    // Since the data at this point is either before or after a %
                    // sentinel, then the entire substrings are UTF-8
                    let raw = FormatSpec::raw(unsafe { core::str::from_utf8_unchecked(self.data) });
                    self.data = &self.data[self.data.len()..];
                    Some(Ok(raw))
                }
                Some(idx) => {
                    let (raw, rest) = self.data.split_at(idx);
                    self.inside_directive = true;
                    self.data = rest;
                    // SAFETY: See above
                    Some(Ok(FormatSpec::raw(unsafe { core::str::from_utf8_unchecked(raw) })))
                }
            }
        }
    }
}

/// Parses a format string into a [`Vec`] of [`FormatSpec`].
///
/// If a parser error occurs then [`Error`] is returned. Note that if
/// the string is known at compile time then the [`format_spec`] macro
/// should be used instead.
pub fn parse_spec(s: &str) -> Result<Vec<FormatSpec<'_>>, Error> {
    FormatSpecParser::new(s).collect()
}

/// Parses and validates format string at compile time.
#[doc(inline)]
#[cfg(feature = "macros")]
pub use eos_format_spec_macro::format_spec;

/// Helper macro to make formatting types a bit less cumbersome.
///
/// This is essentially sugar for `obj.format(format_spec!(literal))`.
///
/// ```
/// use eos::{date, fmt::{format_spec, format_dt}};
/// let date = date!(2021-01-30);
/// assert_eq!(
///     date.format(format_spec!("%Y-%m-%d")).to_string(),
///     format_dt!("%Y-%m-%d", date).to_string(),
/// );
/// ```
#[cfg(all(feature = "macros", feature = "formatting"))]
#[macro_export]
macro_rules! format_dt {
    ($fmt:literal, $dt:expr) => {
        $dt.format($crate::fmt::format_spec!($fmt))
    };
}

#[cfg(feature = "formatting")]
pub use format_dt;

/// A wrapper type that formats [`Date`] instances with the given format spec.
#[cfg(feature = "formatting")]
#[must_use]
pub struct DateFormatter<'a, 'b, Spec>
where
    Spec: AsRef<[FormatSpec<'b>]>,
{
    date: &'a Date,
    spec: Spec,
    phantom: core::marker::PhantomData<&'b Date>,
}

#[cfg(feature = "formatting")]
impl<'a, 'b, Spec> DateFormatter<'a, 'b, Spec>
where
    Spec: AsRef<[FormatSpec<'b>]>,
{
    pub(crate) fn new(date: &'a Date, spec: Spec) -> Self {
        Self {
            date,
            spec,
            phantom: core::marker::PhantomData,
        }
    }
}

/// A wrapper type that formats [`Time`] instances with the given format spec.
#[cfg(feature = "formatting")]
#[must_use]
pub struct TimeFormatter<'a, 'b, Spec>
where
    Spec: AsRef<[FormatSpec<'b>]>,
{
    time: &'a Time,
    spec: Spec,
    phantom: core::marker::PhantomData<&'b Time>,
}

#[cfg(feature = "formatting")]
impl<'a, 'b, Spec> TimeFormatter<'a, 'b, Spec>
where
    Spec: AsRef<[FormatSpec<'b>]>,
{
    pub(crate) fn new(time: &'a Time, spec: Spec) -> Self {
        Self {
            time,
            spec,
            phantom: core::marker::PhantomData,
        }
    }
}

/// A wrapper type that formats [`DateTime`] instances with the given format spec.
#[cfg(feature = "formatting")]
#[must_use]
pub struct DateTimeFormatter<'a, 'b, Tz, Spec>
where
    Spec: AsRef<[FormatSpec<'b>]>,
    Tz: TimeZone,
{
    dt: &'a DateTime<Tz>,
    spec: Spec,
    phantom: core::marker::PhantomData<&'b DateTime>,
}

#[cfg(feature = "formatting")]
impl<'a, 'b, Tz, Spec> DateTimeFormatter<'a, 'b, Tz, Spec>
where
    Spec: AsRef<[FormatSpec<'b>]>,
    Tz: TimeZone,
{
    pub(crate) fn new(dt: &'a DateTime<Tz>, spec: Spec) -> Self {
        Self {
            dt,
            spec,
            phantom: core::marker::PhantomData,
        }
    }
}

/// Formats a [`DateTime`] into [RFC 3339] format.
///
/// [RFC 3339]: https://datatracker.ietf.org/doc/html/rfc3339
#[cfg(feature = "formatting")]
#[must_use]
pub struct Rfc3339Formatter<'a, Tz>
where
    Tz: TimeZone,
{
    pub(crate) dt: &'a DateTime<Tz>,
}

#[cfg(feature = "formatting")]
fn abbreviated_weekday(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Monday => "Mon",
        Weekday::Tuesday => "Tue",
        Weekday::Wednesday => "Wed",
        Weekday::Thursday => "Thu",
        Weekday::Friday => "Fri",
        Weekday::Saturday => "Sat",
        Weekday::Sunday => "Sun",
    }
}

#[cfg(feature = "formatting")]
fn full_weekday(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Monday => "Monday",
        Weekday::Tuesday => "Tuesday",
        Weekday::Wednesday => "Wednesday",
        Weekday::Thursday => "Thursday",
        Weekday::Friday => "Friday",
        Weekday::Saturday => "Saturday",
        Weekday::Sunday => "Sunday",
    }
}

#[cfg(feature = "formatting")]
fn abbreviated_month(month: u8) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        // SAFETY: the month value is already bound checked by construction.
        _ => unsafe { core::hint::unreachable_unchecked() },
    }
}

#[cfg(feature = "formatting")]
fn full_month(month: u8) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        // SAFETY: the month value is already bound checked by construction.
        _ => unsafe { core::hint::unreachable_unchecked() },
    }
}

/// Pads a number to the specified digits given a specified padding.
#[inline]
#[cfg(feature = "formatting")]
fn pad_number<T>(
    f: &mut core::fmt::Formatter<'_>,
    number: T,
    spec: FormatSpecPadding,
    padding: usize,
) -> core::fmt::Result
where
    T: core::fmt::Display,
{
    match spec {
        FormatSpecPadding::Zero => write!(f, "{:0width$}", number, width = padding),
        FormatSpecPadding::Empty => number.fmt(f),
        FormatSpecPadding::Space => write!(f, "{:width$}", number, width = padding),
    }
}

#[cfg(feature = "formatting")]
impl<'a, 'b, Spec> core::fmt::Display for DateFormatter<'a, 'b, Spec>
where
    Spec: AsRef<[FormatSpec<'b>]>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for spec in self.spec.as_ref() {
            match spec.kind {
                FormatSpecKind::Raw(s) => f.write_str(s)?,
                FormatSpecKind::AbbreviatedWeekday => f.write_str(abbreviated_weekday(self.date.weekday()))?,
                FormatSpecKind::FullWeekday => f.write_str(full_weekday(self.date.weekday()))?,
                FormatSpecKind::Weekday => {
                    let weekday = self.date.weekday().number_from_sunday() - 1;
                    f.write_char((weekday + b'0') as char)?;
                }
                FormatSpecKind::IsoWeekday => {
                    let weekday = self.date.weekday().number_from_monday();
                    f.write_char((weekday + b'0') as char)?;
                }
                FormatSpecKind::Day => pad_number(f, self.date.day(), spec.padding, 2)?,
                FormatSpecKind::Ordinal => pad_number(f, self.date.ordinal(), spec.padding, 3)?,
                FormatSpecKind::AbbreviatedMonth => f.write_str(abbreviated_month(self.date.month()))?,
                FormatSpecKind::FullMonth => f.write_str(full_month(self.date.month()))?,
                FormatSpecKind::Month => {
                    pad_number(f, self.date.month(), spec.padding, 2)?;
                }
                FormatSpecKind::Year => {
                    // This one's a bit special since the padding depends on whether
                    // it's 4 or 5 digits
                    let year = self.date.year();
                    let padding = if year < -9999 || year > 9999 { 5 } else { 4 };
                    pad_number(f, year, spec.padding, padding)?;
                }
                FormatSpecKind::SignedYear => {
                    // This one needs to be done manually
                    let year = self.date.year();
                    match spec.padding {
                        FormatSpecPadding::Zero => {
                            let padding = if year < -9999 || year > 9999 { 6 } else { 5 };
                            write!(f, "{:+0width$}", year, width = padding)?
                        }
                        FormatSpecPadding::Empty => write!(f, "{:+}", year)?,
                        FormatSpecPadding::Space => {
                            let padding = if year < -9999 || year > 9999 { 6 } else { 5 };
                            write!(f, "{:+width$}", year, width = padding)?
                        }
                    }
                }
                FormatSpecKind::IsoWeekYear => {
                    pad_number(f, self.date.iso_week().year(), spec.padding, 4)?;
                }
                FormatSpecKind::IsoWeek => {
                    pad_number(f, self.date.iso_week().week(), spec.padding, 2)?;
                }
                FormatSpecKind::Escape => f.write_char('%')?,
                // unsupported
                _ => continue,
            }
        }
        Ok(())
    }
}

#[cfg(feature = "formatting")]
impl<'a, 'b, Spec> core::fmt::Display for TimeFormatter<'a, 'b, Spec>
where
    Spec: AsRef<[FormatSpec<'b>]>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for spec in self.spec.as_ref() {
            match spec.kind {
                FormatSpecKind::Raw(s) => f.write_str(s)?,
                FormatSpecKind::Hour => pad_number(f, self.time.hour(), spec.padding, 2)?,
                FormatSpecKind::Hour12 => {
                    let h = self.time.hour();
                    if h <= 12 {
                        pad_number(f, h, spec.padding, 2)?;
                    } else {
                        pad_number(f, h - 12, spec.padding, 2)?;
                    }
                }
                FormatSpecKind::Meridiem => {
                    if self.time.hour() >= 12 {
                        f.write_str("PM")?
                    } else {
                        f.write_str("AM")?
                    }
                }
                FormatSpecKind::Minute => pad_number(f, self.time.minute(), spec.padding, 2)?,
                FormatSpecKind::Second => {
                    let second = if self.time.nanosecond() >= 1_000_000_000 {
                        self.time.second() + 1
                    } else {
                        self.time.second()
                    };
                    pad_number(f, second, spec.padding, 2)?
                }
                FormatSpecKind::Nanosecond => {
                    let mut ns = self.time.nanosecond();
                    if ns >= 1_000_000_000 {
                        ns -= 1_000_000_000;
                    }
                    pad_number(f, ns, spec.padding, 7)?
                }
                FormatSpecKind::Escape => f.write_char('%')?,
                // Unsupported
                _ => continue,
            }
        }
        Ok(())
    }
}

#[cfg(feature = "formatting")]
impl<'a, 'b, Tz, Spec> core::fmt::Display for DateTimeFormatter<'a, 'b, Tz, Spec>
where
    Spec: AsRef<[FormatSpec<'b>]>,
    Tz: TimeZone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for spec in self.spec.as_ref() {
            match spec.kind {
                FormatSpecKind::Raw(s) => f.write_str(s)?,
                FormatSpecKind::AbbreviatedWeekday => f.write_str(abbreviated_weekday(self.dt.weekday()))?,
                FormatSpecKind::FullWeekday => f.write_str(full_weekday(self.dt.weekday()))?,
                FormatSpecKind::Weekday => {
                    let weekday = self.dt.weekday().number_from_sunday() - 1;
                    f.write_char((weekday + b'0') as char)?;
                }
                FormatSpecKind::IsoWeekday => {
                    let weekday = self.dt.weekday().number_from_monday();
                    f.write_char((weekday + b'0') as char)?;
                }
                FormatSpecKind::Day => pad_number(f, self.dt.day(), spec.padding, 2)?,
                FormatSpecKind::Ordinal => pad_number(f, self.dt.ordinal(), spec.padding, 3)?,
                FormatSpecKind::AbbreviatedMonth => f.write_str(abbreviated_month(self.dt.month()))?,
                FormatSpecKind::FullMonth => f.write_str(full_month(self.dt.month()))?,
                FormatSpecKind::Month => {
                    pad_number(f, self.dt.month(), spec.padding, 2)?;
                }
                FormatSpecKind::Year => {
                    // This one's a bit special since the padding depends on whether
                    // it's 4 or 5 digits
                    let year = self.dt.year();
                    let padding = if year < -9999 || year > 9999 { 5 } else { 4 };
                    pad_number(f, year, spec.padding, padding)?;
                }
                FormatSpecKind::SignedYear => {
                    // This one needs to be done manually
                    let year = self.dt.year();
                    match spec.padding {
                        FormatSpecPadding::Zero => {
                            let padding = if year < -9999 || year > 9999 { 6 } else { 5 };
                            write!(f, "{:+0width$}", year, width = padding)?
                        }
                        FormatSpecPadding::Empty => write!(f, "{:+}", year)?,
                        FormatSpecPadding::Space => {
                            let padding = if year < -9999 || year > 9999 { 6 } else { 5 };
                            write!(f, "{:+width$}", year, width = padding)?
                        }
                    }
                }
                FormatSpecKind::IsoWeekYear => {
                    pad_number(f, self.dt.iso_week().year(), spec.padding, 4)?;
                }
                FormatSpecKind::IsoWeek => {
                    pad_number(f, self.dt.iso_week().week(), spec.padding, 2)?;
                }
                FormatSpecKind::Hour => pad_number(f, self.dt.hour(), spec.padding, 2)?,
                FormatSpecKind::Hour12 => {
                    let h = self.dt.hour();
                    if h <= 12 {
                        pad_number(f, h, spec.padding, 2)?;
                    } else {
                        pad_number(f, h - 12, spec.padding, 2)?;
                    }
                }
                FormatSpecKind::Meridiem => {
                    if self.dt.hour() >= 12 {
                        f.write_str("PM")?
                    } else {
                        f.write_str("AM")?
                    }
                }
                FormatSpecKind::Minute => pad_number(f, self.dt.minute(), spec.padding, 2)?,
                FormatSpecKind::Second => {
                    let second = if self.dt.nanosecond() >= 1_000_000_000 {
                        self.dt.second() + 1
                    } else {
                        self.dt.second()
                    };
                    pad_number(f, second, spec.padding, 2)?
                }
                FormatSpecKind::Nanosecond => {
                    let mut ns = self.dt.nanosecond();
                    if ns >= 1_000_000_000 {
                        ns -= 1_000_000_000;
                    }
                    pad_number(f, ns, spec.padding, 7)?
                }
                FormatSpecKind::UtcOffset => self.dt.offset().fmt(f)?,
                FormatSpecKind::UtcOffsetBrief => {
                    let (hour, minute, second) = self.dt.offset().into_hms();
                    let (minute, second) = (minute.abs(), second.abs());
                    if second > 0 {
                        write!(f, "{:+03}{:02}{:02}", hour, minute, second)?
                    } else {
                        write!(f, "{:+03}{:02}", hour, minute)?
                    }
                }
                FormatSpecKind::ZoneName => {
                    if let Some(name) = self.dt.tzname() {
                        f.write_str(name)?;
                    }
                }
                FormatSpecKind::Escape => f.write_char('%')?,
            }
        }
        Ok(())
    }
}

#[cfg(feature = "formatting")]
impl<'a, Tz> core::fmt::Display for Rfc3339Formatter<'a, Tz>
where
    Tz: TimeZone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (h, m, _) = self.dt.offset().into_hms();
        let m = m.abs();
        let time = self.dt.time();
        let mut us = time.microsecond();
        let mut s = time.second();
        if us >= 1_000_000 {
            s += 1;
            us -= 1_000_000;
        }

        if us != 0 {
            write!(
                f,
                "{} {:02}:{:02}:{:02}.{:06}{:+03}:{:02}",
                self.dt.date(),
                time.hour(),
                time.minute(),
                s,
                us,
                h,
                m
            )
        } else {
            write!(
                f,
                "{} {:02}:{:02}:{:02}{:+03}:{:02}",
                self.dt.date(),
                time.hour(),
                time.minute(),
                s,
                h,
                m
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_year() {
        let mut parser = Parser::new("2012-02-13");
        assert_eq!(parser.parse_year(), Ok(2012));
        assert_eq!(parser.advance(), Some(b'-'));

        let mut parser = Parser::new("-2012-02-13");
        assert_eq!(parser.parse_year(), Ok(-2012));
        assert_eq!(parser.advance(), Some(b'-'));

        let mut parser = Parser::new("-45678-02-13");
        assert_eq!(parser.parse_year(), Err(ParseError::OutOfBounds));

        let mut parser = Parser::new("234");
        assert_eq!(parser.parse_year(), Err(ParseError::UnexpectedEnd));

        let mut parser = Parser::new("234a");
        assert_eq!(parser.parse_year(), Err(ParseError::UnexpectedNonDigit));
    }
}
