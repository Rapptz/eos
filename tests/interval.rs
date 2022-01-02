use core::time::Duration;
use eos::{date, datetime, ext::IntervalLiteral, time, Interval};

#[test]
fn zero() {
    let zero = Interval::ZERO;
    assert_eq!(zero.years(), 0);
    assert_eq!(zero.months(), 0);
    assert_eq!(zero.weeks(), 0);
    assert_eq!(zero.days(), 0);
    assert_eq!(zero.hours(), 0);
    assert_eq!(zero.minutes(), 0);
    assert_eq!(zero.seconds(), 0);
    assert_eq!(zero.milliseconds(), 0);
    assert_eq!(zero.microseconds(), 0);
    assert_eq!(zero.nanoseconds(), 0);
}

#[test]
fn from_unit() {
    assert_eq!(Interval::from_years(2).years(), 2);
    assert_eq!(Interval::from_months(1234).months(), 1234);
    assert_eq!(Interval::from_weeks(1234).weeks(), 1234);
    assert_eq!(Interval::from_days(1234).days(), 1234);
    assert_eq!(Interval::from_hours(1234).hours(), 1234);
    assert_eq!(Interval::from_minutes(1234).minutes(), 1234);
    assert_eq!(Interval::from_seconds(1234).seconds(), 1234);
    assert_eq!(Interval::from_milliseconds(1234).milliseconds(), 1234);
    assert_eq!(Interval::from_microseconds(1234).microseconds(), 1234);
    assert_eq!(Interval::from_nanoseconds(1234).nanoseconds(), 1234);
}

#[test]
fn from_literal() {
    assert_eq!(2.years().years(), 2);
    assert_eq!(1234.months().months(), 1234);
    assert_eq!(1234.weeks().weeks(), 1234);
    assert_eq!(1234.days().days(), 1234);
    assert_eq!(1234.hours().hours(), 1234);
    assert_eq!(1234.minutes().minutes(), 1234);
    assert_eq!(1234.seconds().seconds(), 1234);
    assert_eq!(1234.milliseconds().milliseconds(), 1234);
    assert_eq!(1234.microseconds().microseconds(), 1234);
    assert_eq!(1234.nanoseconds().nanoseconds(), 1234);
}

#[test]
fn default() {
    assert_eq!(Interval::default(), Interval::ZERO);
}

#[test]
fn from_std_duration() {
    assert_eq!(Interval::from(Duration::from_micros(100)).microseconds(), 100);
    assert_eq!(Interval::from(Duration::from_secs(100)).seconds(), 100);
    assert_eq!(Interval::from(Duration::from_millis(100)).milliseconds(), 100);

    let paired = Interval::from(Duration::from_secs_f32(2.8));
    assert_eq!(paired.seconds(), 2);
    assert_eq!(paired.nanoseconds(), 800_000_000);
    assert_eq!(paired.microseconds(), 800_000);
    assert_eq!(paired.milliseconds(), 800);
}

#[test]
fn add_to_date() {
    assert_eq!(date!(2012 - 2 - 29) + 1.days(), date!(2012 - 3 - 1));
    assert_eq!(date!(2012 - 2 - 29) + 1.years(), date!(2013 - 2 - 28));
    assert_eq!(date!(2012 - 1 - 31) + 1.months(), date!(2012 - 2 - 29));
    assert_eq!(date!(2001 - 1 - 31) + 1.months(), date!(2001 - 2 - 28));
}

#[test]
fn noop_date_addition() {
    assert_eq!(date!(2001 - 1 - 31) + 10.minutes(), date!(2001 - 1 - 31));
    assert_eq!(date!(2001 - 1 - 31) + 10.microseconds(), date!(2001 - 1 - 31));
    assert_eq!(date!(2001 - 1 - 31) + 10.hours(), date!(2001 - 1 - 31));
    assert_eq!(date!(2001 - 1 - 31) + 10.milliseconds(), date!(2001 - 1 - 31));
    assert_eq!(date!(2001 - 1 - 31) + 10.nanoseconds(), date!(2001 - 1 - 31));
}

#[test]
fn add_to_time() {
    assert_eq!(time!(00:00:00) + 92.minutes(), time!(1:32:0));
    assert_eq!(time!(00:00:00) - 2.minutes(), time!(23:58:0));
}

#[test]
fn out_of_bounds_wrapping_time() {
    assert_eq!(time!(23:59:59) + Interval::from_seconds(i64::MAX), time!(23:59:59));
    assert_eq!(time!(23:59:59) - Interval::from_seconds(i64::MAX), time!(23:59:59));
}

#[test]
fn random_single_units_to_dates() {
    assert_eq!(date!(268 - 7 - 31) + 10.years(), date!(278 - 7 - 31));
    assert_eq!(date!(221 - 1 - 31) + (-82).years(), date!(139 - 1 - 31));
    assert_eq!(date!(1723 - 3 - 31) - (-35).years(), date!(1758 - 3 - 31));
    assert_eq!(date!(651 - 11 - 30) - 99.days(), date!(651 - 8 - 23));
    assert_eq!(date!(1476 - 10 - 31) + 53.years(), date!(1529 - 10 - 31));
    assert_eq!(date!(927 - 1 - 31) + 3.months(), date!(927 - 4 - 30));
    assert_eq!(date!(348 - 2 - 29) - 38.months(), date!(344 - 12 - 29));
    assert_eq!(date!(1707 - 4 - 30) - (-77).months(), date!(1713 - 9 - 30));
    assert_eq!(date!(1444 - 10 - 31) - 27.months(), date!(1442 - 7 - 31));
    assert_eq!(date!(100 - 1 - 31) - (-68).months(), date!(105 - 9 - 30));
    assert_eq!(date!(1371 - 7 - 31) + (-64).months(), date!(1366 - 3 - 31));
    assert_eq!(date!(1599 - 8 - 31) + 62.months(), date!(1604 - 10 - 31));
    assert_eq!(date!(1031 - 1 - 31) + (-54).days(), date!(1030 - 12 - 8));
    assert_eq!(date!(855 - 10 - 31) + (-57).months(), date!(851 - 1 - 31));
    assert_eq!(date!(1691 - 3 - 31) - (-55).days(), date!(1691 - 5 - 25));
    assert_eq!(date!(927 - 5 - 31) + 89.days(), date!(927 - 8 - 28));
    assert_eq!(date!(904 - 1 - 31) - 20.weeks(), date!(903 - 9 - 13));
    assert_eq!(date!(779 - 12 - 31) + (-72).weeks(), date!(778 - 8 - 14));
    assert_eq!(date!(1689 - 6 - 30) - (-27).weeks(), date!(1690 - 1 - 5));
    assert_eq!(date!(806 - 12 - 31) - (-59).months(), date!(811 - 11 - 30));
}

