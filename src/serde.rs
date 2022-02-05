//! Serialization/Deserialization support for the library.
//!
//! This module allows for alternative formats for serialization than the default
//! ISO-8601 representation. They are meant to be used with the [`with`] annotation.
//!
//! [`with`]: https://serde.rs/field-attrs.html#with

use serde::de::{self, Deserialize, Visitor};
use serde::ser::Serialize;

use crate::fmt::FromIsoFormat;
use crate::{Date, DateTime, Interval, IsoWeekDate, Time, TimeZone, Utc, UtcOffset};

/// Serialize into an ISO-8601 string.
impl Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

/// Serialize into an ISO-8601 string.
impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

/// Serialize into an ISO-8601 string.
impl Serialize for IsoWeekDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

/// Serialize into an ISO-8601 string.
impl<Tz> Serialize for DateTime<Tz>
where
    Tz: TimeZone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

/// Serialize into an ISO-8601 string.
impl Serialize for Interval {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

struct TimeVisitor;

impl<'de> Visitor<'de> for TimeVisitor {
    type Value = Time;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("an ISO-8601 formatted time string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Time::from_iso_format(v).map_err(E::custom)
    }
}

struct DateVisitor;

impl<'de> Visitor<'de> for DateVisitor {
    type Value = Date;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("an ISO-8601 formatted date string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Date::from_iso_format(v).map_err(E::custom)
    }
}

struct IsoWeekDateVisitor;

impl<'de> Visitor<'de> for IsoWeekDateVisitor {
    type Value = IsoWeekDate;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("an ISO-8601 formatted week date string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        IsoWeekDate::from_iso_format(v).map_err(E::custom)
    }
}

struct IntervalVisitor;

impl<'de> Visitor<'de> for IntervalVisitor {
    type Value = Interval;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("an ISO-8601 formatted interval string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Interval::from_iso_format(v).map_err(E::custom)
    }
}

struct DateTimeVisitor;

impl<'de> Visitor<'de> for DateTimeVisitor {
    type Value = DateTime<UtcOffset>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("an ISO-8601 formatted date string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        DateTime::from_iso_format(v).map_err(E::custom)
    }
}

/// Deserialize from an ISO-8601 string.
impl<'de> Deserialize<'de> for Time {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(TimeVisitor)
    }
}

/// Deserialize from an ISO-8601 string.
impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(DateVisitor)
    }
}

/// Deserialize from an ISO-8601 string.
impl<'de> Deserialize<'de> for IsoWeekDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(IsoWeekDateVisitor)
    }
}

/// Deserialize from an ISO-8601 string.
impl<'de> Deserialize<'de> for Interval {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(IntervalVisitor)
    }
}

/// Deserialize from an ISO-8601 string.
impl<'de> Deserialize<'de> for DateTime<UtcOffset> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(DateTimeVisitor)
    }
}

/// Deserialize from an ISO-8601 string.
impl<'de> Deserialize<'de> for DateTime<Utc> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(DateTimeVisitor).map(|dt| dt.into_utc())
    }
}

/// Serialize and deserialize to and from a UNIX timestamp in whole seconds.
///
/// Note that there's no nanosecond precision!
///
/// This is intended to be used with the [`with`] field attribute in `serde`.
///
/// [`with`]: https://serde.rs/field-attrs.html#with
///
/// # Example
///
/// ```
/// # use serde_derive::{Deserialize, Serialize};
/// use eos::{datetime, DateTime, Utc};
///
/// #[derive(Serialize, Deserialize)]
/// struct T {
///     #[serde(with = "eos::serde::timestamp")]
///     dt: DateTime<Utc>,
/// }
///
/// let t = T { dt: datetime!(2022-02-01 12:34:56) };
/// let string = serde_json::to_string(&t)?;
/// assert_eq!(string, r#"{"dt":1643718896}"#);
/// # Ok::<_, serde_json::Error>(())
/// ```
pub mod timestamp {

    use super::*;

    /// Serialize a UTC datetime into a UNIX timestamp in whole seconds.
    ///
    /// This is intended to be used with the [`serialize_with`] field attribute in `serde`.
    ///
    /// [`serialize_with`]: https://serde.rs/field-attrs.html#serialize_with
    ///
    /// # Example
    ///
    /// ```
    /// # use serde_derive::{Deserialize, Serialize};
    /// use eos::{datetime, DateTime, Utc};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct T {
    ///     #[serde(serialize_with = "eos::serde::timestamp::serialize")]
    ///     dt: DateTime<Utc>,
    /// }
    ///
    /// let t = T { dt: datetime!(2022-02-01 12:34:56) };
    /// let string = serde_json::to_string(&t)?;
    /// assert_eq!(string, r#"{"dt":1643718896}"#);
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(dt.timestamp().as_seconds())
    }

    /// Deserialize a UTC datetime from a UNIX timestamp in whole seconds.
    ///
    /// This is intended to be used with the [`deserialize_with`] field attribute in `serde`.
    ///
    /// [`deserialize_with`]: https://serde.rs/field-attrs.html#deserialize_with
    ///
    /// # Example
    ///
    /// ```
    /// # use serde_derive::{Deserialize, Serialize};
    /// use eos::{datetime, DateTime, Utc};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct T {
    ///     #[serde(deserialize_with = "eos::serde::timestamp::deserialize")]
    ///     dt: DateTime<Utc>,
    /// }
    ///
    /// let t: T = serde_json::from_str(r#"{"dt":1643718896}"#)?;
    /// # Ok::<_, serde_json::Error>(())
    /// ```
    pub fn deserialize<'de, D>(d: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_i64(TimestampVisitor)
    }

    struct TimestampVisitor;

    impl<'de> de::Visitor<'de> for TimestampVisitor {
        type Value = DateTime<Utc>;

        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            formatter.write_str("a UNIX timestamp in whole seconds")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(DateTime::from_timestamp(crate::Timestamp::from_seconds(value), Utc))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let value = i64::try_from(value).map_err(E::custom)?;
            Ok(DateTime::from_timestamp(crate::Timestamp::from_seconds(value), Utc))
        }
    }
}
