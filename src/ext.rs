//! Extension traits to numeric types for [`Interval`] construction.

use crate::Interval;

///! Extension traits

mod private {
    pub trait Sealed {}
    impl Sealed for i64 {}
    impl Sealed for i32 {}
    impl Sealed for u64 {}
    impl Sealed for u32 {}
    impl Sealed for f64 {}
}

/// A trait that allows you to create [`Interval`] from integer literals.
///
/// This is merely syntax sugar.
/// # Examples
///
/// Basic construction:
///
/// ```rust
/// use eos::{Interval, ext::IntervalLiteral};
/// assert_eq!(10.years(), Interval::from_years(10));
/// assert_eq!(10.months(), Interval::from_months(10));
/// assert_eq!(10.weeks(), Interval::from_weeks(10));
/// assert_eq!(10.days(), Interval::from_days(10));
/// assert_eq!(10.hours(), Interval::from_hours(10));
/// assert_eq!(10.minutes(), Interval::from_minutes(10));
/// assert_eq!(10.seconds(), Interval::from_seconds(10));
/// assert_eq!(10.milliseconds(), Interval::from_milliseconds(10));
/// assert_eq!(10.microseconds(), Interval::from_microseconds(10));
/// ```
///
/// Negative numbers:
///
/// ```rust
/// use eos::{Interval, ext::IntervalLiteral};
/// assert_eq!((-10).years(), Interval::from_years(-10));
/// assert_eq!((-10).months(), Interval::from_months(-10));
/// assert_eq!((-10).weeks(), Interval::from_weeks(-10));
/// assert_eq!((-10).days(), Interval::from_days(-10));
/// assert_eq!((-10).hours(), Interval::from_hours(-10));
/// assert_eq!((-10).minutes(), Interval::from_minutes(-10));
/// assert_eq!((-10).seconds(), Interval::from_seconds(-10));
/// assert_eq!((-10).milliseconds(), Interval::from_milliseconds(-10));
/// assert_eq!((-10).microseconds(), Interval::from_microseconds(-10));
/// ```
///
/// Arithmetic:
///
/// ```rust
/// use eos::{Interval, ext::IntervalLiteral};
/// assert_eq!(1.years() + 3.months(), Interval::from_years(1) + Interval::from_months(3));
/// assert_eq!(10.seconds() + 500.milliseconds(), Interval::from_seconds(10) + Interval::from_milliseconds(500));
/// ```
pub trait IntervalLiteral: private::Sealed {
    /// Creates a [`Interval`] representing the specified number of years.
    fn years(self) -> Interval;

    /// Creates a [`Interval`] representing the specified number of days.
    fn days(self) -> Interval;

    /// Creates a [`Interval`] representing the specified number of months.
    fn months(self) -> Interval;

    /// Creates a [`Interval`] representing the specified number of weeks.
    fn weeks(self) -> Interval;

    /// Creates a [`Interval`] representing the specified number of hours.
    fn hours(self) -> Interval;

    /// Creates a [`Interval`] representing the specified number of minutes.
    fn minutes(self) -> Interval;

    /// Creates a [`Interval`] representing the specified number of seconds.
    fn seconds(self) -> Interval;

    /// Creates a [`Interval`] representing the specified number of milliseconds.
    fn milliseconds(self) -> Interval;

    /// Creates a [`Interval`] representing the specified number of microseconds.
    fn microseconds(self) -> Interval;
}

macro_rules! impl_for_literal {
    ($($type:ty)+) => {
        $(
            impl IntervalLiteral for $type {
                #[inline]
                fn years(self) -> Interval {
                    Interval::from_years(self as _)
                }

                #[inline]
                fn days(self) -> Interval {
                    Interval::from_days(self as _)
                }

                #[inline]
                fn months(self) -> Interval {
                    Interval::from_months(self as _)
                }

                #[inline]
                fn weeks(self) -> Interval {
                    Interval::from_weeks(self as _)
                }

                #[inline]
                fn hours(self) -> Interval {
                    Interval::from_hours(self as _)
                }

                #[inline]
                fn minutes(self) -> Interval {
                    Interval::from_minutes(self as _)
                }

                #[inline]
                fn seconds(self) -> Interval {
                    Interval::from_seconds(self as _)
                }

                #[inline]
                fn milliseconds(self) -> Interval {
                    Interval::from_milliseconds(self as _)
                }

                #[inline]
                fn microseconds(self) -> Interval {
                    Interval::from_microseconds(self as _)
                }
            }
        )+
    };
}

impl_for_literal!(i32);
