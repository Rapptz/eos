#[cfg(feature = "localtime")]
use crate::sys::localtime;

use crate::{utils::ensure_in_range, Date, DateTime, Error, Time, Timestamp};

/// Represents an offset from UTC.
///
/// This struct can only store values up to ±24:00:00.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtcOffset {
    pub(crate) hours: i8,
    pub(crate) minutes: i8,
    pub(crate) seconds: i8,
}

impl Default for UtcOffset {
    fn default() -> Self {
        Self::UTC
    }
}

impl UtcOffset {
    /// Returns the smallest possible [`UtcOffset`].
    pub const MIN: Self = Self {
        hours: -24,
        minutes: 0,
        seconds: 0,
    };

    /// Returns the largest possible [`UtcOffset`].
    pub const MAX: Self = Self {
        hours: 24,
        minutes: 0,
        seconds: 0,
    };

    /// Returns the [`UtcOffset`] representing UTC.
    pub const UTC: Self = Self {
        hours: 0,
        minutes: 0,
        seconds: 0,
    };

    #[doc(hidden)]
    #[cfg(feature = "macros")]
    #[inline]
    pub const fn __new_unchecked_from_macro(seconds: i32) -> Self {
        Self::from_seconds_unchecked(seconds)
    }

    /// Creates a new [`UtcOffset`] from the given number of hours, minutes, and seconds.
    ///
    /// The sign of all three components should match. If they do not, all components will
    /// have their signs flipped to match the `hour` sign.
    ///
    /// The values must be within the range of ±24:00:00.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::UtcOffset;
    /// assert!(UtcOffset::from_hms(24, 1, 0).is_err()); // invalid range
    /// assert!(UtcOffset::from_hms(24, 0, 0).is_ok());
    /// assert_eq!(UtcOffset::from_hms(23, 56, 59)?.into_hms(), (23, 56, 59));
    /// assert_eq!(UtcOffset::from_hms(0, 30, 0)?.into_hms(), (0, 30, 0));
    /// assert_eq!(UtcOffset::from_hms(0, -30, 30)?.into_hms(), (0, -30, -30));
    /// # Ok::<_, eos::Error>(())
    /// ```
    pub const fn from_hms(hours: i8, mut minutes: i8, mut seconds: i8) -> Result<Self, Error> {
        ensure_in_range!(hours, -24 => 24);
        ensure_in_range!(minutes, -59 => 59);
        ensure_in_range!(seconds, -59 => 59);

        // This is surprisingly well optimised
        if hours.is_negative() {
            if minutes.is_positive() {
                minutes = -minutes;
            }
            if seconds.is_positive() {
                seconds = -seconds;
            }
        } else if hours.is_positive() {
            if minutes.is_negative() {
                minutes = -minutes;
            }
            if seconds.is_negative() {
                seconds = -seconds;
            }
        } else {
            // Special case for 0 hours, it takes the sign of minutes
            // -30:30 => -30:-30
            // 30:-30 => 30:30
            if seconds.is_positive() != minutes.is_positive() {
                seconds = -seconds;
            }
        }

        let seconds = hours as i32 * 3600 + minutes as i32 * 60 + seconds as i32;
        Self::from_seconds(seconds)
    }

    /// Creates a new [`UtcOffset`] from a total number of seconds.
    /// The value must be between `-86400..=86400`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::UtcOffset;
    /// assert_eq!(UtcOffset::from_seconds(23400)?.into_hms(), (6, 30, 0));
    /// assert_eq!(UtcOffset::from_seconds(23400)?.total_seconds(), 23400);
    /// # Ok::<_, eos::Error>(())
    /// ```
    pub const fn from_seconds(seconds: i32) -> Result<Self, Error> {
        ensure_in_range!(seconds, -86400 => 86400);
        Ok(Self::from_seconds_unchecked(seconds))
    }

    pub(crate) const fn from_seconds_unchecked(seconds: i32) -> Self {
        let hours = seconds / 3600;
        let seconds = seconds % 3600;
        let minutes = seconds / 60;
        let seconds = seconds % 60;
        Self {
            hours: hours as i8,
            minutes: minutes as i8,
            seconds: seconds as i8,
        }
    }

