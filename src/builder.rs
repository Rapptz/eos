use crate::{Date, DateTime, Error, IsoWeekDate, Time, TimeZone, Weekday};

/// A builder to construct a [`Date`], [`Time`], or [`DateTime`] instance.
///
/// This builder is useful if constructing an instance from multiple different
/// sources of input and there's a need to only to validation at the very end rather
/// than at every step of the way. One exception to this is timezone construction
/// since there would be no way to generalise this.
///
/// If any component is not given then the defaults are as follows:
///
/// - Hour, minute, seconds, and nanoseconds default to `0`.
/// - Month and day default to `1`.
/// - Year defaults to `1970`.
#[derive(Debug, Clone)]
#[must_use]
pub struct Builder<Tz>
where
    Tz: TimeZone,
{
    year: Option<i16>,
    month: u8,
    day: u8,
    ordinal: Option<u16>,
    iso_week: Option<u8>,
    weekday: Option<Weekday>,
    meridiem: Option<AmPm>,
    hour: u8,
    minute: u8,
    second: u8,
    nanosecond: u32,
    pub(crate) timezone: Tz,
}

#[derive(Debug, Clone, Copy)]
enum AmPm {
    Am,
    Pm,
}

impl Builder<crate::Utc> {
    /// Creates a new [`Builder`] with a UTC timezone.
    pub const fn new() -> Self {
        Self {
            year: None,
            month: 1,
            day: 1,
            ordinal: None,
            iso_week: None,
            weekday: None,
            meridiem: None,
            hour: 0,
            minute: 0,
            second: 0,
            nanosecond: 0,
            timezone: crate::Utc,
        }
    }
}

