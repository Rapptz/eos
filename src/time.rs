use crate::{
    utils::{divmod, ensure_in_range, divrem},
    Error, Interval,
};

use core::{
    ops::{Add, Sub},
    time::Duration,
};

/// Represents a moment in time. This type is not aware of any particular calendar, date, or time zone.
///
/// This type has nanosecond precision. Comparisons assume they're on the same calendar date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time {
    hour: u8,
    minute: u8,
    second: u8,
    nanosecond: u32,
}

const NANOS_PER_SEC: u64 = 1_000_000_000;
const NANOS_PER_MIN: u64 = 60 * NANOS_PER_SEC;
const NANOS_PER_HOUR: u64 = 60 * NANOS_PER_MIN;
const MAXIMUM_SECONDS_FROM_DURATION: u64 = i32::MAX as u64 * 24 * 60 * 60;

impl Time {
    /// Represets the minimum time.
    pub const MIN: Self = Self {
        hour: 0,
        minute: 0,
        second: 0,
        nanosecond: 0,
    };

    /// Represents the time at midnight.
    pub const MIDNIGHT: Self = Self::MIN;

    /// Represents the maximum time.
    ///
    /// This does not include leap seconds.
    pub const MAX: Self = Self {
        hour: 23,
        minute: 59,
        second: 59,
        nanosecond: 999_999_999,
    };

    /// Creates a new [`Time`] from the specified hour, minute, and second.
    ///
    /// The `hour` value must be between `0..24` and the `minute` and `second` values must
    /// be between `0..60`.
    ///
    /// # Panics
    ///
    /// Panics if the values are out of range. If this is undesirable, consider
    /// using [`Time::try_new`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use eos::Time;
    /// let time = Time::new(23, 10, 0);
    ///
    /// assert_eq!(time.hour(), 23);
    /// assert_eq!(time.minute(), 10);
    /// assert_eq!(time.second(), 0);
    /// ```
    pub fn new(hour: u8, minute: u8, second: u8) -> Self {
        Self::try_new(hour, minute, second).expect("input of out range")
    }