#[test]
fn random_double_units_to_dates() {
    assert_eq!(
        date!(806 - 2 - 28) - ((-7).weeks() + (-90).months()),
        date!(813 - 10 - 16)
    );
    assert_eq!(date!(391 - 11 - 30) + ((-7).weeks() + 22.years()), date!(413 - 10 - 12));
    assert_eq!(date!(1925 - 7 - 31) + (56.weeks() + 51.months()), date!(1930 - 11 - 27));
    assert_eq!(
        date!(1423 - 2 - 28) - ((-37).months() + (-51).years()),
        date!(1477 - 3 - 28)
    );
    assert_eq!(date!(66 - 11 - 30) + (-50).weeks(), date!(65 - 12 - 15));
    assert_eq!(date!(2020 - 3 - 31) - 12.months(), date!(2019 - 3 - 31));
    assert_eq!(date!(46 - 7 - 31) + ((-47).days() + (-41).months()), date!(43 - 1 - 12));
    assert_eq!(date!(987 - 7 - 31) - (67.months() + (-26).days()), date!(982 - 1 - 26));
    assert_eq!(date!(1831 - 4 - 30) - (-11).months(), date!(1832 - 3 - 30));
    assert_eq!(date!(812 - 1 - 31) + ((-34).months() + 11.years()), date!(820 - 3 - 31));
    assert_eq!(
        date!(1194 - 5 - 31) - ((-42).years() + (-79).days()),
        date!(1236 - 8 - 18)
    );
    assert_eq!(date!(1943 - 8 - 31) + (-80).days(), date!(1943 - 6 - 12));
    assert_eq!(date!(1609 - 5 - 31) + 13.years(), date!(1622 - 5 - 31));
    assert_eq!(
        date!(614 - 7 - 31) - ((-52).days() + (-66).years()),
        date!(680 - 9 - 21)
    );
    assert_eq!(date!(170 - 4 - 30) + 24.days(), date!(170 - 5 - 24));
    assert_eq!(date!(488 - 2 - 29) + (12.weeks() + 96.months()), date!(496 - 5 - 23));
    assert_eq!(
        date!(1349 - 10 - 31) + ((-90).days() + (-32).weeks()),
        date!(1348 - 12 - 21)
    );
    assert_eq!(date!(1942 - 7 - 31) + 53.weeks(), date!(1943 - 8 - 6));
    assert_eq!(date!(1095 - 7 - 31) - (22.weeks() + 53.days()), date!(1095 - 1 - 5));
    assert_eq!(date!(539 - 4 - 30) + (3.months() + 48.weeks()), date!(540 - 6 - 30));
}
#[test]
fn random_single_units_to_times() {
    assert_eq!(time!(16:51:49) - (-13).minutes(), time!(17:4:49));
    assert_eq!(time!(13:20:48) - (-4).minutes(), time!(13:24:48));
    assert_eq!(time!(6:51:52) - 44.minutes(), time!(6:7:52));
    assert_eq!(time!(20:19:13) - 41.minutes(), time!(19:38:13));
    assert_eq!(time!(1:4:31) - 0.hours(), time!(1:4:31));
    assert_eq!(time!(19:11:31) - 32.seconds(), time!(19:10:59));
    assert_eq!(time!(7:31:3) + 2.hours(), time!(9:31:3));
    assert_eq!(time!(7:5:30) - (-20).hours(), time!(3:5:30));
    assert_eq!(time!(11:56:1) - (-18).minutes(), time!(12:14:1));
    assert_eq!(time!(18:51:17) - (-51).seconds(), time!(18:52:8));
    assert_eq!(time!(21:13:48) - 36.seconds(), time!(21:13:12));
    assert_eq!(time!(19:42:49) - 99.minutes(), time!(18:3:49));
    assert_eq!(time!(0:54:29) + (-70).seconds(), time!(0:53:19));
    assert_eq!(time!(14:20:2) - (-49).minutes(), time!(15:9:2));
    assert_eq!(time!(8:51:55) - 58.seconds(), time!(8:50:57));
    assert_eq!(time!(9:21:3) + 87.seconds(), time!(9:22:30));
    assert_eq!(time!(2:40:56) + (-2).seconds(), time!(2:40:54));
    assert_eq!(time!(15:24:1) + 47.hours(), time!(14:24:1));
    assert_eq!(time!(7:38:1) + (-21).hours(), time!(10:38:1));
    assert_eq!(time!(11:26:23) - 65.minutes(), time!(10:21:23));
}

#[test]
fn random_double_units_to_times() {
    assert_eq!(time!(9:16:42) + (86.hours() + 17.minutes()), time!(23:33:42));
    assert_eq!(time!(15:13:14) + (-47).hours(), time!(16:13:14));
    assert_eq!(time!(17:39:18) + 96.hours(), time!(17:39:18));
    assert_eq!(time!(0:40:43) + (0.seconds() + (-46).hours()), time!(2:40:43));
    assert_eq!(time!(12:48:19) + (-27).seconds(), time!(12:47:52));
    assert_eq!(time!(11:11:30) + ((-33).hours() + 2.seconds()), time!(2:11:32));
    assert_eq!(time!(8:52:53) - (-98).seconds(), time!(8:54:31));
    assert_eq!(time!(18:39:30) - (-56).hours(), time!(2:39:30));
    assert_eq!(time!(13:52:13) + 91.minutes(), time!(15:23:13));
    assert_eq!(time!(18:3:35) - ((-85).minutes() + (-90).hours()), time!(13:28:35));
    assert_eq!(time!(2:17:44) + 93.hours(), time!(23:17:44));
    assert_eq!(time!(8:23:42) + ((-2).hours() + (-42).seconds()), time!(6:23:0));
    assert_eq!(time!(15:44:3) - ((-7).minutes() + 62.hours()), time!(1:51:3));
    assert_eq!(time!(8:48:28) - (51.minutes() + (-77).seconds()), time!(7:58:45));
    assert_eq!(time!(6:7:49) - (50.minutes() + (-12).hours()), time!(17:17:49));
    assert_eq!(time!(3:24:42) + (80.hours() + (-4).minutes()), time!(11:20:42));
    assert_eq!(time!(13:21:23) + (-33).minutes(), time!(12:48:23));
    assert_eq!(time!(8:23:54) + ((-63).seconds() + 56.hours()), time!(16:22:51));
    assert_eq!(time!(20:26:47) + ((-50).seconds() + 53.hours()), time!(1:25:57));
    assert_eq!(time!(11:50:19) - (36.hours() + (-30).minutes()), time!(0:20:19));
}

#[test]
fn add_to_datetime() {
    assert_eq!(datetime!(2012-02-29 00:00) + 1.days(), datetime!(2012-03-1 00:00));
    assert_eq!(datetime!(2012-02-29 00:00) + 1.years(), datetime!(2013-02-28 00:00));
    assert_eq!(datetime!(2012-01-31 00:00) + 1.months(), datetime!(2012-02-29 00:00));
    assert_eq!(datetime!(2001-01-31 00:00) + 1.months(), datetime!(2001-02-28 00:00));
    assert_eq!(datetime!(2001-01-31 00:00) + 92.minutes(), datetime!(2001-01-31 1:32));
    assert_eq!(datetime!(2001-01-31 00:00) - 2.minutes(), datetime!(2001-01-30 23:58));
}

