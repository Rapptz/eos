//! Iterator types for iterating over dates and times.
//!
//! This module contains the named types that are returned in iterators
//! such as [`DateTime::every`].
//!

use crate::{DateTime, Interval, Time, TimeZone, Weekday};

/// An iterator builder over a [`DateTime`] recurrence.
///
/// This struct is created via [`DateTime::every`]. See its documentation
/// for more details.
#[must_use]
pub struct Every<Tz: TimeZone> {
    start: DateTime<Tz>,
    interval: Interval,
    until: Option<DateTime<Tz>>,
    weekday: Option<Weekday>,
    time: Option<Time>,
}

impl<Tz: TimeZone> Every<Tz> {
    pub(crate) fn new(start: DateTime<Tz>, interval: Interval) -> Self {
        Self {
            start,
            interval,
            until: None,
            weekday: None,
            time: None,
        }
    }

    /// Sets the time that the recurrence must fall on.
    ///
    /// Setting the time of the recurrence takes priority over
    /// the initial starting interval. Therefore, for best results it's
    /// recommended not to set an interval that has time shifting values.
    ///
    /// Note that if the time cannot be represented (such as when it's missing)
    /// then it is shifted forward.
    pub fn at(mut self, time: Time) -> Self {
        self.time = Some(time);
        self
    }

    /// Sets the weekday that the recurrence must fall on.
    ///
    /// The date is shifted until it falls on this weekend.
    pub fn on(mut self, weekday: Weekday) -> Self {
        self.weekday = Some(weekday);
        self
    }

    /// Sets the upper bound limit for the recurrence. If this is given
    /// then the final recurrence is less than or equal to this value.
    pub fn until(mut self, dt: DateTime<Tz>) -> Self {
        self.until = Some(dt);
        self
    }

    fn build(mut self) -> EveryIter<Tz> {
        // Check if our initial data needs to be shifted
        if let Some(weekday) = self.weekday {
            if self.start.weekday() != weekday {
                self.start = self.start.next(weekday);
            }
        }
        let fixed = self.start.timezone().is_fixed();

        EveryIter {
            start: self.start,
            interval: self.interval,
            until: self.until,
            weekday: self.weekday,
            first_time: self.time,
            fixed,
        }
    }
}

/// The actual iterator created by [`Every`].
#[must_use]
pub struct EveryIter<Tz: TimeZone> {
    start: DateTime<Tz>,
    interval: Interval,
    until: Option<DateTime<Tz>>,
    weekday: Option<Weekday>,
    first_time: Option<Time>,
    fixed: bool,
}

impl<Tz: TimeZone> IntoIterator for Every<Tz> {
    type Item = DateTime<Tz>;

    type IntoIter = EveryIter<Tz>;

    fn into_iter(self) -> Self::IntoIter {
        self.build()
    }
}

impl<Tz: TimeZone> Iterator for EveryIter<Tz> {
    type Item = DateTime<Tz>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(dt) = &self.until {
            if &self.start >= dt {
                return None;
            }
        }

        let (sub, duration) = self.interval.get_time_duration();
        let (days, time) = if sub {
            self.start.time.sub_with_duration(duration)
        } else {
            self.start.time.add_with_duration(duration)
        };

        let mut date = self
            .start
            .date
            .add_months(self.interval.total_months())
            .add_days(self.interval.days() + days);

        if let Some(weekday) = self.weekday {
            if date.weekday() != weekday {
                date = date.next(weekday);
            }
        }

        let timezone = self.start.timezone.clone();
        let dt = if let Some(first_time) = self.first_time.take() {
            if first_time > self.start.time {
                timezone.resolve(self.start.date, first_time).lenient()
            } else {
                timezone.resolve(date, first_time).lenient()
            }
        } else if self.fixed {
            DateTime {
                date,
                time,
                offset: self.start.offset,
                timezone,
            }
        } else if sub {
            timezone.resolve(date, time).backwards()
        } else {
            timezone.resolve(date, time).lenient()
        };

        self.start.time = dt.time;
        self.start.date = dt.date;
        self.start.offset = dt.offset;
        Some(dt)
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use crate::{datetime, time, Interval};

    const HOUR: Duration = Duration::from_secs(60 * 60);
    const DAY: Duration = Duration::from_secs(60 * 60 * 24);

    #[test]
    fn later_same_day_start() {
        let start = datetime!(2024-12-24 6:00);
        let mut iter = start.every(Interval::from_days(1)).at(time!(07:00)).into_iter();

        let dt = iter.next().unwrap();
        let duration = dt.duration_since(&start);
        assert_eq!(duration, HOUR);

        let dt = iter.next().unwrap();
        let duration = dt.duration_since(&start);
        assert_eq!(duration, DAY + HOUR);
    }

    #[test]
    fn earlier_same_day_start() {
        let start = datetime!(2024-12-24 8:00);
        let mut iter = start.every(Interval::from_days(1)).at(time!(07:00)).into_iter();

        let dt = iter.next().unwrap();
        let duration = dt.duration_since(&start);
        assert_eq!(duration, DAY - HOUR);

        let dt = iter.next().unwrap();
        let duration = dt.duration_since(&start);
        assert_eq!(duration, 2 * DAY - HOUR);
    }

    #[test]
    fn same_same_day_start() {
        let start = datetime!(2024-12-24 7:00);
        let mut iter = start.every(Interval::from_days(1)).at(time!(07:00)).into_iter();

        let dt = iter.next().unwrap();
        let duration = dt.duration_since(&start);
        assert_eq!(duration, DAY);

        let dt = iter.next().unwrap();
        let duration = dt.duration_since(&start);
        assert_eq!(duration, DAY * 2);
    }
}
