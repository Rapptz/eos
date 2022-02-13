use crate::{
    interval::{MICROS_PER_HOUR, MICROS_PER_MIN, MICROS_PER_SEC, NANOS_PER_HOUR, NANOS_PER_MIN, NANOS_PER_SEC},
    step::Advance,
    utils::{divmod, ensure_in_range},
    Date, DateTime, Interval, Utc,
};

use core::{
    ops::{Add, Sub},
    time::Duration,
};

#[cfg(feature = "formatting")]
use crate::fmt::{IsoFormatPrecision, ToIsoFormat};

#[cfg(feature = "parsing")]
use crate::fmt::{FromIsoFormat, ParseError, Parser};

/// A moment in time.
///
/// This type is not aware of any particular calendar, date, or time zone. The maximum
/// supported precision is nanoseconds. Comparisons assume they're on the same calendar date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time {
    pub(crate) hour: u8,
    pub(crate) minute: u8,
    pub(crate) second: u8,
    pub(crate) nanosecond: u32,
}

const MAXIMUM_SECONDS_FROM_DURATION: u64 = i32::MAX as u64 * 24 * 60 * 60;

impl Time {
    /// The minimum valid time.
    pub const MIN: Self = Self {
        hour: 0,
        minute: 0,
        second: 0,
        nanosecond: 0,
    };

    /// The time at midnight.
    pub const MIDNIGHT: Self = Self::MIN;

    /// The maximum valid time.
    ///
    /// This does not include leap seconds.
    pub const MAX: Self = Self {
        hour: 23,
        minute: 59,
        second: 59,
        nanosecond: 999_999_999,
    };

    #[doc(hidden)]
    #[cfg(feature = "macros")]
    #[inline]
    pub const fn __new_unchecked_from_macro(hour: u8, minute: u8, second: u8) -> Self {
        Self {
            hour,
            minute,
            second,
            nanosecond: 0,
        }
    }

    /// Creates a new [`Time`] from the specified hour, minute, and second.
    ///
    /// The `hour` value must be between `0..24` and the `minute` and `second` values must
    /// be between `0..60`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use eos::Time;
    /// # fn test() -> Option<()> {
    /// let time = Time::new(23, 10, 0)?;
    ///
    /// assert_eq!(time.hour(), 23);
    /// assert_eq!(time.minute(), 10);
    /// assert_eq!(time.second(), 0);
    /// assert!(Time::new(10, 0, 0).is_some());
    /// assert!(Time::new(24, 0, 0).is_none());
    /// assert!(Time::new(23, 60, 0).is_none());
    /// assert!(Time::new(23, 59, 60).is_none());
    /// # Some(())
    /// # }
    /// # test();
    /// ```
    #[inline]
    pub const fn new(hour: u8, minute: u8, second: u8) -> Option<Self> {
        ensure_in_range!(hour, 23);
        ensure_in_range!(minute, 59);
        ensure_in_range!(second, 59);
        Some(Self {
            nanosecond: 0,
            hour,
            minute,
            second,
        })
    }

    /// Combines this [`Time`] with a [`Date`] to create a [`DateTime`] in [`Utc`].
    #[inline]
    #[must_use]
    pub const fn at(&self, date: Date) -> DateTime<Utc> {
        DateTime {
            date,
            time: *self,
            offset: crate::UtcOffset::UTC,
            timezone: Utc,
        }
    }

