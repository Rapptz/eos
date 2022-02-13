use core::{
    cmp::Ordering,
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
    time::Duration,
};
use std::fmt::Write;

use crate::{
    utils::{divmod, divrem},
    Date, DateTime, Time, TimeZone, UtcOffset,
};

#[cfg(feature = "formatting")]
use crate::fmt::{IsoFormatPrecision, ToIsoFormat};

#[cfg(feature = "parsing")]
use crate::fmt::{FromIsoFormat, ParseError, Parser};

pub(crate) const NANOS_PER_SEC: u64 = 1_000_000_000;
pub(crate) const NANOS_PER_MIN: u64 = 60 * NANOS_PER_SEC;
pub(crate) const NANOS_PER_HOUR: u64 = 60 * NANOS_PER_MIN;

pub(crate) const MICROS_PER_SEC: i64 = 1_000_000;
pub(crate) const MICROS_PER_MIN: i64 = 60 * MICROS_PER_SEC;
pub(crate) const MICROS_PER_HOUR: i64 = 60 * MICROS_PER_MIN;

/// An interval of time such as 2 years, 30 minutes, etc.
///
/// For performance and memory reasons this only has up to microsecond precision.
///
/// Intervals are stored in whole unit months, days, and microseconds. This format
/// is not guaranteed to be stable between releases. However, this interaction makes the
/// accessors operate in a way is normalized to the nearest unit. For example, `1234` months
/// would be equivalent to 102 years and 10 months. Therefore, [`Interval::months`]
/// would return `10` rather than `1234`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Interval {
    months: i32,
    days: i32,
    microseconds: i64,
}

impl Interval {
    /// A interval that contains only zero values.
    pub const ZERO: Self = Self {
        months: 0,
        days: 0,
        microseconds: 0,
    };

    /// Creates a [`Interval`] representing the specified number of years.
    #[inline]
    #[must_use]
    pub const fn from_years(years: i16) -> Self {
        Self {
            months: years as i32 * 12,
            ..Self::ZERO
        }
    }

    /// Creates a [`Interval`] representing the specified number of days.
    #[inline]
    #[must_use]
    pub const fn from_days(days: i32) -> Self {
        Self { days, ..Self::ZERO }
    }

    /// Creates a [`Interval`] representing the specified number of months.
    #[inline]
    #[must_use]
    pub const fn from_months(months: i32) -> Self {
        Self { months, ..Self::ZERO }
    }

    /// Creates a [`Interval`] representing the specified number of weeks.
    #[inline]
    #[must_use]
    pub const fn from_weeks(weeks: i32) -> Self {
        Self {
            days: weeks * 7,
            ..Self::ZERO
        }
    }

    /// Creates a [`Interval`] representing the specified number of hours.
    #[inline]
    #[must_use]
    pub const fn from_hours(hours: i32) -> Self {
        Self {
            microseconds: hours as i64 * MICROS_PER_HOUR,
            ..Self::ZERO
        }
    }

    /// Creates a [`Interval`] representing the specified number of minutes.
    #[inline]
    #[must_use]
    pub const fn from_minutes(minutes: i32) -> Self {
        Self {
            microseconds: minutes as i64 * MICROS_PER_MIN,
            ..Self::ZERO
        }
    }

    /// Creates a [`Interval`] representing the specified number of seconds.
    #[inline]
    #[must_use]
    pub const fn from_seconds(seconds: i32) -> Self {
        Self {
            microseconds: seconds as i64 * MICROS_PER_SEC,
            ..Self::ZERO
        }
    }

    /// Creates a [`Interval`] representing the specified number of milliseconds.
    ///
    /// Note that the internal structure only stores microseconds. If the computation
    /// would end up overflowing then the value is saturated to the upper bounds.
    #[inline]
    #[must_use]
    pub const fn from_milliseconds(milliseconds: i64) -> Self {
        Self {
            microseconds: milliseconds.saturating_mul(1_000),
            ..Self::ZERO
        }
    }

    /// Creates a [`Interval`] representing the specified number of microseconds.
    ///
    /// Note that the internal structure only stores microseconds. If the computation
    /// would end up overflowing then the value is saturated to the upper bounds.
    #[inline]
    #[must_use]
    pub const fn from_microseconds(microseconds: i64) -> Self {
        Self {
            microseconds,
            ..Self::ZERO
        }
    }

