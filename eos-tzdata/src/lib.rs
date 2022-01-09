pub mod data;

/// Represents an IANA tzdb zone entry.
pub struct ZoneEntry {
    /// The raw zone ID identifier. For example, `America/New_York` or `Europe/London`.
    pub zone: &'static str,
    /// The raw TZif data corresponding to that zone.
    pub data: &'static [u8],
}

/// const-safe string comparison utility
const fn cmp_strings(lhs: &str, rhs: &str) -> i8 {
    let (lhs_len, rhs_len) = (lhs.len(), rhs.len());
    let min_len = if lhs_len < rhs_len { lhs_len } else { rhs_len };
    let lhs = lhs.as_bytes();
    let rhs = rhs.as_bytes();

    let mut i = 0;
    while i < min_len {
        let (a, b) = (lhs[i], rhs[i]);
        if a < b {
            return -1;
        }
        if b < a {
            return 1;
        }
        i += 1;
    }

    if lhs_len == rhs_len {
        0
    } else if lhs_len < rhs_len {
        -1
    } else {
        1
    }
}

/// This is a binary search function except suitable for const contexts
#[doc(hidden)]
pub const fn binary_search_by_zone(data: &'static [ZoneEntry], zone: &str) -> Option<&'static [u8]> {
    let mut size = data.len();
    let mut left = 0;
    let mut right = size;
    while left < right {
        let middle = left + size / 2;
        let entry = &data[middle];
        let cmp = cmp_strings(entry.zone, zone);
        if cmp == -1 {
            left = middle + 1;
        } else if cmp == 1 {
            right = middle;
        } else {
            return Some(entry.data);
        }

        size = right - left;
    }
    None
}

/// Returns the TZif embedded data for a given path.
///
/// If the file is not found then a compiler error is emitted.
///
/// ```
/// # use::eos_tzdata::tzif;
/// let data = tzif!("America/New_York");
/// ```
#[macro_export]
macro_rules! tzif {
    ($location:literal) => {{
        const DATA: &'static [u8] = match $crate::binary_search_by_zone(&$crate::data::MAPPINGS, $location) {
            Some(entry) => entry,
            None => core::panic!(concat!("could not find a zone by name ", stringify!($location))),
        };
        DATA
    }};
}

/// Returns the TZif data associated with the given time zone ID.
///
/// ```
/// use eos_tzdata::locate_tzif;
///
/// assert!(locate_tzif("nonsense").is_none());
/// assert!(locate_tzif("Europe/London").is_some());
/// ```
pub const fn locate_tzif(zone: &str) -> Option<&'static [u8]> {
    binary_search_by_zone(&crate::data::MAPPINGS, zone)
}