    /// Formats this time with a given slice of [`crate::fmt::FormatSpec`].
    ///
    /// Check the [`crate::fmt`] module for more documentation.
    #[cfg(feature = "formatting")]
    pub fn format<'a, 'b, S>(&'a self, spec: S) -> crate::fmt::TimeFormatter<'a, 'b, S>
    where
        S: AsRef<[crate::fmt::FormatSpec<'b>]>,
    {
        crate::fmt::TimeFormatter::new(self, spec)
    }

    /// Returns a [`Time`] moved to the next step of the given value.
    ///
    /// # Examples
    ///
    /// Getting the next unit:
    ///
    /// ```rust
    /// use eos::{time, unit};
    ///
    /// assert_eq!(time!(02:00).next(unit::Hour), time!(03:00));
    /// assert_eq!(time!(02:00).next(unit::Minute), time!(02:01));
    /// assert_eq!(time!(02:00).next(unit::Second), time!(02:00:01));
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn next<A>(self, advance: A) -> Self
    where
        A: Advance<Self>,
    {
        advance.next_from(self)
    }

    /// Returns a [`Time`] moved to the previous step of the given value.
    ///
    /// # Examples
    ///
    /// Getting the previous unit:
    ///
    /// ```rust
    /// use eos::{time, unit};
    ///
    /// assert_eq!(time!(02:00).prev(unit::Hour), time!(01:00));
    /// assert_eq!(time!(02:00).prev(unit::Minute), time!(01:59));
    /// assert_eq!(time!(02:00).prev(unit::Second), time!(01:59:59));
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn prev<A>(self, advance: A) -> Self
    where
        A: Advance<Self>,
    {
        advance.prev_from(self)
    }

    #[inline]
    pub(crate) fn total_seconds(&self) -> i32 {
        self.hour as i32 * 3600 + self.minute as i32 * 60 + self.second as i32
    }

    /// Total number of nanoseconds represented by this time.
    ///
    /// The valid range for this type is [0, 86,401,000,000,000]
    pub(crate) fn total_nanos(&self) -> u64 {
        self.hour as u64 * NANOS_PER_HOUR
            + self.minute as u64 * NANOS_PER_MIN
            + self.second as u64 * NANOS_PER_SEC
            + self.nanosecond as u64
    }

    pub(crate) fn total_micros(&self) -> i64 {
        self.hour as i64 * MICROS_PER_HOUR
            + self.minute as i64 * MICROS_PER_MIN
            + self.second as i64 * MICROS_PER_SEC
            + self.nanosecond as i64 / 1_000
    }

    /// Converts nanoseconds into a date representation and returns the left-over days.
    pub(crate) fn adjust_from_nanos(nanos: i64) -> (i32, Self) {
        // Arithmetic can be done entirely using nanoseconds
        // Duration is comprised of a u64 seconds + u32 nanosecond.
        // The u32 nanosecond doesn't go over NANOS_PER_SEC which means that
        // Duration can only represent 2^64 seconds which is around 213.5 trillion
        // days, a bit over the limit of an i32. There's a constant that allows us
        // to set the maximum number of seconds we can represent in a duration.
        //
        // With that maximum in place, the highest a nanosecond value will be
        // boils down to 3.6 trillion, which fits perfectly fine in a 64-bit number.
        // When it goes through the reduction steps it'll cap at around 1 billion.
        let (hour, nanos) = divmod!(nanos, NANOS_PER_HOUR as i64);
        let (minute, nanos) = divmod!(nanos, NANOS_PER_MIN as i64);
        let (second, nanos) = divmod!(nanos, NANOS_PER_SEC as i64);
        let (days, hour) = divmod!(hour, 24);

        (
            days as i32,
            Self {
                hour: hour as u8,
                minute: minute as u8,
                second: second as u8,
                nanosecond: nanos as u32,
            },
        )
    }

    /// Adds the time with the given duration and returns the number of days that have passed.
    pub(crate) fn add_with_duration(self, duration: Duration) -> (i32, Self) {
        if duration.as_secs() > MAXIMUM_SECONDS_FROM_DURATION {
            (i32::MAX, self)
        } else {
            let diff = self.total_nanos() as i64 + duration.as_nanos() as i64;
            Self::adjust_from_nanos(diff)
        }
    }

    /// Subtracts the time with the given duration and returns the number of days that have passed.
    pub(crate) fn sub_with_duration(self, duration: Duration) -> (i32, Self) {
        if duration.as_secs() > MAXIMUM_SECONDS_FROM_DURATION {
            (i32::MIN + 1, self)
        } else {
            let diff = self.total_nanos() as i64 - duration.as_nanos() as i64;
            Self::adjust_from_nanos(diff)
        }
    }

    // The "common" functions begin here.
    // I want to "unroll" the trait and make them inherent methods since their discoverability
    // is better in the documentation, and the trait usability is mostly subpar.
    // This is done both in Time and Date.

    /// Returns the hour.
    ///
    /// This value will always be within `0..24`.
    #[inline]
    #[must_use]
    pub const fn hour(&self) -> u8 {
        self.hour
    }

    /// Returns the minute within the hour.
    ///
    /// This value will always be within `0..60`.
    #[inline]
    #[must_use]
    pub const fn minute(&self) -> u8 {
        self.minute
    }

    /// Returns the second within the minute.
    ///
    /// This value will always be within `0..60`.
    #[inline]
    #[must_use]
    pub const fn second(&self) -> u8 {
        self.second
    }

    /// Returns the millisecond within the second.
    ///
    /// This value will always be within `0..1000`.
    #[inline]
    #[must_use]
    pub const fn millisecond(&self) -> u16 {
        (self.nanosecond / 1_000_000) as u16
    }

    /// Returns the microsecond within the second.
    ///
    /// This value will always be within `0..1_000_000`.
    #[inline]
    #[must_use]
    pub const fn microsecond(&self) -> u32 {
        self.nanosecond / 1_000
    }

    /// Returns the nanosecond within the second.
    ///
    /// This value will always be within `0..2_000_000_000`.
    #[inline]
    #[must_use]
    pub const fn nanosecond(&self) -> u32 {
        self.nanosecond
    }

    /// Returns a new [`Time`] that points to the given hour.
    /// If the hour is out of bounds (`0..24`) then [`None`] is returned.
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn with_hour(mut self, hour: u8) -> Option<Self> {
        ensure_in_range!(hour, 24);
        self.hour = hour;
        Some(self)
    }

    /// Returns a new [`Time`] that points to the given minute.
    /// If the minute is out of bounds (`0..60`) then [`None`] is returned.
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn with_minute(mut self, minute: u8) -> Option<Self> {
        ensure_in_range!(minute, 59);
        self.minute = minute;
        Some(self)
    }

    /// Returns a new [`Time`] that points to the given second.
    /// If the second is out of bounds (`0..60`) then [`None`] is returned.
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn with_second(mut self, second: u8) -> Option<Self> {
        ensure_in_range!(second, 59);
        self.second = second;
        Some(self)
    }

    /// Returns a new [`Time`] that points to the given millisecond.
    /// If the millisecond is out of bounds (`0..1000`) then [`None`] is returned.
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn with_millisecond(mut self, millisecond: u16) -> Option<Self> {
        ensure_in_range!(millisecond, 1999);
        self.nanosecond = millisecond as u32 * 1_000_000;
        Some(self)
    }

    /// Returns a new [`Time`] that points to the given microsecond.
    /// If the microsecond is out of bounds (`0..1_000_000`) then [`None`] is returned.
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn with_microsecond(mut self, microsecond: u32) -> Option<Self> {
        ensure_in_range!(microsecond, 1_999_999);
        self.nanosecond = microsecond * 1_000;
        Some(self)
    }

    /// Returns a new [`Time`] that points to the given nanosecond.
    /// If the nanosecond is out of bounds (`0..2_000_000_000`) then [`None`] is returned.
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn with_nanosecond(mut self, nanosecond: u32) -> Option<Self> {
        ensure_in_range!(nanosecond, 1_999_999_999);
        self.nanosecond = nanosecond;
        Some(self)
    }
}

impl Add<Interval> for Time {
    type Output = Time;

    fn add(self, rhs: Interval) -> Self::Output {
        let (sub, duration) = rhs.get_time_duration();
        let (_, ret) = if sub {
            self.sub_with_duration(duration)
        } else {
            self.add_with_duration(duration)
        };
        ret
    }
}

impl Sub<Interval> for Time {
    type Output = Time;

    fn sub(self, rhs: Interval) -> Self::Output {
        let (sub, duration) = rhs.get_time_duration();
        let (_, ret) = if sub {
            self.add_with_duration(duration)
        } else {
            self.sub_with_duration(duration)
        };
        ret
    }
}

impl Add<Duration> for Time {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        self.add_with_duration(rhs).1
    }
}

impl Sub<Duration> for Time {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        self.sub_with_duration(rhs).1
    }
}