    /// Returns the number of *whole* years within this interval.
    #[inline]
    #[must_use]
    pub const fn years(&self) -> i16 {
        // This could technically overflow
        (self.months / 12) as i16
    }

    /// Returns the number of *whole* days within this interval.
    ///
    /// Note that 86400 seconds does not equal a day in this interval.
    #[inline]
    #[must_use]
    pub const fn days(&self) -> i32 {
        self.days
    }

    /// Returns the number of *whole* months within this interval.
    #[inline]
    #[must_use]
    pub const fn months(&self) -> i32 {
        self.months % 12
    }

    /// Returns the number of *whole* weeks within this interval.
    #[inline]
    #[must_use]
    pub const fn weeks(&self) -> i32 {
        self.days / 7
    }

    /// Returns the number of *whole* hours within this interval.
    #[inline]
    #[must_use]
    pub const fn hours(&self) -> i64 {
        self.microseconds / MICROS_PER_HOUR
    }

    /// Returns the number of *whole* minutes within this interval.
    #[inline]
    #[must_use]
    pub const fn minutes(&self) -> i64 {
        (self.microseconds % MICROS_PER_HOUR) / MICROS_PER_MIN
    }

    /// Returns the number of *whole* seconds within this interval.
    #[inline]
    #[must_use]
    pub const fn seconds(&self) -> i64 {
        (self.microseconds % MICROS_PER_MIN) / MICROS_PER_SEC
    }

    /// Returns the number of *whole* milliseconds within this interval.
    #[inline]
    #[must_use]
    pub const fn milliseconds(&self) -> i64 {
        self.microseconds() / 1_000
    }

    /// Returns the number of *whole* microseconds within this interval.
    #[inline]
    #[must_use]
    pub const fn microseconds(&self) -> i64 {
        self.microseconds % MICROS_PER_SEC
    }

    /// Constructs an [`Interval`] between two dates.
    ///
    /// If `end` is before `start` then each property will be negative.
    ///
    /// Note that the [`Sub`] implementation is more ergonomic and idiomatic than this API.
    /// Only use this if you need to use references rather than copying.
    ///
    /// ```rust
    /// use eos::{date, Interval};
    ///
    /// let interval = Interval::between_dates(&date!(2012-03-29), &date!(2012-04-30));
    /// assert_eq!(interval.months(), 1);
    /// assert_eq!(interval.days(), 1);
    /// ```
    #[must_use]
    pub fn between_dates(start: &Date, end: &Date) -> Self {
        // Trivial case
        if start == end {
            return Self::ZERO;
        }

        let mut result = *start;
        let years = years_between(&result, end);
        result = result.add_years(years);
        let months = months_between(&result, end);
        result = result.add_months(months);
        let days = end.days_since_epoch() - result.days_since_epoch();
        Self {
            months: years as i32 * 12 + months,
            days,
            ..Self::ZERO
        }
    }

    /// Constructs an [`Interval`] between two times.
    ///
    /// If `end` is before `start` then each property will be negative.
    ///
    /// Note that the [`Sub`] implementation is more ergonomic and idiomatic than this API.
    /// Only use this if you need to use references rather than copying.
    ///
    /// ```rust
    /// use eos::{time, Interval};
    ///
    /// let interval = Interval::between_times(&time!(10:00:30), &time!(23:30:15));
    /// assert_eq!(interval.hours(), 13);
    /// assert_eq!(interval.minutes(), 29);
    /// assert_eq!(interval.seconds(), 45);
    /// ```
    #[must_use]
    pub fn between_times(start: &Time, end: &Time) -> Self {
        // Times are conceptually simple since they're bounded to at most 24 hours
        let microseconds = end.total_micros() - start.total_micros();
        Self {
            microseconds,
            ..Self::ZERO
        }
    }

