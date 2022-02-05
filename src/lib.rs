//! # eos: Simple yet correct date and time for Rust
//!
//! Eos is a simple crate that makes dates, times, and their manipulation as simple as possible.
//! It uses the proleptic Gregorian calendar using the ISO-8601
//!
//! Special care has been taken to ensure that timezone handling is correct and work as people expect.
//! This library has been inspired by world class libraries such as [`java.time`], [Noda Time], and [`<date>`].
//! For example, modifying times and comparing between them properly handle things like ambiguity, skipped times,
//! and DST transitions.
//!
//! Proper timezone support via the IANA database, or [tzdb] for short, has been delegated to a separate first
//! party crate, [`eos-tz`], though its use is highly recommended. Nevertheless, the library's default timezone
//! is UTC, so things work as expected even when dealing with times across internet borders.
//!
//! [`java.time`]: https://docs.oracle.com/javase/8/docs/api/java/time/package-summary.html
//! [Noda Time]: https://nodatime.org
//! [`<date>`]: https://github.com/HowardHinnant/date
//! [tzdb]: https://www.iana.org/time-zones

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate core;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "macros")]
pub mod macros;

#[cfg(any(feature = "formatting", feature = "parsing"))]
pub mod fmt;

#[cfg(all(feature = "parsing", feature = "serde"))]
pub mod serde;

mod builder;
mod date;
mod datetime;
mod error;
pub mod ext;
pub mod gregorian;
mod interval;
pub(crate) mod sys;
mod time;
mod timestamp;
mod timezone;
mod utils;

pub use builder::Builder;
pub use date::{Date, IsoWeekDate, Weekday};
pub use datetime::DateTime;
pub use error::Error;
pub use interval::Interval;
pub use time::Time;
pub use timestamp::Timestamp;
pub use timezone::{DateTimeResolution, DateTimeResolutionKind, System, TimeZone, Utc, UtcOffset};

// Internal helper for the macro_rules
#[doc(hidden)]
#[cfg(feature = "macros")]
pub use datetime::__create_offset_datetime_from_macro;
