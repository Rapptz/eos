use crate::{timezone::Utc, utils::ensure_in_range, Error, TimeZone};

use core::time::Duration;

/// Represents a moment in time.
///
/// This type has nanosecond precision. Comparisons assume they're on the same calendar date and
/// are done on UTC time.
#[derive(Debug, Clone, Copy)]
pub struct Time<Tz = Utc>
where
    Tz: TimeZone,
{
    hour: u8,
    minute: u8,
    second: u8,
    nanosecond: u32,
    timezone: Tz,
}

impl Time {
    /// Represets the minimum time in UTC.
    pub const MIN: Self = Self {
        hour: 0,
        minute: 0,
        second: 0,
        nanosecond: 0,
        timezone: Utc,
    };

    /// Represents the time at midnight UTC.
    pub const MIDNIGHT: Self = Self::MIN;

    /// Represents the maximum time in UTC.
    ///
    /// This does not include leap seconds.
    pub const MAX: Self = Self {
        hour: 23,
        minute: 59,
        second: 59,
        nanosecond: 999_999_999,
        timezone: Utc,
    };

    /// Creates a new [`Time`] from the specified hour, minute, and second at UTC.
    ///
    /// The `hour` value must be between `0..24` and the `minute` and `second` values must
    /// be between `0..60`.
    ///
    /// # Panics
    ///
    /// Panics if the values are out of range. If this is undesirable, consider
    /// using [`Time::from_utc`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use eos::Time;
    /// let time = Time::utc(23, 10, 0);
    ///
    /// assert_eq!(time.hour(), 23);
    /// assert_eq!(time.minute(), 10);
    /// assert_eq!(time.second(), 0);
    /// ```
    pub fn utc(hour: u8, minute: u8, second: u8) -> Self {
        Self::from_utc(hour, minute, second).expect("input of out range")
    }

    /// Creates a new [`Time`] from the specified hour, minute, and second at UTC.
    ///
    /// This functions similar to [`Time::utc`] except if the values are out of bounds
    /// then [`None`] is returned instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use eos::Time;
    /// assert!(Time::from_utc(10, 0, 0).is_ok());
    /// assert!(Time::from_utc(24, 0, 0).is_err());
    /// assert!(Time::from_utc(23, 60, 0).is_err());
    /// assert!(Time::from_utc(23, 59, 60).is_err());
    /// ```
    pub fn from_utc(hour: u8, minute: u8, second: u8) -> Result<Self, Error> {
        Self::try_new(hour, minute, second)
    }
}

impl<Tz> Time<Tz>
where
    Tz: TimeZone + Default,
{
    /// Creates a new [`Time`] from the specified hour, minute, and second. The
    /// timezone is created using the [`Default`] trait.
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
    /// # use eos::{Time, Utc};
    /// let time = Time::<Utc>::new(23, 10, 0);
    ///
    /// assert_eq!(time.hour(), 23);
    /// assert_eq!(time.minute(), 10);
    /// assert_eq!(time.second(), 0);
    /// ```
    pub fn new(hour: u8, minute: u8, second: u8) -> Self {
        Self::try_new(hour, minute, second).expect("input of out range")
    }

    /// Creates a new [`Time`] at midnight with a timezone created using the [`Default`]
    /// trait.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::{Time, Utc};
    /// let midnight = Time::<Utc>::midnight(); // alt: Time::MIDNIGHT
    /// assert_eq!(midnight.hour(), 0);
    /// assert_eq!(midnight.minute(), 0);
    /// assert_eq!(midnight.second(), 0);
    /// assert_eq!(midnight.nanosecond(), 0);
    /// ```
    pub fn midnight() -> Self {
        Self {
            hour: 0,
            minute: 0,
            second: 0,
            nanosecond: 0,
            timezone: Tz::default(),
        }
    }

    /// Creates a new [`Time`] from the specified hour, minute, and second. The
    /// timezone is created using the [`Default`] trait.
    ///
    /// This functions similar to [`Time::new`] except if the values are out of bounds
    /// then [`None`] is returned instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use eos::{Time, Utc};
    /// assert!(Time::<Utc>::try_new(10, 0, 0).is_ok());
    /// assert!(Time::<Utc>::try_new(24, 0, 0).is_err());
    /// assert!(Time::<Utc>::try_new(23, 60, 0).is_err());
    /// assert!(Time::<Utc>::try_new(23, 59, 60).is_err());
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
            timezone: Tz::default(),
        })
    }
}

