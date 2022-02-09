//! Date and time units
//!
//! This is meant to be used with date and time modification functions
//! such as [`DateTime::next`].
//!
//! These are implemented as [Zero-Sized Types] (ZSTs) since implementing them
//! as an enum would make it possible to use variants that don't make sense for the
//! target type. For example, passing in [`Year`] for [`Time`] should not be possible.
//!
//! [Zero-Sized Types]: https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts
//! [`Time`]: crate::Time
//! [`DateTime::next`]: crate::DateTime::next

/// A year.
#[derive(Debug, Copy, Clone)]
pub struct Year;

/// A month.
#[derive(Debug, Copy, Clone)]
pub struct Month;

/// A week. Equal to 7 days.
///
/// A week starts on Monday and ends on a Sunday, following ISO-8601 convention.
#[derive(Debug, Copy, Clone)]
pub struct Week;

/// A day.
#[derive(Debug, Copy, Clone)]
pub struct Day;

/// A hour.
#[derive(Debug, Copy, Clone)]
pub struct Hour;

/// A minute.
#[derive(Debug, Copy, Clone)]
pub struct Minute;

/// A second.
#[derive(Debug, Copy, Clone)]
pub struct Second;

/// A millisecond.
#[derive(Debug, Copy, Clone)]
pub struct Millisecond;

/// A microsecond.
#[derive(Debug, Copy, Clone)]
pub struct Microsecond;

/// A nanosecond.
#[derive(Debug, Copy, Clone)]
pub struct Nanosecond;
