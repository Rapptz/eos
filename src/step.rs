use crate::{Date, DateTime, TimeZone, Weekday, Time};

mod private {
    pub use super::*;
    pub trait Sealed {}
    impl Sealed for Weekday {}
    impl Sealed for Time {}
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

#[cfg(test)]
mod tests {
    use crate::{datetime, time};

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
}