impl<Tz> Time<Tz>
where
    Tz: TimeZone,
{
    #[inline]
    pub(crate) fn total_seconds(&self) -> i32 {
        self.hour as i32 * 3600 + self.minute as i32 * 60 + self.second as i32
    }

    /// Adds the time with the given duration and returns the number of days that have passed.
    pub(crate) fn add_with_duration(self, duration: Duration) -> (i32, Self) {
        let mut nanosecond = self.nanosecond + duration.subsec_nanos();
        let mut secs = duration.as_secs();
        let mut hour = self.hour as u64 + secs / 3600;
        secs %= 3600;
        let mut minute = self.minute as u64 + secs / 60;
        secs %= 60;
        let mut second = self.second as u64 + secs;

        if nanosecond >= 2_000_000_000 {
            second += (nanosecond / 2_000_000_000) as u64;
            nanosecond %= 2_000_000_000;
        }

        if second >= 60 {
            minute += second / 60;
            minute %= 60;
        }

        if minute >= 60 {
            hour += minute / 60;
            minute %= 60;
        }

        let days = if hour >= 24 {
            let d = hour / 24;
            hour %= 24;
            d as i32
        } else {
            0
        };

        (
            days,
            Self {
                hour: hour as u8,
                minute: minute as u8,
                second: second as u8,
                nanosecond,
                timezone: self.timezone,
            },
        )
    }

    /// Subtracts the time with the given duration and returns the number of days that have passed.
    pub(crate) fn sub_with_duration(self, duration: Duration) -> (i32, Self) {
        let mut nanosecond = self.nanosecond as i32 - duration.subsec_nanos() as i32;
        let mut secs = duration.as_secs() as i64;
        let mut hour = self.hour as i64 + secs / 3600;
        secs %= 3600;
        let mut minute = self.minute as i64 + secs / 60;
        secs %= 60;
        let mut second = self.second as i64 + secs;

        if nanosecond >= 2_000_000_000 {
            second += (nanosecond / 2_000_000_000) as i64;
            nanosecond %= 2_000_000_000;
        }

        if second >= 60 {
            minute += second / 60;
            minute %= 60;
        }

        if minute >= 60 {
            hour += minute / 60;
            minute %= 60;
        }

        let days = if hour >= 24 {
            let d = hour / 24;
            hour %= 24;
            d as i32
        } else {
            0
        };

        (
            days,
            Self {
                hour: hour as u8,
                minute: minute as u8,
                second: second as u8,
                nanosecond: nanosecond as u32,
                timezone: self.timezone,
            },
        )
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

    /// Compares two [`Time`] objects without considering their timezone information.
    ///
    /// # Examples
    ///
    /// ```
    /// use eos::{UtcOffset, Time};
    ///
    /// ```
    #[inline]
    pub fn cmp_without_tz<OtherTz>(&self, other: &Time<OtherTz>) -> core::cmp::Ordering
    where
        OtherTz: TimeZone,
    {
        (self.hour, self.minute, self.second, self.nanosecond).cmp(&(
            other.hour,
            other.minute,
            other.second,
            other.nanosecond,
        ))
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

fn compare_times<Tz, OtherTz>(lhs: &Time<Tz>, rhs: &Time<OtherTz>) -> core::cmp::Ordering
where
    Tz: TimeZone,
    OtherTz: TimeZone,
{
    let offset = lhs.timezone.offset::<Tz>(None);
    let rhs_offset = rhs.timezone.offset::<OtherTz>(None);
    if offset == rhs_offset {
        (lhs.hour, lhs.minute, lhs.second, lhs.nanosecond).cmp(&(rhs.hour, rhs.minute, rhs.second, rhs.nanosecond))
    } else {
        let hms = lhs.total_seconds() - offset.total_seconds();
        let rhs_hms = rhs.total_seconds() - rhs_offset.total_seconds();
        (hms, lhs.nanosecond).cmp(&(rhs_hms, rhs.nanosecond))
    }
}

impl<Tz, OtherTz> PartialEq<Time<OtherTz>> for Time<Tz>
where
    Tz: TimeZone,
    OtherTz: TimeZone,
{
    fn eq(&self, other: &Time<OtherTz>) -> bool {
        compare_times(&self, &other) == core::cmp::Ordering::Equal
    }
}

// Rust does not support Eq<Rhs> for some reason
impl<Tz> Eq for Time<Tz> where Tz: TimeZone {}

impl<Tz, OtherTz> PartialOrd<Time<OtherTz>> for Time<Tz>
where
    Tz: TimeZone,
    OtherTz: TimeZone,
{
    fn partial_cmp(&self, other: &Time<OtherTz>) -> Option<std::cmp::Ordering> {
        Some(compare_times(&self, &other))
    }
}

// Rust does not allow Ord<Rhs> for some reason
// see: https://github.com/rust-lang/rfcs/issues/2511
impl<Tz> Ord for Time<Tz>
where
    Tz: TimeZone,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        compare_times(&self, &other)
    }
}

impl<Tz> core::hash::Hash for Time<Tz>
where
    Tz: TimeZone,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        let offset = self.timezone.offset::<Tz>(None);
        if offset.is_utc() {
            self.hour.hash(state);
            self.minute.hash(state);
            self.second.hash(state);
            self.nanosecond.hash(state);
        } else {
            let seconds = self.total_seconds() - offset.total_seconds();
            seconds.hash(state);
            self.nanosecond.hash(state);
        }
    }
}
