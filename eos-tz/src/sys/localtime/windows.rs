use std::mem::MaybeUninit;

use eos::UtcOffset;

use crate::{
    posix::{DstTransitionInfo, DstTransitionRule},
    PosixTimeZone,
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct LocalTime {
    inner: PosixTimeZone,
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

fn windows_utf16_to_utf8(s: &[u16]) -> Option<String> {
    // Find the first "null terminator byte"
    let null = s.iter().position(|&p| p == 0).unwrap_or(s.len());
    if null == 0 {
        None
    } else {
        String::from_utf16(&s[0..null]).ok()
    }
}

fn system_time_to_offset_seconds(s: &SYSTEMTIME) -> i64 {
    s.wHour as i64 * 3600 + s.wMinute as i64 * 60 + s.wSecond as i64
}

#[link(name = "kernel32")]
extern "system" {
    fn GetTimeZoneInformation(lpTimeZoneInformation: *mut TIME_ZONE_INFORMATION) -> u32;
}

impl LocalTime {
    pub(crate) fn new() -> Result<Self, crate::Error> {
        let mut info = MaybeUninit::uninit();

        // SAFETY: the WinAPI call for this is pretty safe, if this fails then
        // TIME_ZONE_ID_INVALID is returned (represented essentially as u32::MAX)
        let code = unsafe { GetTimeZoneInformation(info.as_mut_ptr()) };

        if code == u32::MAX {
            return Err(crate::Error::NoLocalTime);
        }

        // SAFETY: at this point, the WinAPI returned without errors
        let info = unsafe { info.assume_init() };

        // Essentially we want to convert this structure into a pre-existing PosixTimeZone rule
        // since they're usually equivalent and saves us the repetitive code.

        let std_abbr = windows_utf16_to_utf8(&info.StandardName).ok_or(crate::Error::NoLocalTime)?;

        // For these attributes, the unit for the biases are in minutes
        // The Windows documentation mentions that bias is calculated
        // UTC = local time + bias
        // So, e.g. UTC-5 is represented as a bias of 300
        let std_offset =
            UtcOffset::from_seconds((info.Bias + info.StandardBias) * -60).ok_or(crate::Error::NoLocalTime)?;

        // No DST transition information
        let dst = if code == 0 {
            None
        } else {
            let dst_name = windows_utf16_to_utf8(&info.DaylightName).unwrap_or_else(|| std_abbr.clone());
            let offset =
                UtcOffset::from_seconds((info.Bias + info.DaylightBias) * -60).ok_or(crate::Error::NoLocalTime)?;

            if info.DaylightDate.wYear != 0 {
                None
            } else {
                let start = DstTransitionRule::Calendar {
                    month: info.DaylightDate.wMonth as u8,
                    n: info.DaylightDate.wDay as u8,
                    weekday: info.DaylightDate.wDayOfWeek as u8,
                    offset: system_time_to_offset_seconds(&info.DaylightDate),
                };
                let end = DstTransitionRule::Calendar {
                    month: info.StandardDate.wMonth as u8,
                    n: info.StandardDate.wDay as u8,
                    weekday: info.StandardDate.wDayOfWeek as u8,
                    offset: system_time_to_offset_seconds(&info.StandardDate),
                };
                Some(DstTransitionInfo {
                    abbr: dst_name,
                    base_offset: offset.saturating_sub(std_offset),
                    offset,
                    start,
                    end,
                })
            }
        };

        Ok(Self {
            inner: PosixTimeZone {
                std_abbr,
                std_offset,
                dst,
            },
        })
    }

    #[inline]
    pub(crate) fn as_inner(&self) -> &PosixTimeZone {
        &self.inner
    }
}