impl<Tz> Builder<Tz>
where
    Tz: TimeZone,
{
    /// Sets the date to the given year.
    ///
    /// This does *not* do any bound checking. The final build step does.
    pub fn year(&mut self, year: i16) -> &mut Self {
        self.year = Some(year);
        self
    }

    /// Sets the date to the given month.
    ///
    /// This does *not* do any bound checking. The final build step does.
    pub fn month(&mut self, month: u8) -> &mut Self {
        self.month = month;
        self
    }

    /// Sets the date to the given day.
    ///
    /// This does *not* do any bound checking. The final build step does.
    pub fn day(&mut self, day: u8) -> &mut Self {
        self.day = day;
        self
    }

    /// Sets the date to the given weekday.
    ///
    /// This does *not* do any bound checking. The final build step does.
    pub fn weekday(&mut self, weekday: Weekday) -> &mut Self {
        self.weekday = Some(weekday);
        self
    }

    /// Sets the date to the given ISO week.
    ///
    /// This does *not* do any bound checking. The final build step does.
    pub fn iso_week(&mut self, iso_week: u8) -> &mut Self {
        self.iso_week = Some(iso_week);
        self
    }

    /// Sets the date to the given ordinal day.
    ///
    /// This does *not* do any bound checking. The final build step does.
    pub fn ordinal(&mut self, ordinal: u16) -> &mut Self {
        self.ordinal = Some(ordinal);
        self
    }

    /// Sets the time to the given hour.
    ///
    /// This does *not* do any bound checking. The final build step does.
    #[inline]
    pub fn hour(&mut self, hour: u8) -> &mut Self {
        self.hour = hour;
        self
    }

    /// Sets the time to use 12-hour clock in `AM`.
    #[inline]
    pub fn am(&mut self) -> &mut Self {
        self.meridiem = Some(AmPm::Am);
        self
    }

    /// Sets the time to use 12-hour clock in `PM`.
    #[inline]
    pub fn pm(&mut self) -> &mut Self {
        self.meridiem = Some(AmPm::Pm);
        self
    }

    /// Sets the time to the given minute.
    ///
    /// This does *not* do any bound checking. The final build step does.
    #[inline]
    pub fn minute(&mut self, minute: u8) -> &mut Self {
        self.minute = minute;
        self
    }

    /// Sets the time to the given second.
    ///
    /// This does *not* do any bound checking. The final build step does.
    #[inline]
    pub fn second(&mut self, second: u8) -> &mut Self {
        self.second = second;
        self
    }

    /// Sets the time to the given millisecond.
    ///
    /// This does *not* do any bound checking. The final build step does.
    #[inline]
    pub fn millisecond(&mut self, millisecond: u16) -> &mut Self {
        self.nanosecond = millisecond as u32 * 1_000_000;
        self
    }

    /// Sets the time to the given microsecond.
    ///
    /// This does *not* do any bound checking. The final build step does.
    #[inline]
    pub fn microsecond(&mut self, microsecond: u32) -> &mut Self {
        self.nanosecond = microsecond * 1_000;
        self
    }

    /// Sets the time to the given nanosecond.
    ///
    /// This does *not* do any bound checking. The final build step does.
    #[inline]
    pub fn nanosecond(&mut self, nanosecond: u32) -> &mut Self {
        self.nanosecond = nanosecond;
        self
    }

    pub(crate) fn fix_leap_seconds(&mut self) {
        if self.second == 60 {
            self.second -= 1;
            self.nanosecond += 1_000_000_000;
        }
    }

    /// Sets the timezone.
    ///
    /// This does *not* do any time modification.
    #[inline]
    pub fn timezone<OtherTz>(self, timezone: OtherTz) -> Builder<OtherTz>
    where
        OtherTz: TimeZone,
    {
        Builder {
            year: self.year,
            month: self.month,
            day: self.day,
            ordinal: self.ordinal,
            iso_week: self.iso_week,
            weekday: self.weekday,
            meridiem: self.meridiem,
            hour: self.hour,
            minute: self.minute,
            second: self.second,
            nanosecond: self.nanosecond,
            timezone,
        }
    }

    /// Builds the final [`DateTime`] with the given components.
    ///
    /// If the components represent an invalid, missing, or ambiguous date time
    /// then an [`Error`] is returned.
    ///
    /// To build a [`Date`], see [`Self::build_date`]. To build a [`Time`],
    /// see [`Self::build_time`].
    pub fn build(&self) -> Result<DateTime<Tz>, Error> {
        let date = self.build_date()?;
        let time = self.build_time()?;
        self.timezone.clone().at_exactly(date, time)
    }

    /// Builds the final [`Date`] with the given components.
    ///
    /// A date is built with the following priority:
    ///
    /// 1. If an ordinal and year is given, then calculate it using that.
    /// 2. If an ISO week and year is given, then calculate using Monday as the weekday.
    /// 3. If an ISO week, weekday, and year is given then calculate using that.
    /// 4. Calculate the date using the provided values or with their defaults.
    ///
    /// If the components represent an invalid date then an [`Error`]
    /// is returned.
    pub fn build_date(&self) -> Result<Date, Error> {
        if let Some((ordinal, year)) = self.ordinal.zip(self.year) {
            Date::from_ordinal(year, ordinal)
        } else if let Some((week, year)) = self.iso_week.zip(self.year) {
            let weekday = self.weekday.unwrap_or(Weekday::Monday);
            let iso_week = IsoWeekDate::new(year, week, weekday)?;
            Ok(Date::from(iso_week))
        } else {
            Date::new(self.year.unwrap_or(1970), self.month, self.day)
        }
    }

    /// Builds the final [`Time`] with the given components.
    ///
    /// If either [`Self::am`] or [`Self::pm`] are called then the time is assumed to be in
    /// 12-hour clock with a range of `1..=12`. If they're not called then 24-hour time
    /// is assumed.
    ///
    /// If the components represent an invalid time then an [`Error`]
    /// is returned.
    pub fn build_time(&self) -> Result<Time, Error> {
        let hour = match self.meridiem {
            Some(AmPm::Am) => {
                if self.hour == 12 {
                    0
                } else {
                    self.hour
                }
            }
            Some(AmPm::Pm) => {
                if self.hour == 12 {
                    12
                } else {
                    self.hour + 12
                }
            }
            None => self.hour,
        };

        Time::new(hour, self.minute, self.second)?.with_nanosecond(self.nanosecond)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_construction() -> Result<(), Error> {
        let dt = Builder::new().month(12).day(31).year(2022).build()?;
        assert_eq!(dt.date(), &Date::new(2022, 12, 31)?);
        assert_eq!(dt.time(), &Time::MIDNIGHT);
        Ok(())
    }

    #[test]
    fn test_ordinal_construction() -> Result<(), Error> {
        let dt = Builder::new().year(2020).ordinal(60).build()?;
        assert_eq!(dt.date(), &Date::new(2020, 2, 29)?);
        assert_eq!(dt.time(), &Time::MIDNIGHT);
        Ok(())
    }

    #[test]
    fn test_iso_week_construction() -> Result<(), Error> {
        let date = Builder::new()
            .year(2020)
            .weekday(Weekday::Wednesday)
            .iso_week(30)
            .build_date()?;

        assert_eq!(date.year(), 2020);
        assert_eq!(date.month(), 7);
        assert_eq!(date.day(), 22);
        Ok(())
    }
}
