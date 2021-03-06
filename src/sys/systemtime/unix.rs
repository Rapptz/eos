use core::mem::MaybeUninit;
use core::time::Duration;

#[cfg(feature = "alloc")]
use alloc::string::String;

use crate::{DateTime, Error, Utc, UtcOffset};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct SystemTime {
    offset: UtcOffset,
    #[cfg(feature = "alloc")]
    name: Option<String>,
}

extern "C" {
    // Apparently libc doesn't contain this method
    // Also, netbsd needs this compatibility linkage
    #[cfg_attr(target_os = "netbsd", link_name = "__tzset50")]
    fn tzset();
}

impl SystemTime {
    pub(crate) fn new() -> Result<Self, Error> {
        let (_, ts) = get_current_duration_from_epoch()?;
        Self::new_from_time(ts)
    }

    fn new_from_time(timestamp: libc::time_t) -> Result<Self, Error> {
        let mut tm = MaybeUninit::uninit();

        // `localtime_r` does not call this function for some reason
        // SAFETY: tzset is safe to call as long as nothing mutates the environment
        // while it's retrieving the TZ variable.
        //
        // This invariant is impossible to uphold for external programs outside of Rust
        // and unfortunately `std::env::set_var` is marked safe so this invariant is
        // possible to break even in safe Rust.
        //
        // In the future (hopefully), `std::env::set_var` will be deprecated and
        // an `unsafe` alternative will be introduced that removes the ability for
        // safe Rust to cause segfaults.
        //
        // Note that tzset is marked as re-entrant/thread-safe in POSIX documentation
        // despite the above.
        unsafe { tzset() };

        // SAFETY: see above
        // This returns a NULL pointer in case of errors
        let ptr = unsafe { libc::localtime_r(&timestamp, tm.as_mut_ptr()) };
        if ptr.is_null() {
            return Err(Error::NoSystemTime);
        }

        // SAFETY: this returned without errors
        let tm = unsafe { tm.assume_init() };

        // tm_gmtoff is a c_long which can either be i32 or i64
        // I'm not sure this can ever really error out since realistically the bounds
        // from which you can have a UTC offset is either -24*60*60 or 24*60*60
        // both of which fall well below the bounds of an i32, let alone an i64.
        // An i32::MAX number of seconds offset from UTC would be around 596523 hours,
        // a completely bogus value.
        let seconds = tm.tm_gmtoff as i32;
        let offset = UtcOffset::from_seconds_unchecked(seconds);

        #[cfg(feature = "alloc")]
        {
            // SAFETY: the string returned by localtime_r is a standard NUL-terminating string.
            let length = unsafe { libc::strlen(tm.tm_zone) };
            // SAFETY: the string is at least the specified length
            // const char* can also be reinterpreted as const u8* since they're
            // the same aliasing and size
            let slice = unsafe { core::slice::from_raw_parts(tm.tm_zone as *const u8, length) };
            let name = core::str::from_utf8(slice).ok().map(String::from);
            Ok(Self { offset, name })
        }

        #[cfg(not(feature = "alloc"))]
        {
            Ok(Self { offset })
        }
    }

    pub(crate) fn offset(&self) -> UtcOffset {
        self.offset
    }

    #[cfg(feature = "alloc")]
    pub(crate) fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    #[cfg(not(feature = "alloc"))]
    pub(crate) fn name(&self) -> Option<&str> {
        None
    }
}

#[cfg(target_os = "macos")]
fn get_current_duration_from_epoch() -> Result<(Duration, libc::time_t), Error> {
    let mut timeval = MaybeUninit::uninit();

    // MacOS seems to prefer using gettimeofday over clock_gettime so I'll prefer it too
    // SAFETY: This returns non-0 on error, even if rare
    // Note this method is thread-safe
    let code = unsafe { libc::gettimeofday(timeval.as_mut_ptr(), core::ptr::null_mut()) };
    if code != 0 {
        Err(Error::NoSystemTime)
    } else {
        // SAFETY: at this point there is no error
        let timeval = unsafe { timeval.assume_init() };
        let duration = Duration::new(timeval.tv_sec as _, (timeval.tv_usec * 1000) as _);
        Ok((duration, timeval.tv_sec))
    }
}

#[cfg(not(target_os = "macos"))]
fn get_current_duration_from_epoch() -> Result<(Duration, libc::time_t), Error> {
    let mut timespec = MaybeUninit::uninit();

    // SAFETY: this returns a non-zero status code if it fails
    // Note this method is thread-safe
    let code = unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, timespec.as_mut_ptr()) };
    if code != 0 {
        Err(Error::NoSystemTime)
    } else {
        // SAFETY: at this point there is no error
        let timespec = unsafe { timespec.assume_init() };
        let duration = Duration::new(timespec.tv_sec as _, timespec.tv_nsec as _);
        Ok((duration, timespec.tv_sec))
    }
}

pub(crate) fn get_system_time_components() -> Result<(DateTime<Utc>, SystemTime), Error> {
    let (duration, timestamp) = get_current_duration_from_epoch()?;
    let tz = SystemTime::new_from_time(timestamp)?;
    // Have to adjust the time locally ourselves
    let seconds = tz.offset.total_seconds();
    let shift = if seconds.is_negative() {
        duration
            .checked_sub(Duration::from_secs(-seconds as u64))
            .ok_or(Error::NoSystemTime)?
    } else {
        duration
            .checked_add(Duration::from_secs(seconds as u64))
            .ok_or(Error::NoSystemTime)?
    };
    let dt = DateTime::UNIX_EPOCH + shift;
    Ok((dt, tz))
}