    /// Constructs an [`Interval`] between two datetimes.
    ///
    /// If `end` is before `start` then each property will be negative.
    ///
    /// Note that the [`Sub`] implementation is more ergonomic and idiomatic than this API.
    /// Only use this if you need to use references rather than copying.
    ///
    /// ```rust
    /// use eos::{datetime, Interval};
    ///
    /// let start = datetime!(2012-03-10 10:00 am);
    /// let end = datetime!(2012-03-12 2:00 am);
    /// let interval = Interval::between(&start, &end);
    /// assert_eq!(interval.days(), 1);
    /// assert_eq!(interval.hours(), 16);
    ///
    /// let start = datetime!(2012-04-11 9:00 am);
    /// let end = datetime!(2014-05-12 10:00 am);
    /// let interval = Interval::between(&start, &end);
    /// assert_eq!(interval.years(), 2);
    /// assert_eq!(interval.months(), 1);
    /// assert_eq!(interval.days(), 1);
    /// assert_eq!(interval.hours(), 1);
    ///
    /// let start = datetime!(2000-02-29 10:00 am +5:00);
    /// let end = datetime!(2000-03-02 6:00 am);
    /// let interval = Interval::between(&start, &end);
    /// assert_eq!(interval.days(), 2);
    /// assert_eq!(interval.hours(), 1);
    ///
    /// let start = datetime!(2021-12-31 1:00 -5:00);
    /// let end = datetime!(2020-12-19 3:00);
    /// let interval = Interval::between(&start, &end);
    /// assert_eq!(interval.years(), -1);
    /// assert_eq!(interval.days(), -12);
    /// assert_eq!(interval.hours(), -3);
    /// ```
    #[must_use]
    pub fn between<Tz, OtherTz>(start: &DateTime<Tz>, end: &DateTime<OtherTz>) -> Self
    where
        Tz: TimeZone,
        OtherTz: TimeZone,
    {
        let cmp = start.cmp_cross_timezone(end);
        if cmp == Ordering::Equal {
            return Self::ZERO;
        }

        // We need to adjust for when overshooting might be possible.
        // For example, getting the days between March 10th 10AM and March 12th 2AM
        // would naively return 2 days when in reality it's 1 day (and 16 hours).

        // Let's assume we have 4 cases:
        // A: 2012-03-10 10am, B: 2012-03-12 2am
        // C: 2012-04-11 9am,  D: 2014-05-12 10am
        // E: 2000-02-29 10am UTC+5 F: 2000-03-02 6am UTC
        // G: 2021-12-31 1am UTC-5 H: 2020-12-19 3am UTC
        // Each of these account for different edge cases
        // For A -> B we have 1hr 16min
        // For C -> D there's 2y 1mo 1d 1h
        // For E -> F there's 2d 1hr (after considering timezones)
        // For G -> H there's -1y -12d -3h (after timezones)
        // N.B. The timezone cases get more complicated (e.g. imaginary times) but that's outside my scope right now

        // Get the number of months between the two dates
        let mut months = (end.year() as i32 - start.year() as i32) * 12 + end.month() as i32 - start.month() as i32;

        // If there are no months that are different then this is safe to skip
        if months != 0 {
            // I was tempted to optimise this to use mutable variables however when I tried it
            // there was incorrect behaviour when it came to end-of-month boundaries and leap years
            let mut offset = start.clone().add_months(months);

            let (cmp, inc) = if cmp == Ordering::Greater {
                (Ordering::Less, 1)
            } else {
                (Ordering::Greater, -1)
            };
            if offset.cmp_cross_timezone(end) == cmp {
                months += inc;
                offset = start.clone().add_months(months);
            }

            let mut delta = Self::days_between(&offset, end);
            delta.months = months;
            delta
        } else {
            Self::days_between(start, end)
        }
    }

    /// Returns the number of days and seconds between the two dates
    pub(crate) fn days_between<Tz, OtherTz>(start: &DateTime<Tz>, end: &DateTime<OtherTz>) -> Self
    where
        Tz: TimeZone,
        OtherTz: TimeZone,
    {
        let days = end.date().days_since_epoch() - start.date().days_since_epoch();
        let mut seconds = end.time().total_seconds() - start.time().total_seconds();
        let micros = end.time().microsecond() as i64 - start.time().microsecond() as i64;

        if start.offset() != end.offset() {
            seconds = seconds + start.offset().total_seconds() - end.offset().total_seconds();
        }

        // Combine the days and seconds to ensure both of them have the same signage
        let seconds = days * 86_400 + seconds;
        let (days, seconds) = divrem!(seconds, 86_400);
        Self {
            days,
            microseconds: seconds as i64 * MICROS_PER_SEC + micros,
            ..Self::ZERO
        }
    }

    #[inline]
    pub(crate) const fn total_months(&self) -> i32 {
        self.months
    }

    /// Returns a duration representing the time components of this interval.
    ///
    /// The first boolean argument is whether the time ended up being negative.
    pub(crate) fn get_time_duration(&self) -> (bool, Duration) {
        let (seconds, microseconds) = divmod!(self.microseconds, MICROS_PER_SEC);
        let nanoseconds = (microseconds as u32).saturating_mul(1_000);
        (seconds < 0, Duration::new(seconds.abs() as u64, nanoseconds))
    }
}