    /// Get the utc offset's hours.
    pub const fn hours(&self) -> i8 {
        self.hours
    }

    /// Get the utc offset's minutes.
    pub const fn minutes(&self) -> i8 {
        self.minutes
    }

    /// Get the utc offset's seconds.
    pub const fn seconds(&self) -> i8 {
        self.seconds
    }

    /// Returns the total number of seconds this offset represents.
    ///
    /// # Example
    ///
    /// ```
    /// # use eos::UtcOffset;
    /// assert_eq!(UtcOffset::from_hms(6, 30, 0)?.total_seconds(), 23400);
    /// # Ok::<_, eos::Error>(())
    /// ```
    #[inline]
    pub const fn total_seconds(&self) -> i32 {
        self.hours as i32 * 3600 + self.minutes as i32 * 60 + self.seconds as i32
    }

    /// Unwraps this offset into their individual `(hours, minutes, seconds)` components.
    #[inline]
    pub const fn into_hms(self) -> (i8, i8, i8) {
        (self.hours, self.minutes, self.seconds)
    }

    /// Returns `true` if this offset is UTC.
    #[inline]
    pub const fn is_utc(&self) -> bool {
        self.hours == 0 && self.minutes == 0 && self.seconds == 0
    }

    /// Returns `true` if this offset is negative.
    #[inline]
    pub const fn is_negative(&self) -> bool {
        self.hours < 0 && self.minutes < 0 && self.seconds < 0
    }

    /// Subtracts two offsets, returning [`Error`] if the result would be out of bounds.
    ///
    /// ```rust
    /// # use eos::utc_offset;
    /// let east = utc_offset!(-5:00);
    /// let west = utc_offset!(-8:00);
    /// let far  = utc_offset!(18:00);
    ///
    /// assert!(far.checked_sub(west).is_err()); // 18 - -8 => 26
    /// assert_eq!(west.checked_sub(east), Ok(utc_offset!(-3:00)));
    /// ```
    #[inline]
    pub const fn checked_sub(self, other: Self) -> Result<Self, Error> {
        let seconds = self.total_seconds() - other.total_seconds();
        Self::from_seconds(seconds)
    }

    /// Adds two offsets, returning [`Error`] if the result would be out of bounds.
    ///
    /// ```rust
    /// # use eos::utc_offset;
    /// let east  = utc_offset!(-5:00);
    /// let west  = utc_offset!(-8:00);
    /// let far   = utc_offset!(18:00);
    /// let other = utc_offset!(-18:00);
    ///
    /// assert_eq!(far.checked_add(west), Ok(utc_offset!(10:00)));
    /// assert_eq!(west.checked_add(east), Ok(utc_offset!(-13:00)));
    /// assert!(other.checked_add(west).is_err());
    /// assert_eq!(other.checked_add(east), Ok(utc_offset!(-23:00)));
    /// ```
    #[inline]
    pub const fn checked_add(self, other: Self) -> Result<Self, Error> {
        let seconds = self.total_seconds() + other.total_seconds();
        Self::from_seconds(seconds)
    }

    /// Subtracts two offsets, saturating at the bounds if out of bounds.
    ///
    /// ```rust
    /// # use eos::utc_offset;
    /// let east = utc_offset!(-5:00);
    /// let west = utc_offset!(-8:00);
    /// let far  = utc_offset!(18:00);
    ///
    /// assert_eq!(far.saturating_sub(west), utc_offset!(24:00:00)); // 18 - -8 => 26
    /// assert_eq!(west.saturating_sub(east), utc_offset!(-3:00));
    /// ```
    #[inline]
    pub const fn saturating_sub(self, other: Self) -> Self {
        let seconds = self.total_seconds() - other.total_seconds();
        if seconds <= -86400 {
            Self::MIN
        } else if seconds >= 86400 {
            Self::MAX
        } else {
            Self::from_seconds_unchecked(seconds)
        }
    }

