// These tests are adapted from Python's datetime library
// https://github.com/python/cpython/blob/3.10/Lib/test/datetimetester.py

use eos::{
    datetime, ext::IntervalLiteral, utc_offset, Date, DateTime, Interval, Time, TimeZone, Utc, UtcOffset, Weekday,
};

fn this_or_next_sunday(date: Date) -> Date {
    if date.weekday() == Weekday::Sunday {
        date
    } else {
        date.next_weekday(Weekday::Sunday)
    }
}

// DST in America starts on the second Sunday of March and ends on the first Sunday of November at 2am.
// Times have to be converted over to standard time, so 2 AM "summer time" is 1 AM "standard time".
// Yes, these times are technically in "UTC" despite representing local time.
const DST_START: DateTime = datetime!(2021-03-08 2:00 am);
const DST_END: DateTime = datetime!(2021-11-01 1:00 am);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct AmericanTimeZone {
    offset: UtcOffset,
    name: &'static str,
    dst_name: &'static str,
}

impl AmericanTimeZone {
    fn is_dst(&self, date: &Date, time: &Time) -> bool {
        let start = this_or_next_sunday(*DST_START.with_year(date.year()).date());
        assert_eq!(start.weekday(), Weekday::Sunday);
        assert_eq!(start.month(), 3);
        assert!(start.day() > 7);

        let end = this_or_next_sunday(*DST_END.with_year(date.year()).date());
        assert_eq!(end.weekday(), Weekday::Sunday);
        assert_eq!(end.month(), 11);
        assert!(end.day() <= 7);

        let start_dt = (&start, DST_START.time());
        let end_dt = (&end, DST_END.time());
        let dt = (date, time);

        dt >= start_dt && dt < end_dt
    }
}

impl TimeZone for AmericanTimeZone {
    fn name(&self, date: &Date, time: &Time) -> Option<&str> {
        if self.is_dst(date, time) {
            Some(self.name)
        } else {
            Some(self.dst_name)
        }
    }

    fn offset(&self, date: &Date, time: &Time) -> UtcOffset {
        if self.is_dst(date, time) {
            self.offset.saturating_add(utc_offset!(+01:00))
        } else {
            self.offset
        }
    }

    fn convert_utc(self, mut utc: DateTime<Utc>) -> DateTime<Self>
    where
        Self: Sized,
    {
        // This doesn't deal with imaginary or ambiguous times
        utc.shift(self.offset);
        if self.is_dst(utc.date(), utc.time()) {
            utc.shift(utc_offset!(+01:00));
        }
        utc.with_timezone(self)
    }
}

const EAST: AmericanTimeZone = AmericanTimeZone {
    offset: utc_offset!(-5:00),
    name: "EST",
    dst_name: "EDT",
};

const CENTRAL: AmericanTimeZone = AmericanTimeZone {
    offset: utc_offset!(-6:00),
    name: "CST",
    dst_name: "CDT",
};

const MOUNTAIN: AmericanTimeZone = AmericanTimeZone {
    offset: utc_offset!(-7:00),
    name: "MST",
    dst_name: "MDT",
};

const PACIFIC: AmericanTimeZone = AmericanTimeZone {
    offset: utc_offset!(-8:00),
    name: "PST",
    dst_name: "PDT",
};

const DT: DateTime = datetime!(2021-12-31 00:00);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct AlwaysEasternStandard;

impl TimeZone for AlwaysEasternStandard {
    fn offset(&self, _date: &Date, _time: &Time) -> UtcOffset {
        utc_offset!(-5:00)
    }

    fn convert_utc(self, mut utc: DateTime<Utc>) -> DateTime<Self>
    where
        Self: Sized,
    {
        utc.shift(utc_offset!(-5:00));
        utc.with_timezone(self)
    }
}

const DST_START_2021: DateTime = datetime!(2021-3-14 2:00 am);
const DST_END_2021: DateTime = datetime!(2021-11-7 1:00 am);

#[test]
fn test_from_utc() -> Result<(), eos::Error> {
    for tz in [&EAST, &CENTRAL, &MOUNTAIN, &PACIFIC] {
        let local = tz.convert_utc(DT);
        assert_eq!(local - DT.with_timezone(*tz), Interval::from(*local.offset()));
        assert_eq!(local, DT);
    }

    let utc_now = Utc::now();
    let east = EAST.convert_utc(utc_now);
    assert_eq!(utc_now, east);

    /*
        UTC  4:00  5:00 6:00 7:00 8:00 9:00
        EDT  0:00  1:00 2:00 3:00 4:00 5:00
        EST 23:00  0:00 1:00 2:00 3:00 4:00
    */

    // start = EST
    let mut start = DST_START_2021.with_hour(4)?;

    for hour in [23, 0, 1, 3, 4, 5] {
        let mut expected = start.with_hour(hour)?;
        if hour == 23 {
            expected = expected - 1.days();
        }
        let got = EAST.convert_utc(start);
        assert_eq!(expected.with_timezone(EAST), got);

        let got = AlwaysEasternStandard.convert_utc(start);
        let expected = (start + (-5).hours()).with_timezone(AlwaysEasternStandard);
        assert_eq!(expected, got);

        let got = start.in_timezone(AlwaysEasternStandard);
        assert_eq!(expected, got);

        start = start + 1.hours();
    }

    let mut start = DST_END_2021.with_hour(4)?;
    for hour in [0, 1, 1, 2, 3, 4] {
        let expected = start.with_hour(hour)?;
        let got = EAST.convert_utc(start);
        assert_eq!(expected.with_timezone(EAST), got);

        let got = AlwaysEasternStandard.convert_utc(start);
        let expected = (start + (-5).hours()).with_timezone(AlwaysEasternStandard);
        assert_eq!(expected, got);

        let got = start.in_timezone(AlwaysEasternStandard);
        assert_eq!(expected, got);

        start = start + 1.hours();
    }
    Ok(())
}
