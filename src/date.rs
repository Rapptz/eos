use crate::{
    utils::{
        date_from_epoch_days, date_to_epoch_days, date_to_ordinal, days_in_month, divmod, ensure_in_range,
        is_leap_year, DAYS_BEFORE_MONTH,
    },
    Error, Interval,
};

use core::ops::{Add, AddAssign, Sub, SubAssign};

/// An enum representing the different weekdays.
///
/// Due to different orderings of weekdays, this type does not implement `PartialOrd` or `Ord`. Some
/// cultures place either Friday, Saturday, Sunday, or Monday as the first day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weekday {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 7,
}

impl Weekday {
    /// Returns the next weekday.
    ///
    /// |        Current         |          Next          |
    /// |:----------------------:|:----------------------:|
    /// |  [`Weekday::Monday`]   |  [`Weekday::Tuesday`]  |
    /// |  [`Weekday::Tuesday`]  | [`Weekday::Wednesday`] |
    /// | [`Weekday::Wednesday`] | [`Weekday::Thursday`]  |
    /// | [`Weekday::Thursday`]  |  [`Weekday::Friday`]   |
    /// |  [`Weekday::Friday`]   | [`Weekday::Saturday`]  |
    /// | [`Weekday::Saturday`]  |  [`Weekday::Sunday`]   |
    /// |  [`Weekday::Sunday`]   |  [`Weekday::Monday`]   |
    ///
    pub const fn next(self) -> Self {
        match self {
            Self::Monday => Self::Tuesday,
            Self::Tuesday => Self::Wednesday,
            Self::Wednesday => Self::Thursday,
            Self::Thursday => Self::Friday,
            Self::Friday => Self::Saturday,
            Self::Saturday => Self::Sunday,
            Self::Sunday => Self::Monday,
        }
    }

    /// Return the previous weekday.
    ///
    /// |        Current         |        Previous        |
    /// |:----------------------:|:----------------------:|
    /// |  [`Weekday::Monday`]   |  [`Weekday::Sunday`]   |
    /// |  [`Weekday::Tuesday`]  |  [`Weekday::Monday`]   |
    /// | [`Weekday::Wednesday`] |  [`Weekday::Tuesday`]  |
    /// | [`Weekday::Thursday`]  | [`Weekday::Wednesday`] |
    /// |  [`Weekday::Friday`]   | [`Weekday::Thursday`]  |
    /// | [`Weekday::Saturday`]  |  [`Weekday::Friday`]   |
    /// |  [`Weekday::Sunday`]   | [`Weekday::Saturday`]  |
    ///
    pub const fn prev(&self) -> Self {
        match self {
            Self::Monday => Self::Sunday,
            Self::Tuesday => Self::Monday,
            Self::Wednesday => Self::Tuesday,
            Self::Thursday => Self::Wednesday,
            Self::Friday => Self::Thursday,
            Self::Saturday => Self::Friday,
            Self::Sunday => Self::Saturday,
        }
    }
}

/// Represents a concrete date in the proleptic Gregorian calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    /// There is a possibility of using bit compression to represent dates.
    ///
    /// Since days can only be between 1-31 they have a maximum of 5 bits that can be set.
    /// Coincidentally, this maximum value is 0x1F with all 5 bits set to 1. Likewise, the
    /// month can only be between 1-12 and has a maximum of 4 bits. This leaves the remaining
    /// 23 bits for the year or other bit flags.
    ///
    /// This saves on space but it might make access for common fields a bit slow. The
    /// amount of memory lost is not too bad, especially since the range of a 16-bit type
    /// is large enough for any human being alive right now. Therefore I've opted for the
    /// simplest representation, which should be easier to maintain.
    year: i16,
    month: u8,
    day: u8,
}

impl Date {
    /// Represents a [`Date`] at the unix epoch (January 1st, 1970).
    pub const UNIX_EPOCH: Self = Self {
        year: 1970,
        month: 1,
        day: 1,
    };

