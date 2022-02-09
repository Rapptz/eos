use crate::{unit, Date, DateTime, Time, TimeZone, Weekday};
use core::time::Duration;

mod private {
    pub use super::*;
    pub trait Sealed {}
    impl Sealed for Weekday {}
    impl Sealed for Time {}
    impl Sealed for unit::Year {}
    impl Sealed for unit::Month {}
    impl Sealed for unit::Week {}
    impl Sealed for unit::Day {}
    impl Sealed for unit::Hour {}
    impl Sealed for unit::Minute {}
    impl Sealed for unit::Second {}
    impl Sealed for unit::Millisecond {}
    impl Sealed for unit::Microsecond {}
    impl Sealed for unit::Nanosecond {}
}

/// A sealed trait for objects that can increment or decrement a date or time.
pub trait Advance<T>: self::private::Sealed {
    /// Returns the object moved to the next step regardless if it's already at that step.
    fn next_from(self, obj: T) -> T;
    /// Returns the object moved to the previous step regardless if it's already at that step.
    fn prev_from(self, obj: T) -> T;
}

impl Advance<Date> for Weekday {
    #[inline]
    fn next_from(self, date: Date) -> Date {
        let diff = self as i8 - date.weekday() as i8;
        date.add_days(if diff <= 0 { diff + 7 } else { diff } as i32)
    }

    #[inline]
    fn prev_from(self, date: Date) -> Date {
        let diff = self as i8 - date.weekday() as i8;
        date.add_days(if diff >= 0 { diff - 7 } else { diff } as i32)
    }
}

impl<Tz: TimeZone> Advance<DateTime<Tz>> for Weekday {
    #[inline]
    fn next_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        let diff = self as i8 - dt.weekday() as i8;
        dt.date = dt.date.add_days(if diff <= 0 { diff + 7 } else { diff } as i32);
        if dt.timezone().is_fixed() {
            dt
        } else {
            dt.timezone.resolve(dt.date, dt.time).lenient()
        }
    }

    #[inline]
    fn prev_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        let diff = self as i8 - dt.weekday() as i8;
        dt.date = dt.date.add_days(if diff >= 0 { diff - 7 } else { diff } as i32);
        if dt.timezone().is_fixed() {
            dt
        } else {
            dt.timezone.resolve(dt.date, dt.time).lenient()
        }
    }
}

impl<Tz: TimeZone> Advance<DateTime<Tz>> for Time {
    fn next_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        // e.g. 2AM (dt) -> 9AM (self) means +7H but 11AM (dt) -> 9AM (self) means -2H
        // Since zero or negative, add over a day.
        let delta_nanos = self.total_nanos() as i64 - dt.time.total_nanos() as i64;
        if delta_nanos <= 0 {
            dt.date = dt.date.add_days(1);
        }
        if dt.timezone().is_fixed() {
            dt.time = self;
            dt
        } else {
            dt.timezone.resolve(dt.date, self).lenient()
        }
    }

    fn prev_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        // e.g. 2AM (dt) -> 9AM (self) means -7H but 11AM (dt) -> 9AM (self) means +2H
        // Since zero or negative, remove a day.
        let delta_nanos = dt.time.total_nanos() as i64 - self.total_nanos() as i64;
        if delta_nanos <= 0 {
            dt.date = dt.date.add_days(-1);
        }
        if dt.timezone().is_fixed() {
            dt.time = self;
            dt
        } else {
            dt.timezone.resolve(dt.date, self).lenient()
        }
    }
}

impl Advance<Date> for unit::Year {
    fn next_from(self, date: Date) -> Date {
        date.add_years(1)
    }

    fn prev_from(self, date: Date) -> Date {
        date.add_years(-1)
    }
}

impl Advance<Date> for unit::Month {
    fn next_from(self, date: Date) -> Date {
        date.add_months(1)
    }

    fn prev_from(self, date: Date) -> Date {
        date.add_months(-1)
    }
}

impl Advance<Date> for unit::Week {
    fn next_from(self, date: Date) -> Date {
        date.add_days(7)
    }

    fn prev_from(self, date: Date) -> Date {
        date.add_days(-7)
    }
}

impl Advance<Date> for unit::Day {
    fn next_from(self, date: Date) -> Date {
        date.add_days(1)
    }

    fn prev_from(self, date: Date) -> Date {
        date.add_days(-1)
    }
}

impl Advance<Time> for unit::Hour {
    fn next_from(self, time: Time) -> Time {
        time.add_with_duration(Duration::from_secs(3600)).1
    }

    fn prev_from(self, time: Time) -> Time {
        time.sub_with_duration(Duration::from_secs(3600)).1
    }
}

impl Advance<Time> for unit::Minute {
    fn next_from(self, time: Time) -> Time {
        time.add_with_duration(Duration::from_secs(60)).1
    }

