use core::mem::MaybeUninit;

#[cfg(feature = "alloc")]
use alloc::string::String;

use crate::{Date, DateTime, Time, Utc, UtcOffset};

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct SystemTime {
    info: TIME_ZONE_INFORMATION,
    is_dst: bool,
    #[cfg(feature = "alloc")]
    std_name: Option<String>,
    #[cfg(feature = "alloc")]
    dst_name: Option<String>,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, PartialEq, Eq, Hash)]
struct SYSTEMTIME {
    wYear: u16,
    wMonth: u16,
    wDayOfWeek: u16,
    wDay: u16,
    wHour: u16,
    wMinute: u16,
    wSecond: u16,
    wMilliseconds: u16,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[derive(Clone, PartialEq, Eq, Hash)]
struct TIME_ZONE_INFORMATION {
    Bias: i32,
    StandardName: [u16; 32],
    StandardDate: SYSTEMTIME,
    StandardBias: i32,
    DaylightName: [u16; 32],
    DaylightDate: SYSTEMTIME,
    DaylightBias: i32,
}

#[cfg(feature = "alloc")]
fn windows_utf16_to_utf8(s: &[u16]) -> Option<String> {
    // Find the first "null terminator byte"
    let null = s.iter().position(|&p| p == 0).unwrap_or(s.len());
    if null == 0 {
        None
    } else {
        String::from_utf16(&s[0..null]).ok()
    }
}

#[link(name = "kernel32")]
extern "system" {
    fn GetTimeZoneInformation(lpTimeZoneInformation: *mut TIME_ZONE_INFORMATION) -> u32;
    fn GetSystemTime(lpSystemTime: *mut SYSTEMTIME);
}

impl SystemTime {
    pub(crate) fn new() -> Result<Self, crate::Error> {
        let mut tzinfo = MaybeUninit::uninit();
        // SAFETY: the WinAPI call for this is pretty safe, if this fails then
        // TIME_ZONE_ID_INVALID is returned (represented essentially as u32::MAX)
        let code = unsafe { GetTimeZoneInformation(tzinfo.as_mut_ptr()) };

        if code == u32::MAX {
            return Err(crate::Error::NoSystemTime);
        }

        // SAFETY: at this point, the WinAPI returned without errors
        let tzinfo = unsafe { tzinfo.assume_init() };
        #[cfg(feature = "alloc")]
        {
            Ok(Self {
                std_name: windows_utf16_to_utf8(&tzinfo.StandardName),
                dst_name: windows_utf16_to_utf8(&tzinfo.DaylightName),
                info: tzinfo,
                is_dst: code == 2,
            })
        }

        #[cfg(not(feature = "alloc"))]
        {
            Ok(Self {
                info: tzinfo,
                is_dst: code == 2,
            })
        }
    }

    pub(crate) fn offset(&self) -> UtcOffset {
        let seconds = if self.is_dst {
            (self.info.Bias + self.info.DaylightBias) * 60
        } else {
            (self.info.Bias + self.info.StandardBias) * 60
        };
        // The Windows documentation says that the bias is calculated using
        // UTC = local time + bias
        // So the UTC offset is technically -bias.
        UtcOffset::from_seconds_unchecked(-seconds)
    }

    #[cfg(feature = "alloc")]
    pub(crate) fn name(&self) -> Option<&str> {
        if self.is_dst {
            self.dst_name.as_deref()
        } else {
            self.std_name.as_deref()
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub(crate) fn name(&self) -> Option<&str> {
        None
    }
}

pub(crate) fn get_system_time_components() -> Result<(DateTime<Utc>, SystemTime), crate::Error> {
    // SAFETY: this function does not fail according to the windows API docs
    // https://docs.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getlocaltime
    // this page makes no mention of any sort of fallibility, even with GetLastError
    // Since this is the case then it's safe to just call it as-is and assume it's valid
    let dt = unsafe {
        let mut out = MaybeUninit::uninit();
        GetSystemTime(out.as_mut_ptr());
        out.assume_init()
    };

    let date = Date {
        year: dt.wYear as i16,
        month: dt.wMonth as u8,
        day: dt.wDay as u8,
    };
    let time = Time {
        hour: dt.wHour as u8,
        minute: dt.wMinute as u8,
        second: dt.wSecond as u8,
        nanosecond: dt.wMilliseconds as u32 * 1_000_000,
    };
    Ok((DateTime::__new_utc_unchecked_from_macro(date, time), SystemTime::new()?))
}
