use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};

use eos::{
    gregorian::{date_to_epoch_days, days_in_month, is_leap_year, weekday_difference, weekday_from_days},
    utc_offset, Time, UtcOffset,
};

use crate::{error::ParseError, timestamp::NaiveTimestamp};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum DstTransitionRule {
    /// The Julian day. The first element is 1 <= n <= 365.
    /// Leap days aren't counted and it's impossible to refer to February 29.
    /// The second element is offset in seconds from midnight when the transition takes place.
    JulianDay(u16, i64),
    /// A zero-based Julian day. The first element is 0 <= n <= 365. Leap days are counted.
    /// The second element is offset in seconds from midnight when the transition takes place.
    Day(u16, i64),
    /// The `weekday` (0 <= weekday <= 6) of week `n` (1 <= n <= 5) of the `month` (1 <= month <= 12).
    /// A value of `n == 5` means the last `weekday` of the `month`. `weekday == 0` is Sunday.
    /// The offset is seconds from midnight when the transition takes place.
    Calendar { month: u8, n: u8, weekday: u8, offset: i64 },
}

impl DstTransitionRule {
    /// Returns a new DstTransitionRule with a newly set number of seconds from midnight
    /// local time.
    fn with_offset(self, offset: i64) -> Self {
        match self {
            Self::JulianDay(d, _) => Self::JulianDay(d, offset),
            Self::Day(d, _) => Self::Day(d, offset),
            Self::Calendar { month, n, weekday, .. } => Self::Calendar {
                month,
                n,
                weekday,
                offset,
            },
        }
    }
    pub(crate) fn timestamp_in_year(&self, year: i16) -> NaiveTimestamp {
        match self {
            Self::JulianDay(day, offset) => {
                let d = if *day >= 59 && is_leap_year(year) {
                    day + 1
                } else {
                    *day
                };
                let epoch = date_to_epoch_days(year, 1, 1) as i64;
                let seconds = (epoch - 1 + d as i64) * 86400 + offset;
                NaiveTimestamp::from_seconds(seconds)
            }
            Self::Day(day, offset) => {
                // day is already range checked as part of the contract
                let epoch = date_to_epoch_days(year, 1, 1) as i64;
                let seconds = (epoch - 1 + *day as i64) * 86400 + offset;
                NaiveTimestamp::from_seconds(seconds)
            }
            Self::Calendar {
                month,
                n,
                weekday,
                offset,
            } => {
                let first_weekday = weekday_from_days(date_to_epoch_days(year, *month, 1));
                let days_in_month = days_in_month(year, *month);
                let mut day = weekday_difference(*weekday, first_weekday) + 1 + (n - 1) * 7;
                if day > days_in_month {
                    day -= 7;
                }
                let epoch = date_to_epoch_days(year, *month, day) as i64;
                let seconds = epoch * 86400 + offset;
                NaiveTimestamp::from_seconds(seconds)
            }
        }
    }
}

