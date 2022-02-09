use crate::{
    step::Advance,
    timestamp::Timestamp,
    timezone::{Utc, UtcOffset},
    utils::divmod,
    Date, IsoWeekDate, System, Time, TimeZone, Weekday,
};
use crate::{Error, Interval};

#[cfg(feature = "system")]
use crate::sys::systemtime;
use core::time::Duration;
use core::{
    cmp::Ordering,
    fmt::Write,
    ops::{Add, Sub},
};

#[cfg(feature = "std")]
use std::time::SystemTime;

#[cfg(feature = "formatting")]
use crate::fmt::{IsoFormatPrecision, ToIsoFormat};

#[cfg(feature = "parsing")]
use crate::fmt::{FromIsoFormat, ParseError, Parser};

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
    pub(crate) date: Date,
    pub(crate) time: Time,
    pub(crate) offset: UtcOffset,
    pub(crate) timezone: Tz,
}

#[doc(hidden)]
#[cfg(feature = "macros")]
#[inline]
pub const fn __create_offset_datetime_from_macro(date: Date, time: Time, timezone: UtcOffset) -> DateTime<UtcOffset> {
    DateTime {
        date,
        time,
        offset: timezone,
        timezone,
    }
}

impl DateTime<Utc> {
    /// A [`DateTime`] at the unix epoch (January 1st, 1970 00:00:00 UTC).
    pub const UNIX_EPOCH: Self = Self {
        date: Date::UNIX_EPOCH,
        time: Time::MIDNIGHT,
        offset: UtcOffset::UTC,
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
            offset: UtcOffset::UTC,
            timezone: Utc,
        }
    }

    /// Shifts the [`DateTime`] by the given [`UtcOffset`].
    ///
    /// Since this function does the operation in-place, this does not
    /// change the timezone. Note that this method is only available on
    /// datetimes with a [`Utc`] timezone since otherwise it would be
    /// too lossy.
    ///
    /// If a change in timezone is required, check the [`DateTime::with_timezone`]
    /// and [`DateTime::in_timezone`] methods. If you want to change a datetime
    /// by an arbitrary interval then check the [`Interval`] class and add the
    /// datetime to that value.
    pub fn shift(&mut self, offset: UtcOffset) {
        let offset_nanos = offset.total_seconds() as i64 * 1_000_000_000;
        let (days, time) = Time::adjust_from_nanos(self.time.total_nanos() as i64 + offset_nanos);
        self.date = self.date.add_days(days);
        self.time = time;
    }
}

impl DateTime<System> {
    /// Returns the current [`DateTime`] in local time.
    #[cfg(feature = "system")]
    #[inline]
    pub fn now() -> Result<Self, Error> {
        let (dt, local) = systemtime::get_system_time_components()?;
        Ok(dt.with_timezone(System(local)))
    }
}

impl DateTime<UtcOffset> {
    /// Parses a [`DateTime`] with a [`UtcOffset`] from an [RFC 3339] string.
    ///
    /// This differs from ISO-8601 parsing by only allowing the basic format:
    ///
    /// - `YYYY-MM-DD HH:MM:SS.ssssss+HH:MM`
    ///
    /// The offset can be negative or positive or `Z`. The fractional seconds
    /// can be ignored and be up to 9 digits of precision. The space separator
    /// between date and time can be a `T` instead.
    ///
    /// [RFC 3339]: https://datatracker.ietf.org/doc/html/rfc3339
    #[cfg(feature = "parsing")]
    pub fn from_rfc3339(s: &str) -> Result<Self, ParseError> {
        let mut parser = Parser::new(s);
        let year = parser.parse_year()?;
        parser.expect(b'-')?;
        let month = parser.parse_month()?;
        parser.expect(b'-')?;
        let day = parser.parse_two_digits()?;
        let date = Date::new(year, month, day).map_err(|_| ParseError::OutOfBounds)?;
        match parser.advance() {
            Some(b' ' | b'T') => {}
            Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
            None => return Err(ParseError::UnexpectedEnd),
        }
        let time = parser.parse_time()?;
        let offset = match parser.advance() {
            Some(b'Z') => UtcOffset::UTC,
            Some(x @ b'+' | x @ b'-') => {
                let negative = x == b'-';
                let hours = parser.parse_two_digits()? as i8;
                parser.expect(b':')?;
                let minutes = parser.parse_two_digits()? as i8;
                if hours > 23 || minutes > 59 {
                    return Err(ParseError::OutOfBounds);
                }
                if negative {
                    UtcOffset {
                        hours: -hours,
                        minutes: -minutes,
                        seconds: 0,
                    }
                } else {
                    UtcOffset {
                        hours,
                        minutes,
                        seconds: 0,
                    }
                }
            }
            Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
            None => return Err(ParseError::UnexpectedEnd),
        };

        Ok(Self {
            date,
            time,
            offset,
            timezone: offset,
        })
    }

