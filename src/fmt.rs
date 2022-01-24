//! Utilities for formatting and parsing various types in the library.
//!
//! # Usage
//!
//! Formatting is done through functions such as [`crate::DateTime::format`]. These
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
//! # Format Specifiers
//!
//! The format specifiers in this library were mainly modelled after [strftime] but
//! with certain formats either being added, changed, or removed due to being incompatible
//! with the library or legacy reasons. For example, the libc `%y` and `%C` don't make
//! sense since the range of data used in this library are larger than the ones in `<time.h>`.
//!
//! | Specifier | Meaning                                                  | Example                          |
//! |:---------:|:---------------------------------------------------------|:---------------------------------|
//! |   `%a`    | Abbreviated weekday name                                 | Sun, Mon, ..., Sat               |
//! |   `%A`    | Full weekday name                                        | Sunday, Monday, ..., Saturday    |
//! |   `%w`    | Weekday as a number where 0 is Sunday and 6 is Saturday. | 0, 1, ... 6                      |
//! |   `%u`    | Weekday as a number where 1 is Monday and 7 is Saturday  | 1, 2, ... 7                      |
//! |   `%d`    | Day of the month as a zero-padded number                 | 01, 02, ..., 31                  |
//! |   `%j`    | Ordinal day of the year as a zero-padded number          | 001, 002, ..., 365               |
//! |   `%b`    | Abbreviated month name                                   | Jan, Feb, ..., Dec               |
//! |   `%B`    | Full month name                                          | January, February, ..., December |
//! |   `%m`    | Month as a zero-padded number                            | 01, 02, ..., 12                  |
//! |   `%Y`    | Year as a zero-padded number                             | 0001, 0002, ..., 32767           |
//! |   `%y`    | Same as `%Y` but with explicit sign                      | -0001, 0000, ..., +32767         |
//! |   `%G`    | ISO 8601 week calendar year as a zero-padded number      | 0001, 0002, ..., 32767           |
//! |   `%V`    | ISO 8601 week as a zero-padded number                    | 01, 02, ..., 53                  |
//! |   `%H`    | Hour (24-hour clock) as a zero-padded number             | 00, 01, ..., 23                  |
//! |   `%I`    | Hour (12-hour clock) as a zero-padded number             | 01, 02, ..., 12                  |
//! |   `%p`    | The time meridian (am or pm)                             | AM, PM                           |
//! |   `%M`    | Minute as a zero-padded number                           | 00, 01, ..., 59                  |
//! |   `%S`    | Second as a zero-padded number                           | 00, 01, ..., 59                  |
//! |   `%f`    | Nanoseconds as a zero-padded number                      | 0000000, 0000001, ..., 2000000   |
//! |   `%z`    | UTC offset as `±HHMM[SS]` or empty                       | +0000, -0500, +102340, ...       |
//! |   `%o`    | UTC offset as `±HH:MM[:SS]` or empty                     | +00:00, -05:00, +10:23:40, ...   |
//! |   `%Z`    | Timezone name or empty                                   | UTC, EST, ...                    |
//! |   `%%`    | The literal `%` character.                               | %                                |
//!
//! ## Modifiers
//!
//! Directives that are zero-padded support so called modifiers that help modify the formatting behavior.
//! These help change the formatting from zero-padding to either space or no padding. They *must* follow the `%` sign.
//!
//! | Modifier | Meaning                                  | Example                       |
//! |:--------:|:-----------------------------------------|:------------------------------|
//! |   `#`    | Use no padding at all                    | `%#d` outputs 1, 2, ..., 31   |
//! |   `_`    | Use spaces for padding instead of zeroes | `%#d` outputs ` 1`, ` 2`, ... |
//!
//!
//! [strftime]: https://en.cppreference.com/w/cpp/chrono/c/strftime

use crate::{Date, DateTime, Time, TimeZone};
use core::fmt::Write;

/// Represents how a fragment should be formatted.
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
}

/// Represents an error that occurred during parsing in [`parse_spec`].
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

/// Represents the kind of fragment in a format specification.
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
    /// The time meridian (`%p`)
    Meridian,
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
        b'p' => Ok(FormatSpecKind::Meridian),
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

#[cfg(feature = "formatting")]
fn abbreviated_weekday(weekday: crate::Weekday) -> &'static str {
    match weekday {
        crate::Weekday::Monday => "Mon",
        crate::Weekday::Tuesday => "Tue",
        crate::Weekday::Wednesday => "Wed",
        crate::Weekday::Thursday => "Thu",
        crate::Weekday::Friday => "Fri",
        crate::Weekday::Saturday => "Sat",
        crate::Weekday::Sunday => "Sun",
    }
}

#[cfg(feature = "formatting")]
fn full_weekday(weekday: crate::Weekday) -> &'static str {
    match weekday {
        crate::Weekday::Monday => "Monday",
        crate::Weekday::Tuesday => "Tuesday",
        crate::Weekday::Wednesday => "Wednesday",
        crate::Weekday::Thursday => "Thursday",
        crate::Weekday::Friday => "Friday",
        crate::Weekday::Saturday => "Saturday",
        crate::Weekday::Sunday => "Sunday",
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
                FormatSpecKind::Meridian => {
                    if self.time.hour() >= 12 {
                        f.write_str("PM")?
                    } else {
                        f.write_str("AM")?
                    }
                }
                FormatSpecKind::Minute => pad_number(f, self.time.minute(), spec.padding, 2)?,
                FormatSpecKind::Second => pad_number(f, self.time.second(), spec.padding, 2)?,
                FormatSpecKind::Nanosecond => pad_number(f, self.time.nanosecond(), spec.padding, 7)?,
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
                FormatSpecKind::Meridian => {
                    if self.dt.hour() >= 12 {
                        f.write_str("PM")?
                    } else {
                        f.write_str("AM")?
                    }
                }
                FormatSpecKind::Minute => pad_number(f, self.dt.minute(), spec.padding, 2)?,
                FormatSpecKind::Second => pad_number(f, self.dt.second(), spec.padding, 2)?,
                FormatSpecKind::Nanosecond => pad_number(f, self.dt.nanosecond(), spec.padding, 7)?,
                FormatSpecKind::UtcOffset => {
                    let offset = self.dt.timezone().offset(self.dt.date(), self.dt.time());
                    offset.fmt(f)?
                }
                FormatSpecKind::UtcOffsetBrief => {
                    let offset = self.dt.timezone.offset(self.dt.date(), self.dt.time());
                    let (hour, minute, second) = offset.into_hms();
                    let (minute, second) = (minute.abs(), second.abs());
                    if second > 0 {
                        write!(f, "{:+03}{:02}{:02}", hour, minute, second)?
                    } else {
                        write!(f, "{:+03}{:02}", hour, minute)?
                    }
                }
                FormatSpecKind::ZoneName => {
                    if let Some(name) = self.dt.timezone().name(self.dt.date(), self.dt.time()) {
                        f.write_str(name.as_str())?;
                    }
                }
                FormatSpecKind::Escape => f.write_char('%')?,
            }
        }
        Ok(())
    }
}