    fn prev_from(self, time: Time) -> Time {
        time.sub_with_duration(Duration::from_secs(60)).1
    }
}

impl Advance<Time> for unit::Second {
    fn next_from(self, time: Time) -> Time {
        time.add_with_duration(Duration::from_secs(1)).1
    }

    fn prev_from(self, time: Time) -> Time {
        time.sub_with_duration(Duration::from_secs(1)).1
    }
}

impl Advance<Time> for unit::Millisecond {
    fn next_from(self, time: Time) -> Time {
        time.add_with_duration(Duration::from_millis(1)).1
    }

    fn prev_from(self, time: Time) -> Time {
        time.sub_with_duration(Duration::from_millis(1)).1
    }
}

impl Advance<Time> for unit::Microsecond {
    fn next_from(self, time: Time) -> Time {
        time.add_with_duration(Duration::from_micros(1)).1
    }

    fn prev_from(self, time: Time) -> Time {
        time.sub_with_duration(Duration::from_micros(1)).1
    }
}

impl Advance<Time> for unit::Nanosecond {
    fn next_from(self, time: Time) -> Time {
        time.add_with_duration(Duration::from_nanos(1)).1
    }

    fn prev_from(self, time: Time) -> Time {
        time.sub_with_duration(Duration::from_nanos(1)).1
    }
}

impl<Tz: TimeZone> Advance<DateTime<Tz>> for unit::Year {
    fn next_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        dt.date = dt.date.add_years(1);
        if dt.timezone().is_fixed() {
            dt
        } else {
            dt.timezone.resolve(dt.date, dt.time).lenient()
        }
    }

    fn prev_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        dt.date = dt.date.add_years(-1);
        if dt.timezone().is_fixed() {
            dt
        } else {
            dt.timezone.resolve(dt.date, dt.time).lenient()
        }
    }
}

impl<Tz: TimeZone> Advance<DateTime<Tz>> for unit::Month {
    fn next_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        dt.date = dt.date.add_months(1);
        if dt.timezone().is_fixed() {
            dt
        } else {
            dt.timezone.resolve(dt.date, dt.time).lenient()
        }
    }

    fn prev_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        dt.date = dt.date.add_months(-1);
        if dt.timezone().is_fixed() {
            dt
        } else {
            dt.timezone.resolve(dt.date, dt.time).lenient()
        }
    }
}

impl<Tz: TimeZone> Advance<DateTime<Tz>> for unit::Week {
    fn next_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        dt.date = dt.date.add_days(7);
        if dt.timezone().is_fixed() {
            dt
        } else {
            dt.timezone.resolve(dt.date, dt.time).lenient()
        }
    }

    fn prev_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        dt.date = dt.date.add_days(-7);
        if dt.timezone().is_fixed() {
            dt
        } else {
            dt.timezone.resolve(dt.date, dt.time).lenient()
        }
    }
}

impl<Tz: TimeZone> Advance<DateTime<Tz>> for unit::Day {
    fn next_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        dt.date = dt.date.add_days(1);
        if dt.timezone().is_fixed() {
            dt
        } else {
            dt.timezone.resolve(dt.date, dt.time).lenient()
        }
    }

    fn prev_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        dt.date = dt.date.add_days(-1);
        if dt.timezone().is_fixed() {
            dt
        } else {
            dt.timezone.resolve(dt.date, dt.time).lenient()
        }
    }
}

impl<Tz: TimeZone> Advance<DateTime<Tz>> for unit::Hour {
    fn next_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        let (days, time) = dt.time.add_with_duration(Duration::from_secs(3600));
        dt.date = dt.date.add_days(days);
        if dt.timezone().is_fixed() {
            dt.time = time;
            dt
        } else {
            dt.timezone.resolve(dt.date, time).lenient()
        }
    }

    fn prev_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        let (days, time) = dt.time.sub_with_duration(Duration::from_secs(3600));
        dt.date = dt.date.add_days(days);
        if dt.timezone().is_fixed() {
            dt.time = time;
            dt
        } else {
            dt.timezone.resolve(dt.date, time).lenient()
        }
    }
}

impl<Tz: TimeZone> Advance<DateTime<Tz>> for unit::Minute {
    fn next_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        let (days, time) = dt.time.add_with_duration(Duration::from_secs(60));
        dt.date = dt.date.add_days(days);
        if dt.timezone().is_fixed() {
            dt.time = time;
            dt
        } else {
            dt.timezone.resolve(dt.date, time).lenient()
        }
    }

    fn prev_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        let (days, time) = dt.time.sub_with_duration(Duration::from_secs(60));
        dt.date = dt.date.add_days(days);
        if dt.timezone().is_fixed() {
            dt.time = time;
            dt
        } else {
            dt.timezone.resolve(dt.date, time).lenient()
        }
    }
}

