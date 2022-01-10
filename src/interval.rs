use core::{
    cmp::Ordering,
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
    time::Duration,
};

use crate::{utils::divrem, Date, DateTime, Time, TimeZone, UtcOffset};

pub(crate) const NANOS_PER_SEC: u64 = 1_000_000_000;
pub(crate) const NANOS_PER_MIN: u64 = 60 * NANOS_PER_SEC;
pub(crate) const NANOS_PER_HOUR: u64 = 60 * NANOS_PER_MIN;

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
        Self {
            days: weeks * 7,
            ..Self::ZERO
        }
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
        self.days / 7
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
        self.days = self.days - (self.days / 7) + weeks * 7;
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

    /// Normalize the interval so that large units are combined to their larger unit.
    /// For example, this turns 90 minutes into 1 hour and 30 minutes or 13 months
    /// into 1 year and 1 month.
    ///
    /// ```rust
    /// use eos::{Interval, ext::IntervalLiteral};
    /// let mut interval: Interval = 90.minutes() + 9.days() + 13.months() + 1.years();
    /// interval.normalize();
    /// assert_eq!(interval.years(), 2);
    /// assert_eq!(interval.months(), 1);
    /// assert_eq!(interval.days(), 9);
    /// assert_eq!(interval.hours(), 1);
    /// assert_eq!(interval.minutes(), 30);
    /// ```
    pub fn normalize(&mut self) {
        if self.nanoseconds.abs() >= 1_000_000_000 {
            self.seconds += self.nanoseconds / 1_000_000_000;
            self.nanoseconds %= 1_000_000_000;
        }

        if self.seconds.abs() >= 60 {
            self.minutes += self.seconds / 60;
            self.seconds %= 60;
        }

        if self.minutes.abs() >= 60 {
            self.hours += (self.minutes / 60) as i32;
            self.minutes %= 60;
        }

        if self.hours.abs() >= 24 {
            self.days += self.hours / 24;
            self.hours %= 24;
        }

        // Weeks cannot be reduced further... but months can in the gregorian calendar
        // Some edge cases arrive from this reduction such as
        // 1436-2-29 - (77.years() + (-97).months())
        // The formulation can either be 1367-3-28 or 1367-3-29 depending on whether
        // normalisation happens or not.
        // Since the library is assuming a Gregorian calendar, it makes sense to normalise
        // months and years even if other years do not always have 12 months in other calendars
        // Note that this normalisation is done via the total_months method, not this one

        if self.months.abs() >= 12 {
            self.years += (self.months / 12) as i16;
            self.months %= 12;
        }
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
            years,
            days,
            months,
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
    pub fn between_times(start: &Time, end: &Time) -> Self {
        // Times are conceptually simple since they're bounded to at most 24 hours
        let nanos = end.total_nanos() as i64 - start.total_nanos() as i64;
        let (hour, nanos) = divrem!(nanos, NANOS_PER_HOUR as i64);
        let (minutes, nanos) = divrem!(nanos, NANOS_PER_MIN as i64);
        let (seconds, nanos) = divrem!(nanos, NANOS_PER_SEC as i64);
        Self {
            hours: hour as i32,
            minutes,
            seconds,
            nanoseconds: nanos,
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
            let (years, months) = divrem!(months, 12);
            delta.years = years as i16;
            delta.months = months;
            delta.normalize(); // for seconds
            delta
        } else {
            let mut delta = Self::days_between(start, end);
            delta.normalize();
            delta
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
        let nanos = end.time().nanosecond() as i32 - start.time().nanosecond() as i32;

        let offset = start.timezone().offset(start);
        let end_offset = end.timezone().offset(end);
        if offset != end_offset {
            seconds = seconds + offset.total_seconds() - end_offset.total_seconds();
        }

        // Combine the days and seconds to ensure both of them have the same signage
        let seconds = days * 86_400 + seconds;
        let (days, seconds) = divrem!(seconds, 86_400);
        Self {
            days,
            seconds: seconds as i64,
            nanoseconds: nanos as i64,
            ..Self::ZERO
        }
    }

    #[inline]
    pub(crate) const fn total_months(&self) -> i32 {
        self.months + self.years as i32 * 12
    }

    #[inline]
    pub(crate) const fn total_seconds_from_days(&self) -> i64 {
        self.days as i64 * 86400 + self.hours as i64 * 3600 + self.minutes * 60 + self.seconds
    }

    #[inline]
    pub(crate) const fn total_milliseconds_from_days(&self) -> i64 {
        self.total_seconds_from_days() * 1000 + self.nanoseconds / 1_000_000
    }

    /// Returns a duration representing the time components of this interval.
    ///
    /// The first boolean argument is whether the time ended up being negative.
    pub(crate) fn get_time_duration(&self) -> (bool, Duration) {
        let mut total_seconds = self.hours as i64 * 3600 + self.minutes as i64 * 60 + self.seconds;
        let (seconds, nanos) = divrem!(self.nanoseconds, 1_000_000_000);
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
        let (h, m, s) = offset.into_hms();
        Self {
            hours: h as _,
            minutes: m as _,
            seconds: s as _,
            ..Self::ZERO
        }
    }
}

impl Neg for Interval {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            years: -self.years,
            days: -self.days,
            months: -self.months,
            hours: -self.hours,
            minutes: -self.minutes,
            seconds: -self.seconds,
            nanoseconds: -self.nanoseconds,
        }
    }
}

impl Add for Interval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            years: self.years + rhs.years,
            days: self.days + rhs.days,
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

impl Add<DateTime> for Interval {
    type Output = DateTime;

    fn add(self, rhs: DateTime) -> Self::Output {
        rhs + self
    }
}