    /// Adds two offsets, saturating at the bounds if out of bounds.
    ///
    /// ```rust
    /// # use eos::utc_offset;
    /// let east  = utc_offset!(-5:00);
    /// let west  = utc_offset!(-8:00);
    /// let far   = utc_offset!(18:00);
    /// let other = utc_offset!(-18:00);
    ///
    /// assert_eq!(far.saturating_add(west), utc_offset!(10:00));
    /// assert_eq!(west.saturating_add(east), utc_offset!(-13:00));
    /// assert_eq!(other.saturating_add(west), utc_offset!(-24:00:00));
    /// assert_eq!(other.saturating_add(east), utc_offset!(-23:00));
    /// ```
    #[inline]
    pub const fn saturating_add(self, other: Self) -> Self {
        let seconds = self.total_seconds() + other.total_seconds();
        if seconds <= -86400 {
            Self::MIN
        } else if seconds >= 86400 {
            Self::MAX
        } else {
            Self::from_seconds_unchecked(seconds)
        }
    }
}

impl core::fmt::Display for UtcOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (m, s) = (self.minutes.abs(), self.seconds.abs());
        if s > 0 {
            write!(f, "{:+03}:{:02}:{:02}", self.hours, m, s)
        } else {
            write!(f, "{:+03}:{:02}", self.hours, m)
        }
    }
}

impl core::ops::Neg for UtcOffset {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            hours: -self.hours,
            minutes: -self.minutes,
            seconds: -self.seconds,
        }
    }
}

impl core::ops::Add for UtcOffset {
    type Output = Self;

    /// Adds two offsets together.
    ///
    /// # Panics
    ///
    /// If the offset ended up out of bounds.
    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(rhs).expect("out of bounds when adding offsets")
    }
}

impl core::ops::Sub for UtcOffset {
    type Output = Self;

    /// Subtracts two offsets together.
    ///
    /// # Panics
    ///
    /// If the offset ended up out of bounds.
    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs).expect("out of bounds when subtracting offsets")
    }
}

/// An enum representing the kind of [`DateTimeResolution`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DateTimeResolutionKind {
    /// The local datetime does not occur at all in the target time zone. This usually happens
    /// during DST transitions when clocks go forward. For example if the clock changed
    /// from 1AM to 2AM then a time such as 1:30 AM does not exist and cannot be represented.
    Missing,
    /// The local datetime occurs unambiguously and can be represented.
    Unambiguous,
    /// The local datetime occurs twice in the target time zone.This usually happens
    /// during a DST transition when clocks go backwards. For example, if the
    /// clocks changed from 2AM to 1AM then 1:30 AM happened twice. Once before
    /// the transition (the "earlier" time) and once after the transition (the "later" time).
    Ambiguous,
}

/// The result of resolving a local time in one time zone to another time zone.
/// This is returned from the [`TimeZone::resolve`] method. Most users should not
/// be creating these types themselves unless they're making a timezone library with
/// their own `resolve` implementation.
///
/// This allows you to differentiate between two moments in time where the local time
/// is the same. These are called "ambiguous times". For example, 2 AM at November 7th 2021
/// in most places in America was when clocks went backwards for one hour. In this case,
/// a time such as 1:30 AM is ambiguous because it can either refer to 1:30 AM after the
/// clocks have gone back an hour (Standard Time), or 1:30 AM before the clocks have
/// gone back an hour (Daylight Savings Time).
///
/// In other words, when clocks are moved backwards in time, a [*fold*] is created.
/// When clocks are moved forward, a *gap* is created. A local time that falls in the
/// fold is an ambiguous time, while a local time that falls in the gap is called a missing
/// time.
///
/// Likewise, this also allows you to handle the cases where a datetime cannot be represented
/// in the given timezone.
///
/// [*fold*]: https://www.python.org/dev/peps/pep-0495/#terminology
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeResolution<Tz: TimeZone> {
    date: Date,
    time: Time,
    timezone: Tz,
    earlier: UtcOffset,
    later: UtcOffset,
    kind: DateTimeResolutionKind,
}

impl<Tz> core::marker::Copy for DateTimeResolution<Tz> where Tz: Copy + TimeZone {}