fn display_time(f: &mut std::fmt::Formatter<'_>, offset: i64) -> std::fmt::Result {
    let (hours, seconds) = (offset.div_euclid(3600), offset.rem_euclid(3600));
    let (minutes, seconds) = (seconds.div_euclid(60), seconds.rem_euclid(60));
    if minutes != 0 || seconds != 0 {
        write!(f, "{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        write!(f, "{}", hours)
    }
}

impl std::fmt::Display for DstTransitionRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            DstTransitionRule::JulianDay(d, offset) => {
                write!(f, "J{}/", d)?;
                display_time(f, *offset)?;
            }
            DstTransitionRule::Day(d, offset) => {
                write!(f, "{}/", d)?;
                display_time(f, *offset)?;
            }
            DstTransitionRule::Calendar {
                month,
                n,
                weekday,
                offset,
            } => {
                write!(f, "M{}.{}.{}/", month, n, weekday)?;
                display_time(f, *offset)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct DstTransitionInfo {
    pub(crate) abbr: String,
    pub(crate) offset: UtcOffset,
    pub(crate) start: DstTransitionRule,
    pub(crate) end: DstTransitionRule,
    pub(crate) base_offset: UtcOffset,
}

impl DstTransitionInfo {
    /// Returns true if DST is active
    pub(crate) fn is_active(&self, date: &eos::Date, time: &eos::Time) -> bool {
        let ts = NaiveTimestamp::new(date, time);
        let start = self.start.timestamp_in_year(date.year());
        let end = self.end.timestamp_in_year(date.year());
        if start < end {
            start <= ts && ts < end
        } else {
            !(end <= ts && ts < start)
        }
    }

    /// Returns true if the UNIX timestamp is in DST
    pub(crate) fn is_dst_utc(&self, ts: eos::Timestamp, std_offset: &UtcOffset) -> bool {
        let utc = ts.to_utc();
        let start = self.start.timestamp_in_year(utc.year()).to_regular(std_offset);
        let end = self.end.timestamp_in_year(utc.year()).to_regular(&self.offset);
        if start < end {
            start <= ts && ts < end
        } else {
            !(end <= ts && ts < start)
        }
    }
}

/// A POSIX-string specified time zone rule.
///
/// The details of this format are specified under the POSIX TZ rules
/// under [Section 8.3]. These mainly show up either in the `TZ` environment
/// variable or at the end of a TZif footer as specified by [RFC8536].
///
/// The typical way to create a [`PosixTimeZone`] is through the [`FromStr`] trait
/// or via [`PosixTimeZone::new`].
///
/// ```
/// use eos_tz::PosixTimeZone;
/// use std::str::FromStr;
///
/// let tz = PosixTimeZone::from_str("EST+5EDT,M3.2.0/2,M11.1.0/2")?;
/// # Ok::<_, eos_tz::ParseError>(())
/// ```
///
/// [Section 8.3]: https://pubs.opengroup.org/onlinepubs/9699919799/
/// [RFC8536]: https://datatracker.ietf.org/doc/html/rfc8536#section-3.3
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PosixTimeZone {
    pub(crate) std_abbr: String,
    pub(crate) std_offset: UtcOffset,
    pub(crate) dst: Option<DstTransitionInfo>,
}

impl PosixTimeZone {
    /// Creates a new [`PosixTimeZone`] with the given TZ string.
    pub fn new(tz: &str) -> Result<Self, ParseError> {
        // std[offset[dst[offset],start[/time],end[/time]]]
        let mut parser = tz.chars().peekable();
        let std_abbr = parse_abbr(&mut parser)?;
        let std_offset = if parser.peek().is_none() {
            UtcOffset::default()
        } else {
            parse_offset(&mut parser)?
        };
        let dst = {
            match parser.peek() {
                None => None,
                Some(_) => {
                    let abbr = parse_abbr(&mut parser)?;
                    let offset = match parser.peek() {
                        Some(',') => {
                            parser.next();
                            std_offset.saturating_add(utc_offset!(01:00))
                        }
                        Some(_) => {
                            let offset = parse_offset(&mut parser)?;
                            if parser.next_if_eq(&',').is_none() {
                                return Err(ParseError::InvalidPosixTz);
                            }
                            offset
                        }
                        None => return Err(ParseError::InvalidPosixTz),
                    };
                    let start = parse_dst_transition_rule(&mut parser)?;
                    if parser.next_if_eq(&',').is_none() {
                        return Err(ParseError::InvalidPosixTz);
                    }
                    let end = parse_dst_transition_rule(&mut parser)?;
                    let base_offset = offset.saturating_sub(std_offset);
                    Some(DstTransitionInfo {
                        abbr,
                        offset,
                        start,
                        end,
                        base_offset,
                    })
                }
            }
        };
        Ok(PosixTimeZone {
            std_abbr,
            std_offset,
            dst,
        })
    }

    /// Returns `true` if the given local date and time are in DST
    pub fn is_dst(&self, date: &eos::Date, time: &eos::Time) -> bool {
        match &self.dst {
            Some(dst) => dst.is_active(date, time),
            None => false,
        }
    }

    pub(crate) fn shift_utc(&self, utc: &mut eos::DateTime<eos::Utc>) {
        let ts = NaiveTimestamp::new(&utc.date(), &utc.time());
        match self.dst.as_ref() {
            None => {
                utc.shift(self.std_offset);
            }
            Some(dst) => {
                let mut dst_on = dst.start.timestamp_in_year(utc.year());
                let mut dst_off = dst.end.timestamp_in_year(utc.year());
                dst_on.0 -= self.std_offset.total_seconds() as i64;
                dst_off.0 -= dst.offset.total_seconds() as i64;

                let is_dst = if dst_on < dst_off {
                    dst_on <= ts && ts < dst_off
                } else {
                    !(dst_off <= ts && ts < dst_on)
                };
                if is_dst {
                    utc.shift(dst.offset);
                } else {
                    utc.shift(self.std_offset);
                }
            }
        }
    }

    /// A "hack" to partially construct an eos::DateTimeResolution due to the lack of
    /// Copy semantics in this type and how it requires moving the TimeZone type.
    pub(crate) fn partial_resolution(
        &self,
        date: &eos::Date,
        time: &eos::Time,
    ) -> (eos::DateTimeResolutionKind, UtcOffset, UtcOffset) {
        match &self.dst {
            Some(dst) => {
                let ts = NaiveTimestamp::new(date, time).into_inner();
                // Ambiguous time if the offset is positive happens when DST ends,
                // otherwise it happens when DST starts.
                // On the other hand, when the offset is positive missing times happen
                // when DST starts, otherwise when it ends.
                // This can get pretty confusing, admittedly.
                let dst_diff = dst.base_offset.total_seconds() as i64;
                let end = dst.end.timestamp_in_year(date.year()).into_inner();
                let start = dst.start.timestamp_in_year(date.year()).into_inner();
                let is_dst = if start < end {
                    start <= ts && ts < end
                } else {
                    !(end <= ts && ts < start)
                };
                if dst_diff > 0 {
                    // Ambiguous:
                    // if DST ends at 2AM and we go back 1 hour (positive offset)
                    // then 1:30am >= (2am - 1hr) && 1:30am < 2am
                    // Missing:
                    // if DST starts 2AM and we go forward 1 hour (positive offset)
                    // then 2:30am is missing
                    // so: 2:30am >= 2AM && 2:30am < (2am + 1 hour)
                    if (end - dst_diff) <= ts && ts < end {
                        (eos::DateTimeResolutionKind::Ambiguous, dst.offset, self.std_offset)
                    } else if start <= ts && ts < (start + dst_diff) {
                        (eos::DateTimeResolutionKind::Missing, self.std_offset, dst.offset)
                    } else if is_dst {
                        (eos::DateTimeResolutionKind::Unambiguous, dst.offset, dst.offset)
                    } else {
                        (
                            eos::DateTimeResolutionKind::Unambiguous,
                            self.std_offset,
                            self.std_offset,
                        )
                    }
                } else {
                    // Ambiguous
                    // This is actually the opposite of the above
                    // If DST starts at 1AM and we go back an hour (negative offset)
                    // then if 12:30am is ambiguous because
                    // (1am + -1hr) <= 12:30am && 12:30 am < 1am
                    // The earlier time would be before DST starts (e.g. UTC+1) and the later time would
                    // be the DST transition (e.g. UTC+0)
                    // Missing:
                    // If DST ends at 1AM and we go forward an hour (negative offset)
                    // then 1:30AM is unrepresentable
                    // so: 1:30 AM >= 1AM && 1:30AM < (1AM - -1hr)
                    // Similar to above the "earlier" gap is the DST time (UTC+0) and the later time is
                    // after DST ends (UTC+1).
                    if (start + dst_diff) <= ts && ts < start {
                        (eos::DateTimeResolutionKind::Ambiguous, self.std_offset, dst.offset)
                    } else if end <= ts && ts < (end - dst_diff) {
                        (eos::DateTimeResolutionKind::Missing, dst.offset, self.std_offset)
                    } else if is_dst {
                        (eos::DateTimeResolutionKind::Unambiguous, dst.offset, dst.offset)
                    } else {
                        (
                            eos::DateTimeResolutionKind::Unambiguous,
                            self.std_offset,
                            self.std_offset,
                        )
                    }
                }
            }
            None => (
                eos::DateTimeResolutionKind::Unambiguous,
                self.std_offset,
                self.std_offset,
            ),
        }
    }
}

impl eos::TimeZone for PosixTimeZone {
    fn name(&self, ts: eos::Timestamp) -> Option<&str> {
        match &self.dst {
            Some(dst) => {
                if dst.is_dst_utc(ts, &self.std_offset) {
                    Some(dst.abbr.as_str())
                } else {
                    Some(self.std_abbr.as_str())
                }
            }
            None => Some(self.std_abbr.as_str()),
        }
    }

    fn offset(&self, ts: eos::Timestamp) -> UtcOffset {
        match &self.dst {
            Some(dst) => {
                if dst.is_dst_utc(ts, &self.std_offset) {
                    dst.offset
                } else {
                    self.std_offset
                }
            }
            None => self.std_offset,
        }
    }

    fn resolve(self, date: eos::Date, time: Time) -> eos::DateTimeResolution<Self>
    where
        Self: Sized,
    {
        let (kind, earlier, later) = self.partial_resolution(&date, &time);
        match kind {
            eos::DateTimeResolutionKind::Missing => eos::DateTimeResolution::missing(date, time, earlier, later, self),
            eos::DateTimeResolutionKind::Unambiguous => eos::DateTimeResolution::unambiguous(date, time, earlier, self),
            eos::DateTimeResolutionKind::Ambiguous => {
                eos::DateTimeResolution::ambiguous(date, time, earlier, later, self)
            }
        }
    }

    fn convert_utc(self, mut utc: eos::DateTime<eos::Utc>) -> eos::DateTime<Self>
    where
        Self: Sized,
    {
        self.shift_utc(&mut utc);
        utc.with_timezone(self)
    }

    fn is_fixed(&self) -> bool {
        self.dst.is_none()
    }
}

/*
    The format for the TZ string has the following format:

    std[offset[dst[offset],start[/time],end[/time]]]

    std and dst must be between 3 to TZNAME_MAX characters long and
    may be quoted using < and >.

    The format of the offset is `[+-]hh[:mm[:ss]]`.

    The DST rule format is `date[/time],date[/time]`. It must be provided
    if a DST offset is provided. The `date` rule can be:

    J<n> where 1 <= n <= 365 which is days not including leap years. So day 59
    is Feb 28th and day 60 is March 1st.

    <n> where 0 <= n <= 365 which is days including leap years.

    M<m>.<n>.<d> which is `n`th weekday (`d`) of month `m` where `d` starts at
    0 = Sunday and 6 = Monday. `n=5` means last `d` of the month.

    The `time` rule is the same as the `offset` rule except without a leading +/-.
*/

type Parser<'a> = Peekable<Chars<'a>>;
type ParseResult<T> = Result<T, ParseError>;

fn parse_abbr(parser: &mut Parser) -> ParseResult<String> {
    // abbr are disambiguated by the fact that one of these must hold:
    // if it starts with < then it continues until it finds a >
    // else it continues until it finds one of the forbidden characters.
    // In regex terms this boils down to (<[a-zA-Z0-9+\-]+>|[^<0-9:.+-]+)
    let mut abbr = String::new();

    match parser.next() {
        Some('<') => {
            for ch in parser.by_ref() {
                if ch.is_ascii_alphanumeric() || ch == '+' || ch == '-' {
                    abbr.push(ch);
                } else if ch == '>' {
                    break;
                } else {
                    return Err(ParseError::InvalidPosixTz);
                }
            }
        }
        Some(c) => {
            if c.is_alphabetic() {
                abbr.push(c);
            } else {
                return Err(ParseError::InvalidPosixTz);
            }
            while let Some(c) = parser.peek() {
                if c.is_alphabetic() {
                    abbr.push(*c);
                    parser.next();
                } else {
                    break;
                }
            }
        }
        None => {
            return Err(ParseError::InvalidPosixTz);
        }
    }

    if abbr.len() < 3 {
        Err(ParseError::InvalidPosixTz)
    } else {
        Ok(abbr)
    }
}

#[inline]
fn parse_digit(parser: &mut Parser) -> ParseResult<u8> {
    match parser.next() {
        Some(c) if c.is_ascii_digit() => Ok(c as u8 - b'0'),
        _ => Err(ParseError::InvalidPosixTz),
    }
}

fn parse_three_digit_number(parser: &mut Parser) -> ParseResult<u16> {
    let mut read_any: bool = false;
    let mut n: u16 = 0;
    for _ in 0..3 {
        match parser.next_if(char::is_ascii_digit) {
            Some(c) => {
                n = n * 10 + (c as u8 - b'0') as u16;
                read_any = true;
            }
            None => break,
        }
    }

    if read_any {
        Ok(n)
    } else {
        Err(ParseError::InvalidPosixTz)
    }
}

fn parse_offset(parser: &mut Parser) -> ParseResult<UtcOffset> {
    // ([+-]?\d{1,2}(:\d{2}(:\d{2})?)?)
    // In POSIX, this is actually swapped.
    // HST10 is UTC -10 not UTC+10
    // + is *west* or left of UTC (i.e. negative) and - is *east* of UTC (i.e. positive).
    let negative = match parser.peek() {
        Some('+') => {
            parser.next();
            true
        }
        Some('-') => {
            parser.next();
            false
        }
        None => return Err(ParseError::InvalidPosixTz),
        _ => true,
    };

    let hours = match parser.next() {
        Some(c) if c.is_ascii_digit() => match parser.next_if(char::is_ascii_digit) {
            Some(d) => (c as u8 - b'0') * 10 + (d as u8 - b'0'),
            None => c as u8 - b'0',
        },
        _ => return Err(ParseError::InvalidPosixTz),
    };

    let minutes = match parser.next_if(|x| *x == ':') {
        Some(_) => match parser.next().zip(parser.next()) {
            Some((c, d)) => (c as u8 - b'0') * 10 + (d as u8 - b'0'),
            None => return Err(ParseError::InvalidPosixTz),
        },
        None => 0u8,
    };

    let seconds = match parser.next_if(|x| *x == ':') {
        Some(_) => match parser.next().zip(parser.next()) {
            Some((c, d)) => (c as u8 - b'0') * 10 + (d as u8 - b'0'),
            None => return Err(ParseError::InvalidPosixTz),
        },
        None => 0u8,
    };

    let mut seconds = hours as i32 * 3600 + minutes as i32 * 60 + seconds as i32;
    if negative {
        seconds = -seconds;
    }

    UtcOffset::from_seconds(seconds).ok_or(ParseError::InvalidOffset)
}

fn parse_time(parser: &mut Parser) -> ParseResult<i64> {
    let hour = match parser.next() {
        Some(c) if c.is_ascii_digit() => match parser.next_if(char::is_ascii_digit) {
            Some(d) => (c as u8 - b'0') * 10 + (d as u8 - b'0'),
            None => c as u8 - b'0',
        },
        _ => return Err(ParseError::InvalidPosixTz),
    };

    let minute = match parser.next_if(|x| *x == ':') {
        Some(_) => match parser.next().zip(parser.next()) {
            Some((c, d)) => (c as u8 - b'0') * 10 + (d as u8 - b'0'),
            None => return Err(ParseError::InvalidPosixTz),
        },
        None => 0u8,
    };

    let second = match parser.next_if(|x| *x == ':') {
        Some(_) => match parser.next().zip(parser.next()) {
            Some((c, d)) => (c as u8 - b'0') * 10 + (d as u8 - b'0'),
            None => return Err(ParseError::InvalidPosixTz),
        },
        None => 0u8,
    };

    Ok(hour as i64 * 3600 + minute as i64 * 60 + second as i64)
}

fn parse_dst_transition_rule(parser: &mut Parser) -> ParseResult<DstTransitionRule> {
    // date[/time]
    // date can be either (J\d{1,3}|\d{1,3}|M\d{1,2}.\d.\d)
    let rule = match parser.next() {
        Some('J') => {
            let day = parse_three_digit_number(parser)?;
            if day < 1 || day > 365 {
                return Err(ParseError::InvalidPosixTz);
            }
            DstTransitionRule::JulianDay(day, 0)
        }
        Some('M') => {
            let month = match parser.next() {
                Some(c) if c.is_ascii_digit() => match parser.next_if(char::is_ascii_digit) {
                    Some(d) => (c as u8 - b'0') * 10 + (d as u8 - b'0'),
                    None => c as u8 - b'0',
                },
                _ => return Err(ParseError::InvalidPosixTz),
            };
            parser.next_if_eq(&'.').ok_or(ParseError::InvalidPosixTz)?;
            let n = parse_digit(parser)?;
            parser.next_if_eq(&'.').ok_or(ParseError::InvalidPosixTz)?;
            let weekday = parse_digit(parser)?;
            if month > 12 || month < 1 || n < 1 || n > 5 || weekday > 6 {
                return Err(ParseError::InvalidPosixTz);
            }

            DstTransitionRule::Calendar {
                month,
                n,
                weekday,
                offset: 0,
            }
        }
        Some(c) if c.is_ascii_digit() => {
            let day = parse_three_digit_number(parser)?;
            if day > 365 {
                return Err(ParseError::InvalidPosixTz);
            }
            DstTransitionRule::Day(day, 0)
        }
        _ => return Err(ParseError::InvalidPosixTz),
    };

    let offset = match parser.next_if(|&x| x == '/') {
        Some(_) => parse_time(parser)?,
        None => 7200,
    };

    Ok(rule.with_offset(offset))
}

impl FromStr for PosixTimeZone {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

fn display_offset(f: &mut std::fmt::Formatter<'_>, offset: &UtcOffset) -> std::fmt::Result {
    // Offsets are swapped in POSIX timezones.
    // i.e. an offset of -5 UTC is represented as 5
    let hours = -offset.hours();
    let (minutes, seconds) = (offset.minutes().abs(), offset.seconds().abs());
    if minutes != 0 || seconds != 0 {
        write!(f, "{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        write!(f, "{}", hours)
    }
}

impl std::fmt::Display for PosixTimeZone {
    /// Converts the [`PosixTimeZone`] back into its original representation.
    ///
    /// Note that this does *not* roundtrip and makes no guarantee to do so.
    /// It just returns a suitable display representing the original data
    /// faithfully.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // std[offset[dst[offset],start[/time],end[/time]]]
        if self.std_abbr.as_bytes().iter().any(|x| !x.is_ascii_alphabetic()) {
            write!(f, "<{}>", &self.std_abbr)?;
        } else {
            f.write_str(self.std_abbr.as_str())?;
        }
        match &self.dst {
            None => {
                if !self.std_offset.is_utc() {
                    display_offset(f, &self.std_offset)?;
                }
            }
            Some(dst) => {
                display_offset(f, &self.std_offset)?;
                if dst.abbr.as_bytes().iter().any(|x| !x.is_ascii_alphabetic()) {
                    write!(f, "<{}>", &dst.abbr)?;
                } else {
                    f.write_str(dst.abbr.as_str())?;
                }
                let default = self.std_offset.saturating_add(utc_offset!(01:00));
                if dst.offset != default {
                    display_offset(f, &dst.offset)?;
                }
                write!(f, ",{},{}", &dst.start, &dst.end)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use eos::{datetime, ext::IntervalLiteral, DateTime, TimeZone, Utc};

    use super::*;

    #[test]
    fn test_utc_posix() {
        let result = PosixTimeZone::from_str("UTC");
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.std_abbr, "UTC");
        assert_eq!(result.dst, None);
        assert_eq!(result.std_offset, UtcOffset::default());

        // UTC times are always unambiguous
        let dt = datetime!(2012-02-29 3:00 am);
        let resolved = result.resolve(dt.date(), dt.time());
        assert!(resolved.is_unambiguous());
        let resolved = resolved.lenient();
        assert_eq!(resolved, dt);
        assert_eq!(resolved.tzname(), Some("UTC"));
    }

    #[test]
    fn test_est_posix() {
        // In 2007 EST changed the DST transition to 2nd Sunday of March at 2AM
        // to the 1st Sunday of November at 2AM.
        let result = PosixTimeZone::from_str("EST+5EDT,M3.2.0/2,M11.1.0/2");
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.std_abbr, "EST");
        assert_eq!(result.std_offset, utc_offset!(-05:00));
        assert!(result.dst.is_some());
        if let Some(dst) = &result.dst {
            assert_eq!(dst.abbr, "EDT");
            assert_eq!(dst.offset, utc_offset!(-04:00));
            assert_eq!(
                dst.start,
                DstTransitionRule::Calendar {
                    month: 3,
                    n: 2,
                    weekday: 0,
                    offset: 7200,
                }
            );
            assert_eq!(
                dst.end,
                DstTransitionRule::Calendar {
                    month: 11,
                    n: 1,
                    weekday: 0,
                    offset: 7200
                }
            );
        }

        let names = [
            (datetime!(2021-11-07 1:30 am -04:00), "EDT"),
            (datetime!(2021-11-07 1:30 am -05:00), "EST"),
            (datetime!(2021-11-07 12:30 am -04:00), "EDT"),
            (datetime!(2021-03-14 03:30 am -04:00), "EDT"),
        ];

        for (dt, name) in names {
            assert_eq!(result.name(dt.timestamp()), Some(name));
        }

        let local = datetime!(2021-11-07 1:30 am);
        let resolve = result.clone().resolve(local.date(), local.time());
        assert!(resolve.is_ambiguous());
        assert_eq!(resolve.clone().earlier().unwrap(), datetime!(2021-11-07 1:30 am -04:00));
        assert_eq!(resolve.clone().later().unwrap(), datetime!(2021-11-07 1:30 am -05:00));
        assert_eq!(resolve.lenient(), datetime!(2021-11-07 1:30 am -04:00));

        // This is not ambiguous
        let unambiguous = datetime!(2021-11-07 12:30 am);
        let resolve = result.clone().resolve(unambiguous.date(), unambiguous.time());
        assert!(resolve.is_unambiguous());
        assert_eq!(
            resolve.clone().earlier().unwrap(),
            datetime!(2021-11-07 12:30 am -04:00)
        );
        assert_eq!(resolve.lenient(), datetime!(2021-11-07 12:30 am -04:00));

        // This is missing
        let missing = datetime!(2021-03-14 02:30 am);
        let resolve = result.resolve(missing.date(), missing.time());
        assert!(resolve.is_missing());
        assert!(resolve.clone().earlier().is_err());
        assert!(resolve.clone().later().is_err());
        assert_eq!(resolve.lenient(), datetime!(2021-03-14 03:30 am -04:00));
    }

    #[test]
    fn test_aest_posix() {
        let result = PosixTimeZone::from_str("AEST-10AEDT,M10.1.0/2,M4.1.0/3");
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.std_abbr, "AEST");
        assert_eq!(result.std_offset, utc_offset!(10:00));
        assert!(result.dst.is_some());
        if let Some(dst) = &result.dst {
            assert_eq!(dst.abbr, "AEDT");
            assert_eq!(dst.offset, utc_offset!(11:00));
            assert_eq!(
                dst.start,
                DstTransitionRule::Calendar {
                    month: 10,
                    n: 1,
                    weekday: 0,
                    offset: 7200,
                }
            );
            assert_eq!(
                dst.end,
                DstTransitionRule::Calendar {
                    month: 4,
                    n: 1,
                    weekday: 0,
                    offset: 10800,
                }
            );
        }

        let names = [
            (datetime!(2022-04-03 2:30 am +11:00), "AEDT"),
            (datetime!(2022-04-03 2:30 am +10:00), "AEST"),
            (datetime!(2022-04-03 1:30 am +11:00), "AEDT"),
            (datetime!(2021-10-03 03:30 am +11:00), "AEDT"),
        ];

        for (dt, name) in names {
            assert_eq!(result.name(dt.timestamp()), Some(name));
        }

        let local = datetime!(2022-04-03 2:30 am);
        let resolve = result.clone().resolve(local.date(), local.time());
        assert!(resolve.is_ambiguous());
        assert_eq!(resolve.clone().earlier().unwrap(), datetime!(2022-04-03 2:30 am +11:00));
        assert_eq!(resolve.clone().later().unwrap(), datetime!(2022-04-03 2:30 am +10:00));
        assert_eq!(resolve.lenient(), datetime!(2022-04-03 2:30 am +11:00));

        // This is not ambiguous
        let unambiguous = datetime!(2022-04-03 1:30 am);
        let resolve = result.clone().resolve(unambiguous.date(), unambiguous.time());
        assert!(resolve.is_unambiguous());
        assert_eq!(resolve.clone().earlier().unwrap(), datetime!(2022-04-03 1:30 am +11:00));
        assert_eq!(resolve.lenient(), datetime!(2022-04-03 1:30 am +11:00));

        // This is missing
        let missing = datetime!(2021-10-03 02:30 am);
        let resolve = result.resolve(missing.date(), missing.time());
        assert!(resolve.is_missing());
        assert!(resolve.clone().earlier().is_err());
        assert!(resolve.clone().later().is_err());
        assert_eq!(resolve.lenient(), datetime!(2021-10-03 03:30 am +11:00));
    }

    #[test]
    fn test_america_santiago() {
        let result = PosixTimeZone::from_str("<-04>4<-03>,M9.1.6/24,M4.1.6/24");
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.std_abbr, "-04");
        assert_eq!(result.std_offset, utc_offset!(-4:00));
        assert!(result.dst.is_some());
        if let Some(dst) = &result.dst {
            assert_eq!(dst.abbr, "-03");
            assert_eq!(dst.offset, utc_offset!(-03:00));
            assert_eq!(
                dst.start,
                DstTransitionRule::Calendar {
                    month: 9,
                    n: 1,
                    weekday: 6,
                    offset: 86400,
                }
            );
            assert_eq!(
                dst.end,
                DstTransitionRule::Calendar {
                    month: 4,
                    n: 1,
                    weekday: 6,
                    offset: 86400,
                }
            );
        }

        let names = [
            (datetime!(2022-04-02 23:30 -03:00), "-03"),
            (datetime!(2022-04-02 23:30 -04:00), "-04"),
            (datetime!(2022-04-02 22:59:59 -03:00), "-03"),
            (datetime!(2021-09-05 01:00 -03:00), "-03"),
        ];

        for (dt, name) in names {
            assert_eq!(result.name(dt.timestamp()), Some(name));
        }

        // America/Santiago is a bit of a weird edge case
        // DST begins at 2021-09-04 at 23:59:59 so essentially 2021-09-05 00:00
        // and an hour was added making their offset go from -4 -> -3
        // thus making 2021-09-05 00:00 -> 2021-09-05 00:59 unrepresentable
        // DST ends at 2022-04-02 at 23:59:59 so again essentially 2022-04-03 00:00
        // and it went back an hour so 23:00:00 to 00:00:00 is experienced again
        // This means the ambiguous times are 2022-04-02 23:00:00 to 2022-04-03 00:00:00

        let local = datetime!(2022-04-02 23:30:00);
        let resolve = result.clone().resolve(local.date(), local.time());
        assert!(resolve.is_ambiguous());
        assert_eq!(resolve.clone().earlier().unwrap(), datetime!(2022-04-02 23:30 -03:00));
        assert_eq!(resolve.clone().later().unwrap(), datetime!(2022-04-02 23:30 -04:00));
        assert_eq!(resolve.lenient(), datetime!(2022-04-02 23:30 -03:00));

        // This is not ambiguous
        let unambiguous = datetime!(2022-04-02 22:59:59);
        let resolve = result.clone().resolve(unambiguous.date(), unambiguous.time());
        assert!(resolve.is_unambiguous());
        assert_eq!(
            resolve.clone().earlier().unwrap(),
            datetime!(2022-04-02 22:59:59 -03:00)
        );
        assert_eq!(resolve.lenient(), datetime!(2022-04-02 22:59:59 -03:00));

        // This is missing
        let missing = datetime!(2021-09-05 00:00);
        let resolve = result.resolve(missing.date(), missing.time());
        assert!(resolve.is_missing());
        assert!(resolve.clone().earlier().is_err());
        assert!(resolve.clone().later().is_err());
        assert_eq!(resolve.lenient(), datetime!(2021-09-05 01:00 -03:00));
    }

    #[test]
    fn test_europe_dublin() {
        let result = PosixTimeZone::from_str("IST-1GMT0,M10.5.0,M3.5.0/1");
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.std_abbr, "IST");
        assert_eq!(result.std_offset, utc_offset!(1:00));
        assert!(result.dst.is_some());
        if let Some(dst) = &result.dst {
            assert_eq!(dst.abbr, "GMT");
            assert_eq!(dst.offset, utc_offset!(00:00));
            assert_eq!(
                dst.start,
                DstTransitionRule::Calendar {
                    month: 10,
                    n: 5,
                    weekday: 0,
                    offset: 7200,
                }
            );
            assert_eq!(
                dst.end,
                DstTransitionRule::Calendar {
                    month: 3,
                    n: 5,
                    weekday: 0,
                    offset: 3600,
                }
            );
        }

        let names = [
            (datetime!(2021-10-31 01:30 +01:00), "IST"),
            (datetime!(2021-10-31 01:30 +00:00), "GMT"),
            (datetime!(2022-03-27 00:00 +00:00), "GMT"),
            (datetime!(2022-03-27 02:30 +01:00), "IST"),
        ];

        for (dt, name) in names {
            assert_eq!(result.name(dt.timestamp()), Some(name));
        }

        // Europe/Dublin is unique in that it has a negative DST offset.
        // DST begins in 2021-10-31 2AM and we go back an hour to 2021-10-31 1AM (UTC+1 -> UTC+0)
        // DST ends at 2022-03-27 1AM and we go forward an hour to 2AM (UTC+0 -> UTC+1)
        // This means the ambiguous ranges and missing ranges are different from
        // your typical positive DST offset.
        // 2021-10-31 1:30AM is ambiguous since the we experienced it with UTC+1 and again with UTC+0
        // The earlier time would be 2021-10-31 1:30 AM UTC+1 and the later time would be UTC+0
        // Meanwhile, 2022-03-27 1:30 AM is unrepresentable because we skipped to 2AM.
        // The gap skip would mean that'd forward an hour so 2022-03-27 2:30 AM UTC+1

        let local = datetime!(2021-10-31 01:30:00);
        let resolve = result.clone().resolve(local.date(), local.time());
        assert!(resolve.is_ambiguous());
        assert_eq!(resolve.clone().earlier().unwrap(), datetime!(2021-10-31 01:30 +01:00));
        assert_eq!(resolve.clone().later().unwrap(), datetime!(2021-10-31 01:30 +00:00));
        assert_eq!(resolve.lenient(), datetime!(2021-10-31 01:30 +01:00));

        // This is not ambiguous
        let unambiguous = datetime!(2022-03-27 00:00);
        let resolve = result.clone().resolve(unambiguous.date(), unambiguous.time());
        assert!(resolve.is_unambiguous());
        assert_eq!(resolve.clone().earlier().unwrap(), datetime!(2022-03-27 00:00 +00:00));
        assert_eq!(resolve.lenient(), datetime!(2022-03-27 00:00 +00:00));

        // This is missing
        let missing = datetime!(2022-03-27 01:30);
        let resolve = result.resolve(missing.date(), missing.time());
        assert!(resolve.is_missing());
        assert!(resolve.clone().earlier().is_err());
        assert!(resolve.clone().later().is_err());
        assert_eq!(resolve.lenient(), datetime!(2022-03-27 02:30 +01:00));
    }

    const DST_START_2021: DateTime = datetime!(2021-3-14 2:00 am);
    const DST_END_2021: DateTime = datetime!(2021-11-7 1:00 am);

    #[test]
    fn test_timezone_transition() {
        // Modified from timezone_conversion.rs test case
        let tz = PosixTimeZone::from_str("EST+5EDT,M3.2.0/2,M11.1.0/2");
        assert!(tz.is_ok());
        let tz = tz.unwrap();

        let utc = Utc::now();
        // Unfortunately PosixTimeZone is *not* Copy which makes it awkward...
        let current = tz.clone().convert_utc(utc);
        assert_eq!(utc, current);

        /*
            UTC  4:00  5:00 6:00 7:00 8:00 9:00
            EDT  0:00  1:00 2:00 3:00 4:00 5:00
            EST 23:00  0:00 1:00 2:00 3:00 4:00
        */

        // start = UTC
        let mut start = DST_START_2021.with_hour(4).unwrap();
        for hour in [23, 0, 1, 3, 4, 5] {
            let mut expected = start.with_hour(hour).unwrap();
            if hour == 23 {
                expected = expected - 1.days();
            }
            let got = tz.clone().convert_utc(start);
            assert_eq!(expected.with_timezone(tz.clone()), got);

            start = start + 1.hours();
        }

        let mut start = DST_END_2021.with_hour(4).unwrap();
        for hour in [0, 1, 1, 2, 3, 4] {
            let expected = start.with_hour(hour).unwrap();
            let got = tz.clone().convert_utc(start);
            assert_eq!(expected.with_timezone(tz.clone()), got);

            start = start + 1.hours();
        }
    }

    #[test]
    fn america_santiago_regression() -> Result<(), ParseError> {
        let posix = PosixTimeZone::new("<-04>4<-03>,M9.1.6/24,M4.1.6/24")?;
        // DST doesn't end until 2040-04-08 00:00
        let dt = datetime!(2040-04-06 00:00);
        assert!(posix.is_dst(&dt.date(), &dt.time()));
        let end = datetime!(2040-04-08 00:00);
        assert!(!posix.is_dst(&end.date(), &end.time()));
        Ok(())
    }

    #[test]
    fn test_display_repr() -> Result<(), ParseError> {
        let posix = PosixTimeZone::new("GMT5")?;
        assert_eq!(posix.to_string(), "GMT5");
        let posix = PosixTimeZone::new("<-04>4<-03>,M9.1.6/24,M4.1.6/24")?;
        assert_eq!(posix.to_string(), "<-04>4<-03>,M9.1.6/24,M4.1.6/24");
        let posix = PosixTimeZone::from_str("EST+5EDT,M3.2.0/2,M11.1.0/2")?;
        assert_eq!(posix.to_string(), "EST5EDT,M3.2.0/2,M11.1.0/2");
        Ok(())
    }
}
