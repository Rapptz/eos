use crate::{
    timezone::{Utc, UtcOffset},
    Date, Time, TimeZone, Weekday,
};
use crate::{Error, Interval};

use core::ops::{Add, Sub};
use core::time::Duration;
#[cfg(feature = "std")]
use std::time::SystemTime;

/// An ISO 8601 combined date and time component.
///
/// Unlike their individual components, [`DateTime`] have a timezone associated with them.
/// For convenience, the methods of [`Time`] and [`Date`] are flattened and inherent methods
/// of the struct. This means that methods such as [`second`] or [`month`] work as expected.
///
/// [`second`]: DateTime::second
/// [`month`]: DateTime::month
#[derive(Debug, Clone, Copy, Hash)]
pub struct DateTime<Tz = Utc>
where
    Tz: TimeZone,
{
    date: Date,
    time: Time,
    timezone: Tz,
}

#[doc(hidden)]
#[cfg(feature = "macros")]
#[inline]
pub const fn __create_offset_datetime_from_macro(date: Date, time: Time, timezone: UtcOffset) -> DateTime<UtcOffset> {
    DateTime { date, time, timezone }
}

impl DateTime<Utc> {
    /// Represents a [`DateTime`] at the unix epoch (January 1st, 1970 00:00:00 UTC).
    pub const UNIX_EPOCH: Self = Self {
        date: Date::UNIX_EPOCH,
        time: Time::MIDNIGHT,
        timezone: Utc,
    };

    /// Returns the current date and time in UTC.
    #[inline]
    #[cfg(feature = "std")]
    pub fn utc_now() -> Self {
        SystemTime::now().into()
    }

    #[doc(hidden)]
    #[cfg(feature = "macros")]
    #[inline]
    pub const fn __new_utc_unchecked_from_macro(date: Date, time: Time) -> Self {
        Self {
            date,
            time,
            timezone: Utc,
        }
    }

    /// Creates a [`DateTime`] from the given year and ordinal date. The time is set to
    /// midnight UTC.
    ///
    /// If the ordinal is out of bounds (`1..=366`) then [`None`] is returned.
    /// Note that 366 is also invalid if the year is not a leap year.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::{DateTime, Utc};
    /// assert_eq!(DateTime::from_ordinal(1992, 62), Ok(DateTime::<Utc>::new(1992, 3, 2)?)); // leap year
    /// assert!(DateTime::from_ordinal(2013, 366).is_err()); // not a leap year
    /// assert_eq!(DateTime::from_ordinal(2012, 366), Ok(DateTime::<Utc>::new(2012, 12, 31)?));
    /// assert_eq!(DateTime::from_ordinal(2001, 246), Ok(DateTime::<Utc>::new(2001, 9, 3)?));
    /// # Ok::<_, eos::Error>(())
    /// ```
    pub fn from_ordinal(year: i16, ordinal: u16) -> Result<Self, Error> {
        let date = Date::from_ordinal(year, ordinal)?;
        Ok(Self {
            date,
            time: Time::MIDNIGHT,
            timezone: Utc,
        })
    }
}

impl<Tz> DateTime<Tz>
where
    Tz: Default + TimeZone,
{
    /// Creates a new [`DateTime`] from a given year, month, and day with the time set to midnight.
    /// The timezone is created using the [`Default`] trait.
    ///
    /// The month must be between `1..=12` and the day must be between `1..=31`.
    /// Note that the day has to be valid for the specified month, i.e. February
    /// must be either 28 or 29 days depending on the year.
    ///
    /// Returns [`Error`] if the date is out of range. See [`Date::new`] for more info.
    ///
    /// # Examples
    ///
    /// ```
    /// use eos::{DateTime, Time, Utc};
    ///
    /// let dt = DateTime::<Utc>::new(2003, 4, 19)?; // creates a DateTime at UTC
    /// assert_eq!(dt.year(), 2003);
    /// assert_eq!(dt.month(), 4);
    /// assert_eq!(dt.day(), 19);
    /// assert_eq!(dt.time(), &Time::MIDNIGHT);
    /// assert!(DateTime::<Utc>::new(2013, 2, 29).is_err()); // 2013 was not a leap year
    /// # Ok::<_, eos::Error>(())
    /// ```
    pub fn new(year: i16, month: u8, day: u8) -> Result<Self, Error> {
        Ok(Self {
            date: Date::new(year, month, day)?,
            time: Time::MIDNIGHT,
            timezone: Tz::default(),
        })
    }
}