impl<Tz: TimeZone> DateTimeResolution<Tz> {
    /// Creates a new [`DateTimeResolution`] that's ambiguous within two separate offsets.
    ///
    /// Note that in this case the earlier offset is the one *before* the transition and
    /// the after offset is the one *after* transition. Ergo, if the jump is UTC-4 -> UTC-5
    /// then UTC-4 is "earlier" and UTC-5 is "later".
    pub fn ambiguous(date: Date, time: Time, earlier: UtcOffset, later: UtcOffset, timezone: Tz) -> Self {
        Self {
            date,
            time,
            timezone,
            earlier,
            later,
            kind: DateTimeResolutionKind::Ambiguous,
        }
    }

    /// Creates a new [`DateTimeResolution`] that cannot be represented.
    ///
    /// Note that in this case the earlier offset is the one *before* the transition and
    /// the after offset is the one *after* transition. Ergo, if the jump is UTC-5 -> UTC-4
    /// then UTC-5 is "earlier" and UTC-4 is "later".
    pub fn missing(date: Date, time: Time, earlier: UtcOffset, later: UtcOffset, timezone: Tz) -> Self {
        Self {
            date,
            time,
            timezone,
            earlier,
            later,
            kind: DateTimeResolutionKind::Missing,
        }
    }

    /// Creates a new [`DateTimeResolution`] that's is unambiguous.
    pub fn unambiguous(date: Date, time: Time, offset: UtcOffset, timezone: Tz) -> Self {
        Self {
            date,
            time,
            timezone,
            earlier: offset,
            later: offset,
            kind: DateTimeResolutionKind::Unambiguous,
        }
    }

    /// Creates a new [`DateTimeResolution`] pointing to a new timezone.
    pub fn with_timezone<OtherTz: TimeZone>(self, timezone: OtherTz) -> DateTimeResolution<OtherTz> {
        DateTimeResolution {
            date: self.date,
            time: self.time,
            timezone,
            earlier: self.earlier,
            later: self.later,
            kind: self.kind,
        }
    }

    /// Returns the associated [`DateResolutionKind`] for this resolution.
    pub fn kind(&self) -> DateTimeResolutionKind {
        self.kind
    }

    /// Returns a reference to the date time resolution's date.
    pub fn date(&self) -> &Date {
        &self.date
    }

    /// Returns a reference to the date time resolution's time.
    pub fn time(&self) -> &Time {
        &self.time
    }

    /// Returns a reference to the date time resolution's timezone.
    pub fn timezone(&self) -> &Tz {
        &self.timezone
    }

    /// Returns a reference to the date time resolution's earlier.
    pub fn earlier_offset(&self) -> &UtcOffset {
        &self.earlier
    }

    /// Returns a reference to the date time resolution's later.
    pub fn later_offset(&self) -> &UtcOffset {
        &self.later
    }

    /// Returns `true` if the date time resolution is [`Ambiguous`].
    ///
    /// [`Ambiguous`]: DateTimeResolutionKind::Ambiguous
    pub fn is_ambiguous(&self) -> bool {
        matches!(self.kind, DateTimeResolutionKind::Ambiguous)
    }

    /// Returns `true` if the date time resolution is [`Unambiguous`].
    ///
    /// [`Unambiguous`]: DateTimeResolutionKind::Unambiguous
    pub fn is_unambiguous(&self) -> bool {
        matches!(self.kind, DateTimeResolutionKind::Unambiguous)
    }

    /// Returns `true` if the date time resolution is [`Missing`].
    ///
    /// [`Missing`]: DateTimeResolutionKind::Missing
    pub fn is_missing(&self) -> bool {
        matches!(self.kind, DateTimeResolutionKind::Missing)
    }

    /// Returns the earlier date time that was resolved.
    ///
    /// If the date time was skipped then an [`Error`] is returned.
    pub fn earlier(self) -> Result<DateTime<Tz>, Error> {
        match self.kind {
            DateTimeResolutionKind::Missing => Err(Error::SkippedDateTime(self.date, self.time)),
            DateTimeResolutionKind::Unambiguous | DateTimeResolutionKind::Ambiguous => Ok(DateTime {
                date: self.date,
                time: self.time,
                offset: self.earlier,
                timezone: self.timezone,
            }),
        }
    }

