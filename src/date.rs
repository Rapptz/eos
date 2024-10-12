use crate::{
    gregorian::{
        date_from_epoch_days, date_to_epoch_days, date_to_ordinal, days_in_month, find_iso_week_start_epoch,
        is_leap_year, iso_week_start_epoch_from_year, iso_weeks_in_year, weekday_from_days,
    },
    step::Advance,
    utils::{divrem, ensure_in_range},
    DateTime, Interval, Time, TimeZone, Utc,
};

use core::ops::{Add, AddAssign, Sub, SubAssign};

#[cfg(feature = "formatting")]
use crate::fmt::ToIsoFormat;

#[cfg(feature = "formatting")]
use alloc::string::{String, ToString};

#[cfg(feature = "parsing")]
use crate::fmt::{FromIsoFormat, ParseError, Parser};

/// An enum representing the different weekdays.
///
/// Due to different orderings of weekdays, this type does not implement `PartialOrd` or `Ord`. Some
/// cultures place either Friday, Saturday, Sunday, or Monday as the first day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
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
    /// Current | `Monday`  | `Tuesday`   | `Wednesday` | `Thursday` | `Friday`   | `Saturday` | `Sunday`
    /// --------|-----------|-------------|-------------|------------|------------|------------|---------
    /// Next    | `Tuesday` | `Wednesday` | `Thursday`  | `Friday`   | `Saturday` | `Sunday`   | `Monday`
    ///
    #[inline]
    #[must_use]
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
    /// Current  | `Monday` | `Tuesday` | `Wednesday` | `Thursday`  | `Friday`   | `Saturday` | `Sunday`
    /// ---------|----------|-----------|-------------|-------------|------------|------------|-----------
    /// Previous | `Sunday` | `Monday`  | `Tuesday`   | `Wednesday` | `Thursday` | `Friday`   | `Saturday`
    ///
    #[inline]
    #[must_use]
    pub const fn prev(self) -> Self {
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

    /// Returns the day of the week number starting from Monday. This is also known as the ISO weekday.
    ///
    /// Current | `Monday` | `Tuesday` | `Wednesday` | `Thursday` | `Friday` | `Saturday` | `Sunday`
    /// --------|----------|-----------|-------------|------------|----------|------------|---------
    /// Number  | 1        | 2         | 3           | 4          | 5        | 6          | 7
    ///
    #[inline]
    #[must_use]
    pub const fn number_from_monday(self) -> u8 {
        self as u8
    }

    /// Returns the day of the week number starting from Sunday.
    ///
    /// Current | `Sunday`| `Monday` | `Tuesday` | `Wednesday` | `Thursday` | `Friday` | `Saturday`
    /// --------|---------|----------|-----------|-------------|------------|----------|------------
    /// Number  | 1       | 2        | 3         | 4           | 5          | 6        | 7
    ///
    #[inline]
    #[must_use]
    pub const fn number_from_sunday(self) -> u8 {
        match self {
            Self::Monday => 2,
            Self::Tuesday => 3,
            Self::Wednesday => 4,
            Self::Thursday => 5,
            Self::Friday => 6,
            Self::Saturday => 7,
            Self::Sunday => 1,
        }
    }

    /// Returns the number of days from Monday.
    ///
    /// Current | `Monday` | `Tuesday` | `Wednesday` | `Thursday` | `Friday` | `Saturday` | `Sunday`
    /// --------|----------|-----------|-------------|------------|----------|------------|---------
    /// Number  | 0        | 1         | 2           | 3          | 4        | 5          | 6
    ///
    #[inline]
    #[must_use]
    pub const fn days_from_monday(self) -> u8 {
        match self {
            Self::Monday => 0,
            Self::Tuesday => 1,
            Self::Wednesday => 2,
            Self::Thursday => 3,
            Self::Friday => 4,
            Self::Saturday => 5,
            Self::Sunday => 6,
        }
    }

    /// Returns the number of days from Sunday.
    ///
    /// Current | `Sunday`| `Monday` | `Tuesday` | `Wednesday` | `Thursday` | `Friday` | `Saturday`
    /// --------|---------|----------|-----------|-------------|------------|----------|------------
    /// Number  | 0       | 1        | 2         | 3           | 4          | 5        | 6
    ///
    #[inline]
    #[must_use]
    pub const fn days_from_sunday(self) -> u8 {
        match self {
            Self::Monday => 1,
            Self::Tuesday => 2,
            Self::Wednesday => 3,
            Self::Thursday => 4,
            Self::Friday => 5,
            Self::Saturday => 6,
            Self::Sunday => 0,
        }
    }
}

