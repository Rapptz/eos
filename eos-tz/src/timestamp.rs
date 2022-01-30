use eos::{DateTime, Interval, TimeZone, Utc};

/// Represents a naive Unix timestamp.
///
/// A naive timestamp is defined by the number of seconds since the Unix epoch,
/// defined as January 1st 1970 12:00 AM UTC. This does *not* have nanosecond precision.
///
/// Naive timestamps have no notion of timezone.They are generally not used except when
/// dealing with [`TimeZone`] calculations.
///
/// To convert a [`DateTime`] into a [`NaiveTimestamp`], the [`From`] trait should be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub(crate) struct NaiveTimestamp(pub(crate) i64);

impl<Tz> From<DateTime<Tz>> for NaiveTimestamp
where
    Tz: TimeZone,
{
    fn from(dt: DateTime<Tz>) -> Self {
        let ts = dt.days_since_epoch() as i64 * 86400
            + dt.hour() as i64 * 3600
            + dt.minute() as i64 * 60
            + dt.second() as i64;
        Self(ts)
    }
}

impl From<i64> for NaiveTimestamp {
    fn from(s: i64) -> Self {
        Self(s)
    }
}

impl NaiveTimestamp {
    /// Denotes the minimum possible NaiveTimestamp.
    ///
    /// This is usually used to represent the beginning of time.
    pub(crate) const MIN: Self = Self::from_seconds(i64::MIN);

    /// Creates a new [`NaiveTimestamp`] from the given date and time.
    pub(crate) const fn new(date: &eos::Date, time: &eos::Time) -> Self {
        let ts = date.days_since_epoch() as i64 * 86400
            + time.hour() as i64 * 3600
            + time.minute() as i64 * 60
            + time.second() as i64;

        Self(ts)
    }

    /// Creates a new [`NaiveTimestamp`] from the given number of seconds.
    pub(crate) const fn from_seconds(secs: i64) -> Self {
        Self(secs)
    }

    /// Returns the inner value. These are the number of seconds.
    pub(crate) const fn into_inner(self) -> i64 {
        self.0
    }

    /// Converts the naive timestamp into a UTC [`DateTime`].
    pub(crate) fn to_utc(self) -> DateTime<Utc> {
        DateTime::UNIX_EPOCH + Interval::from_seconds(self.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eos::datetime;

    #[test]
    fn test_to_utc() {
        let dt = datetime!(2021-01-12 12:34 -05:00);
        assert_eq!(NaiveTimestamp::from(dt).to_utc(), datetime!(2021-01-12 12:34));
    }
}
