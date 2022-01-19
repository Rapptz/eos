//! Traits and types that help with parsing to and from the ISO-8601 standard.
//!
//! Note that ISO-8601 output is the default [`std::fmt::Display`] output for
//! all types in the library. Despite the fact that this module depends on the `format`
//! feature, the `Display` implementations are still enabled without that feature.
//!
//! The library does not aim for strict ISO-8601 compliance, for example ISO-8601 does not
//! support concepts such as negative duration and have a lot of esoteric formats that aren't
//! supported. The support in this library is similar to the [`java.time`] library, where things
//! are supported in a way that make the most sense for the given domain and restriction of the
//! library.
//!
//! [`java.time`]: https://docs.oracle.com/javase/8/docs/api/java/time/package-summary.html

use crate::utils::divmod;
use core::fmt::Write;

/// An enum that specifies how the [`ToIso`] trait should handle precision of the components.
///
/// If a given precision would omit certain values from displaying, these values are *omitted*
/// rather than rounded to that value. For example, if the [`IsoFormatPrecision::Hour`] precision
/// is given at a time representing `12:59` then `12:00` will be returned not `13:00`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[cfg(feature = "format")]
pub enum IsoFormatPrecision {
    /// Display up to the hour, leaving remaining values either as 0 or omitted if possible.
    Hour,
    /// Display up to the minute, leaving remaining values either as 0 or omitted if possible.
    Minute,
    /// Display up to the second, leaving the fractional seconds omitted.
    Second,
    /// Display fractional seconds up to millisecond precision.
    Millisecond,
    /// Display fractional seconds up to microsecond precision.
    Microsecond,
    /// Display fractional seconds up to nanosecond precision.
    Nanosecond,
}

/// Converts a value to an ISO-8601 extended format string. Note that does not
/// aim for strict ISO-8601 compliance, for example ISO-8601 does not support concepts
/// such as negative duration and have a lot of esoteric formats that aren't supported.
///
/// This conversion should be infallible (other than allocation errors...).
#[cfg(feature = "format")]
pub trait ToIsoFormat {
    /// Converts to an appropriate ISO-8601 extended formatted string with the given precision.
    ///
    /// Certain types do not make use of this precision and will be ignored. A much simpler
    /// alternative is provided under [`Self::to_iso`].
    fn to_iso_format_with_precision(&self, precision: IsoFormatPrecision) -> String;

    /// Converts to an appropriate ISO-8601 extended formatted string.
    ///
    /// This function attempts to convert with appropriate precision for the given type.
    /// This means that certain values (typically fractional seconds) will be omitted if they
    /// can be.
    fn to_iso_format(&self) -> String;
}

#[cfg(feature = "format")]
impl ToIsoFormat for core::time::Duration {
    fn to_iso_format_with_precision(&self, _precision: IsoFormatPrecision) -> String {
        self.to_iso_format()
    }

    fn to_iso_format(&self) -> String {
        let mut buffer = String::new();
        let total_secs = self.as_secs_f64();
        if total_secs < 60.0 {
            // Simple case with (just) fractional seconds
            write!(&mut buffer, "PT{}S", self.as_secs_f64()).expect("unexpected error when writing to string");
        } else {
            let (hours, seconds) = divmod!(total_secs, 3600.0);
            let (minutes, seconds) = divmod!(seconds, 60.0);
            buffer.push('P');
            buffer.push('T');
            if hours > 0.0 {
                buffer
                    .write_fmt(format_args!("{}H", hours))
                    .expect("unexpected error when writing to string");
            }

            if minutes > 0.0 {
                buffer
                    .write_fmt(format_args!("{}M", minutes))
                    .expect("unexpected error when writing to string");
            }

            if seconds > 0.0 {
                buffer
                    .write_fmt(format_args!("{}S", hours))
                    .expect("unexpected error when writing to string");
            }
        }
        buffer
    }
}
