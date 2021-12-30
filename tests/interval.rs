use core::time::Duration;
use eos::{ext::IntervalLiteral, Date, Error, Interval, Time};

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
fn add_to_date() -> Result<(), Error> {
    assert_eq!(Date::new(2012, 2, 29)? + 1.days(), Date::new(2012, 3, 1)?);
    assert_eq!(Date::new(2012, 2, 29)? + 1.years(), Date::new(2013, 2, 28)?);
    assert_eq!(Date::new(2012, 1, 31)? + 1.months(), Date::new(2012, 2, 29)?);
    assert_eq!(Date::new(2001, 1, 31)? + 1.months(), Date::new(2001, 2, 28)?);
    Ok(())
}

#[test]
fn noop_date_addition() -> Result<(), Error> {
    assert_eq!(Date::new(2001, 1, 31)? + 10.minutes(), Date::new(2001, 1, 31)?);
    assert_eq!(Date::new(2001, 1, 31)? + 10.microseconds(), Date::new(2001, 1, 31)?);
    assert_eq!(Date::new(2001, 1, 31)? + 10.hours(), Date::new(2001, 1, 31)?);
    assert_eq!(Date::new(2001, 1, 31)? + 10.milliseconds(), Date::new(2001, 1, 31)?);
    assert_eq!(Date::new(2001, 1, 31)? + 10.nanoseconds(), Date::new(2001, 1, 31)?);
    Ok(())
}

#[test]
fn add_to_time() -> Result<(), Error> {
    assert_eq!(Time::new(0, 0, 0)? + 92.minutes(), Time::new(1, 32, 0)?);
    assert_eq!(Time::new(0, 0, 0)? - 2.minutes(), Time::new(23, 58, 0)?);
    Ok(())
}

#[test]
fn out_of_bounds_wrapping_time() -> Result<(), Error> {
    assert_eq!(
        Time::new(23, 59, 59)? + Interval::from_seconds(i64::MAX),
        Time::new(23, 59, 59)?
    );
    assert_eq!(
        Time::new(23, 59, 59)? - Interval::from_seconds(i64::MAX),
        Time::new(23, 59, 59)?
    );
    Ok(())
}

#[test]
fn random_single_units_to_dates() -> Result<(), Error> {
    assert_eq!(Date::new(268, 7, 31)? + 10.years(), Date::new(278, 7, 31)?);
    assert_eq!(Date::new(221, 1, 31)? + (-82).years(), Date::new(139, 1, 31)?);
    assert_eq!(Date::new(1723, 3, 31)? - (-35).years(), Date::new(1758, 3, 31)?);
    assert_eq!(Date::new(651, 11, 30)? - 99.days(), Date::new(651, 8, 23)?);
    assert_eq!(Date::new(1476, 10, 31)? + 53.years(), Date::new(1529, 10, 31)?);
    assert_eq!(Date::new(927, 1, 31)? + 3.months(), Date::new(927, 4, 30)?);
    assert_eq!(Date::new(348, 2, 29)? - 38.months(), Date::new(344, 12, 29)?);
    assert_eq!(Date::new(1707, 4, 30)? - (-77).months(), Date::new(1713, 9, 30)?);
    assert_eq!(Date::new(1444, 10, 31)? - 27.months(), Date::new(1442, 7, 31)?);
    assert_eq!(Date::new(100, 1, 31)? - (-68).months(), Date::new(105, 9, 30)?);
    assert_eq!(Date::new(1371, 7, 31)? + (-64).months(), Date::new(1366, 3, 31)?);
    assert_eq!(Date::new(1599, 8, 31)? + 62.months(), Date::new(1604, 10, 31)?);
    assert_eq!(Date::new(1031, 1, 31)? + (-54).days(), Date::new(1030, 12, 8)?);
    assert_eq!(Date::new(855, 10, 31)? + (-57).months(), Date::new(851, 1, 31)?);
    assert_eq!(Date::new(1691, 3, 31)? - (-55).days(), Date::new(1691, 5, 25)?);
    assert_eq!(Date::new(927, 5, 31)? + 89.days(), Date::new(927, 8, 28)?);
    assert_eq!(Date::new(904, 1, 31)? - 20.weeks(), Date::new(903, 9, 13)?);
    assert_eq!(Date::new(779, 12, 31)? + (-72).weeks(), Date::new(778, 8, 14)?);
    assert_eq!(Date::new(1689, 6, 30)? - (-27).weeks(), Date::new(1690, 1, 5)?);
    assert_eq!(Date::new(806, 12, 31)? - (-59).months(), Date::new(811, 11, 30)?);
    Ok(())
}