// Lower level algorithms to compute intervals
fn years_between(start: &Date, end: &Date) -> i16 {
    // Assume we're starting from 2019-01-30 and ending at 2021-02-14
    // First get the raw difference in years (in this example, 2)
    let diff = end.year() - start.year();
    // Check the start date at the ending year... (in this example, 2021-01-30)
    let location = start.add_years(diff);

    // If our start time is earlier then we've moved forward in time
    if start <= end {
        // In this case we need to check whether we actually landed at date
        // If we haven't overshot our date then we really are N years away
        // Otherwise we're in "year and N months" territory.
        if &location <= end {
            diff
        } else {
            diff - 1
        }
    } else {
        // This operates the same way except in the opposite direction
        if &location >= end {
            diff
        } else {
            diff + 1
        }
    }
}

fn months_between(start: &Date, end: &Date) -> i32 {
    // see years_between for an explanation
    // this is the same except we also subtract years and they're 12 months each
    let diff = (end.year() as i32 - start.year() as i32) * 12 + end.month() as i32 - start.month() as i32;
    let location = start.add_months(diff);

    if start <= end {
        if &location <= end {
            diff
        } else {
            diff - 1
        }
    } else if &location >= end {
        diff
    } else {
        diff + 1
    }
}

impl From<UtcOffset> for Interval {
    fn from(offset: UtcOffset) -> Self {
        Self::from_seconds(offset.total_seconds())
    }
}

impl Neg for Interval {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            months: -self.months,
            days: -self.days,
            microseconds: -self.microseconds,
        }
    }
}

impl Add for Interval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            months: self.months + rhs.months,
            days: self.days + rhs.days,
            microseconds: self.microseconds + rhs.microseconds,
        }
    }
}

impl AddAssign for Interval {
    fn add_assign(&mut self, rhs: Self) {
        self.months += rhs.months;
        self.days += rhs.days;
        self.microseconds += rhs.microseconds;
    }
}

impl Sub for Interval {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            months: self.months - rhs.months,
            days: self.days - rhs.days,
            microseconds: self.microseconds - rhs.microseconds,
        }
    }
}

impl SubAssign for Interval {
    fn sub_assign(&mut self, rhs: Self) {
        self.months -= rhs.months;
        self.days -= rhs.days;
        self.microseconds -= rhs.microseconds;
    }
}

impl TryFrom<Duration> for Interval {
    type Error = crate::Error;

    /// Attempts to convert a [`Duration`] into an [`Interval`].
    ///
    /// Since an interval stores rich relative information, it cannot assume
    /// that a month is a set number of days since the number of days vary between
    /// 28 to 31.
    ///
    /// If the number of seconds in this duration cannot fit the in the interval then
    /// an [`crate::Error`] is returned.
    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        let (days, seconds) = divmod!(value.as_secs(), 86400);
        let days = i32::try_from(days).map_err(|_| crate::Error::OutOfRange)?;
        let micros = seconds
            .saturating_mul(MICROS_PER_SEC as u64)
            .saturating_add(value.subsec_micros() as u64);
        let micros = i64::try_from(micros).map_err(|_| crate::Error::OutOfRange)?;
        Ok(Self {
            months: 0,
            days,
            microseconds: micros,
        })
    }
}

impl Add<Duration> for Interval {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        self + Self::try_from(rhs).expect("duration overflowed")
    }
}

impl Sub<Duration> for Interval {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        self - Self::try_from(rhs).expect("duration overflowed")
    }
}

impl Add<Date> for Interval {
    type Output = Date;

    fn add(self, rhs: Date) -> Self::Output {
        rhs + self
    }
}

impl Add<Time> for Interval {
    type Output = Time;

    fn add(self, rhs: Time) -> Self::Output {
        rhs + self
    }
}

impl Add<DateTime> for Interval {
    type Output = DateTime;

    fn add(self, rhs: DateTime) -> Self::Output {
        rhs + self
    }
}