impl<Tz> DateTime<Tz>
where
    Tz: TimeZone,
{
    /// Returns a reference to the time component.
    pub fn time(&self) -> &Time {
        &self.time
    }

    /// Returns a mutable reference to the time component.
    pub fn time_mut(&mut self) -> &mut Time {
        &mut self.time
    }

    /// Returns a reference to the date component.
    pub fn date(&self) -> &Date {
        &self.date
    }

    /// Returns a mutable reference to the date component.
    pub fn date_mut(&mut self) -> &mut Date {
        &mut self.date
    }

    /// Returns a new [`DateTime`] with the newly specified [`Time`].
    ///
    /// This does not do timezone conversion.
    pub fn with_time(mut self, time: Time) -> Self {
        self.time = time;
        self
    }

    /// Returns a new [`DateTime`] with the newly specified [`Date`].
    pub fn with_date(mut self, date: Date) -> Self {
        self.date = date;
        self
    }

    /// Returns a reference to the [`TimeZone`] associated with this datetime.
    pub fn timezone(&self) -> &Tz {
        &self.timezone
    }

    /// Returns a mutable reference to the [`TimeZone`] associated with this datetime.
    pub fn timezone_mut(&mut self) -> &mut Tz {
        &mut self.timezone
    }

    // The "common" functions begin here.
    // I want to "unroll" the trait and make them inherent methods since their discoverability
    // is better in the documentation, and the trait usability is mostly subpar.
    // This is done both in Time and Date.

    /// Returns the year.
    ///
    /// Note that year 0 is equivalent to 1 BC (or BCE) and year 1 is equivalent
    /// to 1 AD (or CE).
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::datetime;
    /// let date = datetime!(2012-01-15 00:00);
    /// assert_eq!(date.year(), 2012);
    /// # Ok::<_, eos::Error>(())
    /// ```
    #[inline]
    pub fn year(&self) -> i16 {
        self.date.year()
    }

    /// Returns the month.
    ///
    /// This value will always be within `1..=12`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::datetime;
    /// let date = datetime!(2012-01-15 00:00);
    /// assert_eq!(date.month(), 1);
    /// # Ok::<_, eos::Error>(())
    /// ```
    #[inline]
    pub fn month(&self) -> u8 {
        self.date.month()
    }

    /// Returns the day.
    ///
    /// This value will always be within `1..=31`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::datetime;
    /// let date = datetime!(2012-01-15 00:00);
    /// assert_eq!(date.day(), 15);
    /// # Ok::<_, eos::Error>(())
    /// ```
    #[inline]
    pub fn day(&self) -> u8 {
        self.date.day()
    }

    /// Returns the ISO ordinal date.
    ///
    /// January 1st is 1 and December 31st is either 365 or 366 depending on leap year.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::{DateTime, Utc};
    /// let date = DateTime::<Utc>::new(2013, 3, 17)?;
    /// let leap = DateTime::<Utc>::new(2012, 3, 17)?;
    ///
    /// assert_eq!(date.ordinal(), 76);
    /// assert_eq!(leap.ordinal(), 77); // 2012 was a leap year
    /// # Ok::<_, eos::Error>(())
    /// ```
    #[inline]
    pub fn ordinal(&self) -> u16 {
        self.date.ordinal()
    }

    /// Returns the weekday.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::{DateTime, Utc};
    /// # use eos::Weekday;
    /// assert_eq!(DateTime::<Utc>::new(2021, 12, 25)?.weekday(), Weekday::Saturday);
    /// assert_eq!(DateTime::<Utc>::new(2012, 2, 29)?.weekday(), Weekday::Wednesday);
    /// # Ok::<_, eos::Error>(())
    /// ```
    pub fn weekday(&self) -> Weekday {
        self.date.weekday()
    }

    /// Returns a new [`DateTime`] with the date pointing to the given year.
    pub fn with_year(mut self, year: i16) -> Self {
        self.date = self.date.with_year(year);
        self
    }

    /// Returns a new [`DateTime`] that points to the given month.
    /// If the month is out of bounds (`1..=12`) or if the month
    /// does not have as many days as is currently specified then
    /// an [`Error`] is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::{DateTime, Utc};
    /// assert!(DateTime::<Utc>::new(2012, 3, 30)?.with_month(2).is_err());
    /// assert!(DateTime::<Utc>::new(2014, 12, 31)?.with_month(1).is_ok());
    /// assert!(DateTime::<Utc>::new(2019, 4, 28)?.with_month(2).is_ok());
    /// # Ok::<_, eos::Error>(())
    /// ```
    pub fn with_month(mut self, month: u8) -> Result<Self, Error> {
        self.date = self.date.with_month(month)?;
        Ok(self)
    }

    /// Returns a new [`Date`] that points to the given day.
    /// If the day is out of bounds (`1..=31`) then an [`Error`] is returned.
    ///
    /// Note that the actual maximum day depends on the specified month.
    /// For example, `30` is always invalid with a month of February since
    /// the maximum day for the given month is `29`.
    pub fn with_day(mut self, day: u8) -> Result<Self, Error> {
        self.date = self.date.with_day(day)?;
        Ok(self)
    }

    /// Returns the hour.
    ///
    /// This value will always be within `0..24`.
    #[inline]
    pub fn hour(&self) -> u8 {
        self.time.hour()
    }

    /// Returns the minute within the hour.
    ///
    /// This value will always be within `0..60`.
    #[inline]
    pub fn minute(&self) -> u8 {
        self.time.minute()
    }

    /// Returns the second within the minute.
    ///
    /// This value will always be within `0..60`.
    #[inline]
    pub fn second(&self) -> u8 {
        self.time.second()
    }

    /// Returns the millisecond within the second.
    ///
    /// This value will always be within `0..1000`.
    #[inline]
    pub fn millisecond(&self) -> u16 {
        self.time.millisecond()
    }

    /// Returns the microsecond within the second.
    ///
    /// This value will always be within `0..1_000_000`.
    #[inline]
    pub fn microsecond(&self) -> u32 {
        self.time.microsecond()
    }

    /// Returns the nanosecond within the second.
    ///
    /// This value will always be within `0..2_000_000_000`.
    #[inline]
    pub fn nanosecond(&self) -> u32 {
        self.time.nanosecond()
    }

    /// Returns a new [`DateTime`] that points to the given hour.
    /// If the hour is out of bounds (`0..24`) then [`Error`] is returned.
    ///
    /// This does not do timezone conversion.
    #[inline]
    pub fn with_hour(mut self, hour: u8) -> Result<Self, Error> {
        self.time = self.time.with_hour(hour)?;
        Ok(self)
    }

    /// Returns a new [`DateTime`] that points to the given minute.
    /// If the minute is out of bounds (`0..60`) then [`Error`] is returned.
    ///
    /// This does not do timezone conversion.
    #[inline]
    pub fn with_minute(mut self, minute: u8) -> Result<Self, Error> {
        self.time = self.time.with_minute(minute)?;
        Ok(self)
    }

    /// Returns a new [`DateTime`] that points to the given second.
    /// If the second is out of bounds (`0..60`) then [`Error`] is returned.
    ///
    /// This does not do timezone conversion.
    #[inline]
    pub fn with_second(mut self, second: u8) -> Result<Self, Error> {
        self.time = self.time.with_second(second)?;
        Ok(self)
    }

    /// Returns a new [`DateTime`] that points to the given millisecond.
    /// If the millisecond is out of bounds (`0..1000`) then [`Error`] is returned.
    ///
    /// This does not do timezone conversion.
    #[inline]
    pub fn with_millisecond(mut self, millisecond: u16) -> Result<Self, Error> {
        self.time = self.time.with_millisecond(millisecond)?;
        Ok(self)
    }

    /// Returns a new [`DateTime`] that points to the given microsecond.
    /// If the microsecond is out of bounds (`0..1_000_000`) then [`Error`] is returned.
    ///
    /// This does not do timezone conversion.
    #[inline]
    pub fn with_microsecond(mut self, microsecond: u32) -> Result<Self, Error> {
        self.time = self.time.with_microsecond(microsecond)?;
        Ok(self)
    }

    /// Returns a new [`DateTime`] that points to the given nanosecond.
    /// If the nanosecond is out of bounds (`0..2_000_000_000`) then [`Error`] is returned.
    ///
    /// This does not do timezone conversion.
    #[inline]
    pub fn with_nanosecond(mut self, nanosecond: u32) -> Result<Self, Error> {
        self.time = self.time.with_nanosecond(nanosecond)?;
        Ok(self)
    }
}