#[test]
fn random_single_units_to_datetimes() {
    assert_eq!(datetime!(1572-2-29 2:28:40) - 67.days(), datetime!(1571-12-24 2:28:40));
    assert_eq!(
        datetime!(1288-4-30 3:24:10) + (-71).days(),
        datetime!(1288-2-19 3:24:10)
    );
    assert_eq!(datetime!(1391-7-31 1:1:41) + (-58).weeks(), datetime!(1390-6-20 1:1:41));
    assert_eq!(datetime!(1950-8-31 6:46:36) - 53.years(), datetime!(1897-8-31 6:46:36));
    assert_eq!(datetime!(779-7-31 7:6:16) + (-58).days(), datetime!(779-6-3 7:6:16));
    assert_eq!(
        datetime!(1148-11-30 18:52:18) - 2.days(),
        datetime!(1148-11-28 18:52:18)
    );
    assert_eq!(
        datetime!(317-12-31 20:12:28) + 28.months(),
        datetime!(320-4-30 20:12:28)
    );
    assert_eq!(datetime!(639-11-30 3:2:35) - (-82).days(), datetime!(640-2-20 3:2:35));
    assert_eq!(
        datetime!(1561-4-30 0:47:47) - (-77).years(),
        datetime!(1638-4-30 0:47:47)
    );
    assert_eq!(datetime!(98-9-30 5:6:37) + 4.weeks(), datetime!(98-10-28 5:6:37));
    assert_eq!(datetime!(1453-9-30 6:12:17) - 29.months(), datetime!(1451-4-30 6:12:17));
    assert_eq!(datetime!(402-4-30 15:55:5) - 98.weeks(), datetime!(400-6-13 15:55:5));
    assert_eq!(datetime!(652-4-30 0:31:20) - 64.days(), datetime!(652-2-26 0:31:20));
    assert_eq!(datetime!(1626-1-31 3:19:54) - 61.days(), datetime!(1625-12-1 3:19:54));
    assert_eq!(datetime!(1991-8-31 4:52:25) - 58.days(), datetime!(1991-7-4 4:52:25));
    assert_eq!(datetime!(416-4-30 0:15:7) - (-97).months(), datetime!(424-5-30 0:15:7));
    assert_eq!(datetime!(1379-3-31 21:24:18) - 34.weeks(), datetime!(1378-8-5 21:24:18));
    assert_eq!(
        datetime!(900-9-30 11:42:14) - (-88).years(),
        datetime!(988-9-30 11:42:14)
    );
    assert_eq!(datetime!(1835-2-28 4:42:11) + 75.months(), datetime!(1841-5-28 4:42:11));
    assert_eq!(datetime!(921-8-31 23:9:17) - 4.days(), datetime!(921-8-27 23:9:17));
}

#[test]
fn random_double_units_to_datetimes() {
    assert_eq!(
        datetime!(832-4-30 12:41:58) - ((-26).months() + (-88).weeks()),
        datetime!(836-3-7 12:41:58)
    );
    assert_eq!(
        datetime!(1558-2-28 22:45:39) - ((-94).weeks() + 71.months()),
        datetime!(1554-1-15 22:45:39)
    );
    assert_eq!(
        datetime!(70-8-31 22:37:9) + ((-80).weeks() + 90.days()),
        datetime!(69-5-18 22:37:9)
    );
    assert_eq!(
        datetime!(1279-5-31 14:52:35) - 50.weeks(),
        datetime!(1278-6-15 14:52:35)
    );
    assert_eq!(
        datetime!(1829-3-31 11:27:57) - (62.weeks() + (-80).days()),
        datetime!(1828-4-11 11:27:57)
    );
    assert_eq!(
        datetime!(1682-12-31 13:31:15) - (-39).days(),
        datetime!(1683-2-8 13:31:15)
    );
    assert_eq!(
        datetime!(405-1-31 4:33:44) - (22.months() + 1.years()),
        datetime!(402-3-31 4:33:44)
    );
    assert_eq!(
        datetime!(1436-2-29 22:59:42) - (77.years() + (-97).months()),
        datetime!(1367-3-29 22:59:42)
    );
    assert_eq!(datetime!(1014-7-31 6:30:12) - 58.weeks(), datetime!(1013-6-20 6:30:12));
    assert_eq!(
        datetime!(803-5-31 8:54:49) - ((-85).years() + (-92).weeks()),
        datetime!(890-3-6 8:54:49)
    );
    assert_eq!(datetime!(1147-12-31 2:0:13) - 40.years(), datetime!(1107-12-31 2:0:13));
    assert_eq!(
        datetime!(633-12-31 2:19:19) + (59.years() + (-26).weeks()),
        datetime!(692-7-2 2:19:19)
    );
    assert_eq!(
        datetime!(1841-1-31 0:53:36) + ((-23).days() + 14.months()),
        datetime!(1842-3-8 0:53:36)
    );
    assert_eq!(
        datetime!(1834-11-30 4:45:6) - (30.weeks() + (-73).years()),
        datetime!(1907-5-4 4:45:6)
    );
    assert_eq!(
        datetime!(1495-6-30 8:8:35) + ((-59).weeks() + 20.days()),
        datetime!(1494-6-2 8:8:35)
    );
    assert_eq!(
        datetime!(313-9-30 20:54:41) + (74.weeks() + 30.months()),
        datetime!(317-8-30 20:54:41)
    );
    assert_eq!(
        datetime!(1588-3-31 20:44:4) - ((-99).weeks() + 3.months()),
        datetime!(1589-11-23 20:44:4)
    );
    assert_eq!(
        datetime!(1642-3-31 5:34:21) + ((-26).days() + (-7).years()),
        datetime!(1635-3-5 5:34:21)
    );
    assert_eq!(datetime!(588-7-31 14:5:45) - (-53).years(), datetime!(641-7-31 14:5:45));
    assert_eq!(
        datetime!(1154-4-30 20:13:54) + 11.weeks(),
        datetime!(1154-7-16 20:13:54)
    );
}