impl Sub for Time {
    type Output = Interval;

    fn sub(self, rhs: Self) -> Self::Output {
        Interval::between_times(&rhs, &self)
    }
}

#[cfg(feature = "formatting")]
pub(crate) fn fmt_iso_time<W>(f: &mut W, t: &Time, precision: IsoFormatPrecision) -> core::fmt::Result
where
    W: core::fmt::Write,
{
    match precision {
        IsoFormatPrecision::Hour => write!(f, "{:02}:00", t.hour),
        IsoFormatPrecision::Minute => write!(f, "{:02}:{:02}", t.hour, t.minute),
        IsoFormatPrecision::Second => write!(f, "{:02}:{:02}:{:02}", t.hour, t.minute, t.second),
        IsoFormatPrecision::Millisecond => {
            let ms = t.millisecond();
            write!(f, "{:02}:{:02}:{:02}.{:03}", t.hour, t.minute, t.second, ms)
        }
        IsoFormatPrecision::Microsecond => {
            let ms = t.microsecond();
            write!(f, "{:02}:{:02}:{:02}.{:06}", t.hour, t.minute, t.second, ms)
        }
        IsoFormatPrecision::Nanosecond => {
            write!(f, "{:02}:{:02}:{:02}.{:07}", t.hour, t.minute, t.second, t.nanosecond)
        }
    }
}

impl core::fmt::Display for Time {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.nanosecond != 0 {
            write!(
                f,
                "{:02}:{:02}:{:02}.{:07}",
                self.hour, self.minute, self.second, self.nanosecond
            )
        } else {
            write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
        }
    }
}

#[cfg(feature = "formatting")]
impl ToIsoFormat for Time {
    fn to_iso_format_with_precision(&self, precision: IsoFormatPrecision) -> String {
        let mut buffer = String::with_capacity(16);
        fmt_iso_time(&mut buffer, self, precision).unwrap();
        buffer
    }

    fn to_iso_format(&self) -> String {
        if self.nanosecond != 0 {
            self.to_iso_format_with_precision(IsoFormatPrecision::Microsecond)
        } else {
            self.to_iso_format_with_precision(IsoFormatPrecision::Second)
        }
    }
}

#[cfg(feature = "parsing")]
impl FromIsoFormat for Time {
    /// Parse an ISO-8601 formatted string to a [`Time`].
    ///
    /// The syntax accepted by this function are:
    ///
    /// - `HH:MM` (e.g. `10:23`)
    /// - `HH:MM:SS` (e.g. `10:24:30`)
    /// - `HH:MM:SS.sssssssss`, up to 9 digits of precision (e.g. `10:24:30.999999999)
    /// - `HH:MM:SS,sssssssss`, similar to above except with `,` instead of `.`
    ///
    /// Notably, formats *without* the colon are not allowed despite being part of the
    /// ISO-8601 standard.
    fn from_iso_format(s: &str) -> Result<Self, ParseError> {
        let mut parser = Parser::new(s);
        parser.parse_time()
    }
}
