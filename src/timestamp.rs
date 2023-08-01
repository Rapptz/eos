use crate::{
    date::Date,
    datetime::DateTime,
    gregorian::{date_from_epoch_days, MAX_EPOCH_DAYS, MIN_EPOCH_DAYS},
    time::Time,
    timezone::{Utc, UtcOffset},
    utils::{divmod, divrem},
};

const NANOS_PER_SEC: u32 = 1_000_000_000;
const NANOS_PER_MILLI: u32 = 1_000_000;
const NANOS_PER_MICRO: u32 = 1_000;
const MILLIS_PER_SEC: u64 = 1_000;
const MICROS_PER_SEC: u64 = 1_000_000;

/// A UNIX timestamp.
///
/// This is defined by the number of seconds since the Unix epoch,
/// defined as January 1st 1970 12:00 AM UTC.
///
/// Each `Timestamp` is composed of a whole number of seconds and a fractional
/// part representing the nanoseconds.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Timestamp {
    seconds: i64,
    pub(crate) nanoseconds: u32,
}

impl Timestamp {
    const MIN_VALID: i64 = MIN_EPOCH_DAYS as i64 * 86400;
    const MAX_VALID: i64 = MAX_EPOCH_DAYS as i64 * 86400 + (23 * 3600) + (59 * 60) + 59;

    /// Creates a new `Timestamp` from the specified whole seconds and additional nanoseconds.
    ///
    /// If the number of nanoseconds is greater than 2 billion then it is clamped to that value.
    #[inline]
    #[must_use]
    pub const fn new(seconds: i64, nanoseconds: u32) -> Self {
        Self {
            seconds,
            nanoseconds: if nanoseconds >= 2_000_000_000 {
                1_999_999_999
            } else {
                nanoseconds
            },
        }
    }

    /// Creates a new `Timestamp` from the given number of seconds.
    #[inline]
    #[must_use]
    pub const fn from_seconds(seconds: i64) -> Self {
        Self {
            seconds,
            nanoseconds: 0,
        }
    }

    /// Creates a new `Timestamp` from the given number of milliseconds.
    #[inline]
    #[must_use]
    pub const fn from_milliseconds(milliseconds: i64) -> Self {
        let (seconds, millis) = divmod!(milliseconds, MILLIS_PER_SEC as i64);
        Self {
            seconds,
            nanoseconds: millis as u32 * NANOS_PER_MILLI,
        }
    }

    /// Creates a new `Timestamp` from the given number of microseconds.
    #[inline]
    #[must_use]
    pub const fn from_microseconds(microseconds: i64) -> Self {
        let (seconds, micros) = divmod!(microseconds, MICROS_PER_SEC as i64);
        Self {
            seconds,
            nanoseconds: micros as u32 * NANOS_PER_MICRO,
        }
    }

    /// Returns the number of whole seconds in this timestamp.
    ///
    /// This does not return the nanoseconds component.
    #[inline]
    #[must_use]
    pub const fn as_seconds(&self) -> i64 {
        self.seconds
    }

    /// Returns the total number of milliseconds in this timestamp.
    #[inline]
    #[must_use]
    pub const fn as_milliseconds(&self) -> i128 {
        self.seconds as i128 * MILLIS_PER_SEC as i128 + (self.nanoseconds / NANOS_PER_MILLI) as i128
    }

    /// Returns the number of seconds as an `f64`.
    ///
    /// This contains the fractional seconds.
    #[inline]
    #[must_use]
    pub fn as_seconds_f64(&self) -> f64 {
        (self.seconds as f64) + (self.nanoseconds as f64) / (NANOS_PER_SEC as f64)
    }

    /// Returns the number of seconds as an `f32`.
    ///
    /// This contains the fractional seconds.
    #[inline]
    #[must_use]
    pub fn as_seconds_f32(&self) -> f32 {
        (self.seconds as f32) + (self.nanoseconds as f32) / (NANOS_PER_SEC as f32)
    }

    /// Converts the `Timestamp` into a [`DateTime`] in UTC.
    ///
    /// If the timestamp is out of range whether in the negative or positive
    /// direction then it saturates towards the overflowing side.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn to_utc(self) -> DateTime<Utc> {
        // This is a manual implementation due to the `const fn` requirement.

        if self.seconds >= Self::MAX_VALID {
            return DateTime {
                date: Date::MAX,
                time: Time::MAX,
                offset: UtcOffset::UTC,
                timezone: Utc,
            };
        } else if self.seconds <= Self::MIN_VALID {
            return DateTime {
                date: Date::MIN,
                time: Time::MIN,
                offset: UtcOffset::UTC,
                timezone: Utc,
            };
        }

        let (days, seconds) = divmod!(self.seconds, 86400);
        // At this point this cast should be safe since it was bound checked earlier
        let (year, month, day) = date_from_epoch_days(days as i32);
        let (hours, seconds) = divrem!(seconds, 3600);
        let (minutes, seconds) = divrem!(seconds, 60);

        DateTime {
            date: Date { year, month, day },
            time: Time {
                hour: hours as u8,
                minute: minutes as u8,
                second: seconds as u8,
                nanosecond: self.nanoseconds,
            },
            offset: UtcOffset::UTC,
            timezone: Utc,
        }
    }
}

impl core::fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.nanoseconds == 0 {
            f.debug_tuple("Timestamp").field(&self.seconds).finish()
        } else {
            f.debug_tuple("Timestamp").field(&self.as_seconds_f64()).finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datetime;

    #[test]
    fn test_to_utc() {
        assert_eq!(Timestamp::from_seconds(0).to_utc(), DateTime::UNIX_EPOCH);
        assert_eq!(Timestamp::from_seconds(3723).to_utc(), datetime!(1970-01-01 1:02:03));
        assert_eq!(
            Timestamp::from_seconds(1641155925).to_utc(),
            datetime!(2022-01-02 20:38:45)
        );
        assert_eq!(
            Timestamp::from_seconds(1641173925).to_utc(),
            datetime!(2022-01-02 20:38:45 -5:00)
        );
    }
}
