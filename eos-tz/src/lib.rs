#![allow(clippy::manual_range_contains)]

pub(crate) mod error;
mod local;
mod posix;
pub(crate) mod reader;
pub(crate) mod sys;
pub(crate) mod timestamp;
mod timezone;
mod transitions;

pub use error::{Error, ParseError};
pub use posix::PosixTimeZone;
pub use timezone::TimeZone;

#[cfg(feature = "localtime")]
pub use local::Local;