#[test]
fn diff_between_dates() {
    assert_eq!(
        date!(1064 - 7 - 31) - date!(325 - 2 - 28),
        Interval::default().with_years(739).with_months(5).with_days(3)
    );
    assert_eq!(
        date!(325 - 2 - 28) - date!(1064 - 7 - 31),
        Interval::default().with_years(-739).with_months(-5)
    );

    assert_eq!(
        date!(113 - 6 - 30) - date!(550 - 2 - 28),
        Interval::default().with_years(-436).with_months(-7).with_days(-28)
    );
    assert_eq!(
        date!(1948 - 9 - 30) - date!(309 - 9 - 30),
        Interval::default().with_years(1639)
    );
    assert_eq!(
        date!(1292 - 9 - 30) - date!(199 - 4 - 30),
        Interval::default().with_years(1093).with_months(5)
    );
    assert_eq!(
        date!(1915 - 3 - 31) - date!(1765 - 1 - 31),
        Interval::default().with_years(150).with_months(2)
    );
    assert_eq!(
        date!(476 - 10 - 31) - date!(731 - 5 - 31),
        Interval::default().with_years(-254).with_months(-7)
    );
    assert_eq!(
        date!(41 - 4 - 30) - date!(1235 - 8 - 31),
        Interval::default().with_years(-1194).with_months(-4)
    );
    assert_eq!(
        date!(9 - 7 - 31) - date!(1390 - 4 - 30),
        Interval::default().with_years(-1380).with_months(-8).with_days(-30)
    );
    assert_eq!(
        date!(1479 - 8 - 31) - date!(2048 - 5 - 31),
        Interval::default().with_years(-568).with_months(-9)
    );
    assert_eq!(
        date!(1965 - 1 - 31) - date!(1515 - 7 - 31),
        Interval::default().with_years(449).with_months(6)
    );
    assert_eq!(
        date!(538 - 7 - 31) - date!(1760 - 1 - 31),
        Interval::default().with_years(-1221).with_months(-6)
    );
    assert_eq!(
        date!(1904 - 7 - 31) - date!(1127 - 2 - 28),
        Interval::default().with_years(777).with_months(5).with_days(3)
    );
    assert_eq!(
        date!(631 - 6 - 30) - date!(529 - 9 - 30),
        Interval::default().with_years(101).with_months(9)
    );
    assert_eq!(
        date!(2021 - 11 - 30) - date!(1351 - 10 - 31),
        Interval::default().with_years(670).with_months(1)
    );
    assert_eq!(
        date!(1600 - 8 - 31) - date!(1345 - 4 - 30),
        Interval::default().with_years(255).with_months(4).with_days(1)
    );
    assert_eq!(
        date!(87 - 8 - 31) - date!(205 - 9 - 30),
        Interval::default().with_years(-118).with_days(-30)
    );
    assert_eq!(
        date!(816 - 10 - 31) - date!(1496 - 10 - 31),
        Interval::default().with_years(-680)
    );
    assert_eq!(
        date!(260 - 5 - 31) - date!(1733 - 2 - 28),
        Interval::default().with_years(-1472).with_months(-8).with_days(-28)
    );
    assert_eq!(
        date!(100 - 3 - 31) - date!(1213 - 5 - 31),
        Interval::default().with_years(-1113).with_months(-2)
    );
    assert_eq!(
        date!(1170 - 6 - 30) - date!(1980 - 7 - 31),
        Interval::default().with_years(-810).with_months(-1)
    );
    assert_eq!(
        date!(337 - 10 - 31) - date!(40 - 11 - 30),
        Interval::default().with_years(296).with_months(11).with_days(1)
    );
    assert_eq!(
        date!(1102 - 12 - 31) - date!(687 - 11 - 30),
        Interval::default().with_years(415).with_months(1).with_days(1)
    );
    assert_eq!(
        date!(1820 - 5 - 31) - date!(1737 - 3 - 31),
        Interval::default().with_years(83).with_months(2)
    );
    assert_eq!(
        date!(1046 - 11 - 30) - date!(1382 - 5 - 31),
        Interval::default().with_years(-335).with_months(-6)
    );
    assert_eq!(
        date!(1138 - 2 - 28) - date!(408 - 3 - 31),
        Interval::default().with_years(729).with_months(11)
    );
    assert_eq!(
        date!(1187 - 1 - 31) - date!(430 - 7 - 31),
        Interval::default().with_years(756).with_months(6)
    );
    assert_eq!(
        date!(1422 - 11 - 30) - date!(1357 - 3 - 31),
        Interval::default().with_years(65).with_months(8)
    );
    assert_eq!(
        date!(1259 - 2 - 28) - date!(430 - 3 - 31),
        Interval::default().with_years(828).with_months(11)
    );
    assert_eq!(
        date!(880 - 12 - 31) - date!(982 - 8 - 31),
        Interval::default().with_years(-101).with_months(-8)
    );
    assert_eq!(
        date!(400 - 6 - 30) - date!(750 - 9 - 30),
        Interval::default().with_years(-350).with_months(-3)
    );
    assert_eq!(
        date!(1930 - 12 - 31) - date!(1506 - 3 - 31),
        Interval::default().with_years(424).with_months(9)
    );
    assert_eq!(
        date!(4 - 1 - 31) - date!(1408 - 7 - 31),
        Interval::default().with_years(-1404).with_months(-6)
    );
    assert_eq!(
        date!(1555 - 4 - 30) - date!(2027 - 3 - 31),
        Interval::default().with_years(-471).with_months(-11)
    );
    assert_eq!(
        date!(252 - 9 - 30) - date!(1680 - 1 - 31),
        Interval::default().with_years(-1427).with_months(-4)
    );
    assert_eq!(
        date!(419 - 8 - 31) - date!(516 - 6 - 30),
        Interval::default().with_years(-96).with_months(-9).with_days(-30)
    );
    assert_eq!(
        date!(1311 - 8 - 31) - date!(1013 - 11 - 30),
        Interval::default().with_years(297).with_months(9).with_days(1)
    );
    assert_eq!(
        date!(1395 - 6 - 30) - date!(76 - 8 - 31),
        Interval::default().with_years(1318).with_months(10)
    );
    assert_eq!(
        date!(930 - 3 - 31) - date!(1829 - 3 - 31),
        Interval::default().with_years(-899)
    );
    assert_eq!(
        date!(1560 - 3 - 31) - date!(1683 - 6 - 30),
        Interval::default().with_years(-123).with_months(-2).with_days(-30)
    );
    assert_eq!(
        date!(31 - 3 - 31) - date!(1984 - 12 - 31),
        Interval::default().with_years(-1953).with_months(-9)
    );
    assert_eq!(
        date!(685 - 9 - 30) - date!(1706 - 11 - 30),
        Interval::default().with_years(-1021).with_months(-2)
    );
    assert_eq!(
        date!(877 - 1 - 31) - date!(43 - 9 - 30),
        Interval::default().with_years(833).with_months(4).with_days(1)
    );
}

