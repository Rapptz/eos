use core::{
    ops::{Add, AddAssign, Sub, SubAssign},
    time::Duration,
};

use crate::{utils::divmod, Date, Time};

/// Represents a interval of time such as 2 years, 30 minutes, etc.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Interval {
    // There is an alternative data format that allows us to fit in
    // every component necessary without taking as much as memory
    // while retaining functionality, inspired by PostgreSQL.
    // By storing 32-bit months we get both years and months for free.
    // The next granularity is 32-bit days, which are fixed length of 7
    // days and we get days and weeks for free.
    // Afterwards we can store 64-bit seconds and 32-bit nanoseconds.
    //
    // However, this does complicate certain retrieval operations when we begin to clamp
    // them down into their own separate type. For example with 32-bit months / 12
    // we can't end up with 16-bit years since it could overflow.
    // I want to prioritise correctness before focusing on the
    // perceived benefits of minimising the memory, even if I want to.
    //
    // Likewise, by hardcoding these assumptions it becomes hard to break out of the
    // ISO8601 calendar if I want to in the future.
    years: i16,
    days: i32,
    weeks: i32,
    months: i32,
    hours: i32,
    minutes: i64,
    seconds: i64,
    nanoseconds: i64,
}

impl Interval {
    /// A interval that contains only zero values.
    pub const ZERO: Self = Self {
        years: 0,
        days: 0,
        weeks: 0,
        months: 0,
        hours: 0,
        minutes: 0,
        seconds: 0,
        nanoseconds: 0,
    };

    /// Creates a [`Interval`] representing the specified number of years.
    #[inline]
    pub const fn from_years(years: i16) -> Self {
        Self { years, ..Self::ZERO }
    }

    /// Creates a [`Interval`] representing the specified number of days.
    #[inline]
    pub const fn from_days(days: i32) -> Self {
        Self { days, ..Self::ZERO }
    }

    /// Creates a [`Interval`] representing the specified number of months.
    #[inline]
    pub const fn from_months(months: i32) -> Self {
        Self { months, ..Self::ZERO }
    }

    /// Creates a [`Interval`] representing the specified number of weeks.
    #[inline]
    pub const fn from_weeks(weeks: i32) -> Self {
        Self { weeks, ..Self::ZERO }
    }

    /// Creates a [`Interval`] representing the specified number of hours.
    #[inline]
    pub const fn from_hours(hours: i32) -> Self {
        Self { hours, ..Self::ZERO }
    }

    /// Creates a [`Interval`] representing the specified number of minutes.
    #[inline]
    pub const fn from_minutes(minutes: i64) -> Self {
        Self { minutes, ..Self::ZERO }
    }

    /// Creates a [`Interval`] representing the specified number of seconds.
    #[inline]
    pub const fn from_seconds(seconds: i64) -> Self {
        Self { seconds, ..Self::ZERO }
    }

    /// Creates a [`Interval`] representing the specified number of milliseconds.
    ///
    /// Note that the internal structure only stores nanoseconds. If the computation
    /// would end up overflowing then the value is saturated to the upper bounds.
    #[inline]
    pub const fn from_milliseconds(milliseconds: i64) -> Self {
        Self {
            nanoseconds: milliseconds.saturating_mul(1_000_000),
            ..Self::ZERO
        }
    }

    /// Creates a [`Interval`] representing the specified number of microseconds.
    ///
    /// Note that the internal structure only stores nanoseconds. If the computation
    /// would end up overflowing then the value is saturated to the upper bounds.
    #[inline]
    pub const fn from_microseconds(microseconds: i64) -> Self {
        Self {
            nanoseconds: microseconds.saturating_mul(1_000),
            ..Self::ZERO
        }
    }

    /// Creates a [`Interval`] representing the specified number of nanoseconds.
    #[inline]
    pub const fn from_nanoseconds(nanoseconds: i64) -> Self {
        Self {
            nanoseconds,
            ..Self::ZERO
        }
    }

    /// Returns the number of years within this interval.
    #[inline]
    pub const fn years(&self) -> i16 {
        self.years
    }

    /// Returns the number of days within this interval.
    #[inline]
    pub const fn days(&self) -> i32 {
        self.days
    }

    /// Returns the number of months within this interval.
    #[inline]
    pub const fn months(&self) -> i32 {
        self.months
    }

    /// Returns the number of weeks within this interval.
    #[inline]
    pub const fn weeks(&self) -> i32 {
        self.weeks
    }

    /// Returns the number of hours within this interval.
    #[inline]
    pub const fn hours(&self) -> i32 {
        self.hours
    }

    /// Returns the number of minutes within this interval.
    #[inline]
    pub const fn minutes(&self) -> i64 {
        self.minutes
    }

    /// Returns the number of seconds within this interval.
    #[inline]
    pub const fn seconds(&self) -> i64 {
        self.seconds
    }

    /// Returns the number of milliseconds within this interval.
    #[inline]
    pub const fn milliseconds(&self) -> i64 {
        self.nanoseconds / 1_000_000
    }

    /// Returns the number of microseconds within this interval.
    #[inline]
    pub const fn microseconds(&self) -> i64 {
        self.nanoseconds / 1000
    }

    /// Returns the number of nanoseconds within this interval.
    #[inline]
    pub const fn nanoseconds(&self) -> i64 {
        self.nanoseconds
    }

    /// Returns a new [`Interval`] with the given number of years.
    pub fn with_years(mut self, years: i16) -> Self {
        self.years = years;
        self
    }

    /// Returns a new [`Interval`] with the given number of days.
    pub fn with_days(mut self, days: i32) -> Self {
        self.days = days;
        self
    }