    #[doc(hidden)]
    #[cfg(feature = "macros")]
    #[inline]
    pub const fn __new_unchecked_from_macro(year: i16, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    /// Creates a new [`Date`] from a given year, month, and day.
    ///
    /// The month must be between `1..=12` and the day must be between `1..=31`.
    /// Note that the day has to be valid for the specified month, i.e. February
    /// must be either 28 or 29 days depending on the year.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::Date;
    /// let date = Date::new(2003, 4, 19)?;
    /// assert_eq!(date.year(), 2003);
    /// assert_eq!(date.month(), 4);
    /// assert_eq!(date.day(), 19);
    /// # Ok::<_, eos::Error>(())
    /// ```
    pub fn new(year: i16, month: u8, day: u8) -> Result<Self, Error> {
        ensure_in_range!(month, 1 => 12);
        ensure_in_range!(day, 1 => days_in_month(year, month));
        Ok(Self { year, month, day })
    }

    pub(crate) fn add_days(&self, days: i32) -> Self {
        if days == 0 {
            return *self;
        }

        let days = self.epoch_days() + days;
        let (year, month, day) = date_from_epoch_days(days);
        Self { year, month, day }
    }

    pub(crate) fn add_months(&self, months: i32) -> Self {
        if months == 0 {
            return *self;
        }

        let m = self.month as i32 - 1 + months;
        let (year, month) = if m >= 0 {
            let (r, q) = divmod!(m, 12);
            (r, q + 1)
        } else {
            let y = (m / 12) - 1;
            let mut rem = m.abs() % 12;
            if rem == 0 {
                rem = 12;
            }
            let m = 12 - rem + 1;
            if m == 1 {
                (y + 1, m)
            } else {
                (y, m)
            }
        };
        let month = month as u8;
        let year = (self.year as i32 + year).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        let days = days_in_month(year, month);
        let day = days.min(self.day);
        Self { year, month, day }
    }

    pub(crate) fn add_years(&self, years: i16) -> Self {
        if years == 0 {
            return *self;
        }

        let year = self.year + years;
        let days = days_in_month(year, self.month);
        Self {
            year,
            month: self.month,
            day: days.min(self.day),
        }
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
    /// # use eos::date;
    /// let date = date!(2012-01-15);
    /// // or:
    /// // let date = Date::new(2012, 1, 15)?;
    /// assert_eq!(date.year(), 2012);
    /// # Ok::<_, eos::Error>(())
    /// ```
    #[inline]
    pub const fn year(&self) -> i16 {
        self.year
    }

    /// Returns the month.
    ///
    /// This value will always be within `1..=12`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::date;
    /// let date = date!(2012-01-15);
    /// // or:
    /// // let date = Date::new(2012, 1, 15)?;
    /// assert_eq!(date.month(), 1);
    /// # Ok::<_, eos::Error>(())
    /// ```
    #[inline]
    pub const fn month(&self) -> u8 {
        self.month
    }

    /// Returns the day.
    ///
    /// This value will always be within `1..=31`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::date;
    /// let date = date!(2012-01-15);
    /// // or:
    /// // let date = Date::new(2012, 1, 15)?;
    /// assert_eq!(date.day(), 15);
    /// # Ok::<_, eos::Error>(())
    /// ```
    #[inline]
    pub const fn day(&self) -> u8 {
        self.day
    }

    /// Returns the ISO ordinal date.
    ///
    /// January 1st is 1 and December 31st is either 365 or 366 depending on leap year.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::date;
    /// let date = date!(2013-03-17);
    /// let leap = date!(2012-03-17);
    ///
    /// assert_eq!(date.ordinal(), 76);
    /// assert_eq!(leap.ordinal(), 77); // 2012 was a leap year
    /// # Ok::<_, eos::Error>(())
    /// ```
    #[inline]
    pub const fn ordinal(&self) -> u16 {
        date_to_ordinal(self.year, self.month, self.day)
    }

    pub(crate) const fn epoch_days(&self) -> i32 {
        date_to_epoch_days(self.year, self.month, self.day)
    }

    /// Returns the weekday.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::date;
    /// # use eos::Weekday;
    /// assert_eq!(date!(2021-12-25).weekday(), Weekday::Saturday);
    /// assert_eq!(date!(2012-2-29).weekday(), Weekday::Wednesday);
    /// # Ok::<_, eos::Error>(())
    /// ```
    pub fn weekday(&self) -> Weekday {
        let days = self.epoch_days();
        let d = (days + 4).rem_euclid(7) as u8;
        match d {
            0 => Weekday::Sunday,
            1 => Weekday::Monday,
            2 => Weekday::Tuesday,
            3 => Weekday::Wednesday,
            4 => Weekday::Thursday,
            5 => Weekday::Friday,
            6 => Weekday::Saturday,
            // rustc seems incapable of optimising out this panic, not sure why.
            _ => unreachable!(),
        }
    }

    /// Returns a new [`Date] that points to the given year.
    pub fn with_year(mut self, year: i16) -> Self {
        // TODO: needs to error out when switching from e.g. 2012-02-29 -> 2013-02-29
        self.year = year;
        self
    }

    /// Returns a new [`Date`] that points to the given month.
    /// If the month is out of bounds (`1..=12`) or if the month
    /// does not have as many days as is currently specified then
    /// an [`Error`] is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::date;
    /// assert!(date!(2012-3-30).with_month(2).is_err());
    /// assert!(date!(2014-12-31).with_month(1).is_ok());
    /// assert!(date!(2019-4-28).with_month(2).is_ok());
    /// # Ok::<_, eos::Error>(())
    /// ```
    pub fn with_month(mut self, month: u8) -> Result<Self, Error> {
        ensure_in_range!(month, 1 => 12);
        ensure_in_range!(self.day, 1 => days_in_month(self.year, month));
        self.month = month;
        Ok(self)
    }

    /// Returns a new [`Date`] that points to the given day.
    /// If the day is out of bounds (`1..=31`) then an [`Error`] is returned.
    ///
    /// Note that the actual maximum day depends on the specified month.
    /// For example, `30` is always invalid with a month of February since
    /// the maximum day for the given month is `29`.
    pub fn with_day(mut self, day: u8) -> Result<Self, Error> {
        ensure_in_range!(day, 1 => days_in_month(self.year, self.month));
        self.day = day;
        Ok(self)
    }

    /// Creates a date from the given year and ordinal date.
    ///
    /// If the ordinal is out of bounds (`1..=366`) then [`None`] is returned.
    /// Note that 366 is also invalid if the year is not a leap year.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::{Date, date};
    /// assert_eq!(Date::from_ordinal(1992, 62), Ok(date!(1992-3-2))); // leap year
    /// assert!(Date::from_ordinal(2013, 366).is_err()); // not a leap year
    /// assert_eq!(Date::from_ordinal(2012, 366), Ok(date!(2012-12-31)));
    /// assert_eq!(Date::from_ordinal(2001, 246), Ok(date!(2001-9-3)));
    /// # Ok::<_, eos::Error>(())
    /// ```
    pub fn from_ordinal(year: i16, ordinal: u16) -> Result<Self, Error> {
        ensure_in_range!(ordinal, 1 => 366);
        if ordinal == 366 && !is_leap_year(year) {
            return Err(Error::OutOfRange);
        }

        let month = DAYS_BEFORE_MONTH.iter().position(|p| *p > ordinal).unwrap_or(13) - 1;
        let offset = month > 2 && is_leap_year(year);
        let day = ordinal - DAYS_BEFORE_MONTH[month] - offset as u16;
        Ok(Self {
            year,
            month: month as u8,
            day: day as u8,
        })
    }
}

impl Add<Interval> for Date {
    type Output = Self;

    fn add(self, rhs: Interval) -> Self::Output {
        self.add_years(rhs.years())
            .add_months(rhs.months())
            .add_days(rhs.total_days())
    }
}

impl Sub<Interval> for Date {
    type Output = Self;

    fn sub(self, rhs: Interval) -> Self::Output {
        self.add_years(rhs.years().wrapping_neg())
            .add_months(rhs.months().wrapping_neg())
            .add_days(rhs.total_days().wrapping_neg())
    }
}

impl AddAssign<Interval> for Date {
    fn add_assign(&mut self, rhs: Interval) {
        *self = *self + rhs;
    }
}

impl SubAssign<Interval> for Date {
    fn sub_assign(&mut self, rhs: Interval) {
        *self = *self - rhs;
    }
}