    /// Returns the later date time that was resolved.
    ///
    /// If the date time was skipped then an [`Error`] is returned.
    pub fn later(self) -> Result<DateTime<Tz>, Error> {
        match self.kind {
            DateTimeResolutionKind::Missing => Err(Error::SkippedDateTime(self.date, self.time)),
            DateTimeResolutionKind::Unambiguous | DateTimeResolutionKind::Ambiguous => Ok(DateTime {
                date: self.date,
                time: self.time,
                offset: self.later,
                timezone: self.timezone,
            }),
        }
    }

    /// Converts into a pair representing the earlier and later time.
    ///
    /// The first element of the pair is the earlier date time that was
    /// resolved, and the second element of the pair is the later date time
    /// that was resolved.
    ///
    /// This is only available to tests and is not stable API.
    #[doc(hidden)]
    pub fn into_pair(self) -> (DateTime<Tz>, DateTime<Tz>) {
        let earlier = DateTime {
            date: self.date,
            time: self.time,
            offset: self.earlier,
            timezone: self.timezone.clone(),
        };
        let later = DateTime {
            date: self.date,
            time: self.time,
            offset: self.later,
            timezone: self.timezone,
        };
        (earlier, later)
    }

    /// Returns a lenient date time that can represent this resolution.
    ///
    /// This allows retrieving a [`DateTime`] regardless of the resolution.
    /// A missing datetime is forward shifted to skip the gap. In other words,
    /// if 1:30 AM cannot be represented because 1AM was skipped into 2AM then 2:30AM
    /// is returned. An ambiguous date time will resolve into the earlier date time,
    /// or the one that happened first.
    pub fn lenient(self) -> DateTime<Tz> {
        match self.kind {
            DateTimeResolutionKind::Missing => {
                // UTC -5 -> UTC -4
                // -4 - -5 => +1 (offset)
                let mut as_utc = self.date.at(self.time);
                let delta = self.later.saturating_sub(self.earlier);
                as_utc.shift(delta);
                DateTime {
                    date: as_utc.date,
                    time: as_utc.time,
                    offset: self.later,
                    timezone: self.timezone,
                }
            }
            DateTimeResolutionKind::Unambiguous | DateTimeResolutionKind::Ambiguous => DateTime {
                date: self.date,
                time: self.time,
                offset: self.earlier,
                timezone: self.timezone,
            },
        }
    }

    /// Returns an exact date time that can represent this resolution.
    ///
    /// If the date time was skipped or is ambiguous then an [`Error`] is returned.
    pub fn exact(self) -> Result<DateTime<Tz>, Error> {
        match self.kind {
            DateTimeResolutionKind::Missing => Err(Error::SkippedDateTime(self.date, self.time)),
            DateTimeResolutionKind::Unambiguous => Ok(DateTime {
                date: self.date,
                time: self.time,
                offset: self.earlier,
                timezone: self.timezone,
            }),
            DateTimeResolutionKind::Ambiguous => Err(Error::AmbiguousDateTime(self.date, self.time)),
        }
    }

    pub(crate) fn backwards(self) -> DateTime<Tz> {
        match self.kind {
            DateTimeResolutionKind::Missing => {
                // UTC-4 -> UTC-5
                // -5 - -4 => -1 (offset)
                let mut as_utc = self.date.at(self.time);
                let delta = self.earlier.saturating_sub(self.later);
                as_utc.shift(delta);
                DateTime {
                    date: as_utc.date,
                    time: as_utc.time,
                    offset: self.earlier,
                    timezone: self.timezone,
                }
            }
            DateTimeResolutionKind::Unambiguous | DateTimeResolutionKind::Ambiguous => DateTime {
                date: self.date,
                time: self.time,
                offset: self.earlier,
                timezone: self.timezone,
            },
        }
    }
}

/// A trait that defines timezone behaviour.
pub trait TimeZone: Clone {
    /// Returns the name of the timezone at a given UNIX timestamp.
    fn name(&self, _ts: Timestamp) -> Option<&str> {
        None
    }

