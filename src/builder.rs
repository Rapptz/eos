use crate::{Date, DateTime, Error, Time, TimeZone};

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
pub struct Builder<Tz>
where
    Tz: TimeZone,
{
    year: i16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    nanosecond: u32,
    timezone: Tz,
}

impl Builder<crate::Utc> {
    /// Creates a new [`Builder`] with a UTC timezone.
    pub const fn new() -> Self {
        Self {
            year: 1970,
            month: 1,
            day: 1,
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
        self.year = year;
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

    /// Sets the time to the given hour.
    ///
    /// This does *not* do any bound checking. The final build step does.
    #[inline]
    pub fn hour(&mut self, hour: u8) -> &mut Self {
        self.hour = hour;
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
            hour: self.hour,
            minute: self.minute,
            second: self.second,
            nanosecond: self.nanosecond,
            timezone,
        }
    }

    /// Builds the final [`DateTime`] with the given components.
    ///
    /// If the components represent an invalid date or time then an [`Error`]
    /// is returned.
    ///
    /// To build a [`Date`], see [`Self::build_date`]. To build a [`Time`],
    /// see [`Self::build_time`].
    pub fn build(&self) -> Result<DateTime<Tz>, Error> {
        let date = self.build_date()?;
        let time = self.build_time()?;
        Ok(DateTime {
            date,
            time,
            timezone: self.timezone.clone(),
        })
    }

    /// Builds the final [`Date`] with the given components.
    ///
    /// If the components represent an invalid date then an [`Error`]
    /// is returned.
    pub fn build_date(&self) -> Result<Date, Error> {
        Date::new(self.year, self.month, self.day)
    }

    /// Builds the final [`Time`] with the given components.
    ///
    /// If the components represent an invalid time then an [`Error`]
    /// is returned.
    pub fn build_time(&self) -> Result<Time, Error> {
        Time::new(self.hour, self.minute, self.second)?.with_nanosecond(self.nanosecond)
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
}