impl core::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self == &Self::ZERO {
            return f.write_str("PT0S");
        }
        f.write_char('P')?;
        if self.years() != 0 {
            write!(f, "{}Y", self.years())?;
        }

        if self.months() != 0 {
            write!(f, "{}M", self.months())?;
        }

        if self.days != 0 {
            write!(f, "{}D", self.days)?;
        }

        if self.microseconds != 0 {
            f.write_char('T')?;
        }

        if self.hours() != 0 {
            write!(f, "{}H", self.hours())?;
        }

        if self.minutes() != 0 {
            write!(f, "{}M", self.minutes())?;
        }

        let microseconds = self.microseconds();
        if microseconds == 0 {
            if self.seconds() != 0 {
                write!(f, "{}S", self.seconds())?;
            }
        } else {
            let as_frac = (self.microseconds % MICROS_PER_MIN) as f64 / MICROS_PER_SEC as f64;
            if as_frac != 0.0 {
                write!(f, "{}S", as_frac)?
            }
        }

        Ok(())
    }
}

#[cfg(feature = "formatting")]
impl ToIsoFormat for Interval {
    fn to_iso_format_with_precision(&self, _precision: IsoFormatPrecision) -> String {
        self.to_string()
    }

    /// Converts to an ISO-8601 format string such as `PnYnMnDTnHnMnS` where `Y`, `M`, `D`,`H`
    /// and `S` represent units. Unlike the ISO-8601 format, this outputs negative values if the
    /// corresponding unit is negative.
    fn to_iso_format(&self) -> String {
        self.to_string()
    }
}

#[cfg(feature = "parsing")]
#[derive(Copy, Clone, Default)]
struct ParseState {
    years: i16,
    months: i32,
    days: i32,
    hours: i32,
    minutes: i32,
    seconds: i32,
    microseconds: i32,
}

#[cfg(feature = "parsing")]
impl ParseState {
    fn to_micros(self) -> Option<i64> {
        let hours = (self.hours as i64).checked_mul(MICROS_PER_HOUR)?;
        let minutes = (self.minutes as i64).checked_mul(MICROS_PER_MIN)?;
        let seconds = (self.seconds as i64).checked_mul(MICROS_PER_SEC)?;
        hours
            .checked_add(minutes)?
            .checked_add(seconds)?
            .checked_add(self.microseconds as i64)
    }
}

#[cfg(feature = "parsing")]
impl FromIsoFormat for Interval {
    /// Parses an ISO-8601 formatted string to an [`Interval`].
    ///
    /// The syntax accepted by this function deviated from the actual ISO-8601 standard
    /// since it accepts negative numbers. The base syntax accepted is something similar
    /// to `PnYnMnDTnHnMn.nS`.
    ///
    /// The string can start with an optional sign, denoted by the ASCII negative or positive symbol.
    /// If negative, the whole period is negated. The accepted units are `Y`, `M`, `D`, `H`, and `S`.
    /// They must be in uppercase. Up to 9 digits of precision are supported by all units except years,
    /// which must be up to 5 digits. Note that fractions are only supported in the seconds position
    /// and only up to 6 digits of precision are supported.
    ///
    /// Some example strings:
    ///
    /// - `PT15M` (15 minutes)
    /// - `PT20.5S` (20.5 seconds)
    /// - `P10Y2M3DT10S` (10 years, 2 months, 3 days, and 10 seconds).
    /// - `-P30D` (-30 days)
    /// - `P-30D` (-30 days)
    /// - `-P-30DT30S` (30 days and -30 seconds).
    ///
    fn from_iso_format(s: &str) -> Result<Self, ParseError> {
        let mut parser = Parser::new(s);
        let negative = parser.parse_sign();
        parser.expect(b'P')?;
        let mut time_units = parser.advance_if_equal(b'T').is_some();
        let mut parsed_once = false;
        let mut result = ParseState::default();

        // This parser technically accepts repeated units when it shouldn't be possible
        // e.g. P10M30M
        // This is a defect but it makes the parser "simpler". Hopefully in the future
        // these can be fixed.

        loop {
            match parser.peek() {
                Some(b'T') => {
                    if time_units {
                        return Err(ParseError::UnexpectedNonDigit);
                    }
                    time_units = true;
                    parser.advance();
                }
                None => {
                    if parsed_once {
                        break;
                    } else {
                        return Err(ParseError::UnexpectedEnd);
                    }
                }
                _ => {}
            }

            let value = parser.parse_i32()?;
            match parser.advance() {
                Some(b'Y') => {
                    if time_units {
                        return Err(ParseError::UnexpectedChar('Y'));
                    }
                    result.years = i16::try_from(value)?;
                }
                Some(b'M') => {
                    if time_units {
                        result.minutes = value;
                    } else {
                        result.months = value;
                    }
                }
                Some(b'D') => {
                    if time_units {
                        return Err(ParseError::UnexpectedChar('D'));
                    }
                    result.days = value;
                }
                Some(b'H') => {
                    if !time_units {
                        return Err(ParseError::UnexpectedChar('H'));
                    }
                    result.hours = value;
                }
                Some(b'S') => {
                    if !time_units {
                        return Err(ParseError::UnexpectedChar('S'));
                    }
                    result.seconds = value;
                }
                Some(b'.') => {
                    if !time_units {
                        return Err(ParseError::UnexpectedChar('.'));
                    }

                    let mut micros = i32::try_from(parser.parse_microseconds()?)?;
                    parser.expect(b'S')?;

                    // Expect end of string
                    if let Some(c) = parser.advance() {
                        return Err(ParseError::UnexpectedChar(c as char));
                    }

                    if value < 0 {
                        micros = -micros;
                    }
                    result.microseconds = micros;
                    result.seconds = value;
                    break;
                }
                Some(b) => return Err(ParseError::UnexpectedChar(b as char)),
                None => return Err(ParseError::UnexpectedEnd),
            }
            parsed_once = true;
        }

        let months = (result.years as i32 * 12)
            .checked_add(result.months)
            .ok_or(ParseError::OutOfBounds)?;
        let days = result.days;
        let microseconds = result.to_micros().ok_or(ParseError::OutOfBounds)?;
        Ok(if negative {
            Self {
                months: -months,
                days: -days,
                microseconds: -microseconds,
            }
        } else {
            Self {
                months,
                days,
                microseconds,
            }
        })
    }
}