impl Add<Duration> for DateTime {
    type Output = DateTime;

    fn add(self, rhs: Duration) -> Self::Output {
        let (days, time) = self.time.add_with_duration(rhs);
        let date = self.date.add_days(days);
        Self {
            date,
            time,
            timezone: self.timezone,
        }
    }
}

impl Sub<Duration> for DateTime {
    type Output = DateTime;

    fn sub(self, rhs: Duration) -> Self::Output {
        let (days, time) = self.time.sub_with_duration(rhs);
        let date = self.date.add_days(days);
        Self {
            date,
            time,
            timezone: self.timezone,
        }
    }
}

#[cfg(feature = "std")]
impl From<SystemTime> for DateTime {
    /// Creates
    fn from(time: SystemTime) -> Self {
        match time.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => Self::UNIX_EPOCH + duration,
            Err(e) => Self::UNIX_EPOCH - e.duration(),
        }
    }
}

impl<Tz, OtherTz> PartialEq<DateTime<OtherTz>> for DateTime<Tz>
where
    Tz: TimeZone,
    OtherTz: TimeZone,
{
    fn eq(&self, other: &DateTime<OtherTz>) -> bool {
        self.date.eq(&other.date) && self.time.eq(&other.time)
    }
}

// Rust does not support Eq<Rhs> for some reason
impl<Tz> Eq for DateTime<Tz> where Tz: TimeZone {}

