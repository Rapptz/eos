use crate::{Date, DateTime, TimeZone, Weekday};

mod private {
    pub use super::*;
    pub trait Sealed {}
    impl Sealed for Weekday {}
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