    /// Parses a [`DateTime`] with a given slice of [`crate::fmt::FormatSpec`].
    ///
    /// Check the [`crate::fmt`] module for more information.
    ///
    /// ```
    /// use eos::{datetime, DateTime, fmt::format_spec};
    ///
    /// let input = "2022-09-14 1:00 PM";
    /// let dt = DateTime::parse_from_spec(input, format_spec!("%Y-%m-%d %I:%M %p"));
    /// assert!(dt.is_ok());
    /// assert_eq!(dt?, datetime!(2022-09-14 13:00 +00:00));
    /// # Ok::<_, eos::fmt::ParseError>(())
    /// ```
    #[cfg(feature = "parsing")]
    pub fn parse_from_spec<'a, T, S>(s: T, spec: S) -> Result<Self, ParseError>
    where
        T: AsRef<str>,
        S: AsRef<[crate::fmt::FormatSpec<'a>]>,
    {
        let spec = spec.as_ref();
        let mut parser = Parser::new(s.as_ref());
        let mut builder = crate::Builder::new().timezone(UtcOffset::UTC);
        for s in spec {
            s.parse_into(&mut builder, &mut parser)?;
        }
        builder.fix_leap_seconds();
        builder.build().map_err(|_| ParseError::OutOfBounds)
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
        let date = Date::new(year, month, day)?;
        let time = Time::MIDNIGHT;
        Ok(Tz::default().resolve(date, time).lenient())
    }
}

impl<Tz> DateTime<Tz>
where
    Tz: TimeZone,
{
    /// Creates a [`DateTime`] from the given year, ordinal date, and timezone. The time is set to
    /// midnight UTC.
    ///
    /// If the ordinal is out of bounds (`1..=366`) then [`None`] is returned.
    /// Note that 366 is also invalid if the year is not a leap year.
    ///
    /// # Examples
    ///
    /// ```
    /// use eos::{DateTime, Utc};
    /// assert_eq!(DateTime::from_ordinal(1992, 62, Utc), Ok(DateTime::<Utc>::new(1992, 3, 2)?)); // leap year
    /// assert!(DateTime::from_ordinal(2013, 366, Utc).is_err()); // not a leap year
    /// assert_eq!(DateTime::from_ordinal(2012, 366, Utc), Ok(DateTime::<Utc>::new(2012, 12, 31)?));
    /// assert_eq!(DateTime::from_ordinal(2001, 246, Utc), Ok(DateTime::<Utc>::new(2001, 9, 3)?));
    /// # Ok::<_, eos::Error>(())
    /// ```
    pub fn from_ordinal(year: i16, ordinal: u16, timezone: Tz) -> Result<Self, Error> {
        let date = Date::from_ordinal(year, ordinal)?;
        let time = Time::MIDNIGHT;
        Ok(timezone.resolve(date, time).lenient())
    }

    /// Creates a [`DateTime`] from a POSIX timestamp in seconds, a nanosecond component, and a timezone.
    ///
    /// ```
    /// use eos::{datetime, utc_offset, DateTime, Utc, Timestamp};
    /// assert_eq!(
    ///     DateTime::from_timestamp(Timestamp::from_seconds(0), Utc),
    ///     DateTime::UNIX_EPOCH
    /// );
    /// assert_eq!(
    ///     DateTime::from_timestamp(Timestamp::from_seconds(1641173925), Utc),
    ///     datetime!(2022-01-03 1:38:45)
    /// );
    /// assert_eq!(
    ///     DateTime::from_timestamp(Timestamp::from_seconds(1641173925), utc_offset!(-05:00)),
    ///     datetime!(2022-01-02 20:38:45 -05:00)
    /// );
    /// # Ok::<_, eos::Error>(())
    /// ```
    pub fn from_timestamp(timestamp: Timestamp, timezone: Tz) -> Self {
        timezone.convert_utc(timestamp.to_utc())
    }

    /// Creates a [`DateTime`] representing the current day at midnight.
    #[cfg(feature = "std")]
    pub fn today(tz: Tz) -> Self {
        DateTime::utc_now().in_timezone(tz).with_time(Time::MIDNIGHT)
    }

    /// Formats this datetime with a given slice of [`crate::fmt::FormatSpec`].
    ///
    /// Check the [`crate::fmt`] module for more documentation.
    #[cfg(feature = "formatting")]
    pub fn format<'a, 'b, S>(&'a self, spec: S) -> crate::fmt::DateTimeFormatter<'a, 'b, Tz, S>
    where
        S: AsRef<[crate::fmt::FormatSpec<'b>]>,
    {
        crate::fmt::DateTimeFormatter::new(self, spec)
    }

    /// Formats this datetime using [RFC 3339] formatting rules.
    ///
    /// This is mostly the same as ISO-8601 except a space is used
    /// instead of a `T` for the separator between the date and time.
    /// Microsecond precision is used for the fractional component
    /// rather than nanoseconds. Likewise, the seconds component of
    /// the UTC offset is always ignored.
    ///
    /// [RFC 3339]: https://datatracker.ietf.org/doc/html/rfc3339
    #[cfg(feature = "formatting")]
    pub fn to_rfc3339(&self) -> crate::fmt::Rfc3339Formatter<'_, Tz> {
        crate::fmt::Rfc3339Formatter { dt: self }
    }

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

    /// Returns a reference to the [`UtcOffset`] this datetime currently resides in.
    pub fn offset(&self) -> &UtcOffset {
        &self.offset
    }

    /// Returns the time zone name, if any.
    pub fn tzname(&self) -> Option<&str> {
        self.timezone.name(self.timestamp())
    }

    /// Compares two datetime instances that do not share a timezone.
    ///
    /// Due to [a limitation][bad-ord] with the Rust [`Ord`] trait, this cannot be implemented
    /// using the standard trait. Therefore, `cmp` is only implemented on same-type
    /// [`TimeZone`] datetimes.
    ///
    /// [bad-ord]: https://github.com/rust-lang/rfcs/issues/2511
    pub fn cmp_cross_timezone<OtherTz>(&self, other: &DateTime<OtherTz>) -> Ordering
    where
        OtherTz: TimeZone,
    {
        if self.offset == other.offset {
            return (self.date, self.time).cmp(&(other.date, other.time));
        }

        // Get how many days the two dates differ by and also whether
        // they are the same "moment" after accounting for their UTC offsets
        let (days, same) = {
            // Initially this is just a difference in days via epoch
            // If the UTC offsets at that date resolve to the same one, then they're on the same
            // date and the calculation is correct
            let days = self.date().days_since_epoch() - other.date().days_since_epoch();
            if self.offset == other.offset {
                (days, true)
            } else {
                // If they're not, then we need to take into consideration the time component.
                // A time component is essentially constrained to 24 hours
                // So the difference in seconds between two of them won't go too much over
                // 24 hours, we can use this to our advantage because by the same virtue
                // offsets are also only within 24 hour bounds, making this a rather simple
                // second-wise addition
                let delta_offsets = other.offset.total_seconds() - self.offset.total_seconds();
                let seconds = self.time.total_seconds() - other.time.total_seconds() + delta_offsets;

                let (d, s) = divmod!(seconds, 86_400);
                (
                    days + d,
                    s == 0 && self.time().nanosecond() == other.time().nanosecond(),
                )
            }
        };

        // If the number of days we differ by is negative then lhs is less than rhs by virtue

        if days < 0 {
            Ordering::Less
        } else if days == 0 && same {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }

    /// Compares two datetime instances without caring about their timezone information.
    ///
    /// This essentially just compares their individual [`Date`] and [`Time`] components.
    pub fn cmp_without_tz<OtherTz>(&self, other: &DateTime<OtherTz>) -> Ordering
    where
        OtherTz: TimeZone,
    {
        match self.date.cmp(&other.date) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.time.cmp(&other.time)
    }

    /// Returns an iterator builder to create a recurrent range over date times.
    ///
    /// At its most basic form, it allows you to iterate as if repeatedly adding
    /// the given interval. However, it can also be chained to create certain
    /// conditions to iterate over.
    ///
    /// It's important to note that this does not actually implement the entire
    /// [`rrule`] specification since it differs from this function when it comes
    /// to missing or ambiguous times and is more complicated than this method.
    ///
    /// [`rrule`]: https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.10
    ///
    /// # Examples
    ///
    /// Getting every 3 days starting from now:
    ///
    /// ```ignore
    /// let now = Utc::now();
    /// for dt in now.every(3.days()) {
    ///     println!("{}", dt);
    /// }
    /// ```
    ///
    /// Getting the first 5 of the previous iteration:
    ///
    /// ```ignore
    /// let now = Utc::now();
    /// for dt in now.every(3.days()).into_iter().take(5) {
    ///     println!("{}", dt);
    /// }
    /// ```
    ///
    /// Getting every week on Sunday:
    ///
    /// ```ignore
    /// let now = Utc::now();
    /// for dt in now.every(1.weeks()).on(Weekday::Sunday).into_iter().take(5) {
    ///     println!("{}", dt);
    /// }
    /// ```
    ///
    /// Getting every 5 days at 3:30 AM:
    ///
    /// ```ignore
    /// let now = Utc::now();
    /// for dt in now.every(5.days()).at(time!(03:30)).into_iter().take(10) {
    ///     println!("{}", dt);
    /// }
    /// ```
    ///
    /// Getting every 3 days until a limit date is reached:
    ///
    /// ```ignore
    /// let now = Utc::now();
    /// let until = datetime!(2023-04-10);
    /// for dt in now.every(3.days()).at(time!(03:30)).until(until) {
    ///     println!("{}", dt);
    /// }
    /// ```
    pub fn every(self, interval: Interval) -> crate::iter::Every<Tz> {
        crate::iter::Every::new(self, interval)
    }

    #[inline]
    pub(crate) fn into_utc(self) -> DateTime<Utc> {
        let offset = self.offset; // Copy value before moving
        let mut utc = DateTime {
            date: self.date,
            time: self.time,
            offset: UtcOffset::UTC,
            timezone: Utc,
        };
        utc.shift(-offset);
        utc
    }

    /// Returns a new [`DateTime`] with the newly specified [`TimeZone`],
    /// adjusting the date and time components to point to the same internal UTC
    /// time but in the given timezone's local time.
    ///
    /// If you merely want to change the internal timezone without making adjustments
    /// for the date and time, then [`DateTime::with_timezone`] should be used instead.
    pub fn in_timezone<OtherTz>(self, timezone: OtherTz) -> DateTime<OtherTz>
    where
        OtherTz: TimeZone,
    {
        timezone.convert_utc(self.into_utc())
    }

    /// Returns a new [`DateTime`] with the timezone component changed.
    /// This attempts to retain the local date and time as much as possible.
    /// The local date time is only changed if the date time cannot represented
    /// in that timezone, such as when there's a gap due to a transition.
    ///
    /// If you want to change the local date time and the timezone then
    /// [`DateTime::in_timezone`] should be used instead.
    pub fn with_timezone<OtherTz>(self, timezone: OtherTz) -> DateTime<OtherTz>
    where
        OtherTz: TimeZone,
    {
        timezone.resolve(self.date, self.time).lenient()
    }

    /// Returns a new [`DateTime`] with the newly specified [`TimeZone`],
    /// adjusting the date and time components to point to the same internal UTC
    /// time but in the given timezone's local time.
    ///
    /// This is an alias to [`Self::in_timezone`].
    pub fn at<OtherTz>(self, timezone: OtherTz) -> DateTime<OtherTz>
    where
        OtherTz: TimeZone,
    {
        timezone.convert_utc(self.into_utc())
    }

    /// Returns the UNIX timestamp.
    pub fn timestamp(&self) -> Timestamp {
        let seconds = self.days_since_epoch() as i64 * 86400
            + self.hour() as i64 * 3600
            + self.minute() as i64 * 60
            + self.second() as i64
            // This is deliberately subtracted
            // e.g. UTC-5 means we need to add +5 to get back to UTC.
            - self.offset.total_seconds() as i64;

        Timestamp::new(seconds, self.nanosecond())
    }

    pub(crate) fn add_months(mut self, months: i32) -> Self {
        self.date = self.date.add_months(months);
        self
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

    /// Returns the number of days since the UNIX Epoch (1970-01-01).
    pub fn days_since_epoch(&self) -> i32 {
        self.date().days_since_epoch()
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

    /// Returns a [`DateTime`] moved to the next step of the given value.
    ///
    /// # Examples
    ///
    /// Getting the next weekday:
    ///
    /// ```rust
    /// use eos::{datetime, Weekday};
    ///
    /// // March 17th 2021 was a Wednesday
    /// assert_eq!(datetime!(2021-3-17 00:00).next(Weekday::Monday), datetime!(2021-3-22 00:00));
    /// assert_eq!(datetime!(2021-3-17 00:00).next(Weekday::Tuesday), datetime!(2021-3-23 00:00));
    /// assert_eq!(datetime!(2021-3-17 00:00).next(Weekday::Wednesday), datetime!(2021-3-24 00:00));
    /// assert_eq!(datetime!(2021-3-17 00:00).next(Weekday::Thursday), datetime!(2021-3-18 00:00));
    /// assert_eq!(datetime!(2021-3-17 00:00).next(Weekday::Friday), datetime!(2021-3-19 00:00));
    /// assert_eq!(datetime!(2021-3-17 00:00).next(Weekday::Saturday), datetime!(2021-3-20 00:00));
    /// assert_eq!(datetime!(2021-3-17 00:00).next(Weekday::Sunday), datetime!(2021-3-21 00:00));
    /// ```
    ///
    /// Getting the next time:
    ///
    /// ```rust
    /// use eos::{datetime, time};
    ///
    /// assert_eq!(datetime!(2021-3-17 02:00).next(time!(03:00)), datetime!(2021-3-17 03:00));
    /// assert_eq!(datetime!(2021-3-17 02:00).next(time!(02:00)), datetime!(2021-3-18 02:00));
    /// ```
    ///
    /// Getting the next unit:
    ///
    /// ```rust
    /// use eos::{datetime, unit};
    ///
    /// assert_eq!(datetime!(2021-3-17 02:00).next(unit::Year), datetime!(2022-3-17 02:00));
    /// assert_eq!(datetime!(2021-3-17 02:00).next(unit::Month), datetime!(2021-4-17 02:00));
    /// assert_eq!(datetime!(2021-3-17 02:00).next(unit::Week), datetime!(2021-3-24 02:00));
    /// assert_eq!(datetime!(2021-3-17 02:00).next(unit::Day), datetime!(2021-3-18 02:00));
    /// assert_eq!(datetime!(2021-3-17 02:00).next(unit::Hour), datetime!(2021-3-17 03:00));
    /// assert_eq!(datetime!(2021-3-17 02:00).next(unit::Minute), datetime!(2021-3-17 02:01));
    /// assert_eq!(datetime!(2021-3-17 02:00).next(unit::Second), datetime!(2021-3-17 02:00:01));
    /// ```
    pub fn next<A>(self, advance: A) -> Self
    where
        A: Advance<Self>,
    {
        advance.next_from(self)
    }

    /// Returns a [`DateTime`] moved to the previous step of the given value.
    ///
    /// # Examples
    ///
    /// Getting the previous weekday:
    ///
    /// ```rust
    /// use eos::{datetime, Weekday};
    ///
    /// // March 17th 2021 was a Wednesday
    /// assert_eq!(datetime!(2021-3-17 00:00).prev(Weekday::Monday), datetime!(2021-3-15 00:00));
    /// assert_eq!(datetime!(2021-3-17 00:00).prev(Weekday::Tuesday), datetime!(2021-3-16 00:00));
    /// assert_eq!(datetime!(2021-3-17 00:00).prev(Weekday::Wednesday), datetime!(2021-3-10 00:00));
    /// assert_eq!(datetime!(2021-3-17 00:00).prev(Weekday::Thursday), datetime!(2021-3-11 00:00));
    /// assert_eq!(datetime!(2021-3-17 00:00).prev(Weekday::Friday), datetime!(2021-3-12 00:00));
    /// assert_eq!(datetime!(2021-3-17 00:00).prev(Weekday::Saturday), datetime!(2021-3-13 00:00));
    /// assert_eq!(datetime!(2021-3-17 00:00).prev(Weekday::Sunday), datetime!(2021-3-14 00:00));
    /// ```
    ///
    /// Getting the previous time:
    ///
    /// ```rust
    /// use eos::{datetime, time};
    ///
    /// assert_eq!(datetime!(2021-3-17 04:00).prev(time!(03:00)), datetime!(2021-3-17 03:00));
    /// assert_eq!(datetime!(2021-3-17 03:00).prev(time!(03:00)), datetime!(2021-3-16 03:00));
    /// ```
    ///
    /// Getting the previous unit:
    ///
    /// ```rust
    /// use eos::{datetime, unit};
    ///
    /// assert_eq!(datetime!(2021-3-17 02:00).prev(unit::Year), datetime!(2020-3-17 02:00));
    /// assert_eq!(datetime!(2021-3-17 02:00).prev(unit::Month), datetime!(2021-2-17 02:00));
    /// assert_eq!(datetime!(2021-3-17 02:00).prev(unit::Week), datetime!(2021-3-10 02:00));
    /// assert_eq!(datetime!(2021-3-17 02:00).prev(unit::Day), datetime!(2021-3-16 02:00));
    /// assert_eq!(datetime!(2021-3-17 02:00).prev(unit::Hour), datetime!(2021-3-17 01:00));
    /// assert_eq!(datetime!(2021-3-17 02:00).prev(unit::Minute), datetime!(2021-3-17 01:59));
    /// assert_eq!(datetime!(2021-3-17 02:00).prev(unit::Second), datetime!(2021-3-17 01:59:59));
    /// ```
    pub fn prev<A>(self, advance: A) -> Self
    where
        A: Advance<Self>,
    {
        advance.prev_from(self)
    }

    /// Returns the ISO week date for this datetime.
    ///
    /// See [`IsoWeekDate`] for more information.
    ///
    /// Note that the familiar notion of a year is different under the ISO week date.
    ///
    /// ```
    /// use eos::{datetime, Weekday};
    ///
    /// // January 1st 1995 is a Sunday
    /// let iso = datetime!(1995-01-01 00:00).iso_week();
    ///
    /// assert_eq!(iso.weekday(), Weekday::Sunday);
    /// // Despite being 1995 in Gregorian it is the 52nd week of 1994
    /// assert_eq!(iso.year(), 1994);
    /// assert_eq!(iso.week(), 52);
    ///
    /// // Despite December 31st 1996 being in 1996, it's the 1st week of ISO year 1997.
    /// let iso = datetime!(1996-12-31 00:00).iso_week();
    /// assert_eq!(iso.weekday(), Weekday::Tuesday);
    /// assert_eq!(iso.year(), 1997);
    /// assert_eq!(iso.week(), 1);
    /// ```
    #[inline]
    pub fn iso_week(&self) -> IsoWeekDate {
        self.date.iso_week()
    }

    /// Returns a new [`DateTime`] with the date pointing to the given year.
    ///
    /// If the year causes the day to go out of bounds, then [`Error`]
    /// is returned. For example, switching from a leap year to a non-leap
    /// year on February 29th.
    pub fn with_year(mut self, year: i16) -> Result<Self, Error> {
        self.date = self.date.with_year(year)?;
        Ok(self)
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

impl<Tz> core::fmt::Display for DateTime<Tz>
where
    Tz: TimeZone,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}T{}{}", self.date, self.time, self.offset)
    }
}

#[cfg(feature = "formatting")]
impl<Tz> ToIsoFormat for DateTime<Tz>
where
    Tz: TimeZone,
{
    fn to_iso_format_with_precision(&self, precision: IsoFormatPrecision) -> String {
        let mut buffer = String::with_capacity(40);
        write!(&mut buffer, "{}", &self.date).expect("unexpected error when writing string");
        buffer.push('T');
        crate::time::fmt_iso_time(&mut buffer, &self.time, precision).expect("unexpected error when writing string");
        write!(&mut buffer, "{}", self.offset).expect("unexpected error when writing string");
        buffer
    }

    fn to_iso_format(&self) -> String {
        self.to_string()
    }
}

#[cfg(feature = "parsing")]
impl FromIsoFormat for DateTime<UtcOffset> {
    /// Parses an ISO-8601 formatted string into a [`DateTime`] with a [`UtcOffset`].
    ///
    /// This accepts the formats accepted by [`Date`], followed with a `'T'`,
    /// then the formats accepted by [`Time`] and an optional UTC offset.
    ///
    /// The UTC offset syntax must be in the following forms:
    ///
    /// - `±HH` (e.g. `+12`)
    /// - `±HH:MM` (e.g. `+12:23`)
    /// - `±HH:MM:SS` (e.g. `+12:23:45`).
    /// - `Z` (represents UTC)
    ///
    /// Note that strict ISO-8601 compliance would forbid the seconds component and would
    /// make the `:` optional. This function does not currently accept such syntax.
    fn from_iso_format(s: &str) -> Result<Self, ParseError> {
        let mut parser = Parser::new(s);
        let date = parser.parse_date()?;
        parser.expect(b'T')?;
        let time = parser.parse_time()?;
        let offset = if let None | Some(b'Z') = parser.peek() {
            UtcOffset::UTC
        } else {
            let negative = parser.parse_required_sign()?;
            let hours = parser.parse_two_digits()? as i8;
            if hours > 23 {
                return Err(ParseError::OutOfBounds);
            }

            let (minutes, seconds) = match parser.advance_if_equal(b':') {
                Some(_) => {
                    let minute = parser.parse_two_digits()? as i8;
                    if minute > 59 {
                        return Err(ParseError::OutOfBounds);
                    }
                    match parser.advance_if_equal(b':') {
                        Some(_) => {
                            let second = parser.parse_two_digits()? as i8;
                            if second > 59 {
                                return Err(ParseError::OutOfBounds);
                            }
                            (minute, second)
                        }
                        None => (minute, 0),
                    }
                }
                None => (0, 0),
            };

            if negative {
                UtcOffset {
                    hours: -hours,
                    minutes: -minutes,
                    seconds: -seconds,
                }
            } else {
                UtcOffset {
                    hours,
                    minutes,
                    seconds,
                }
            }
        };

        Ok(Self {
            date,
            time,
            offset,
            timezone: offset,
        })
    }
}

impl Add<Duration> for DateTime {
    type Output = DateTime;

    fn add(self, rhs: Duration) -> Self::Output {
        let (days, time) = self.time.add_with_duration(rhs);
        let date = self.date.add_days(days);
        if self.timezone.is_fixed() {
            DateTime {
                date,
                time,
                offset: self.offset,
                timezone: self.timezone,
            }
        } else {
            self.timezone.resolve(date, time).lenient()
        }
    }
}

impl Sub<Duration> for DateTime {
    type Output = DateTime;

    fn sub(self, rhs: Duration) -> Self::Output {
        let (days, time) = self.time.sub_with_duration(rhs);
        let date = self.date.add_days(days);
        if self.timezone.is_fixed() {
            DateTime {
                date,
                time,
                offset: self.offset,
                timezone: self.timezone,
            }
        } else {
            self.timezone.resolve(date, time).backwards()
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
        self.cmp_cross_timezone(other) == Ordering::Equal
    }
}

// Rust does not support Eq<Rhs> for some reason
impl<Tz> Eq for DateTime<Tz> where Tz: TimeZone {}

impl<Tz, OtherTz> PartialOrd<DateTime<OtherTz>> for DateTime<Tz>
where
    Tz: TimeZone,
    OtherTz: TimeZone,
{
    fn partial_cmp(&self, other: &DateTime<OtherTz>) -> Option<Ordering> {
        Some(self.cmp_cross_timezone(other))
    }
}

// Rust does not allow Ord<Rhs> for some reason
// see: https://github.com/rust-lang/rfcs/issues/2511
impl<Tz> Ord for DateTime<Tz>
where
    Tz: TimeZone,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_cross_timezone(other)
    }
}

impl<Tz> Add<Interval> for DateTime<Tz>
where
    Tz: TimeZone,
{
    type Output = Self;

    fn add(self, rhs: Interval) -> Self::Output {
        let (sub, duration) = rhs.get_time_duration();
        let (days, time) = if sub {
            self.time.sub_with_duration(duration)
        } else {
            self.time.add_with_duration(duration)
        };

        let date = self.date.add_months(rhs.total_months()).add_days(rhs.days() + days);
        if self.timezone.is_fixed() {
            DateTime {
                date,
                time,
                offset: self.offset,
                timezone: self.timezone,
            }
        } else if sub {
            self.timezone.resolve(date, time).backwards()
        } else {
            self.timezone.resolve(date, time).lenient()
        }
    }
}

impl<Tz> Sub<Interval> for DateTime<Tz>
where
    Tz: TimeZone,
{
    type Output = Self;

    fn sub(self, rhs: Interval) -> Self::Output {
        let (sub, duration) = rhs.get_time_duration();
        let (days, time) = if sub {
            self.time.add_with_duration(duration)
        } else {
            self.time.sub_with_duration(duration)
        };

        #[allow(clippy::suspicious_arithmetic_impl)]
        let date = self
            .date
            .add_months(rhs.total_months().wrapping_neg())
            .add_days(rhs.days().wrapping_neg() + days);

        if self.timezone.is_fixed() {
            DateTime {
                date,
                time,
                offset: self.offset,
                timezone: self.timezone,
            }
        } else if sub {
            self.timezone.resolve(date, time).lenient()
        } else {
            self.timezone.resolve(date, time).backwards()
        }
    }
}

impl<Tz, OtherTz> Sub<DateTime<OtherTz>> for DateTime<Tz>
where
    Tz: TimeZone,
    OtherTz: TimeZone,
{
    type Output = Interval;

    fn sub(self, rhs: DateTime<OtherTz>) -> Self::Output {
        Interval::between(&rhs, &self)
    }
}

// Some basic tests to ensure comparisons work

#[cfg(test)]
// TODO: remove when this is standardised inside clippy
#[allow(clippy::eq_op)]
mod tests {
    use super::*;
    use crate::{datetime, utc_offset};

    #[test]
    fn test_regular_comparisons() {
        let dt = datetime!(2012-01-12 00:00);
        assert_eq!(dt, dt);
        assert_ne!(dt, datetime!(2012-02-13 00:00));
        let tomorrow = datetime!(2012-01-13 00:00);
        assert!(dt < tomorrow);
        assert!(tomorrow > dt);
        assert!(dt <= tomorrow);
        assert!(dt != tomorrow);
        assert!(tomorrow >= dt);
    }

    #[test]
    fn test_mixed_timezone_comparisons() {
        let dt = datetime!(2000-01-02 03:04:05 +3:00);
        let utc = datetime!(2000-01-02 00:04:05);
        let off = utc.with_hour(3).unwrap();

        assert!(dt == dt);
        assert!(dt == utc);
        assert!(dt != off);
        assert!(off != utc);
        assert!(off == off);
        assert!(off > dt);
        assert!(dt < off);
        assert!(off >= dt);
        assert!(off >= utc);
        assert!(off > utc);
        assert!(utc < off);

        assert_eq!(dt.cmp_without_tz(&off), Ordering::Equal);
        assert_eq!(off.cmp_without_tz(&dt), Ordering::Equal);
        assert_eq!(dt.cmp_without_tz(&utc), Ordering::Greater);
        assert_eq!(utc.cmp_without_tz(&dt), Ordering::Less);
        assert_eq!(off.cmp_without_tz(&utc), Ordering::Greater);
        assert_eq!(utc.cmp_without_tz(&off), Ordering::Less);

        let utc = datetime!(2021-12-31 00:00);
        let offset = utc_offset!(-5:00);
        let left = offset.convert_utc(utc);
        assert_eq!(left, utc);
    }

    #[test]
    fn test_timestamp() {
        assert_eq!(datetime!(1970-01-01 00:00).timestamp().as_seconds(), 0);
        assert_eq!(datetime!(1970-01-01 1:02:03).timestamp().as_seconds(), 3723);
        assert_eq!(datetime!(2022-01-02 20:38:45).timestamp().as_seconds(), 1641155925);
        assert_eq!(
            datetime!(2022-01-02 20:38:45 -5:00).timestamp().as_seconds(),
            1641173925
        );
    }

    #[test]
    fn test_tzname() {
        assert_eq!(DateTime::UNIX_EPOCH.tzname(), Some("UTC"));
    }
}