#[cfg(feature = "parsing")]
impl FromIsoFormat for core::time::Duration {
    /// Parses an ISO-8601 formatted string to an [`std::time::Duration`].
    ///
    /// The base syntax accepted is something similar to `PTnHnMn.nS`.
    /// Unlike the syntax accepted by [`Interval`] this does not accept
    /// negative numbers since [`std::time::Duration`] does not allow negative
    /// durations. Likewise, since duration deals only in time units then the corresponding
    /// date units such as day, month, and year are unsupported.
    ///
    /// The accepted units are `H`, `M`, and `S`. They must be in uppercase. Up to 9 digits of precision
    /// are supported by all units. Note that fractions are only supported in the seconds position.
    ///
    /// Some example strings:
    ///
    /// - `PT15M` (15 minutes)
    /// - `PT20.5S` (20.5 seconds)
    /// - `PT10H` (10 hours)
    /// - `PT6H30M20.5S` (6 hours, 30 minutes, 20.5 seconds)
    ///
    fn from_iso_format(s: &str) -> Result<Self, ParseError> {
        let mut parser = Parser::new(s);
        parser.expect(b'P')?;
        parser.expect(b'T')?;
        let mut total_seconds = 0;
        let mut nanoseconds = 0;
        let mut parsed_units = [false, false, false];

        loop {
            if parser.peek().is_none() {
                if parsed_units.iter().any(|f| *f) {
                    break;
                } else {
                    return Err(ParseError::UnexpectedEnd);
                }
            }
            let value = parser.parse_u32()?;
            match parser.advance() {
                Some(b'M') => {
                    if parsed_units[1] {
                        return Err(ParseError::UnexpectedChar('M'));
                    }
                    total_seconds += value as u64 * 60;
                    parsed_units[1] = true;
                }
                Some(b'S') => {
                    if parsed_units[2] {
                        return Err(ParseError::UnexpectedChar('S'));
                    }
                    total_seconds += value as u64;
                    parsed_units[2] = true;
                }
                Some(b'H') => {
                    if parsed_units[0] {
                        return Err(ParseError::UnexpectedChar('H'));
                    }
                    total_seconds += value as u64 * 3600;
                    parsed_units[0] = true;
                }
                Some(b'.') => {
                    nanoseconds = parser.parse_nanoseconds()?;
                    parser.expect(b'S')?;
                    if let Some(c) = parser.advance() {
                        return Err(ParseError::UnexpectedChar(c as char));
                    }
                    total_seconds += value as u64;
                    break;
                }
                Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
                None => return Err(ParseError::UnexpectedEnd),
            }
        }

        Ok(core::time::Duration::new(total_seconds, nanoseconds))
    }
}