/// A date in the [ISO 8601 week date system].
///
/// The ISO week date system is a commonly used variant of the Gregorian calendar, mainly
/// in financial systems and other forms of businesses that revolve around fiscal
/// years.
///
/// The ISO year is made up of either 52 or 53 weeks, where a week always starts on
/// Monday and always ends on Sunday even if the boundary would not make sense
/// in a traditional Gregorian calendar. The first week of an ISO year begins on
/// the Monday following the first Thursday, with the year being the same year
/// as that Thursday.
///
/// [ISO 8601 week date system]: https://en.wikipedia.org/wiki/ISO_week_date
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IsoWeekDate {
    year: i16,
    week: u8,
    weekday: Weekday,
}

impl IsoWeekDate {
    /// Creates a new [`IsoWeekDate`] from the given year, week, and weekday.
    ///
    /// If the week is out of bounds for the given year (53 or higher) then
    /// [`None`] is returned.
    ///
    #[inline]
    pub const fn new(year: i16, week: u8, weekday: Weekday) -> Option<Self> {
        ensure_in_range!(week, 1 => iso_weeks_in_year(year));
        Some(Self { year, week, weekday })
    }

    /// Returns the ISO year.
    ///
    /// Note that the ISO year might be different from the Gregorian year.
    #[inline]
    #[must_use]
    pub const fn year(&self) -> i16 {
        self.year
    }

    /// Returns the ISO week.
    ///
    /// This value will always be within `1..=53`.
    #[inline]
    #[must_use]
    pub const fn week(&self) -> u8 {
        self.week
    }

    /// Returns the ISO weekday.
    #[inline]
    #[must_use]
    pub const fn weekday(&self) -> Weekday {
        self.weekday
    }
}

impl PartialOrd for IsoWeekDate {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IsoWeekDate {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match self.year.cmp(&other.year) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.week.cmp(&other.week) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.weekday
            .number_from_monday()
            .cmp(&other.weekday.number_from_monday())
    }
}

/// A concrete date in the proleptic Gregorian calendar.
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
    pub(crate) year: i16,
    pub(crate) month: u8,
    pub(crate) day: u8,
}

impl Date {
    /// A [`Date`] at the unix epoch (January 1st, 1970).
    pub const UNIX_EPOCH: Self = Self {
        year: 1970,
        month: 1,
        day: 1,
    };

    /// The minimum valid date.
    pub const MIN: Self = Self {
        year: i16::MIN,
        month: 1,
        day: 1,
    };

    /// The maximum valid date.
    pub const MAX: Self = Self {
        year: i16::MAX,
        month: 12,
        day: 31,
    };