#[test]
fn random_double_units_to_dates() -> Result<(), Error> {
    assert_eq!(
        Date::new(806, 2, 28)? - ((-7).weeks() + (-90).months()),
        Date::new(813, 10, 16)?
    );
    assert_eq!(
        Date::new(391, 11, 30)? + ((-7).weeks() + 22.years()),
        Date::new(413, 10, 12)?
    );
    assert_eq!(
        Date::new(1925, 7, 31)? + (56.weeks() + 51.months()),
        Date::new(1930, 11, 27)?
    );
    assert_eq!(
        Date::new(1423, 2, 28)? - ((-37).months() + (-51).years()),
        Date::new(1477, 3, 28)?
    );
    assert_eq!(Date::new(66, 11, 30)? + (-50).weeks(), Date::new(65, 12, 15)?);
    assert_eq!(Date::new(2020, 3, 31)? - 12.months(), Date::new(2019, 3, 31)?);
    assert_eq!(
        Date::new(46, 7, 31)? + ((-47).days() + (-41).months()),
        Date::new(43, 1, 12)?
    );
    assert_eq!(
        Date::new(987, 7, 31)? - (67.months() + (-26).days()),
        Date::new(982, 1, 26)?
    );
    assert_eq!(Date::new(1831, 4, 30)? - (-11).months(), Date::new(1832, 3, 30)?);
    assert_eq!(
        Date::new(812, 1, 31)? + ((-34).months() + 11.years()),
        Date::new(820, 3, 31)?
    );
    assert_eq!(
        Date::new(1194, 5, 31)? - ((-42).years() + (-79).days()),
        Date::new(1236, 8, 18)?
    );
    assert_eq!(Date::new(1943, 8, 31)? + (-80).days(), Date::new(1943, 6, 12)?);
    assert_eq!(Date::new(1609, 5, 31)? + 13.years(), Date::new(1622, 5, 31)?);
    assert_eq!(
        Date::new(614, 7, 31)? - ((-52).days() + (-66).years()),
        Date::new(680, 9, 21)?
    );
    assert_eq!(Date::new(170, 4, 30)? + 24.days(), Date::new(170, 5, 24)?);
    assert_eq!(
        Date::new(488, 2, 29)? + (12.weeks() + 96.months()),
        Date::new(496, 5, 23)?
    );
    assert_eq!(
        Date::new(1349, 10, 31)? + ((-90).days() + (-32).weeks()),
        Date::new(1348, 12, 21)?
    );
    assert_eq!(Date::new(1942, 7, 31)? + 53.weeks(), Date::new(1943, 8, 6)?);
    assert_eq!(
        Date::new(1095, 7, 31)? - (22.weeks() + 53.days()),
        Date::new(1095, 1, 5)?
    );
    assert_eq!(
        Date::new(539, 4, 30)? + (3.months() + 48.weeks()),
        Date::new(540, 6, 30)?
    );
    Ok(())
}
#[test]
fn random_single_units_to_times() -> Result<(), Error> {
    assert_eq!(Time::new(16, 51, 49)? - (-13).minutes(), Time::new(17, 4, 49)?);
    assert_eq!(Time::new(13, 20, 48)? - (-4).minutes(), Time::new(13, 24, 48)?);
    assert_eq!(Time::new(6, 51, 52)? - 44.minutes(), Time::new(6, 7, 52)?);
    assert_eq!(Time::new(20, 19, 13)? - 41.minutes(), Time::new(19, 38, 13)?);
    assert_eq!(Time::new(1, 4, 31)? - 0.hours(), Time::new(1, 4, 31)?);
    assert_eq!(Time::new(19, 11, 31)? - 32.seconds(), Time::new(19, 10, 59)?);
    assert_eq!(Time::new(7, 31, 3)? + 2.hours(), Time::new(9, 31, 3)?);
    assert_eq!(Time::new(7, 5, 30)? - (-20).hours(), Time::new(3, 5, 30)?);
    assert_eq!(Time::new(11, 56, 1)? - (-18).minutes(), Time::new(12, 14, 1)?);
    assert_eq!(Time::new(18, 51, 17)? - (-51).seconds(), Time::new(18, 52, 8)?);
    assert_eq!(Time::new(21, 13, 48)? - 36.seconds(), Time::new(21, 13, 12)?);
    assert_eq!(Time::new(19, 42, 49)? - 99.minutes(), Time::new(18, 3, 49)?);
    assert_eq!(Time::new(0, 54, 29)? + (-70).seconds(), Time::new(0, 53, 19)?);
    assert_eq!(Time::new(14, 20, 2)? - (-49).minutes(), Time::new(15, 9, 2)?);
    assert_eq!(Time::new(8, 51, 55)? - 58.seconds(), Time::new(8, 50, 57)?);
    assert_eq!(Time::new(9, 21, 3)? + 87.seconds(), Time::new(9, 22, 30)?);
    assert_eq!(Time::new(2, 40, 56)? + (-2).seconds(), Time::new(2, 40, 54)?);
    assert_eq!(Time::new(15, 24, 1)? + 47.hours(), Time::new(14, 24, 1)?);
    assert_eq!(Time::new(7, 38, 1)? + (-21).hours(), Time::new(10, 38, 1)?);
    assert_eq!(Time::new(11, 26, 23)? - 65.minutes(), Time::new(10, 21, 23)?);
    Ok(())
}

