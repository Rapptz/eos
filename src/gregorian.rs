//! Low level algorithms pertaining to the Gregorian calendar.

/*

A vast majority of these algorithms came from sources far smarter than I.

Thanks to Howard Hinnant for his clear and easy to read algorithms (http://howardhinnant.github.io/date_algorithms.html)
Thanks to the PSF for their `datetime` class being an inspiration of design.
Thanks to Joda-Time and Noda-Time for being inspirations as well.

*/

// The first index is unused
pub(crate) const DAYS_IN_MONTH: [u8; 13] = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
pub(crate) const DAYS_BEFORE_MONTH: [u16; 13] = [0, 0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

/// Returns `true` if the year is a leap year or not.
#[inline]
pub const fn is_leap_year(year: i16) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Returns the number of days in that given month and year.
#[inline]
pub const fn days_in_month(year: i16, month: u8) -> u8 {
    if month == 2 && is_leap_year(year) {
        29
    } else {
        DAYS_IN_MONTH[month as usize]
    }
}

/// Returns how many days preceed the first day of the given month in the year.
#[inline]
pub const fn days_before_month(year: i16, month: u8) -> u16 {
    let offset = month > 2 && is_leap_year(year);
    DAYS_BEFORE_MONTH[month as usize] + offset as u16
}

/// Returns the ordinal date of a given year, month, and day.
///
/// # Panics
///
/// In debug mode, this panics if the day is out of range for the given month.
/// Note that in `const` contexts this becomes a compile time error.
#[inline]
pub const fn date_to_ordinal(year: i16, month: u8, day: u8) -> u16 {
    debug_assert!(day >= 1 && day <= days_in_month(year, month), "day is out of range");
    days_before_month(year, month) + day as u16
}

const ERA_DURATION: i32 = 400;
/// The number of days in a 400 year period
const DAYS_IN_ERA: i32 = 146097;
/// The number of days to go from 0000-03-01 to 1970-01-01
const DAYS_TO_EPOCH: i32 = 719468;

/// Returns the number of days from 1970-01-01 of a given year, month, and day.
///
/// Note that this method does *not* do any bound checking on `month` or `day` values.
/// Feeding the algorithm garbage values will return garbage output, a classic case of
/// garbage-in-garbage-out.
#[inline]
pub const fn date_to_epoch_days(year: i16, month: u8, day: u8) -> i32 {
    let y = year as i32 - (month <= 2) as i32;
    let era = y.div_euclid(ERA_DURATION);
    let year_of_era = y - era * ERA_DURATION; // [0, 399]
    let m = if month > 2 { month - 3 } else { month + 9 } as i32;
    let day_of_year = (153 * m + 2) / 5 + day as i32 - 1; // [0, 365]
    let day_of_era = 365 * year_of_era + year_of_era / 4 - year_of_era / 100 + day_of_year; // [0, 146096]
    era * DAYS_IN_ERA + day_of_era - DAYS_TO_EPOCH
}

/// Converts the number of days since 1970-01-01 into a (year, month, day) tuple.
#[inline]
pub const fn date_from_epoch_days(mut days: i32) -> (i16, u8, u8) {
    // Should be noted that the bounds within `epoch` far exceed our i16 year
    // In this case, it should saturate
    days += DAYS_TO_EPOCH;
    let era = days.div_euclid(DAYS_IN_ERA);
    let day_of_era = days - era * DAYS_IN_ERA; // [0, 146096]
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365; // [0, 399]
    let mut y = year_of_era + era * ERA_DURATION;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100); // [0, 365]
    let mp = (5 * day_of_year + 2) / 153; // [0, 11]
    let d = day_of_year - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    y += (m <= 2) as i32;
    let y = if y > i16::MAX as i32 {
        i16::MAX
    } else if y < i16::MIN as i32 {
        i16::MIN
    } else {
        y as i16
    };
    (y, m as u8, d as u8)
}

/// The minimum allowed epoch days that this library supports.
pub const MIN_EPOCH_DAYS: i32 = date_to_epoch_days(i16::MIN, 1, 1);

/// The maximum allowed epoch days that this library supports.
pub const MAX_EPOCH_DAYS: i32 = date_to_epoch_days(i16::MAX, 12, 31);

/// Returns the weekday for December 31st of a given year
///
/// Note that 0 is Sunday and 6 is Saturday.
#[inline]
pub const fn end_of_year_weekday(year: i16) -> u8 {
    let count = year + year / 4 - year / 100 + year / 400;
    count.rem_euclid(7) as u8
}

