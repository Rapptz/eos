use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};

use eos::{
    gregorian::{date_to_epoch_days, days_in_month, is_leap_year, weekday_difference, weekday_from_days},
    time, utc_offset, Time, UtcOffset,
};

use crate::error::ParseError;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum DstTransitionRule {
    JulianDay(u16),
    Day(u16),
    Calendar { month: u8, n: u8, weekday: u8 },
}

impl DstTransitionRule {
    pub(crate) fn date_in(&self, year: i16) -> eos::Date {
        match self {
            Self::JulianDay(day) => {
                let d = if *day >= 59 && is_leap_year(year) {
                    day + 1
                } else {
                    *day
                };
                // day is already range checked as part of the contract
                eos::Date::from_ordinal(year, d).unwrap()
            }
            Self::Day(day) => {
                // day is already range checked as part of the contract
                eos::Date::from_ordinal(year, *day).unwrap()
            }
            Self::Calendar { month, n, weekday } => {
                let first_weekday = weekday_from_days(date_to_epoch_days(year, *month, 1));
                let days_in_month = days_in_month(year, *month);
                let mut day = weekday_difference(*weekday, first_weekday) + 1 + (n - 1) * 7;
                if day > days_in_month {
                    day -= 7;
                }
                eos::Date::__new_unchecked_from_macro(year, *month, day)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct DstTransitionInfo {
    abbr: String,
    offset: UtcOffset,
    start_rule: DstTransitionRule,
    start_time: Time,
    end_rule: DstTransitionRule,
    end_time: Time,
    base_offset: UtcOffset,
}

impl DstTransitionInfo {
    /// Returns true if DST is active
    pub(crate) fn is_active(&self, date: &eos::Date, time: &eos::Time) -> bool {
        let dt = (date, time);
        let start_date = self.start_rule.date_in(date.year());
        let end_date = self.end_rule.date_in(date.year());
        let start = (&start_date, &self.start_time);
        let end = (&end_date, &self.end_time);
        dt >= start && dt < end
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
/// # Ok::<_, eos_tz::ParserError>(())
/// ```
///
/// [Section 8.3]: https://pubs.opengroup.org/onlinepubs/9699919799/
/// [RFC8536]: https://datatracker.ietf.org/doc/html/rfc8536#section-3.3
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PosixTimeZone {
    std_abbr: String,
    std_offset: UtcOffset,
    dst: Option<DstTransitionInfo>,
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
                        Some(_) => parse_offset(&mut parser)?,
                        None => return Err(ParseError::InvalidPosixTz),
                    };
                    let (start_rule, start_time) = parse_dst_transition_rule(&mut parser)?;
                    if parser.next_if(|&x| x == ',').is_none() {
                        return Err(ParseError::InvalidPosixTz);
                    }
                    let (end_rule, end_time) = parse_dst_transition_rule(&mut parser)?;
                    let base_offset = offset.saturating_sub(std_offset);
                    Some(DstTransitionInfo {
                        abbr,
                        offset,
                        start_rule,
                        start_time,
                        end_rule,
                        end_time,
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

    /// Return the transitions in UTC for the given year, if any.
    pub(crate) fn transitions(&self, year: i16) -> Option<(eos::DateTime<eos::Utc>, eos::DateTime<eos::Utc>)> {
        match &self.dst {
            Some(dst) => {
                let start_date = dst.start_rule.date_in(year);
                let end_date = dst.end_rule.date_in(year);
                let mut dst_on = dst.start_time.at(start_date);
                let mut dst_off = dst.end_time.at(end_date);
                dst_on.shift(-self.std_offset);
                dst_off.shift(-dst.offset);
                Some((dst_on, dst_off))
            }
            None => None,
        }
    }

    pub(crate) fn shift_utc(&self, utc: &mut eos::DateTime<eos::Utc>) {
        // TODO: This does not handle imaginary or ambiguous times
        match self.dst.as_ref().zip(self.transitions(utc.year())) {
            None => {
                utc.shift(self.std_offset);
            }
            Some((dst, (dst_on, dst_off))) => {
                let is_dst = if dst_on < dst_off {
                    dst_on <= *utc && *utc < dst_off
                } else {
                    !(dst_off <= *utc && *utc < dst_on)
                };
                if is_dst {
                    utc.shift(dst.offset);
                } else {
                    utc.shift(self.std_offset);
                }
            }
        }
    }
}

impl eos::TimeZone for PosixTimeZone {
    fn name(&self, date: &eos::Date, time: &Time) -> Option<String> {
        match &self.dst {
            Some(dst) => {
                if dst.is_active(date, time) {
                    Some(dst.abbr.clone())
                } else {
                    Some(self.std_abbr.clone())
                }
            }
            None => Some(self.std_abbr.clone()),
        }
    }

    fn offset(&self, date: &eos::Date, time: &Time) -> UtcOffset {
        match &self.dst {
            Some(dst) => {
                if dst.is_active(date, time) {
                    dst.offset
                } else {
                    self.std_offset
                }
            }
            None => self.std_offset,
        }
    }

    fn at(self, mut utc: eos::DateTime<eos::Utc>) -> eos::DateTime<Self>
    where
        Self: Sized,
    {
        self.shift_utc(&mut utc);
        utc.with_timezone(self)
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
            Some((c, d)) => ((c as u8 - b'0') * 10 + (d as u8 - b'0')),
            None => return Err(ParseError::InvalidPosixTz),
        },
        None => 0u8,
    };

    let seconds = match parser.next_if(|x| *x == ':') {
        Some(_) => match parser.next().zip(parser.next()) {
            Some((c, d)) => ((c as u8 - b'0') * 10 + (d as u8 - b'0')),
            None => return Err(ParseError::InvalidPosixTz),
        },
        None => 0u8,
    };

    let mut seconds = hours as i32 * 3600 + minutes as i32 * 60 + seconds as i32;
    if negative {
        seconds = -seconds;
    }

    UtcOffset::from_seconds(seconds).map_err(|_| ParseError::InvalidOffset)
}

fn parse_time(parser: &mut Parser) -> ParseResult<Time> {
    let hour = match parser.next() {
        Some(c) if c.is_ascii_digit() => match parser.next_if(char::is_ascii_digit) {
            Some(d) => (c as u8 - b'0') * 10 + (d as u8 - b'0'),
            None => c as u8 - b'0',
        },
        _ => return Err(ParseError::InvalidPosixTz),
    };

    let minute = match parser.next_if(|x| *x == ':') {
        Some(_) => match parser.next().zip(parser.next()) {
            Some((c, d)) => ((c as u8 - b'0') * 10 + (d as u8 - b'0')),
            None => return Err(ParseError::InvalidPosixTz),
        },
        None => 0u8,
    };

    let second = match parser.next_if(|x| *x == ':') {
        Some(_) => match parser.next().zip(parser.next()) {
            Some((c, d)) => ((c as u8 - b'0') * 10 + (d as u8 - b'0')),
            None => return Err(ParseError::InvalidPosixTz),
        },
        None => 0u8,
    };

    Time::new(hour, minute, second).map_err(|_| ParseError::InvalidPosixTz)
}

fn parse_dst_transition_rule(parser: &mut Parser) -> ParseResult<(DstTransitionRule, Time)> {
    // date[/time]
    // date can be either (J\d{1,3}|\d{1,3}|M\d{1,2}.\d.\d)
    let date = match parser.next() {
        Some('J') => {
            let day = parse_three_digit_number(parser)?;
            if day < 1 || day > 365 {
                return Err(ParseError::InvalidPosixTz);
            }
            DstTransitionRule::JulianDay(day)
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

            DstTransitionRule::Calendar { month, n, weekday }
        }
        Some(c) if c.is_ascii_digit() => {
            let day = parse_three_digit_number(parser)?;
            if day > 365 {
                return Err(ParseError::InvalidPosixTz);
            }
            DstTransitionRule::Day(day)
        }
        _ => return Err(ParseError::InvalidPosixTz),
    };

    let time = match parser.next_if(|&x| x == '/') {
        Some(_) => parse_time(parser)?,
        None => time!(02:00:00),
    };
    Ok((date, time))
}

impl FromStr for PosixTimeZone {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
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
                dst.start_rule,
                DstTransitionRule::Calendar {
                    month: 3,
                    n: 2,
                    weekday: 0
                }
            );
            assert_eq!(dst.start_time, time!(02:00:00));
            assert_eq!(
                dst.end_rule,
                DstTransitionRule::Calendar {
                    month: 11,
                    n: 1,
                    weekday: 0
                }
            );
            assert_eq!(dst.end_time, time!(02:00:00));
        }
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
                dst.start_rule,
                DstTransitionRule::Calendar {
                    month: 10,
                    n: 1,
                    weekday: 0
                }
            );
            assert_eq!(dst.start_time, time!(02:00:00));
            assert_eq!(
                dst.end_rule,
                DstTransitionRule::Calendar {
                    month: 4,
                    n: 1,
                    weekday: 0
                }
            );
            assert_eq!(dst.end_time, time!(03:00:00));
        }
    }

    const DST_START_2021: DateTime = datetime!(2021-3-14 2:00 am);
    const DST_END_2021: DateTime = datetime!(2021-11-7 1:00 am);

    #[test]
    fn test_timezone_transition() -> Result<(), eos::Error> {
        // Modified from timezone_conversion.rs test case
        let tz = PosixTimeZone::from_str("EST+5EDT,M3.2.0/2,M11.1.0/2");
        assert!(tz.is_ok());
        let tz = tz.unwrap();

        let utc = Utc::now();
        // Unfortunately PosixTimeZone is *not* Copy which makes it awkward...
        let current = tz.clone().at(utc);
        assert_eq!(utc, current);

        /*
            UTC  4:00  5:00 6:00 7:00 8:00 9:00
            EDT  0:00  1:00 2:00 3:00 4:00 5:00
            EST 23:00  0:00 1:00 2:00 3:00 4:00
        */

        // start = UTC
        let mut start = DST_START_2021.with_hour(4)?;
        for hour in [23, 0, 1, 3, 4, 5] {
            let mut expected = start.with_hour(hour)?;
            if hour == 23 {
                expected = expected - 1.days();
            }
            let got = tz.clone().at(start);
            assert_eq!(expected.with_timezone(tz.clone()), got);

            start = start + 1.hours();
        }

        let mut start = DST_END_2021.with_hour(4)?;
        for hour in [0, 1, 1, 2, 3, 4] {
            let expected = start.with_hour(hour)?;
            let got = tz.clone().at(start);
            assert_eq!(expected.with_timezone(tz.clone()), got);

            start = start + 1.hours();
        }
        Ok(())
    }
}