#[test]
fn random_double_units_to_times() -> Result<(), Error> {
    assert_eq!(
        Time::new(9, 16, 42)? + (86.hours() + 17.minutes()),
        Time::new(23, 33, 42)?
    );
    assert_eq!(Time::new(15, 13, 14)? + (-47).hours(), Time::new(16, 13, 14)?);
    assert_eq!(Time::new(17, 39, 18)? + 96.hours(), Time::new(17, 39, 18)?);
    assert_eq!(
        Time::new(0, 40, 43)? + (0.seconds() + (-46).hours()),
        Time::new(2, 40, 43)?
    );
    assert_eq!(Time::new(12, 48, 19)? + (-27).seconds(), Time::new(12, 47, 52)?);
    assert_eq!(
        Time::new(11, 11, 30)? + ((-33).hours() + 2.seconds()),
        Time::new(2, 11, 32)?
    );
    assert_eq!(Time::new(8, 52, 53)? - (-98).seconds(), Time::new(8, 54, 31)?);
    assert_eq!(Time::new(18, 39, 30)? - (-56).hours(), Time::new(2, 39, 30)?);
    assert_eq!(Time::new(13, 52, 13)? + 91.minutes(), Time::new(15, 23, 13)?);
    assert_eq!(
        Time::new(18, 3, 35)? - ((-85).minutes() + (-90).hours()),
        Time::new(13, 28, 35)?
    );
    assert_eq!(Time::new(2, 17, 44)? + 93.hours(), Time::new(23, 17, 44)?);
    assert_eq!(
        Time::new(8, 23, 42)? + ((-2).hours() + (-42).seconds()),
        Time::new(6, 23, 0)?
    );
    assert_eq!(
        Time::new(15, 44, 3)? - ((-7).minutes() + 62.hours()),
        Time::new(1, 51, 3)?
    );
    assert_eq!(
        Time::new(8, 48, 28)? - (51.minutes() + (-77).seconds()),
        Time::new(7, 58, 45)?
    );
    assert_eq!(
        Time::new(6, 7, 49)? - (50.minutes() + (-12).hours()),
        Time::new(17, 17, 49)?
    );
    assert_eq!(
        Time::new(3, 24, 42)? + (80.hours() + (-4).minutes()),
        Time::new(11, 20, 42)?
    );
    assert_eq!(Time::new(13, 21, 23)? + (-33).minutes(), Time::new(12, 48, 23)?);
    assert_eq!(
        Time::new(8, 23, 54)? + ((-63).seconds() + 56.hours()),
        Time::new(16, 22, 51)?
    );
    assert_eq!(
        Time::new(20, 26, 47)? + ((-50).seconds() + 53.hours()),
        Time::new(1, 25, 57)?
    );
    assert_eq!(
        Time::new(11, 50, 19)? - (36.hours() + (-30).minutes()),
        Time::new(0, 20, 19)?
    );
    Ok(())
}