#[test]
fn diff_between_times() {
    assert_eq!(
        time!(18:56:51) - time!(15:16:55),
        Interval::default().with_hours(3).with_minutes(39).with_seconds(56)
    );
    assert_eq!(
        time!(10:20:14) - time!(22:35:34),
        Interval::default().with_hours(-12).with_minutes(-15).with_seconds(-20)
    );
    assert_eq!(
        time!(18:21:9) - time!(9:22:45),
        Interval::default().with_hours(8).with_minutes(58).with_seconds(24)
    );
    assert_eq!(
        time!(8:1:56) - time!(8:56:45),
        Interval::default().with_minutes(-54).with_seconds(-49)
    );
    assert_eq!(
        time!(3:25:57) - time!(14:29:4),
        Interval::default().with_hours(-11).with_minutes(-3).with_seconds(-7)
    );
    assert_eq!(time!(12:15:18) - time!(12:56:18), Interval::default().with_minutes(-41));
    assert_eq!(
        time!(20:46:27) - time!(7:33:18),
        Interval::default().with_hours(13).with_minutes(13).with_seconds(9)
    );
    assert_eq!(
        time!(0:40:9) - time!(17:23:27),
        Interval::default().with_hours(-16).with_minutes(-43).with_seconds(-18)
    );
    assert_eq!(
        time!(10:12:13) - time!(17:28:11),
        Interval::default().with_hours(-7).with_minutes(-15).with_seconds(-58)
    );
    assert_eq!(
        time!(3:54:15) - time!(13:32:5),
        Interval::default().with_hours(-9).with_minutes(-37).with_seconds(-50)
    );
    assert_eq!(
        time!(7:18:32) - time!(1:28:51),
        Interval::default().with_hours(5).with_minutes(49).with_seconds(41)
    );
    assert_eq!(
        time!(5:42:46) - time!(23:25:18),
        Interval::default().with_hours(-17).with_minutes(-42).with_seconds(-32)
    );
    assert_eq!(
        time!(20:12:46) - time!(10:12:45),
        Interval::default().with_hours(10).with_seconds(1)
    );
    assert_eq!(
        time!(2:48:24) - time!(22:47:37),
        Interval::default().with_hours(-19).with_minutes(-59).with_seconds(-13)
    );
    assert_eq!(
        time!(9:50:11) - time!(13:40:3),
        Interval::default().with_hours(-3).with_minutes(-49).with_seconds(-52)
    );
    assert_eq!(
        time!(22:14:21) - time!(1:17:59),
        Interval::default().with_hours(20).with_minutes(56).with_seconds(22)
    );
    assert_eq!(
        time!(23:30:37) - time!(21:50:13),
        Interval::default().with_hours(1).with_minutes(40).with_seconds(24)
    );
    assert_eq!(
        time!(5:6:54) - time!(16:6:27),
        Interval::default().with_hours(-10).with_minutes(-59).with_seconds(-33)
    );
    assert_eq!(
        time!(16:15:53) - time!(19:45:50),
        Interval::default().with_hours(-3).with_minutes(-29).with_seconds(-57)
    );
    assert_eq!(
        time!(14:6:15) - time!(4:59:21),
        Interval::default().with_hours(9).with_minutes(6).with_seconds(54)
    );
    assert_eq!(
        time!(6:54:44) - time!(12:36:43),
        Interval::default().with_hours(-5).with_minutes(-41).with_seconds(-59)
    );
    assert_eq!(
        time!(3:13:54) - time!(23:6:2),
        Interval::default().with_hours(-19).with_minutes(-52).with_seconds(-8)
    );
    assert_eq!(
        time!(0:52:35) - time!(6:52:41),
        Interval::default().with_hours(-6).with_seconds(-6)
    );
    assert_eq!(
        time!(4:36:7) - time!(7:17:17),
        Interval::default().with_hours(-2).with_minutes(-41).with_seconds(-10)
    );
    assert_eq!(
        time!(16:49:17) - time!(8:45:25),
        Interval::default().with_hours(8).with_minutes(3).with_seconds(52)
    );
    assert_eq!(
        time!(18:33:56) - time!(12:41:43),
        Interval::default().with_hours(5).with_minutes(52).with_seconds(13)
    );
    assert_eq!(
        time!(15:55:59) - time!(20:38:14),
        Interval::default().with_hours(-4).with_minutes(-42).with_seconds(-15)
    );
    assert_eq!(
        time!(4:35:9) - time!(1:11:15),
        Interval::default().with_hours(3).with_minutes(23).with_seconds(54)
    );
    assert_eq!(
        time!(11:42:55) - time!(9:2:58),
        Interval::default().with_hours(2).with_minutes(39).with_seconds(57)
    );
    assert_eq!(
        time!(1:27:3) - time!(14:34:17),
        Interval::default().with_hours(-13).with_minutes(-7).with_seconds(-14)
    );
    assert_eq!(
        time!(16:8:54) - time!(6:30:51),
        Interval::default().with_hours(9).with_minutes(38).with_seconds(3)
    );
    assert_eq!(
        time!(20:58:7) - time!(3:6:19),
        Interval::default().with_hours(17).with_minutes(51).with_seconds(48)
    );
    assert_eq!(
        time!(3:10:20) - time!(22:12:17),
        Interval::default().with_hours(-19).with_minutes(-1).with_seconds(-57)
    );
    assert_eq!(
        time!(10:29:21) - time!(0:45:26),
        Interval::default().with_hours(9).with_minutes(43).with_seconds(55)
    );
    assert_eq!(
        time!(18:53:42) - time!(21:32:33),
        Interval::default().with_hours(-2).with_minutes(-38).with_seconds(-51)
    );
    assert_eq!(
        time!(17:22:25) - time!(3:36:17),
        Interval::default().with_hours(13).with_minutes(46).with_seconds(8)
    );
    assert_eq!(
        time!(18:17:37) - time!(20:41:24),
        Interval::default().with_hours(-2).with_minutes(-23).with_seconds(-47)
    );
    assert_eq!(
        time!(22:14:39) - time!(19:53:36),
        Interval::default().with_hours(2).with_minutes(21).with_seconds(3)
    );
    assert_eq!(
        time!(22:21:31) - time!(11:30:57),
        Interval::default().with_hours(10).with_minutes(50).with_seconds(34)
    );
    assert_eq!(
        time!(1:30:29) - time!(23:27:35),
        Interval::default().with_hours(-21).with_minutes(-57).with_seconds(-6)
    );
}

