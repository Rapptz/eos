use core::time::Duration;
use eos::{date, ext::IntervalLiteral, time, Interval};

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