    /// Creates a new [`Time`] from the specified hour, minute, and second.
    ///
    /// This functions similar to [`Time::new`] except if the values are out of bounds
    /// then [`None`] is returned instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use eos::Time;
    /// assert!(Time::try_new(10, 0, 0).is_ok());
    /// assert!(Time::try_new(24, 0, 0).is_err());
    /// assert!(Time::try_new(23, 60, 0).is_err());
    /// assert!(Time::try_new(23, 59, 60).is_err());
    /// ```
    pub fn try_new(hour: u8, minute: u8, second: u8) -> Result<Self, Error> {
        ensure_in_range!(hour, 23);
        ensure_in_range!(minute, 59);
        ensure_in_range!(second, 59);
        Ok(Self {
            nanosecond: 0,
            hour,
            minute,
            second,
        })
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
        let (hour, nanos) = divrem!(nanos, NANOS_PER_HOUR as i64);
        let (minute, nanos) = divrem!(nanos, NANOS_PER_MIN as i64);
        let (second, nanos) = divrem!(nanos, NANOS_PER_SEC as i64);
        let (days, hour) = divrem!(hour, 24);

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
    pub fn hour(&self) -> u8 {
        self.hour
    }

    /// Returns the minute within the hour.
    ///
    /// This value will always be within `0..60`.
    #[inline]
    pub fn minute(&self) -> u8 {
        self.minute
    }

    /// Returns the second within the minute.
    ///
    /// This value will always be within `0..60`.
    #[inline]
    pub fn second(&self) -> u8 {
        self.second
    }

    /// Returns the millisecond within the second.
    ///
    /// This value will always be within `0..1000`.
    #[inline]
    pub fn millisecond(&self) -> u16 {
        (self.nanosecond / 1_000_000) as u16
    }

    /// Returns the microsecond within the second.
    ///
    /// This value will always be within `0..1_000_000`.
    #[inline]
    pub fn microsecond(&self) -> u32 {
        self.nanosecond / 1_000
    }

    /// Returns the nanosecond within the second.
    ///
    /// This value will always be within `0..2_000_000_000`.
    #[inline]
    pub fn nanosecond(&self) -> u32 {
        self.nanosecond
    }

    /// Returns a new [`Time`] that points to the given hour.
    ///
    /// # Panics
    ///
    /// Panics if the hour is out of bounds (`0..24`). If this is
    /// undesirable, see [`Time::try_with_hour`].
    #[inline]
    pub fn with_hour(self, hour: u8) -> Self {
        self.try_with_hour(hour).expect("hour is out of range")
    }

    /// Returns a new [`Time`] that points to the given hour.
    ///
    /// This is similar to [`Time::with_hour`] except [`None`] is returned
    /// when the value is out of bounds.
    #[inline]
    pub fn try_with_hour(mut self, hour: u8) -> Result<Self, Error> {
        ensure_in_range!(hour, 24);
        self.hour = hour;
        Ok(self)
    }

    /// Returns a new [`Time`] that points to the given minute.
    ///
    /// # Panics
    ///
    /// Panics if the minute is out of bounds (`0..60`). If this is
    /// undesirable, see [`Time::try_with_minute`].
    #[inline]
    pub fn with_minute(self, minute: u8) -> Self {
        self.try_with_minute(minute).expect("minute is out of range")
    }

    /// Returns a new [`Time`] that points to the given minute.
    ///
    /// This is similar to [`Time::with_minute`] except [`None`] is returned
    /// when the value is out of bounds.
    #[inline]
    pub fn try_with_minute(mut self, minute: u8) -> Result<Self, Error> {
        ensure_in_range!(minute, 59);
        self.minute = minute;
        Ok(self)
    }

    /// Returns a new [`Time`] that points to the given second.
    ///
    /// # Panics
    ///
    /// Panics if the second is out of bounds (`0..60`). If this is
    /// undesirable, see [`Time::try_with_second`].
    #[inline]
    pub fn with_second(self, second: u8) -> Self {
        self.try_with_second(second).expect("second is out of range")
    }

    /// Returns a new [`Time`] that points to the given second.
    ///
    /// This is similar to [`Time::with_second`] except [`None`] is returned
    /// when the value is out of bounds.
    #[inline]
    pub fn try_with_second(mut self, second: u8) -> Result<Self, Error> {
        ensure_in_range!(second, 59);
        self.second = second;
        Ok(self)
    }

    /// Returns a new [`Time`] that points to the given millisecond.
    ///
    /// # Panics
    ///
    /// Panics if the millisecond is out of bounds (`0..1000`). If this is
    /// undesirable, see [`Time::try_with_millisecond`].
    #[inline]
    pub fn with_millisecond(self, millisecond: u16) -> Self {
        self.try_with_millisecond(millisecond)
            .expect("millisecond is out of range")
    }

    /// Returns a new [`Time`] that points to the given millisecond.
    ///
    /// This is similar to [`Time::with_millisecond`] except [`None`] is returned
    /// when the value is out of bounds.
    #[inline]
    pub fn try_with_millisecond(mut self, millisecond: u16) -> Result<Self, Error> {
        ensure_in_range!(millisecond, 1999);
        self.nanosecond = millisecond as u32 * 1_000_000;
        Ok(self)
    }

    /// Returns a new [`Time`] that points to the given microsecond.
    ///
    /// # Panics
    ///
    /// Panics if the microsecond is out of bounds (`0..1_000_000`). If this is
    /// undesirable, see [`Time::try_with_microsecond`].
    #[inline]
    pub fn with_microsecond(self, microsecond: u32) -> Self {
        self.try_with_microsecond(microsecond)
            .expect("microsecond is out of range")
    }

    /// Returns a new [`Time`] that points to the given microsecond.
    ///
    /// This is similar to [`Time::with_microsecond`] except [`None`] is returned
    /// when the value is out of bounds.
    #[inline]
    pub fn try_with_microsecond(mut self, microsecond: u32) -> Result<Self, Error> {
        ensure_in_range!(microsecond, 1_999_999);
        self.nanosecond = microsecond * 1_000;
        Ok(self)
    }

    /// Returns a new [`Time`] that points to the given nanosecond.
    ///
    /// # Panics
    ///
    /// Panics if the nanosecond is out of bounds (`0..2_000_000_000`). If this is
    /// undesirable, see [`Time::try_with_nanosecond`].
    #[inline]
    pub fn with_nanosecond(self, nanosecond: u32) -> Self {
        self.try_with_nanosecond(nanosecond)
            .expect("nanosecond is out of range")
    }

    /// Returns a new [`Time`] that points to the given nanosecond.
    ///
    /// This is similar to [`Time::with_nanosecond`] except [`None`] is returned
    /// when the value is out of bounds.
    #[inline]
    pub fn try_with_nanosecond(mut self, nanosecond: u32) -> Result<Self, Error> {
        ensure_in_range!(nanosecond, 1_999_999_999);
        self.nanosecond = nanosecond;
        Ok(self)
    }
}

impl Add<Interval> for Time {
    type Output = Time;

    fn add(self, rhs: Interval) -> Self::Output {
        let (sub, duration) = rhs.into_time_duration();
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
        let (sub, duration) = rhs.into_time_duration();
        let (_, ret) = if sub {
            self.add_with_duration(duration)
        } else {
            self.sub_with_duration(duration)
        };
        ret
    }
}