    /// Returns the UTC offset of the timezone at a given UNIX timestamp.
    ///
    /// If DST is being observed then the offset must take that into account.
    fn offset(&self, ts: Timestamp) -> UtcOffset;

    /// Resolves the given date and time to this time zone.
    ///
    /// The resolution could either be unambiguous, ambiguous, or missing
    /// depending on when it falls. See the [`DateTimeResolution`] documentation
    /// for more information.
    ///
    /// The `date` and `time` parameters represent the local date and time.
    fn resolve(self, date: Date, time: Time) -> DateTimeResolution<Self>
    where
        Self: Sized;

    /// Resolves the given date and time to this time zone leniently.
    ///
    /// If the time cannot be represented in local time then the "gap"
    /// is skipped and the time moves forward. If the time is ambiguous
    /// then the earlier value is returned. This is equivalent to the
    /// [`DateTimeResolution::lenient`] method.
    ///
    /// If more control is needed from this, consider using the
    /// [`TimeZone::resolve`] method instead.
    ///
    /// The `date` and `time` parameters represent the local date and time.
    fn at(self, date: Date, time: Time) -> DateTime<Self>
    where
        Self: Sized,
    {
        self.resolve(date, time).lenient()
    }

    /// Resolves the given date and time to this time zone exactly.
    ///
    /// If the time cannot be represented in local time unambiguously then
    /// an [`Error`] is returned.
    ///
    /// If more control is needed from this, consider using the
    /// [`TimeZone::resolve`] method instead.
    ///
    /// The `date` and `time` parameters represent the local date and time.
    fn at_exactly(self, date: Date, time: Time) -> Result<DateTime<Self>, Error>
    where
        Self: Sized,
    {
        self.resolve(date, time).exact()
    }

    /// Converts from a UTC [`DateTime`] to a datetime in this timezone.
    fn convert_utc(self, utc: DateTime<Utc>) -> DateTime<Self>
    where
        Self: Sized;
}

impl TimeZone for UtcOffset {
    fn offset(&self, _ts: Timestamp) -> UtcOffset {
        *self
    }

    fn resolve(self, date: Date, time: Time) -> DateTimeResolution<Self>
    where
        Self: Sized,
    {
        // This is always unambiguous
        DateTimeResolution::unambiguous(date, time, self, self)
    }

    fn convert_utc(self, mut utc: DateTime<Utc>) -> DateTime<Self>
    where
        Self: Sized,
    {
        utc.shift(self);
        utc.with_timezone(self)
    }
}

/// Represents the UTC timezone.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Utc;

impl TimeZone for Utc {
    #[cfg(feature = "alloc")]
    fn name(&self, _ts: Timestamp) -> Option<&str> {
        Some("UTC")
    }

    fn offset(&self, _ts: Timestamp) -> UtcOffset {
        UtcOffset::UTC
    }

    fn resolve(self, date: Date, time: Time) -> DateTimeResolution<Self>
    where
        Self: Sized,
    {
        // This is always unambiguous
        DateTimeResolution::unambiguous(date, time, UtcOffset::UTC, self)
    }

    fn convert_utc(self, utc: DateTime<Utc>) -> DateTime<Self>
    where
        Self: Sized,
    {
        utc
    }
}

impl Utc {
    /// Returns the current [`DateTime`] in UTC.
    #[cfg(feature = "std")]
    #[inline(always)]
    pub fn now() -> DateTime<Self> {
        DateTime::utc_now()
    }
}