/// Returns the number of ISO weeks in a given year
#[inline]
pub const fn iso_weeks_in_year(year: i16) -> u8 {
    if end_of_year_weekday(year) == 4 || end_of_year_weekday(year - 1) == 3 {
        53
    } else {
        52
    }
}

/// Returns the epoch representing the start of an ISO year.
#[inline]
pub(crate) const fn iso_week_start_epoch_from_year(year: i16) -> i32 {
    let epoch = date_to_epoch_days(year, 1, 4);
    let weekday = weekday_from_days(epoch);
    // difference from Monday
    epoch - weekday_difference(weekday, 1) as i32
}

/// Determines where the ISO week starts from a given year and epoch of date.
pub(crate) const fn find_iso_week_start_epoch(year: i16, epoch: i32) -> i32 {
    let start = iso_week_start_epoch_from_year(year);
    if epoch < start {
        return iso_week_start_epoch_from_year(year - 1);
    }
    let next_start = iso_week_start_epoch_from_year(year + 1);
    if epoch >= next_start {
        next_start
    } else {
        start
    }
}

/// Returns the difference between two weekdays.
///
/// This returns a number between [0, 6] and assumes both x and y are <= 6.
///
/// Note that 0 is Sunday and 6 is Saturday.
#[inline]
pub const fn weekday_difference(x: u8, y: u8) -> u8 {
    let x = x.wrapping_sub(y);
    if x <= 6 {
        x
    } else {
        x.wrapping_add(7)
    }
}

/// Returns the weekday from the days after 1970-01-01.
///
/// Note that 0 is Sunday and 6 is Saturday.
#[inline]
pub const fn weekday_from_days(civil: i32) -> u8 {
    (civil + 4).rem_euclid(7) as u8
}

/// Returns the next weekday, given a weekday.
///
/// Note that 0 is Sunday and 6 is Saturday
#[inline]
pub const fn next_weekday(wd: u8) -> u8 {
    if wd < 6 {
        wd + 1
    } else {
        0
    }
}

/// Returns the previous weekday, given a weekday.
///
/// Note that 0 is Sunday and 6 is Saturday
#[inline]
pub const fn prev_weekday(wd: u8) -> u8 {
    if wd > 0 {
        wd - 1
    } else {
        6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_level_algorithms() {
        assert_eq!(date_to_epoch_days(1970, 1, 1), 0);
        assert_eq!(date_from_epoch_days(0), (1970, 1, 1));
        assert_eq!(weekday_from_days(date_to_epoch_days(1970, 1, 1)), 4);

        let mut prev_z = date_to_epoch_days(i16::MIN, 1, 1) - 1;
        assert!(prev_z < 0);
        let mut prev_wd = weekday_from_days(prev_z);
        assert!(prev_wd <= 6);

        for y in i16::MIN..=i16::MAX {
            for m in 1..=12 {
                let e = days_in_month(y, m);
                for d in 1..=e {
                    let z = date_to_epoch_days(y, m, d);
                    assert!(prev_z < z);
                    assert_eq!(prev_z + 1, z);
                    let (y2, m2, d2) = date_from_epoch_days(z);
                    assert_eq!((y, m, d), (y2, m2, d2));
                    let wd = weekday_from_days(z);
                    assert!(wd <= 6);
                    assert_eq!(wd, next_weekday(prev_wd));
                    assert_eq!(prev_wd, prev_weekday(wd));
                    prev_z = z;
                    prev_wd = wd;
                }
            }
        }
    }

    #[test]
    fn test_weekday_diff() {
        #[rustfmt::skip]
        const TESTS: [[u8; 7]; 7] =
            [// -    Sun Mon Tue Wed Thu Fri Sat
             /*Sun*/ [0,  6,  5,  4,  3,  2,  1],
             /*Mon*/ [1,  0,  6,  5,  4,  3,  2],
             /*Tue*/ [2,  1,  0,  6,  5,  4,  3],
             /*Wed*/ [3,  2,  1,  0,  6,  5,  4],
             /*Thu*/ [4,  3,  2,  1,  0,  6,  5],
             /*Fri*/ [5,  4,  3,  2,  1,  0,  6],
             /*Sat*/ [6,  5,  4,  3,  2,  1,  0],
            ];

        for x in 0..7 {
            for y in 0..7 {
                assert_eq!(weekday_difference(x, y), TESTS[x as usize][y as usize]);
            }
        }
    }
}