#[test]
fn diff_between_datetimes() {
    assert_eq!(
        datetime!(2049-11-30 19:51:39) - datetime!(1247-6-30 18:31:7),
        Interval::default()
            .with_years(802)
            .with_months(5)
            .with_hours(1)
            .with_minutes(20)
            .with_seconds(32)
    );
    assert_eq!(
        datetime!(1826-6-30 11:14:55) - datetime!(1521-11-30 14:58:52),
        Interval::default()
            .with_years(304)
            .with_months(6)
            .with_days(30)
            .with_hours(20)
            .with_minutes(16)
            .with_seconds(3)
    );
    assert_eq!(
        datetime!(1191-7-31 17:57:28) - datetime!(530-12-31 23:24:27),
        Interval::default()
            .with_years(660)
            .with_months(6)
            .with_days(30)
            .with_hours(18)
            .with_minutes(33)
            .with_seconds(1)
    );
    assert_eq!(
        datetime!(1768-2-29 13:20:33) - datetime!(612-12-31 16:15:22),
        Interval::default()
            .with_years(1155)
            .with_months(1)
            .with_days(28)
            .with_hours(21)
            .with_minutes(5)
            .with_seconds(11)
    );
    assert_eq!(
        datetime!(1378-8-31 9:29:4) - datetime!(1601-5-31 0:12:43),
        Interval::default()
            .with_years(-222)
            .with_months(-8)
            .with_days(-29)
            .with_hours(-14)
            .with_minutes(-43)
            .with_seconds(-39)
    );
    assert_eq!(
        datetime!(1086-7-31 1:31:2) - datetime!(108-12-31 20:46:43),
        Interval::default()
            .with_years(977)
            .with_months(6)
            .with_days(30)
            .with_hours(4)
            .with_minutes(44)
            .with_seconds(19)
    );
    assert_eq!(
        datetime!(166-12-31 15:2:32) - datetime!(213-2-28 20:57:16),
        Interval::default()
            .with_years(-46)
            .with_months(-1)
            .with_days(-28)
            .with_hours(-5)
            .with_minutes(-54)
            .with_seconds(-44)
    );
    assert_eq!(
        datetime!(1191-9-30 11:35:53) - datetime!(321-6-30 0:4:34),
        Interval::default()
            .with_years(870)
            .with_months(3)
            .with_hours(11)
            .with_minutes(31)
            .with_seconds(19)
    );
    assert_eq!(
        datetime!(1384-8-31 23:24:41) - datetime!(307-3-31 21:40:56),
        Interval::default()
            .with_years(1077)
            .with_months(5)
            .with_hours(1)
            .with_minutes(43)
            .with_seconds(45)
    );
    assert_eq!(
        datetime!(1673-12-31 23:24:50) - datetime!(1422-9-30 5:53:12),
        Interval::default()
            .with_years(251)
            .with_months(3)
            .with_days(1)
            .with_hours(17)
            .with_minutes(31)
            .with_seconds(38)
    );
    assert_eq!(
        datetime!(1145-12-31 7:24:7) - datetime!(788-10-31 5:37:18),
        Interval::default()
            .with_years(357)
            .with_months(2)
            .with_hours(1)
            .with_minutes(46)
            .with_seconds(49)
    );
    assert_eq!(
        datetime!(639-11-30 15:3:39) - datetime!(1103-12-31 9:23:21),
        Interval::default()
            .with_years(-464)
            .with_days(-30)
            .with_hours(-18)
            .with_minutes(-19)
            .with_seconds(-42)
    );
    assert_eq!(
        datetime!(174-2-28 12:23:17) - datetime!(1140-4-30 5:42:59),
        Interval::default()
            .with_years(-966)
            .with_months(-1)
            .with_days(-29)
            .with_hours(-17)
            .with_minutes(-19)
            .with_seconds(-42)
    );
    assert_eq!(
        datetime!(1089-7-31 5:24:26) - datetime!(1058-3-31 11:14:1),
        Interval::default()
            .with_years(31)
            .with_months(3)
            .with_days(30)
            .with_hours(18)
            .with_minutes(10)
            .with_seconds(25)
    );
    assert_eq!(
        datetime!(386-7-31 19:5:15) - datetime!(1652-5-31 14:15:16),
        Interval::default()
            .with_years(-1265)
            .with_months(-9)
            .with_days(-30)
            .with_hours(-19)
            .with_minutes(-10)
            .with_seconds(-1)
    );
    assert_eq!(
        datetime!(39-7-31 0:27:35) - datetime!(1705-8-31 13:17:34),
        Interval::default()
            .with_years(-1666)
            .with_months(-1)
            .with_hours(-12)
            .with_minutes(-49)
            .with_seconds(-59)
    );
    assert_eq!(
        datetime!(199-11-30 14:46:55) - datetime!(1250-3-31 11:8:7),
        Interval::default()
            .with_years(-1050)
            .with_months(-3)
            .with_days(-30)
            .with_hours(-20)
            .with_minutes(-21)
            .with_seconds(-12)
    );
    assert_eq!(
        datetime!(210-5-31 5:35:51) - datetime!(1139-8-31 1:23:44),
        Interval::default()
            .with_years(-929)
            .with_months(-2)
            .with_days(-29)
            .with_hours(-19)
            .with_minutes(-47)
            .with_seconds(-53)
    );
    assert_eq!(
        datetime!(1427-8-31 4:35:58) - datetime!(989-3-31 18:14:33),
        Interval::default()
            .with_years(438)
            .with_months(4)
            .with_days(30)
            .with_hours(10)
            .with_minutes(21)
            .with_seconds(25)
    );
    assert_eq!(
        datetime!(677-3-31 23:25:27) - datetime!(1952-12-31 7:27:0),
        Interval::default()
            .with_years(-1275)
            .with_months(-8)
            .with_days(-29)
            .with_hours(-8)
            .with_minutes(-1)
            .with_seconds(-33)
    );
    assert_eq!(
        datetime!(72-5-31 15:47:33) - datetime!(142-7-31 16:37:42),
        Interval::default()
            .with_years(-70)
            .with_months(-2)
            .with_minutes(-50)
            .with_seconds(-9)
    );
    assert_eq!(
        datetime!(1569-4-30 6:46:1) - datetime!(1557-1-31 14:19:0),
        Interval::default()
            .with_years(12)
            .with_months(2)
            .with_days(29)
            .with_hours(16)
            .with_minutes(27)
            .with_seconds(1)
    );
    assert_eq!(
        datetime!(671-7-31 7:59:36) - datetime!(137-3-31 2:40:53),
        Interval::default()
            .with_years(534)
            .with_months(4)
            .with_hours(5)
            .with_minutes(18)
            .with_seconds(43)
    );
    assert_eq!(
        datetime!(828-9-30 1:13:15) - datetime!(948-7-31 22:19:5),
        Interval::default()
            .with_years(-119)
            .with_months(-10)
            .with_hours(-21)
            .with_minutes(-5)
            .with_seconds(-50)
    );
    assert_eq!(
        datetime!(2027-11-30 2:41:32) - datetime!(237-7-31 23:42:54),
        Interval::default()
            .with_years(1790)
            .with_months(3)
            .with_days(29)
            .with_hours(2)
            .with_minutes(58)
            .with_seconds(38)
    );
    assert_eq!(
        datetime!(671-6-30 5:43:54) - datetime!(398-1-31 1:22:39),
        Interval::default()
            .with_years(273)
            .with_months(5)
            .with_hours(4)
            .with_minutes(21)
            .with_seconds(15)
    );
    assert_eq!(
        datetime!(1238-6-30 10:28:40) - datetime!(1073-11-30 10:45:51),
        Interval::default()
            .with_years(164)
            .with_months(6)
            .with_days(30)
            .with_hours(23)
            .with_minutes(42)
            .with_seconds(49)
    );
    assert_eq!(
        datetime!(389-2-28 13:2:18) - datetime!(1174-1-31 1:34:9),
        Interval::default()
            .with_years(-784)
            .with_months(-10)
            .with_days(-30)
            .with_hours(-12)
            .with_minutes(-31)
            .with_seconds(-51)
    );
    assert_eq!(
        datetime!(1522-6-30 6:58:44) - datetime!(834-10-31 4:14:6),
        Interval::default()
            .with_years(687)
            .with_months(8)
            .with_hours(2)
            .with_minutes(44)
            .with_seconds(38)
    );
    assert_eq!(
        datetime!(731-12-31 17:36:52) - datetime!(1729-2-28 5:47:40),
        Interval::default()
            .with_years(-997)
            .with_months(-1)
            .with_days(-27)
            .with_hours(-12)
            .with_minutes(-10)
            .with_seconds(-48)
    );
    assert_eq!(
        datetime!(458-6-30 22:21:7) - datetime!(1575-5-31 21:24:59),
        Interval::default()
            .with_years(-1116)
            .with_months(-10)
            .with_days(-30)
            .with_hours(-23)
            .with_minutes(-3)
            .with_seconds(-52)
    );
    assert_eq!(
        datetime!(878-3-31 17:12:50) - datetime!(1595-9-30 10:31:53),
        Interval::default()
            .with_years(-717)
            .with_months(-5)
            .with_days(-29)
            .with_hours(-17)
            .with_minutes(-19)
            .with_seconds(-3)
    );
    assert_eq!(
        datetime!(1545-1-31 15:52:59) - datetime!(1671-12-31 17:16:24),
        Interval::default()
            .with_years(-126)
            .with_months(-11)
            .with_hours(-1)
            .with_minutes(-23)
            .with_seconds(-25)
    );
    assert_eq!(
        datetime!(333-10-31 23:29:51) - datetime!(206-8-31 22:8:25),
        Interval::default()
            .with_years(127)
            .with_months(2)
            .with_hours(1)
            .with_minutes(21)
            .with_seconds(26)
    );
    assert_eq!(
        datetime!(630-2-28 0:16:53) - datetime!(1941-8-31 10:19:3),
        Interval::default()
            .with_years(-1311)
            .with_months(-6)
            .with_hours(-10)
            .with_minutes(-2)
            .with_seconds(-10)
    );
    assert_eq!(
        datetime!(1121-9-30 18:1:43) - datetime!(274-1-31 4:0:20),
        Interval::default()
            .with_years(847)
            .with_months(8)
            .with_hours(14)
            .with_minutes(1)
            .with_seconds(23)
    );
    assert_eq!(
        datetime!(1084-6-30 17:22:19) - datetime!(460-3-31 18:54:27),
        Interval::default()
            .with_years(624)
            .with_months(2)
            .with_days(29)
            .with_hours(22)
            .with_minutes(27)
            .with_seconds(52)
    );
    assert_eq!(
        datetime!(1211-3-31 2:37:40) - datetime!(676-11-30 8:45:55),
        Interval::default()
            .with_years(534)
            .with_months(4)
            .with_hours(17)
            .with_minutes(51)
            .with_seconds(45)
    );
    assert_eq!(
        datetime!(218-6-30 15:10:51) - datetime!(1872-1-31 12:36:34),
        Interval::default()
            .with_years(-1653)
            .with_months(-6)
            .with_days(-30)
            .with_hours(-21)
            .with_minutes(-25)
            .with_seconds(-43)
    );
    assert_eq!(
        datetime!(152-5-31 12:42:17) - datetime!(1906-5-31 18:18:52),
        Interval::default()
            .with_years(-1754)
            .with_hours(-5)
            .with_minutes(-36)
            .with_seconds(-35)
    );
}