/// Represents the machine's local timezone.
///
/// Due to differences in operating systems, the information returned by this
/// struct isn't necessarily the most detailed.
///
/// This requires the `localtime` feature to be enabled.
///
/// # Underlying OS APIs
///
/// Currently, the following OS APIs are being used to get the local timezone:
///
/// | Platform |                  Function Call                  |
/// |----------|-------------------------------------------------|
/// | POSIX    | [`localtime_r`]                                 |
/// | Windows  | [`GetTimeZoneInformation`] and [`GetLocalTime`] |
///
/// **Disclaimer**: These OS APIs might change over time.
///
/// [`localtime_r`]: https://linux.die.net/man/3/localtime_r
/// [`GetTimeZoneInformation`]: https://docs.microsoft.com/en-us/windows/win32/api/timezoneapi/nf-timezoneapi-gettimezoneinformation
/// [`GetLocalTime`]: https://docs.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getlocaltime
///
/// ## Why `localtime_r`?
///
/// Users familiar with the [`chrono`] or the [`time`] crate would be aware that this underlying C API has been
/// the [source] of an CVE ([CVE-2020-26235][CVE]). However, as more time has passed on the issue there are dissenting
/// views within both the Rust community and the C community on whether the issue lied within `localtime_r` or `setenv`. This
/// is because the `localtime_r` function only *reads* from the environment, it does not modify it. Many users have
/// expressed the opinion that the issue is the underlying [`std::env::set_env`] call not being marked `unsafe`
/// since by its nature it is not a thread safe function. There have been many proposals that have gained significant
/// traction to mark `set_env` as `unsafe`. Given the direction the standard library is heading, the opinion of various
/// experts, and the fact that there is no feasible alternative API in POSIX, this library opts to remain using
/// `localtime_r` despite its previously tainted status causing an CVE.
///
/// If an alternative API were to exist that would fill this gap of functionality for POSIX systems the library gladly
/// switch to it, however, this ideal has yet to be met.
///
/// [`chrono`]: https://github.com/chronotope/chrono/
/// [`time`]: https://github.com/time-rs/time/
/// [source]: https://passcod.name/technical/no-time-for-chrono.html
/// [CVE]: https://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2020-26235
///
#[cfg(feature = "localtime")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Local(pub(crate) localtime::LocalTime);

impl core::fmt::Debug for Local {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(feature = "alloc") {
            f.debug_struct("Local")
                .field("offset", &self.0.offset())
                .field("name", &self.0.name())
                .finish()
        } else {
            f.debug_struct("Local").field("offset", &self.0.offset()).finish()
        }
    }
}

#[cfg(feature = "localtime")]
impl Local {
    /// Creates a new [`Local`].
    #[inline]
    pub fn new() -> Result<Self, Error> {
        Ok(Self(localtime::LocalTime::new()?))
    }

    /// Returns the current [`DateTime`] in local time.
    #[inline]
    pub fn now() -> Result<DateTime<Self>, Error> {
        DateTime::now()
    }
}

impl TimeZone for Local {
    fn name(&self, _ts: Timestamp) -> Option<&str> {
        self.0.name()
    }

    fn offset(&self, _ts: Timestamp) -> UtcOffset {
        self.0.offset()
    }

    fn resolve(self, date: Date, time: Time) -> DateTimeResolution<Self>
    where
        Self: Sized,
    {
        // This is, unfortunately, always unambiguous
        // This could probably be improved if the APIs allow for it
        DateTimeResolution::unambiguous(date, time, self.0.offset(), self)
    }

    fn convert_utc(self, mut utc: DateTime<Utc>) -> DateTime<Self>
    where
        Self: Sized,
    {
        // TODO: proper algorithm
        utc.shift(self.0.offset());
        utc.with_timezone(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction_ranges() {
        assert!(UtcOffset::from_hms(-32, 0, 0).is_err());
        assert!(UtcOffset::from_hms(24, 0, 0).is_ok());
        assert!(UtcOffset::from_hms(23, 60, 0).is_err());
        assert!(UtcOffset::from_hms(-23, -60, 0).is_err());
        assert!(UtcOffset::from_hms(-23, -60, -60).is_err());
        assert!(UtcOffset::from_hms(24, -60, -60).is_err());

        assert!(UtcOffset::from_hms(-5, 30, 0).is_ok());

        assert!(UtcOffset::from_seconds(-86400).is_ok());
        assert!(UtcOffset::from_seconds(86400).is_ok());
        assert!(UtcOffset::from_seconds(3600).is_ok());
        assert!(UtcOffset::from_seconds(-3600).is_ok());
    }
}
