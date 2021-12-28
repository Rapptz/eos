use crate::Error;
use crate::{timezone::Utc, Date, Time, TimeZone, Weekday};

use core::ops::{Add, Sub};
use core::time::Duration;
#[cfg(feature = "std")]
use std::time::SystemTime;

/// An ISO 8601 combined date and time component.
///
/// [`DateTime`] tend to have a timezone associated with them. For convenience,
/// the methods of [`Time`] and [`Date`] are flattened and inherent methods of
/// the struct. This means that methods such as [`second`] or [`month`] work as expected.
///
/// [`second`]: DateTime::second
/// [`month`]: DateTime::month
#[derive(Debug, Clone, Copy, Hash)]
pub struct DateTime<Tz = Utc>
where
    Tz: TimeZone,
{
    date: Date,
    time: Time<Tz>,
}

impl DateTime<Utc> {
    /// Represents a [`DateTime`] at the unix epoch (January 1st, 1970 00:00:00 UTC).
    pub const UNIX_EPOCH: Self = Self {
        date: Date::UNIX_EPOCH,
        time: Time::MIDNIGHT,
    };

    /// Returns the current date and time in UTC.
    #[inline]
    #[cfg(feature = "std")]
    pub fn utc_now() -> Self {
        SystemTime::now().into()
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
    /// assert_eq!(DateTime::from_ordinal(1992, 62), Ok(DateTime::<Utc>::new(1992, 3, 2))); // leap year
    /// assert!(DateTime::from_ordinal(2013, 366).is_err()); // not a leap year
    /// assert_eq!(DateTime::from_ordinal(2012, 366), Ok(DateTime::<Utc>::new(2012, 12, 31)));
    /// assert_eq!(DateTime::from_ordinal(2001, 246), Ok(DateTime::<Utc>::new(2001, 9, 3)));
    /// ```
    pub fn from_ordinal(year: i16, ordinal: u16) -> Result<Self, Error> {
        let date = Date::from_ordinal(year, ordinal)?;
        Ok(Self {
            date,
            time: Time::MIDNIGHT,
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
    /// # Examples
    ///
    /// ```
    /// use eos::{DateTime, Time, Utc};
    ///
    /// let dt = DateTime::<Utc>::new(2003, 4, 19); // creates a DateTime at UTC
    /// assert_eq!(dt.year(), 2003);
    /// assert_eq!(dt.month(), 4);
    /// assert_eq!(dt.day(), 19);
    /// assert_eq!(dt.time(), &Time::MIDNIGHT);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the date is out of range. If this is undesirable, consider
    /// using [`Date::try_new`].
    pub fn new(year: i16, month: u8, day: u8) -> Self {
        Self::try_new(year, month, day).expect("invalid or out-of-range date")
    }

    /// Creates a new [`DateTime`] from a given year, month, and day.
    /// The timezone is created using the [`Default`] trait.
    ///
    /// This functions similar to [`DateTime::<Utc>::new`] except if the values are out of bounds
    /// then [`None`] is returned instead.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::{DateTime, Utc};
    /// assert!(DateTime::<Utc>::try_new(2013, 2, 29).is_err()); // 2013 was not a leap year
    /// ```
    pub fn try_new(year: i16, month: u8, day: u8) -> Result<Self, Error> {
        Ok(Self {
            date: Date::try_new(year, month, day)?,
            time: Time::<Tz>::midnight(),
        })
    }
}

impl<Tz> DateTime<Tz>
where
    Tz: TimeZone,
{
    /// Returns a reference to the time component.
    pub fn time(&self) -> &Time<Tz> {
        &self.time
    }

    /// Returns a mutable reference to the time component.
    pub fn time_mut(&mut self) -> &mut Time<Tz> {
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

    /// Unwraps this datetime into its separate [`Date`] and [`Time`] components.
    pub fn into_inner(self) -> (Date, Time<Tz>) {
        (self.date, self.time)
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
    /// # use eos::{DateTime, Utc};
    /// let date = DateTime::<Utc>::new(2012, 1, 15);
    /// assert_eq!(date.year(), 2012);
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
    /// # use eos::{DateTime, Utc};
    /// let date = DateTime::<Utc>::new(2012, 1, 15);
    /// assert_eq!(date.month(), 1);
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
    /// # use eos::{DateTime, Utc};
    /// let date = DateTime::<Utc>::new(2012, 1, 15);
    /// assert_eq!(date.day(), 15);
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
    /// let date = DateTime::<Utc>::new(2013, 3, 17);
    /// let leap = DateTime::<Utc>::new(2012, 3, 17);
    ///
    /// assert_eq!(date.ordinal(), 76);
    /// assert_eq!(leap.ordinal(), 77); // 2012 was a leap year
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
    /// assert_eq!(DateTime::<Utc>::new(2021, 12, 25).weekday(), Weekday::Saturday);
    /// assert_eq!(DateTime::<Utc>::new(2012, 2, 29).weekday(), Weekday::Wednesday);
    /// ```
    pub fn weekday(&self) -> Weekday {
        self.date.weekday()
    }

    /// Modify the date to point to the given year.
    pub fn with_year(&mut self, year: i16) -> &mut Self {
        self.date.with_year(year);
        self
    }

    /// Modify the date to point to the given month.
    ///
    /// # Panics
    ///
    /// Panics if the month is out of bounds (`1..=12`) or if the month
    /// does not have as many days as is currently specified. If this is
    /// undesirable, see [`DateTime::try_with_month`].
    pub fn with_month(&mut self, month: u8) -> &mut Self {
        self.try_with_month(month).expect("out of range month or day for month")
    }

    /// Modify the date to point to the given month.
    ///
    /// This is similar to [`DateTime::with_month`] except [`None`] is returned
    /// when the value is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::{DateTime, Utc};
    /// assert!(DateTime::<Utc>::new(2012, 3, 30).try_with_month(2).is_err());
    /// assert!(DateTime::<Utc>::new(2014, 12, 31).try_with_month(1).is_ok());
    /// assert!(DateTime::<Utc>::new(2019, 4, 28).try_with_month(2).is_ok());
    /// ```
    pub fn try_with_month(&mut self, month: u8) -> Result<&mut Self, Error> {
        self.date.try_with_month(month)?;
        Ok(self)
    }

    /// Modify the date to point to the given day.
    ///
    /// # Panics
    ///
    /// Panics if the day is out of bounds (`1..=31`). Note that the actual maximum
    /// day depends on the specified month. For example, `30` is always invalid with
    /// a month of February since the maximum day for the given month is `29`.
    ///
    /// If this is undesirable, see [`DateTime::try_with_day`].
    pub fn with_day(&mut self, day: u8) -> &mut Self {
        self.try_with_day(day).expect("out of range day")
    }

    /// Modify the date to point to the given day.
    ///
    /// This is similar to [`DateTime::with_day`] except [`None`] is returned
    /// when the value is out of bounds.
    pub fn try_with_day(&mut self, day: u8) -> Result<&mut Self, Error> {
        self.date.try_with_day(day)?;
        Ok(self)
    }
}

impl Add<Duration> for DateTime {
    type Output = DateTime;

    fn add(self, rhs: Duration) -> Self::Output {
        let (days, time) = self.time.add_with_duration(rhs);
        let date = self.date.add_days(days);
        Self { date, time }
    }
}

impl Sub<Duration> for DateTime {
    type Output = DateTime;

    fn sub(self, rhs: Duration) -> Self::Output {
        let (days, time) = self.time.sub_with_duration(rhs);
        let date = self.date.add_days(days);
        Self { date, time }
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