    /// Returns a new [`Interval`] with the given number of weeks.
    pub fn with_weeks(mut self, weeks: i32) -> Self {
        self.weeks = weeks;
        self
    }

    /// Returns a new [`Interval`] with the given number of months.
    pub fn with_months(mut self, months: i32) -> Self {
        self.months = months;
        self
    }

    /// Returns a new [`Interval`] with the given number of hours.
    pub fn with_hours(mut self, hours: i32) -> Self {
        self.hours = hours;
        self
    }

    /// Returns a new [`Interval`] with the given number of minutes.
    pub fn with_minutes(mut self, minutes: i64) -> Self {
        self.minutes = minutes;
        self
    }

    /// Returns a new [`Interval`] with the given number of seconds.
    pub fn with_seconds(mut self, seconds: i64) -> Self {
        self.seconds = seconds;
        self
    }

    /// Returns a new [`Interval`] with the given number of milliseconds.
    ///
    /// Note that the internal structure only stores nanoseconds. If the computation
    /// would end up overflowing then the value is saturated to the upper bounds. If
    /// nanoseconds are already set then this would remove the previous value.
    #[inline]
    pub fn with_milliseconds(mut self, milliseconds: i64) -> Self {
        self.nanoseconds = milliseconds.saturating_mul(1_000_000);
        self
    }

    /// Returns a new [`Interval`] with the given number of microseconds.
    ///
    /// Note that the internal structure only stores nanoseconds. If the computation
    /// would end up overflowing then the value is saturated to the upper bounds. If
    /// nanoseconds are already set then this would remove the previous value.
    #[inline]
    pub fn with_microseconds(mut self, microseconds: i64) -> Self {
        self.nanoseconds = microseconds.saturating_mul(1_000);
        self
    }

    /// Returns a new [`Interval`] with the given number of nanoseconds.
    pub fn with_nanoseconds(mut self, nanoseconds: i64) -> Self {
        self.nanoseconds = nanoseconds;
        self
    }

    /* internal helpers */

    #[inline]
    pub(crate) const fn total_days(&self) -> i32 {
        self.weeks * 7 + self.days
    }

    /// Returns a duration representing the time components of this interval.
    ///
    /// The first boolean argument is whether the time ended up being negative.
    pub(crate) fn to_time_duration(&self) -> (bool, Duration) {
        let mut total_seconds = self.hours as i64 * 3600 + self.minutes as i64 * 60 + self.seconds;
        let (seconds, nanos) = divmod!(self.nanoseconds, 1_000_000_000);
        total_seconds += seconds;
        match (total_seconds.is_positive(), nanos.is_positive()) {
            (true, true) => (false, Duration::new(total_seconds as u64, nanos as u32)),
            (false, false) => (true, Duration::new(-total_seconds as u64, -nanos as u32)),
            (true, false) => (
                false,
                Duration::from_secs(total_seconds as u64) - Duration::from_nanos(-nanos as u64),
            ),
            (false, true) => (
                true,
                Duration::from_secs(-total_seconds as u64) - Duration::from_nanos(nanos as u64),
            ),
        }
    }
}

impl Add for Interval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            years: self.years + rhs.years,
            days: self.days + rhs.days,
            weeks: self.weeks + rhs.weeks,
            months: self.months + rhs.months,
            hours: self.hours + rhs.hours,
            minutes: self.minutes + rhs.minutes,
            seconds: self.seconds + rhs.seconds,
            nanoseconds: self.nanoseconds + rhs.nanoseconds,
        }
    }
}

impl AddAssign for Interval {
    fn add_assign(&mut self, rhs: Self) {
        self.years += rhs.years;
        self.days += rhs.days;
        self.weeks += rhs.weeks;
        self.months += rhs.months;
        self.hours += rhs.hours;
        self.minutes += rhs.minutes;
        self.seconds += rhs.seconds;
        self.nanoseconds += rhs.nanoseconds;
    }
}

impl Sub for Interval {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            years: self.years - rhs.years,
            days: self.days - rhs.days,
            weeks: self.weeks - rhs.weeks,
            months: self.months - rhs.months,
            hours: self.hours - rhs.hours,
            minutes: self.minutes - rhs.minutes,
            seconds: self.seconds - rhs.seconds,
            nanoseconds: self.nanoseconds - rhs.nanoseconds,
        }
    }
}

impl SubAssign for Interval {
    fn sub_assign(&mut self, rhs: Self) {
        self.years -= rhs.years;
        self.days -= rhs.days;
        self.weeks -= rhs.weeks;
        self.months -= rhs.months;
        self.hours -= rhs.hours;
        self.minutes -= rhs.minutes;
        self.seconds -= rhs.seconds;
        self.nanoseconds -= rhs.nanoseconds;
    }
}

impl From<Duration> for Interval {
    fn from(dt: Duration) -> Self {
        Self {
            seconds: dt.as_secs() as i64,
            nanoseconds: dt.subsec_nanos() as i64,
            ..Self::ZERO
        }
    }
}

impl Add<Duration> for Interval {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        self + Self::from(rhs)
    }
}

impl Sub<Duration> for Interval {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        self - Self::from(rhs)
    }
}

impl AddAssign<Duration> for Interval {
    fn add_assign(&mut self, rhs: Duration) {
        self.seconds += rhs.as_secs() as i64;
        self.nanoseconds += rhs.subsec_nanos() as i64;
    }
}

impl SubAssign<Duration> for Interval {
    fn sub_assign(&mut self, rhs: Duration) {
        self.seconds -= rhs.as_secs() as i64;
        self.nanoseconds -= rhs.subsec_nanos() as i64;
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