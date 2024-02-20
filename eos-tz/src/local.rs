#[cfg(feature = "localtime")]
use crate::sys::localtime::LocalTime;

/// The system's local timezone.
///
/// Due to differences in operating systems, the information returned by this
/// struct isn't necessarily the most detailed.
///
/// This requires the `localtime` feature to be enabled, which is enabled by default.
///
/// # Underlying OS APIs
///
/// Currently, the following OS APIs are being used to get the local timezone:
///
/// | Platform |                  Function Call                  |
/// |----------|-------------------------------------------------|
/// | POSIX    | None (pure Rust)                                |
/// | Windows  | [`GetTimeZoneInformation`]                      |
///
/// **Disclaimer**: These OS APIs might change over time.
///
/// This does *not* parse the `TZ` environment variable on POSIX platforms. If you desire
/// this functionality, you can use [`eos_tz::TimeZone::from_tz_str`] with [`std::env::get_var`].
///
/// [`GetTimeZoneInformation`]: https://docs.microsoft.com/en-us/windows/win32/api/timezoneapi/nf-timezoneapi-gettimezoneinformation
///
#[cfg(feature = "localtime")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Local(pub(crate) LocalTime);

#[cfg(feature = "localtime")]
impl Local {
    /// Creates a new `Local`.
    #[inline]
    pub fn new() -> Result<Self, crate::Error> {
        Ok(Self(crate::sys::localtime::LocalTime::new()?))
    }

    /// Returns the current date and time in the local timezone.
    #[inline]
    pub fn now() -> Result<eos::DateTime<Self>, crate::Error> {
        let tz = Self::new()?;
        Ok(eos::DateTime::utc_now().in_timezone(tz))
    }
}

#[cfg(feature = "localtime")]
impl eos::TimeZone for Local {
    fn offset(&self, ts: eos::Timestamp) -> eos::UtcOffset {
        self.0.as_inner().offset(ts)
    }

    fn resolve(self, date: eos::Date, time: eos::Time) -> eos::DateTimeResolution<Self>
    where
        Self: Sized,
    {
        let tz = self.0.as_inner().clone();
        tz.resolve(date, time).with_timezone(self)
    }

    fn convert_utc(self, utc: eos::DateTime<eos::Utc>) -> eos::DateTime<Self>
    where
        Self: Sized,
    {
        let tz = self.0.as_inner().clone();
        tz.convert_utc(utc).with_timezone(self)
    }

    fn name(&self, ts: eos::Timestamp) -> Option<&str> {
        self.0.as_inner().name(ts)
    }

    fn is_fixed(&self) -> bool {
        self.0.as_inner().is_fixed()
    }
}