impl<Tz, OtherTz> PartialOrd<DateTime<OtherTz>> for DateTime<Tz>
where
    Tz: TimeZone,
    OtherTz: TimeZone,
{
    fn partial_cmp(&self, other: &DateTime<OtherTz>) -> Option<std::cmp::Ordering> {
        match self.date.partial_cmp(&other.date) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.time.partial_cmp(&other.time)
    }
}

// Rust does not allow Ord<Rhs> for some reason
// see: https://github.com/rust-lang/rfcs/issues/2511
impl<Tz> Ord for DateTime<Tz>
where
    Tz: TimeZone,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.date.cmp(&other.date) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.time.cmp(&other.time)
    }
}

impl<Tz> Add<Interval> for DateTime<Tz>
where
    Tz: TimeZone,
{
    type Output = Self;

    fn add(self, rhs: Interval) -> Self::Output {
        let (sub, duration) = rhs.to_time_duration();
        let (days, time) = if sub {
            self.time.sub_with_duration(duration)
        } else {
            self.time.add_with_duration(duration)
        };

        let date = self
            .date
            .add_months(rhs.total_months())
            .add_days(rhs.total_days() + days);

        Self {
            date,
            time,
            timezone: self.timezone,
        }
    }
}

impl<Tz> Sub<Interval> for DateTime<Tz>
where
    Tz: TimeZone,
{
    type Output = Self;

    fn sub(self, rhs: Interval) -> Self::Output {
        let (sub, duration) = rhs.to_time_duration();
        let (days, time) = if sub {
            self.time.add_with_duration(duration)
        } else {
            self.time.sub_with_duration(duration)
        };

        let date = self
            .date
            .add_months(rhs.total_months().wrapping_neg())
            .add_days(rhs.total_days().wrapping_neg() + days);

        Self {
            date,
            time,
            timezone: self.timezone,
        }
    }
}