    #[doc(hidden)]
    #[cfg(feature = "macros")]
    #[inline]
    pub const fn __new_unchecked_from_macro(year: i16, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    /// Creates a new [`Date`] representing today's date in UTC.
    #[cfg(feature = "std")]
    #[inline]
    #[must_use]
    pub fn today_utc() -> Self {
        let dt = crate::DateTime::utc_now();
        dt.date()
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
    /// # fn test() -> Option<()> {
    /// let date = Date::new(2003, 4, 19)?;
    /// assert_eq!(date.year(), 2003);
    /// assert_eq!(date.month(), 4);
    /// assert_eq!(date.day(), 19);
    /// # Some(())
    /// # }
    /// # test();
    /// ```
    #[inline]
    pub fn new(year: i16, month: u8, day: u8) -> Option<Self> {
        ensure_in_range!(month, 1 => 12);
        ensure_in_range!(day, 1 => days_in_month(year, month));
        Some(Self { year, month, day })
    }

    /// Combines this [`Date`] with a [`Time`] to create a [`DateTime`] in [`Utc`].
    #[inline]
    #[must_use]
    pub fn at(&self, time: Time) -> DateTime<Utc> {
        DateTime {
            date: *self,
            time,
            offset: crate::UtcOffset::UTC,
            timezone: Utc,
        }
    }

    /// Formats this date with a given slice of [`crate::fmt::FormatSpec`].
    ///
    /// Check the [`crate::fmt`] module for more documentation.
    #[cfg(feature = "formatting")]
    pub fn format<'a, 'b, S>(&'a self, spec: S) -> crate::fmt::DateFormatter<'a, 'b, S>
    where
        S: AsRef<[crate::fmt::FormatSpec<'b>]>,
    {
        crate::fmt::DateFormatter::new(self, spec)
    }

    /// Returns the earliest valid [`DateTime`] in the given [`TimeZone`] from this date.
    ///
    /// If midnight can exist unambiguously in this timezone, then it's returned.
    /// If the time is ambiguous, then the earlier time is returned. Otherwise if
    /// there is no midnight then the earliest valid date time is returned. Note that
    /// this could cause an entire day to be skipped in certain (rare) cases, e.g.
    /// `Pacific/Apia` (Samoa) skipped 2011-12-30.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn in_timezone<Tz>(self, tz: Tz) -> DateTime<Tz>
    where
        Tz: TimeZone,
    {
        tz.resolve(self, Time::MIDNIGHT).lenient()
    }

    pub(crate) fn add_days(&self, days: i32) -> Self {
        if days == 0 {
            return *self;
        }

        let days = self.days_since_epoch().saturating_add(days);
        let (year, month, day) = date_from_epoch_days(days);
        Self { year, month, day }
    }

    pub(crate) fn add_months(&self, months: i32) -> Self {
        if months == 0 {
            return *self;
        }

        let m = (self.month as i32 - 1).saturating_add(months);
        let (year, month) = if m >= 0 {
            let (r, q) = divrem!(m, 12);
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

        let year = self.year.saturating_add(years);
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub const fn ordinal(&self) -> u16 {
        date_to_ordinal(self.year, self.month, self.day)
    }

    /// Returns the number of days since the UNIX Epoch (1970-01-01).
    /// # Examples
    ///
    /// ```
    /// # use eos::date;
    /// assert_eq!(date!(1978 - 06 - 19).days_since_epoch(), 3091);
    /// assert_eq!(date!(1969 - 12 - 31).days_since_epoch(), -1);
    /// # Ok::<_, eos::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub const fn days_since_epoch(&self) -> i32 {
        date_to_epoch_days(self.year, self.month, self.day)
    }

    /// Returs the number of days since another date.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::date;
    /// assert_eq!(date!(2024 - 10 - 12).days_since(date!(1978 - 06 - 19)), 16917);
    /// # Ok::<_, eos::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub const fn days_since(&self, other: Date) -> i32 {
        self.days_since_epoch() - other.days_since_epoch()
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
    #[inline]
    #[must_use]
    pub fn weekday(&self) -> Weekday {
        let days = self.days_since_epoch();
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

    /// Returns a [`Date`] moved to the next step of the given value.
    ///
    /// # Examples
    ///
    /// Getting the next weekday:
    ///
    /// ```rust
    /// use eos::{date, Weekday};
    ///
    /// // March 17th 2021 was a Wednesday
    /// assert_eq!(date!(2021-3-17).next(Weekday::Monday), date!(2021-3-22));
    /// assert_eq!(date!(2021-3-17).next(Weekday::Tuesday), date!(2021-3-23));
    /// assert_eq!(date!(2021-3-17).next(Weekday::Wednesday), date!(2021-3-24));
    /// assert_eq!(date!(2021-3-17).next(Weekday::Thursday), date!(2021-3-18));
    /// assert_eq!(date!(2021-3-17).next(Weekday::Friday), date!(2021-3-19));
    /// assert_eq!(date!(2021-3-17).next(Weekday::Saturday), date!(2021-3-20));
    /// assert_eq!(date!(2021-3-17).next(Weekday::Sunday), date!(2021-3-21));
    /// ```
    ///
    /// Getting the next unit:
    ///
    /// ```rust
    /// use eos::{date, unit};
    ///
    /// assert_eq!(date!(2021-3-17).next(unit::Year), date!(2022-3-17));
    /// assert_eq!(date!(2021-3-17).next(unit::Month), date!(2021-4-17));
    /// assert_eq!(date!(2021-3-17).next(unit::Week), date!(2021-3-24));
    /// assert_eq!(date!(2021-3-17).next(unit::Day), date!(2021-3-18));
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn next<A>(self, advance: A) -> Self
    where
        A: Advance<Self>,
    {
        advance.next_from(self)
    }

    /// Returns a [`Date`] moved to the previous step of the given value.
    ///
    /// # Examples
    ///
    /// Getting the previous weekday:
    ///
    /// ```rust
    /// use eos::{date, Weekday};
    ///
    /// // March 17th 2021 was a Wednesday
    /// assert_eq!(date!(2021-3-17).prev(Weekday::Monday), date!(2021-3-15));
    /// assert_eq!(date!(2021-3-17).prev(Weekday::Tuesday), date!(2021-3-16));
    /// assert_eq!(date!(2021-3-17).prev(Weekday::Wednesday), date!(2021-3-10));
    /// assert_eq!(date!(2021-3-17).prev(Weekday::Thursday), date!(2021-3-11));
    /// assert_eq!(date!(2021-3-17).prev(Weekday::Friday), date!(2021-3-12));
    /// assert_eq!(date!(2021-3-17).prev(Weekday::Saturday), date!(2021-3-13));
    /// assert_eq!(date!(2021-3-17).prev(Weekday::Sunday), date!(2021-3-14));
    /// ```
    /// Getting the previous unit:
    ///
    /// ```rust
    /// use eos::{date, unit};
    ///
    /// assert_eq!(date!(2021-3-17).prev(unit::Year), date!(2020-3-17));
    /// assert_eq!(date!(2021-3-17).prev(unit::Month), date!(2021-2-17));
    /// assert_eq!(date!(2021-3-17).prev(unit::Week), date!(2021-3-10));
    /// assert_eq!(date!(2021-3-17).prev(unit::Day), date!(2021-3-16));
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn prev<A>(self, advance: A) -> Self
    where
        A: Advance<Self>,
    {
        advance.prev_from(self)
    }

    /// Returns the ISO week date for this date.
    ///
    /// See [`IsoWeekDate`] for more information.
    ///
    /// Note that the familiar notion of a year is different under the ISO week date.
    ///
    /// ```
    /// use eos::{date, Weekday};
    ///
    /// // January 1st 1995 is a Sunday
    /// let iso = date!(1995-01-01).iso_week();
    ///
    /// assert_eq!(iso.weekday(), Weekday::Sunday);
    /// // Despite being 1995 in Gregorian it is the 52nd week of 1994
    /// assert_eq!(iso.year(), 1994);
    /// assert_eq!(iso.week(), 52);
    ///
    /// // Despite December 31st 1996 being in 1996, it's the 1st week of ISO year 1997.
    /// let iso = date!(1996-12-31).iso_week();
    /// assert_eq!(iso.weekday(), Weekday::Tuesday);
    /// assert_eq!(iso.year(), 1997);
    /// assert_eq!(iso.week(), 1);
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn iso_week(&self) -> IsoWeekDate {
        let epoch = self.days_since_epoch();
        let start_epoch = find_iso_week_start_epoch(self.year, epoch);
        let weekday = weekday_from_days(epoch);
        let week = (epoch - start_epoch) / 7 + 1; // range: [1, 53]
        let (year, _, _) = date_from_epoch_days(start_epoch + 3); // Thursday - Monday = 3

        let weekday = match weekday {
            0 => Weekday::Sunday,
            1 => Weekday::Monday,
            2 => Weekday::Tuesday,
            3 => Weekday::Wednesday,
            4 => Weekday::Thursday,
            5 => Weekday::Friday,
            6 => Weekday::Saturday,
            _ => unreachable!(),
        };

        IsoWeekDate {
            year,
            week: week as _,
            weekday,
        }
    }

    /// Returns a new [`Date] that points to the given year.
    ///
    /// If the year causes the day to go out of bounds, then [`None`]
    /// is returned. For example, switching from a leap year to a non-leap
    /// year on February 29th.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn with_year(mut self, year: i16) -> Option<Self> {
        ensure_in_range!(self.day, 1 => days_in_month(year, self.month));
        self.year = year;
        Some(self)
    }

    /// Returns a new [`Date`] that points to the given month.
    /// If the month is out of bounds (`1..=12`) or if the month
    /// does not have as many days as is currently specified then
    /// an [`None`] is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::date;
    /// assert!(date!(2012-3-30).with_month(2).is_none());
    /// assert!(date!(2014-12-31).with_month(1).is_some());
    /// assert!(date!(2019-4-28).with_month(2).is_some());
    /// # Ok::<_, eos::Error>(())
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn with_month(mut self, month: u8) -> Option<Self> {
        ensure_in_range!(month, 1 => 12);
        ensure_in_range!(self.day, 1 => days_in_month(self.year, month));
        self.month = month;
        Some(self)
    }

    /// Returns a new [`Date`] that points to the given day.
    /// If the day is out of bounds (`1..=31`) then an [`None`] is returned.
    ///
    /// Note that the actual maximum day depends on the specified month.
    /// For example, `30` is always invalid with a month of February since
    /// the maximum day for the given month is `29`.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn with_day(mut self, day: u8) -> Option<Self> {
        ensure_in_range!(day, 1 => days_in_month(self.year, self.month));
        self.day = day;
        Some(self)
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
    /// assert_eq!(Date::from_ordinal(1992, 62), Some(date!(1992-3-2))); // leap year
    /// assert_eq!(Date::from_ordinal(2013, 366), None); // not a leap year
    /// assert_eq!(Date::from_ordinal(2012, 366), Some(date!(2012-12-31)));
    /// assert_eq!(Date::from_ordinal(2012, 59), Some(date!(2012-2-28)));
    /// assert_eq!(Date::from_ordinal(2012, 60), Some(date!(2012-2-29)));
    /// assert_eq!(Date::from_ordinal(2001, 246), Some(date!(2001-9-3)));
    /// # Ok::<_, eos::Error>(())
    /// ```
    pub fn from_ordinal(year: i16, ordinal: u16) -> Option<Self> {
        ensure_in_range!(ordinal, 1 => 366);
        if ordinal == 366 && !is_leap_year(year) {
            return None;
        }

        let epoch = date_to_epoch_days(year, 1, 1) - 1 + ordinal as i32;
        let (year, month, day) = date_from_epoch_days(epoch);
        Some(Self { year, month, day })
    }
}

impl Add<Interval> for Date {
    type Output = Self;

    fn add(self, rhs: Interval) -> Self::Output {
        self.add_months(rhs.total_months()).add_days(rhs.days())
    }
}

impl Sub<Interval> for Date {
    type Output = Self;

    fn sub(self, rhs: Interval) -> Self::Output {
        self.add_months(rhs.total_months().wrapping_neg())
            .add_days(rhs.days().wrapping_neg())
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

impl Sub for Date {
    type Output = Interval;

    fn sub(self, rhs: Self) -> Self::Output {
        Interval::between_dates(&rhs, &self)
    }
}

impl From<IsoWeekDate> for Date {
    fn from(iso: IsoWeekDate) -> Self {
        let epoch = iso_week_start_epoch_from_year(iso.year)
            + (iso.week as i32 - 1) * 7
            + iso.weekday.days_from_monday() as i32;
        let (year, month, day) = date_from_epoch_days(epoch);
        Self { year, month, day }
    }
}

impl core::fmt::Display for Date {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.year >= 0 && self.year <= 9999 {
            write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
        } else {
            write!(f, "{:+05}-{:02}-{:02}", self.year, self.month, self.day)
        }
    }
}

#[cfg(feature = "formatting")]
impl ToIsoFormat for Date {
    fn to_iso_format_with_precision(&self, _precision: crate::fmt::IsoFormatPrecision) -> String {
        self.to_iso_format()
    }

    fn to_iso_format(&self) -> String {
        self.to_string()
    }
}

#[cfg(feature = "parsing")]
impl FromIsoFormat for Date {
    /// Parse an ISO-8601 formatted string to a [`Date`].
    ///
    /// The syntax accepted by this function are:
    ///
    /// - `±YYYYY-MM-DD` (e.g. `2012-02-13` or `-9999-10-12`)
    /// - `±YYYYY-MM` (e.g. `2012-02`)
    /// - `±YYYYY-Www` (e.g. `2012-W10`)
    /// - `±YYYYY-Www-D` (e.g. `2012-W10-1`)
    /// - `±YYYYY-DDD` (e.g. `2021-048`)
    fn from_iso_format(s: &str) -> Result<Self, ParseError> {
        let mut parser = Parser::new(s);
        parser.parse_date()
    }
}

impl core::fmt::Display for IsoWeekDate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:04}-W{:02}-{}",
            self.year,
            self.week,
            self.weekday.number_from_monday()
        )
    }
}

#[cfg(feature = "formatting")]
impl ToIsoFormat for IsoWeekDate {
    fn to_iso_format_with_precision(&self, _precision: crate::fmt::IsoFormatPrecision) -> String {
        self.to_string()
    }

    fn to_iso_format(&self) -> String {
        self.to_string()
    }
}

#[cfg(feature = "parsing")]
impl FromIsoFormat for IsoWeekDate {
    /// Parse an ISO-8601 formatted string to a [`IsoWeekDate`].
    ///
    /// The syntax accepted by this function are:
    ///
    /// - `±YYYYY-Www` (e.g. `2012-W10`)
    /// - `±YYYYY-Www-D` (e.g. `2012-W10-1`)
    fn from_iso_format(s: &str) -> Result<Self, ParseError> {
        let mut parser = Parser::new(s);
        let year = parser.parse_year()?;
        parser.expect(b'-')?;
        parser.expect(b'W')?;
        // week date parsing, i.e. 2012-W10-1
        let week = parser.parse_two_digits()?;
        if week == 0 || week > iso_weeks_in_year(year) {
            return Err(ParseError::OutOfBounds);
        }
        let weekday = match parser.advance_if_equal(b'-') {
            Some(_) => match parser.parse_digit()? {
                1 => Weekday::Monday,
                2 => Weekday::Tuesday,
                3 => Weekday::Wednesday,
                4 => Weekday::Thursday,
                5 => Weekday::Friday,
                6 => Weekday::Saturday,
                7 => Weekday::Sunday,
                _ => return Err(ParseError::OutOfBounds),
            },
            None => Weekday::Monday,
        };
        Ok(Self { year, week, weekday })
    }
}

#[cfg(test)]
mod tests {
    use crate::date;

    use super::*;

    #[test]
    fn test_iso_week() {
        assert_eq!(
            date!(2008 - 12 - 29).iso_week(),
            IsoWeekDate {
                year: 2009,
                week: 1,
                weekday: Weekday::Monday
            }
        );
        assert_eq!(
            Date::from(IsoWeekDate {
                year: 2009,
                week: 1,
                weekday: Weekday::Monday
            }),
            date!(2008 - 12 - 29)
        );
    }
}