#[test]
fn diff_between_different_offset_datetimes() {
    assert_eq!(
        datetime!(1604-2-29 17:59:39 -14:20) - datetime!(987-8-31 14:47:0 +11:30),
        Interval::default()
            .with_years(616)
            .with_months(6)
            .with_days(1)
            .with_hours(5)
            .with_minutes(2)
            .with_seconds(39)
    );
    assert_eq!(
        datetime!(1132-9-30 13:24:47 -10:20) - datetime!(941-1-31 16:45:27 +16:00),
        Interval::default()
            .with_years(191)
            .with_months(8)
            .with_hours(22)
            .with_minutes(59)
            .with_seconds(20)
    );
    assert_eq!(
        datetime!(1513-1-31 8:52:21 -08:35) - datetime!(597-12-31 16:38:22 +11:25),
        Interval::default()
            .with_years(915)
            .with_months(1)
            .with_hours(12)
            .with_minutes(13)
            .with_seconds(59)
    );
    assert_eq!(
        datetime!(703-2-28 2:51:22 +03:45) - datetime!(444-6-30 4:34:21 +09:25),
        Interval::default()
            .with_years(258)
            .with_months(8)
            .with_hours(3)
            .with_minutes(57)
            .with_seconds(1)
    );
    assert_eq!(
        datetime!(670-12-31 10:19:43 +10:10) - datetime!(75-7-31 14:2:0 -14:25),
        Interval::default()
            .with_years(595)
            .with_months(4)
            .with_days(29)
            .with_hours(19)
            .with_minutes(42)
            .with_seconds(43)
    );
    assert_eq!(
        datetime!(173-3-31 16:23:42 -19:05) - datetime!(613-8-31 16:39:50 +16:25),
        Interval::default()
            .with_years(-440)
            .with_months(-4)
            .with_days(-28)
            .with_hours(-12)
            .with_minutes(-46)
            .with_seconds(-8)
    );
    assert_eq!(
        datetime!(15-9-30 9:3:2 +18:30) - datetime!(899-10-31 22:16:51 +13:15),
        Interval::default()
            .with_years(-884)
            .with_months(-1)
            .with_hours(-18)
            .with_minutes(-28)
            .with_seconds(-49)
    );
    assert_eq!(
        datetime!(877-3-31 17:54:21 +13:40) - datetime!(1609-2-28 21:29:7 +20:35),
        Interval::default()
            .with_years(-731)
            .with_months(-10)
            .with_days(-27)
            .with_hours(-20)
            .with_minutes(-39)
            .with_seconds(-46)
    );
    assert_eq!(
        datetime!(404-3-31 16:6:38 -10:00) - datetime!(169-4-30 9:43:34 -09:05),
        Interval::default()
            .with_years(234)
            .with_months(11)
            .with_days(1)
            .with_hours(7)
            .with_minutes(18)
            .with_seconds(4)
    );
    assert_eq!(
        datetime!(640-12-31 15:18:5 +20:00) - datetime!(534-3-31 7:14:37 -07:20),
        Interval::default()
            .with_years(106)
            .with_months(8)
            .with_days(30)
            .with_hours(4)
            .with_minutes(43)
            .with_seconds(28)
    );
    assert_eq!(
        datetime!(293-10-31 12:20:53 -08:15) - datetime!(442-12-31 16:34:7 -15:40),
        Interval::default()
            .with_years(-149)
            .with_months(-2)
            .with_hours(-11)
            .with_minutes(-38)
            .with_seconds(-14)
    );
    assert_eq!(
        datetime!(1489-11-30 5:32:19 +22:35) - datetime!(1352-5-31 18:40:46 +12:30),
        Interval::default()
            .with_years(137)
            .with_months(5)
            .with_days(29)
            .with_minutes(46)
            .with_seconds(33)
    );
    assert_eq!(
        datetime!(402-4-30 6:32:46 -13:10) - datetime!(2035-9-30 2:21:48 -17:45),
        Interval::default()
            .with_years(-1633)
            .with_months(-5)
            .with_minutes(-24)
            .with_seconds(-2)
    );
    assert_eq!(
        datetime!(1128-12-31 11:14:18 +18:05) - datetime!(1547-4-30 16:49:27 +20:45),
        Interval::default()
            .with_years(-418)
            .with_months(-3)
            .with_days(-30)
            .with_hours(-2)
            .with_minutes(-55)
            .with_seconds(-9)
    );
    assert_eq!(
        datetime!(69-7-31 1:45:59 +16:15) - datetime!(311-6-30 12:33:13 -06:45),
        Interval::default()
            .with_years(-241)
            .with_months(-11)
            .with_hours(-9)
            .with_minutes(-47)
            .with_seconds(-14)
    );
    assert_eq!(
        datetime!(869-2-28 9:26:44 +14:10) - datetime!(1856-10-31 9:31:19 -19:40),
        Interval::default()
            .with_years(-987)
            .with_months(-8)
            .with_days(-1)
            .with_hours(-9)
            .with_minutes(-54)
            .with_seconds(-35)
    );
    assert_eq!(
        datetime!(2040-3-31 3:24:17 +13:20) - datetime!(1827-5-31 12:38:57 +14:20),
        Interval::default()
            .with_years(212)
            .with_months(9)
            .with_days(30)
            .with_hours(15)
            .with_minutes(45)
            .with_seconds(20)
    );
    assert_eq!(
        datetime!(171-10-31 4:12:47 -14:35) - datetime!(538-3-31 19:39:23 +10:35),
        Interval::default()
            .with_years(-366)
            .with_months(-4)
            .with_days(-29)
            .with_hours(-14)
            .with_minutes(-16)
            .with_seconds(-36)
    );
    assert_eq!(
        datetime!(1079-8-31 15:45:42 -13:05) - datetime!(1096-6-30 23:22:35 -23:20),
        Interval::default()
            .with_years(-16)
            .with_months(-9)
            .with_days(-30)
            .with_hours(-17)
            .with_minutes(-51)
            .with_seconds(-53)
    );
    assert_eq!(
        datetime!(539-3-31 1:21:43 -05:30) - datetime!(854-1-31 11:56:58 -14:10),
        Interval::default()
            .with_years(-314)
            .with_months(-10)
            .with_hours(-19)
            .with_minutes(-15)
            .with_seconds(-15)
    );
    assert_eq!(
        datetime!(99-7-31 3:48:31 +04:40) - datetime!(129-6-30 10:34:25 +03:20),
        Interval::default()
            .with_years(-29)
            .with_months(-10)
            .with_days(-30)
            .with_hours(-8)
            .with_minutes(-5)
            .with_seconds(-54)
    );
    assert_eq!(
        datetime!(1190-10-31 14:10:29 -17:25) - datetime!(1278-11-30 18:26:55 -15:10),
        Interval::default()
            .with_years(-88)
            .with_days(-30)
            .with_hours(-2)
            .with_minutes(-1)
            .with_seconds(-26)
    );
    assert_eq!(
        datetime!(695-4-30 7:37:50 +13:45) - datetime!(673-9-30 4:12:56 +15:15),
        Interval::default()
            .with_years(21)
            .with_months(7)
            .with_hours(4)
            .with_minutes(54)
            .with_seconds(54)
    );
    assert_eq!(
        datetime!(1180-9-30 15:48:47 +14:10) - datetime!(628-1-31 6:55:35 +14:20),
        Interval::default()
            .with_years(552)
            .with_months(8)
            .with_hours(9)
            .with_minutes(3)
            .with_seconds(12)
    );
    assert_eq!(
        datetime!(1763-10-31 3:15:31 +13:35) - datetime!(790-1-31 13:46:6 -12:35),
        Interval::default()
            .with_years(973)
            .with_months(8)
            .with_days(29)
            .with_hours(11)
            .with_minutes(19)
            .with_seconds(25)
    );
    assert_eq!(
        datetime!(993-1-31 3:54:26 -04:10) - datetime!(1349-7-31 5:45:34 +12:55),
        Interval::default()
            .with_years(-356)
            .with_months(-5)
            .with_days(-27)
            .with_hours(-8)
            .with_minutes(-46)
            .with_seconds(-8)
    );
    assert_eq!(
        datetime!(1906-10-31 17:45:41 -16:25) - datetime!(1484-11-30 21:52:55 -00:50),
        Interval::default()
            .with_years(421)
            .with_months(11)
            .with_days(1)
            .with_hours(11)
            .with_minutes(27)
            .with_seconds(46)
    );
    assert_eq!(
        datetime!(1435-10-31 15:54:22 +06:05) - datetime!(1663-10-31 12:11:51 +10:25),
        Interval::default()
            .with_years(-227)
            .with_months(-11)
            .with_days(-29)
            .with_hours(-15)
            .with_minutes(-57)
            .with_seconds(-29)
    );
    assert_eq!(
        datetime!(2001-8-31 8:50:49 +20:10) - datetime!(1955-2-28 20:2:29 +06:45),
        Interval::default()
            .with_years(46)
            .with_months(6)
            .with_days(1)
            .with_hours(23)
            .with_minutes(23)
            .with_seconds(20)
    );
    assert_eq!(
        datetime!(1293-12-31 4:23:27 -22:15) - datetime!(150-11-30 8:17:8 -00:15),
        Interval::default()
            .with_years(1143)
            .with_months(1)
            .with_days(1)
            .with_hours(18)
            .with_minutes(6)
            .with_seconds(19)
    );
    assert_eq!(
        datetime!(2007-2-28 11:38:41 -20:25) - datetime!(2047-10-31 23:5:51 -03:20),
        Interval::default()
            .with_years(-40)
            .with_months(-7)
            .with_days(-30)
            .with_hours(-18)
            .with_minutes(-22)
            .with_seconds(-10)
    );
    assert_eq!(
        datetime!(495-5-31 11:18:31 -15:00) - datetime!(1084-6-30 23:17:53 -08:55),
        Interval::default()
            .with_years(-589)
            .with_days(-30)
            .with_hours(-5)
            .with_minutes(-54)
            .with_seconds(-22)
    );
    assert_eq!(
        datetime!(1659-1-31 0:54:20 -17:15) - datetime!(239-3-31 19:17:2 +09:40),
        Interval::default()
            .with_years(1419)
            .with_months(10)
            .with_hours(8)
            .with_minutes(32)
            .with_seconds(18)
    );
    assert_eq!(
        datetime!(1401-2-28 6:6:15 -17:00) - datetime!(907-11-30 16:52:52 -00:30),
        Interval::default()
            .with_years(493)
            .with_months(3)
            .with_hours(5)
            .with_minutes(43)
            .with_seconds(23)
    );
    assert_eq!(
        datetime!(605-4-30 10:16:3 -09:30) - datetime!(1615-6-30 15:39:33 +09:15),
        Interval::default()
            .with_years(-1010)
            .with_months(-1)
            .with_days(-29)
            .with_hours(-10)
            .with_minutes(-38)
            .with_seconds(-30)
    );
    assert_eq!(
        datetime!(1353-11-30 9:15:32 -21:55) - datetime!(1341-6-30 16:30:45 -19:00),
        Interval::default()
            .with_years(12)
            .with_months(4)
            .with_days(30)
            .with_hours(19)
            .with_minutes(39)
            .with_seconds(47)
    );
    assert_eq!(
        datetime!(967-9-30 5:6:10 +06:45) - datetime!(929-12-31 15:26:30 -11:05),
        Interval::default()
            .with_years(37)
            .with_months(8)
            .with_days(28)
            .with_hours(19)
            .with_minutes(49)
            .with_seconds(40)
    );
    assert_eq!(
        datetime!(336-12-31 16:11:39 +07:20) - datetime!(1696-8-31 15:18:23 -01:25),
        Interval::default()
            .with_years(-1359)
            .with_months(-8)
            .with_hours(-7)
            .with_minutes(-51)
            .with_seconds(-44)
    );
    assert_eq!(
        datetime!(1211-1-31 12:38:44 +12:50) - datetime!(1382-3-31 4:5:59 -23:25),
        Interval::default()
            .with_years(-171)
            .with_months(-2)
            .with_days(-1)
            .with_hours(-3)
            .with_minutes(-42)
            .with_seconds(-15)
    );
    assert_eq!(
        datetime!(718-6-30 7:8:5 -20:15) - datetime!(920-6-30 3:51:19 +08:45),
        Interval::default()
            .with_years(-201)
            .with_months(-11)
            .with_days(-28)
            .with_hours(-15)
            .with_minutes(-43)
            .with_seconds(-14)
    );
}