impl<Tz: TimeZone> Advance<DateTime<Tz>> for unit::Second {
    fn next_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        let (days, time) = dt.time.add_with_duration(Duration::from_secs(1));
        dt.date = dt.date.add_days(days);
        if dt.timezone().is_fixed() {
            dt.time = time;
            dt
        } else {
            dt.timezone.resolve(dt.date, time).lenient()
        }
    }

    fn prev_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        let (days, time) = dt.time.sub_with_duration(Duration::from_secs(1));
        dt.date = dt.date.add_days(days);
        if dt.timezone().is_fixed() {
            dt.time = time;
            dt
        } else {
            dt.timezone.resolve(dt.date, time).lenient()
        }
    }
}

impl<Tz: TimeZone> Advance<DateTime<Tz>> for unit::Millisecond {
    fn next_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        let (days, time) = dt.time.add_with_duration(Duration::from_millis(1));
        dt.date = dt.date.add_days(days);
        if dt.timezone().is_fixed() {
            dt.time = time;
            dt
        } else {
            dt.timezone.resolve(dt.date, time).lenient()
        }
    }

    fn prev_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        let (days, time) = dt.time.sub_with_duration(Duration::from_millis(1));
        dt.date = dt.date.add_days(days);
        if dt.timezone().is_fixed() {
            dt.time = time;
            dt
        } else {
            dt.timezone.resolve(dt.date, time).lenient()
        }
    }
}

impl<Tz: TimeZone> Advance<DateTime<Tz>> for unit::Microsecond {
    fn next_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        let (days, time) = dt.time.add_with_duration(Duration::from_micros(1));
        dt.date = dt.date.add_days(days);
        if dt.timezone().is_fixed() {
            dt.time = time;
            dt
        } else {
            dt.timezone.resolve(dt.date, time).lenient()
        }
    }

    fn prev_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        let (days, time) = dt.time.sub_with_duration(Duration::from_micros(1));
        dt.date = dt.date.add_days(days);
        if dt.timezone().is_fixed() {
            dt.time = time;
            dt
        } else {
            dt.timezone.resolve(dt.date, time).lenient()
        }
    }
}

impl<Tz: TimeZone> Advance<DateTime<Tz>> for unit::Nanosecond {
    fn next_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        let (days, time) = dt.time.add_with_duration(Duration::from_nanos(1));
        dt.date = dt.date.add_days(days);
        if dt.timezone().is_fixed() {
            dt.time = time;
            dt
        } else {
            dt.timezone.resolve(dt.date, time).lenient()
        }
    }

    fn prev_from(self, mut dt: DateTime<Tz>) -> DateTime<Tz> {
        let (days, time) = dt.time.sub_with_duration(Duration::from_nanos(1));
        dt.date = dt.date.add_days(days);
        if dt.timezone().is_fixed() {
            dt.time = time;
            dt
        } else {
            dt.timezone.resolve(dt.date, time).lenient()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{datetime, time, unit};

    #[test]
    fn test_advance_time() {
        let dt = datetime!(2022-02-08 03:00);
        assert_eq!(dt.next(time!(04:00)), datetime!(2022-02-08 04:00));
        assert_eq!(dt.next(time!(09:00)), datetime!(2022-02-08 09:00));
        assert_eq!(dt.next(time!(02:00)), datetime!(2022-02-09 02:00));
        assert_eq!(dt.next(time!(03:00)), datetime!(2022-02-09 03:00));

        assert_eq!(dt.prev(time!(09:00)), datetime!(2022-02-07 09:00));
        assert_eq!(dt.prev(time!(03:00)), datetime!(2022-02-07 03:00));
        assert_eq!(dt.prev(time!(04:00)), datetime!(2022-02-07 04:00));
        assert_eq!(dt.prev(time!(01:00)), datetime!(2022-02-08 01:00));
        assert_eq!(dt.prev(time!(00:00)), datetime!(2022-02-08 00:00));
    }

    #[test]
    fn test_advance_units() {
        let dt = datetime!(2022-02-08 03:00);
        assert_eq!(dt.next(unit::Year), datetime!(2023-02-08 03:00));
        assert_eq!(dt.next(unit::Month), datetime!(2022-03-08 03:00));
        assert_eq!(dt.next(unit::Week), datetime!(2022-02-15 03:00));
        assert_eq!(dt.next(unit::Day), datetime!(2022-02-09 03:00));
        assert_eq!(dt.next(unit::Hour), datetime!(2022-02-08 04:00));
        assert_eq!(dt.next(unit::Minute), datetime!(2022-02-08 03:01));
        assert_eq!(dt.next(unit::Second), datetime!(2022-02-08 03:00:01));
    }
}
