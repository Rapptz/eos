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
