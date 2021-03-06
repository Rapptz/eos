use std::time::Duration;

use eos::{
    date, datetime,
    ext::IntervalLiteral,
    fmt::{FromIsoFormat, ToIsoFormat},
    time, utc_offset, Date, DateTime, Interval, IsoWeekDate, Time, Weekday,
};

#[test]
fn test_duration_isoformat() {
    assert_eq!(Duration::from_millis(500).to_iso_format(), "PT0.5S");
    assert_eq!(Duration::from_secs_f64(20.345).to_iso_format(), "PT20.345S");
    assert_eq!(Duration::from_secs(15 * 60).to_iso_format(), "PT15M");
    assert_eq!(Duration::from_secs(10 * 3600).to_iso_format(), "PT10H");
    assert_eq!(Duration::from_secs(2 * 86400).to_iso_format(), "PT48H");
}

#[test]
fn test_time_isoformat() {
    assert_eq!(time!(21:57:52).to_iso_format(), "21:57:52");
    assert_eq!(time!(12:53:22).to_iso_format(), "12:53:22");
    assert_eq!(time!(23:13:19).to_iso_format(), "23:13:19");
    assert_eq!(time!(12:48:16).to_iso_format(), "12:48:16");
    assert_eq!(time!(08:54:00).to_iso_format(), "08:54:00");
    assert_eq!(time!(11:09:41).to_iso_format(), "11:09:41");
    assert_eq!(time!(15:34:22).to_iso_format(), "15:34:22");
    assert_eq!(time!(01:03:36).to_iso_format(), "01:03:36");
    assert_eq!(time!(12:24:52).to_iso_format(), "12:24:52");
    assert_eq!(time!(18:51:04).to_iso_format(), "18:51:04");
    assert_eq!(time!(16:22:01).to_iso_format(), "16:22:01");
    assert_eq!(time!(07:21:47).to_iso_format(), "07:21:47");
    assert_eq!(time!(21:11:37).to_iso_format(), "21:11:37");
    assert_eq!(time!(19:18:45).to_iso_format(), "19:18:45");
    assert_eq!(time!(08:04:47).to_iso_format(), "08:04:47");
    assert_eq!(time!(02:41:18).to_iso_format(), "02:41:18");
    assert_eq!(time!(20:57:25).to_iso_format(), "20:57:25");
    assert_eq!(time!(15:59:46).to_iso_format(), "15:59:46");
    assert_eq!(time!(14:09:53).to_iso_format(), "14:09:53");
    assert_eq!(time!(06:38:00).to_iso_format(), "06:38:00");
    assert_eq!(time!(22:00:24).to_iso_format(), "22:00:24");
    assert_eq!(time!(23:43:25).to_iso_format(), "23:43:25");
    assert_eq!(time!(13:53:35).to_iso_format(), "13:53:35");
    assert_eq!(time!(00:45:56).to_iso_format(), "00:45:56");
    assert_eq!(time!(08:36:05).to_iso_format(), "08:36:05");
    assert_eq!(time!(21:19:20).to_iso_format(), "21:19:20");
    assert_eq!(time!(11:26:31).to_iso_format(), "11:26:31");
    assert_eq!(time!(04:50:07).to_iso_format(), "04:50:07");
    assert_eq!(time!(10:01:56).to_iso_format(), "10:01:56");
    assert_eq!(time!(20:18:31).to_iso_format(), "20:18:31");

    let base_time = time!(20:10:30);
    let a1 = base_time.with_microsecond(123).unwrap();
    let a2 = base_time.with_millisecond(123).unwrap();
    assert_eq!(a1.to_iso_format(), "20:10:30.000123");
    assert_eq!(a2.to_iso_format(), "20:10:30.123000");
}

#[test]
fn test_datetime_isoformat() {
    assert_eq!(
        datetime!(8780-01-10 20:34:07).to_iso_format(),
        "8780-01-10T20:34:07+00:00"
    );
    assert_eq!(
        datetime!(5680-07-17 08:17:39).to_iso_format(),
        "5680-07-17T08:17:39+00:00"
    );
    assert_eq!(
        datetime!(7653-04-07 00:25:04).to_iso_format(),
        "7653-04-07T00:25:04+00:00"
    );
    assert_eq!(
        datetime!(8588-06-27 10:42:16).to_iso_format(),
        "8588-06-27T10:42:16+00:00"
    );
    assert_eq!(
        datetime!(2322-07-07 17:20:56).to_iso_format(),
        "2322-07-07T17:20:56+00:00"
    );
    assert_eq!(
        datetime!(691-11-27 15:48:10).to_iso_format(),
        "0691-11-27T15:48:10+00:00"
    );
    assert_eq!(
        datetime!(6975-12-13 17:33:40).to_iso_format(),
        "6975-12-13T17:33:40+00:00"
    );
    assert_eq!(
        datetime!(3695-06-07 00:19:02).to_iso_format(),
        "3695-06-07T00:19:02+00:00"
    );
    assert_eq!(
        datetime!(6190-01-27 19:34:34).to_iso_format(),
        "6190-01-27T19:34:34+00:00"
    );
    assert_eq!(
        datetime!(2166-01-08 23:48:14).to_iso_format(),
        "2166-01-08T23:48:14+00:00"
    );
    assert_eq!(
        datetime!(6785-03-05 02:21:04).to_iso_format(),
        "6785-03-05T02:21:04+00:00"
    );
    assert_eq!(
        datetime!(9254-08-21 18:21:29).to_iso_format(),
        "9254-08-21T18:21:29+00:00"
    );
    assert_eq!(
        datetime!(3966-11-28 15:35:01).to_iso_format(),
        "3966-11-28T15:35:01+00:00"
    );
    assert_eq!(
        datetime!(3190-07-21 06:45:53).to_iso_format(),
        "3190-07-21T06:45:53+00:00"
    );
    assert_eq!(
        datetime!(7113-04-27 10:22:06).to_iso_format(),
        "7113-04-27T10:22:06+00:00"
    );
    assert_eq!(
        datetime!(5627-04-24 01:07:28).to_iso_format(),
        "5627-04-24T01:07:28+00:00"
    );
    assert_eq!(
        datetime!(1692-08-23 12:40:09).to_iso_format(),
        "1692-08-23T12:40:09+00:00"
    );
    assert_eq!(
        datetime!(2433-02-03 16:44:14).to_iso_format(),
        "2433-02-03T16:44:14+00:00"
    );
    assert_eq!(
        datetime!(580-03-15 21:54:53).to_iso_format(),
        "0580-03-15T21:54:53+00:00"
    );
    assert_eq!(
        datetime!(6005-10-19 01:58:23).to_iso_format(),
        "6005-10-19T01:58:23+00:00"
    );
    assert_eq!(
        datetime!(8625-02-07 18:28:31).to_iso_format(),
        "8625-02-07T18:28:31+00:00"
    );
    assert_eq!(
        datetime!(9055-09-06 11:57:46).to_iso_format(),
        "9055-09-06T11:57:46+00:00"
    );
    assert_eq!(
        datetime!(3961-12-23 14:04:24).to_iso_format(),
        "3961-12-23T14:04:24+00:00"
    );
    assert_eq!(
        datetime!(4352-01-08 05:49:05).to_iso_format(),
        "4352-01-08T05:49:05+00:00"
    );
    assert_eq!(
        datetime!(5208-02-03 20:55:51).to_iso_format(),
        "5208-02-03T20:55:51+00:00"
    );
    assert_eq!(
        datetime!(8360-04-12 15:05:23).to_iso_format(),
        "8360-04-12T15:05:23+00:00"
    );
    assert_eq!(
        datetime!(6981-12-20 10:56:45).to_iso_format(),
        "6981-12-20T10:56:45+00:00"
    );
    assert_eq!(
        datetime!(644-05-24 01:39:16).to_iso_format(),
        "0644-05-24T01:39:16+00:00"
    );
    assert_eq!(
        datetime!(4904-11-01 18:46:19).to_iso_format(),
        "4904-11-01T18:46:19+00:00"
    );
    assert_eq!(
        datetime!(2685-01-21 23:34:24).to_iso_format(),
        "2685-01-21T23:34:24+00:00"
    );
}

#[test]
fn test_date_isoformat() {
    assert_eq!(date!(6921 - 11 - 17).to_iso_format(), "6921-11-17");
    assert_eq!(date!(7773 - 01 - 16).to_iso_format(), "7773-01-16");
    assert_eq!(date!(5389 - 05 - 08).to_iso_format(), "5389-05-08");
    assert_eq!(date!(5672 - 05 - 24).to_iso_format(), "5672-05-24");
    assert_eq!(date!(6917 - 10 - 03).to_iso_format(), "6917-10-03");
    assert_eq!(date!(1002 - 04 - 23).to_iso_format(), "1002-04-23");
    assert_eq!(date!(4313 - 03 - 21).to_iso_format(), "4313-03-21");
    assert_eq!(date!(941 - 01 - 09).to_iso_format(), "0941-01-09");
    assert_eq!(date!(3741 - 10 - 29).to_iso_format(), "3741-10-29");
    assert_eq!(date!(9294 - 12 - 13).to_iso_format(), "9294-12-13");
    assert_eq!(date!(3937 - 04 - 18).to_iso_format(), "3937-04-18");
    assert_eq!(date!(3081 - 11 - 16).to_iso_format(), "3081-11-16");
    assert_eq!(date!(6476 - 04 - 21).to_iso_format(), "6476-04-21");
    assert_eq!(date!(666 - 11 - 12).to_iso_format(), "0666-11-12");
    assert_eq!(date!(9773 - 11 - 19).to_iso_format(), "9773-11-19");
    assert_eq!(date!(4594 - 12 - 25).to_iso_format(), "4594-12-25");
    assert_eq!(date!(7557 - 11 - 14).to_iso_format(), "7557-11-14");
    assert_eq!(date!(802 - 06 - 17).to_iso_format(), "0802-06-17");
    assert_eq!(date!(2345 - 04 - 12).to_iso_format(), "2345-04-12");
    assert_eq!(date!(5738 - 07 - 15).to_iso_format(), "5738-07-15");
    assert_eq!(date!(9344 - 01 - 06).to_iso_format(), "9344-01-06");
    assert_eq!(date!(3634 - 02 - 17).to_iso_format(), "3634-02-17");
    assert_eq!(date!(7763 - 06 - 25).to_iso_format(), "7763-06-25");
    assert_eq!(date!(8412 - 05 - 30).to_iso_format(), "8412-05-30");
    assert_eq!(date!(7438 - 05 - 31).to_iso_format(), "7438-05-31");
    assert_eq!(date!(8028 - 06 - 11).to_iso_format(), "8028-06-11");
    assert_eq!(date!(9639 - 03 - 19).to_iso_format(), "9639-03-19");
    assert_eq!(date!(5029 - 04 - 14).to_iso_format(), "5029-04-14");
    assert_eq!(date!(2403 - 03 - 11).to_iso_format(), "2403-03-11");
    assert_eq!(date!(5080 - 02 - 14).to_iso_format(), "5080-02-14");
}

#[test]
fn test_interval_isoformat() {
    assert_eq!(eos::Interval::ZERO.to_iso_format(), "PT0S");
    assert_eq!((1.days() + 3.months() + 1.years()).to_iso_format(), "P1Y3M1D");
    assert_eq!((3.seconds() + 500.milliseconds()).to_iso_format(), "PT3.5S");
}

#[test]
fn test_interval_normalization_equivalency() {
    assert_eq!(
        (2.years() + 1.months() + 2.days() + 10.minutes()),
        Interval::from_iso_format("P2Y1M2DT10M").unwrap()
    );
    assert_eq!(
        (9.hours() + 75.minutes() + (-94).seconds()),
        Interval::from_iso_format("PT9H75M-94S").unwrap()
    );
    assert_eq!(
        (9.hours() + 75.minutes() + (-94).seconds() + 24.milliseconds()),
        Interval::from_iso_format("PT9H75M-93.976S").unwrap()
    );
    assert_eq!(
        ((-88).hours() + 69.minutes() + (-53).seconds() + (-18).milliseconds()),
        Interval::from_iso_format("PT-88H69M-53.018S").unwrap()
    );
    assert_eq!(
        (96.hours() + (-63).minutes() + (-15).seconds() + 47.milliseconds()),
        Interval::from_iso_format("PT96H-63M-14.953S").unwrap()
    );
    assert_eq!(
        ((-67).hours() + 24.minutes() + 62.seconds()),
        Interval::from_iso_format("PT-67H24M62S").unwrap()
    );
    assert_eq!(
        ((-67).hours() + 24.minutes() + 62.seconds() + 64.milliseconds()),
        Interval::from_iso_format("PT-67H24M62.064S").unwrap()
    );
    assert_eq!(
        ((-91).hours() + 59.minutes() + 72.seconds() + 22.milliseconds()),
        Interval::from_iso_format("PT-91H59M72.022S").unwrap()
    );
    assert_eq!(
        (84.hours() + 25.minutes() + (-76).seconds() + (-67).milliseconds()),
        Interval::from_iso_format("PT84H25M-76.067S").unwrap()
    );
    assert_eq!(
        ((-80).hours() + 62.minutes() + (-20).seconds() + (-28).milliseconds()),
        Interval::from_iso_format("PT-80H62M-20.028S").unwrap()
    );
    assert_eq!(
        ((-73).hours() + (-75).minutes() + (-75).seconds() + (-22).milliseconds()),
        Interval::from_iso_format("PT-73H-75M-75.022S").unwrap()
    );
    assert_eq!(
        (7.hours() + 2.minutes() + 37.seconds()),
        Interval::from_iso_format("PT7H2M37S").unwrap()
    );
    assert_eq!(
        (27.hours() + 50.minutes() + 19.seconds()),
        Interval::from_iso_format("PT27H50M19S").unwrap()
    );
    assert_eq!(
        ((-38).hours() + 95.minutes() + 12.seconds()),
        Interval::from_iso_format("PT-38H95M12S").unwrap()
    );
    assert_eq!(
        (75.hours() + 94.minutes() + 88.seconds()),
        Interval::from_iso_format("PT75H94M88S").unwrap()
    );
    assert_eq!(
        ((-47).years() + 43.months() + 20.days()),
        Interval::from_iso_format("P-47Y43M20D").unwrap()
    );
    assert_eq!(
        ((-68).years() + 83.months() + 35.days()),
        Interval::from_iso_format("P-68Y83M35D").unwrap()
    );
    assert_eq!(
        (7.years() + (-4).months() + 58.days()),
        Interval::from_iso_format("P7Y-4M58D").unwrap()
    );
    assert_eq!(
        ((-12).years() + (-16).months() + (-69).days()),
        Interval::from_iso_format("P-12Y-16M-69D").unwrap()
    );
    assert_eq!(
        (80.years() + (-43).months() + 92.days()),
        Interval::from_iso_format("P80Y-43M92D").unwrap()
    );
    assert_eq!(
        ((-93).years() + (-87).months() + 99.days()),
        Interval::from_iso_format("P-93Y-87M99D").unwrap()
    );
    assert_eq!(
        (46.years() + 16.months() + (-33).days()),
        Interval::from_iso_format("P46Y16M-33D").unwrap()
    );
    assert_eq!(
        (46.years() + 42.months() + 1.days()),
        Interval::from_iso_format("P46Y42M1D").unwrap()
    );
    assert_eq!(
        ((-39).years() + (-36).months() + (-55).days()),
        Interval::from_iso_format("P-39Y-36M-55D").unwrap()
    );
    assert_eq!(
        (50.years() + 87.months() + (-90).days()),
        Interval::from_iso_format("P50Y87M-90D").unwrap()
    );
    assert_eq!(
        ((-41).years() + (-69).months() + (-19).days()),
        Interval::from_iso_format("P-41Y-69M-19D").unwrap()
    );
    assert_eq!(
        (92.years() + 46.months() + 93.days()),
        Interval::from_iso_format("P92Y46M93D").unwrap()
    );
    assert_eq!(
        ((-63).years() + 81.months() + 89.days() + 30.hours() + (-23).minutes() + 68.seconds() + (-99).milliseconds()),
        Interval::from_iso_format("P-63Y81M89DT30H-23M67.901S").unwrap()
    );
    assert_eq!(
        ((-76).years() + 5.months() + 85.days() + 84.hours() + 20.minutes() + 59.seconds() + (-51).milliseconds()),
        Interval::from_iso_format("P-76Y5M85DT84H20M58.949S").unwrap()
    );
    assert_eq!(
        ((-50).years()
            + 55.months()
            + 98.days()
            + (-31).hours()
            + (-59).minutes()
            + 94.seconds()
            + (-79).milliseconds()),
        Interval::from_iso_format("P-50Y55M98DT-31H-59M93.921S").unwrap()
    );
    assert_eq!(
        ((-79).years()
            + (-38).months()
            + 73.days()
            + 29.hours()
            + (-34).minutes()
            + (-73).seconds()
            + 1.milliseconds()),
        Interval::from_iso_format("P-79Y-38M73DT29H-34M-72.999S").unwrap()
    );
    assert_eq!(
        (4.years()
            + (-100).months()
            + (-54).days()
            + 79.hours()
            + 60.minutes()
            + (-29).seconds()
            + (-9).milliseconds()),
        Interval::from_iso_format("P4Y-100M-54DT79H60M-29.009S").unwrap()
    );
    assert_eq!(
        ((-89).years()
            + (-52).months()
            + (-95).days()
            + 66.hours()
            + (-71).minutes()
            + (-7).seconds()
            + 93.milliseconds()),
        Interval::from_iso_format("P-89Y-52M-95DT66H-71M-6.907S").unwrap()
    );
    assert_eq!(
        (36.years()
            + (-85).months()
            + (-12).days()
            + (-51).hours()
            + (-18).minutes()
            + 56.seconds()
            + (-28).milliseconds()),
        Interval::from_iso_format("P36Y-85M-12DT-51H-18M55.972S").unwrap()
    );
    assert_eq!(
        (78.years() + (-16).months() + (-11).days() + 87.hours() + 58.minutes() + 68.seconds()),
        Interval::from_iso_format("P78Y-16M-11DT87H58M68S").unwrap()
    );
    assert_eq!(
        ((-2).years() + (-36).months() + 41.days() + (-95).hours() + 62.seconds()),
        Interval::from_iso_format("P-2Y-36M41DT-95H62S").unwrap()
    );
    assert_eq!(
        (26.years() + 88.months() + 96.hours() + 68.seconds()),
        Interval::from_iso_format("P26Y88MT96H68S").unwrap()
    );
    assert_eq!(
        ((-8).years() + (-60).months() + 35.minutes() + 44.milliseconds()),
        Interval::from_iso_format("P-8Y-60MT35M0.044S").unwrap()
    );
    assert_eq!(
        (35.days() + 25.hours() + 80.seconds() + (-10).milliseconds()),
        Interval::from_iso_format("P35DT25H79.99S").unwrap()
    );
}

#[test]
fn test_valid_from_year_month() -> Result<(), eos::fmt::ParseError> {
    assert_eq!(Date::from_iso_format("2012-02")?, date!(2012 - 02 - 01));
    assert_eq!(Date::from_iso_format("2021-04")?, date!(2021 - 04 - 01));
    assert_eq!(Date::from_iso_format("-9999-04")?, date!(-9999 - 04 - 01));
    Ok(())
}

#[test]
fn test_valid_from_year_ordinal() -> Result<(), eos::fmt::ParseError> {
    assert_eq!(Date::from_iso_format("2009-123")?, date!(2009 - 05 - 03));
    assert_eq!(Date::from_iso_format("2009-222")?, date!(2009 - 08 - 10));
    assert_eq!(Date::from_iso_format("2009-001")?, date!(2009 - 01 - 01));
    Ok(())
}

#[test]
fn test_valid_year_week() -> Result<(), eos::fmt::ParseError> {
    assert_eq!(Date::from_iso_format("2009-W01-1")?, date!(2008 - 12 - 29));
    assert_eq!(Date::from_iso_format("2009-W53-7")?, date!(2010 - 01 - 03));
    assert_eq!(Date::from_iso_format("2009-W51-1")?, date!(2009 - 12 - 14));
    assert_eq!(Date::from_iso_format("2008-W39-6")?, date!(2008 - 09 - 27));
    assert_eq!(Date::from_iso_format("2009-W33")?, date!(2009 - 08 - 10));
    assert_eq!(Date::from_iso_format("2009-W51-1")?, date!(2009 - 12 - 14));
    assert_eq!(Date::from_iso_format("2009-W21-2")?, date!(2009 - 05 - 19));

    assert_eq!(
        IsoWeekDate::from_iso_format("2009-W01-1")?,
        date!(2008 - 12 - 29).iso_week()
    );
    assert_eq!(
        IsoWeekDate::from_iso_format("2009-W53-7")?,
        date!(2010 - 01 - 03).iso_week()
    );
    assert_eq!(
        IsoWeekDate::from_iso_format("2009-W51-1")?,
        date!(2009 - 12 - 14).iso_week()
    );
    assert_eq!(
        IsoWeekDate::from_iso_format("2008-W39-6")?,
        date!(2008 - 09 - 27).iso_week()
    );
    assert_eq!(
        IsoWeekDate::from_iso_format("2009-W33")?,
        date!(2009 - 08 - 10).iso_week()
    );
    assert_eq!(
        IsoWeekDate::from_iso_format("2009-W51-1")?,
        date!(2009 - 12 - 14).iso_week()
    );
    assert_eq!(
        IsoWeekDate::from_iso_format("2009-W21-2")?,
        date!(2009 - 05 - 19).iso_week()
    );

    assert_eq!(
        IsoWeekDate::from_iso_format("2009-W01-1")?,
        IsoWeekDate::new(2009, 1, Weekday::Monday).unwrap()
    );
    assert_eq!(
        IsoWeekDate::from_iso_format("2009-W53-7")?,
        IsoWeekDate::new(2009, 53, Weekday::Sunday).unwrap()
    );
    assert_eq!(
        IsoWeekDate::from_iso_format("2009-W51-1")?,
        IsoWeekDate::new(2009, 51, Weekday::Monday).unwrap()
    );
    assert_eq!(
        IsoWeekDate::from_iso_format("2008-W39-6")?,
        IsoWeekDate::new(2008, 39, Weekday::Saturday).unwrap()
    );
    assert_eq!(
        IsoWeekDate::from_iso_format("2009-W33")?,
        IsoWeekDate::new(2009, 33, Weekday::Monday).unwrap()
    );
    assert_eq!(
        IsoWeekDate::from_iso_format("2009-W51-1")?,
        IsoWeekDate::new(2009, 51, Weekday::Monday).unwrap()
    );
    assert_eq!(
        IsoWeekDate::from_iso_format("2009-W21-2")?,
        IsoWeekDate::new(2009, 21, Weekday::Tuesday).unwrap()
    );
    Ok(())
}

#[test]
fn test_valid_time() -> Result<(), eos::fmt::ParseError> {
    assert_eq!(
        Time::from_iso_format("06:14:00.000123")?,
        time!(06:14:00).with_microsecond(123).unwrap()
    );
    assert_eq!(Time::from_iso_format("06:40:34.00")?, time!(06:40:34));
    assert_eq!(Time::from_iso_format("19:20")?, time!(19:20));
    assert_eq!(Time::from_iso_format("08:00:00")?, time!(08:00:00));
    assert_eq!(
        Time::from_iso_format("15:34:56.123")?,
        time!(15:34:56).with_millisecond(123).unwrap()
    );
    assert_eq!(Time::from_iso_format("15:34:56")?, time!(15:34:56));
    assert_eq!(
        Time::from_iso_format("11:43:55.328")?,
        time!(11:43:55).with_millisecond(328).unwrap()
    );
    assert_eq!(
        Time::from_iso_format("15:34:56.123")?,
        time!(15:34:56).with_millisecond(123).unwrap()
    );
    assert_eq!(
        Time::from_iso_format("15:34:56.123456789")?,
        time!(15:34:56).with_nanosecond(123456789).unwrap()
    );
    assert_eq!(Time::from_iso_format("18:30")?, time!(18:30));
    Ok(())
}

#[test]
fn test_valid_datetime() -> Result<(), eos::fmt::ParseError> {
    assert_eq!(
        DateTime::from_iso_format("2007-06-23T06:40:34.00Z")?,
        datetime!(2007-6-23 6:40:34+00:00)
    );
    assert_eq!(
        DateTime::from_iso_format("1997-07-16T19:20+01:00")?,
        datetime!(1997-7-16 19:20:00+01:00)
    );
    assert_eq!(
        DateTime::from_iso_format("2007-01-01T08:00:00")?,
        datetime!(2007-1-1 08:00:00+00:00)
    );
    assert_eq!(
        DateTime::from_iso_format("2006-10-20T15:34:56.123+02:30")?,
        datetime!(2006-10-20 15:34:56+02:30).with_microsecond(123000).unwrap()
    );
    assert_eq!(
        DateTime::from_iso_format("2006-10-20T15:34:56Z")?,
        datetime!(2006-10-20 15:34:56+00:00)
    );
    assert_eq!(
        DateTime::from_iso_format("2007-05-07T11:43:55.328Z")?,
        datetime!(2007-5-7 11:43:55+00:00).with_microsecond(328000).unwrap()
    );
    assert_eq!(
        DateTime::from_iso_format("2006-10-20T15:34:56.123Z")?,
        datetime!(2006-10-20 15:34:56+00:00).with_microsecond(123000).unwrap()
    );
    assert_eq!(
        DateTime::from_iso_format("2013-10-15T18:30Z")?,
        datetime!(2013-10-15 18:30:00+00:00)
    );
    assert_eq!(
        DateTime::from_iso_format("2013-10-15T22:30+04")?,
        datetime!(2013-10-15 22:30:00+04:00)
    );
    assert_eq!(
        DateTime::from_iso_format("2013-10-15T11:30-07:00")?,
        datetime!(2013-10-15 11:30:00-07:00)
    );
    assert_eq!(
        DateTime::from_iso_format("2013-10-15T11:30+07:00")?,
        datetime!(2013-10-15 11:30:00+07:00)
    );
    assert_eq!(
        DateTime::from_iso_format("2013-10-15T11:30+07")?,
        datetime!(2013-10-15 11:30:00+07:00)
    );
    assert_eq!(
        DateTime::from_iso_format("2013-10-15T11:30-07")?,
        datetime!(2013-10-15 11:30:00-07:00)
    );
    assert_eq!(
        DateTime::from_iso_format("2013-10-15T15:00-03:30")?,
        datetime!(2013-10-15 15:00:00-03:30)
    );
    assert_eq!(
        DateTime::from_iso_format("2013-10-15T18:31:23Z")?,
        datetime!(2013-10-15 18:31:23+00:00)
    );
    assert_eq!(
        DateTime::from_iso_format("2013-10-15T18:31Z")?,
        datetime!(2013-10-15 18:31:00+00:00)
    );
    assert_eq!(
        DateTime::from_iso_format("2013-10-15T18:00Z")?,
        datetime!(2013-10-15 18:00:00+00:00)
    );
    assert_eq!(
        DateTime::from_iso_format("2013-10-15T18:30Z")?,
        datetime!(2013-10-15 18:30:00+00:00)
    );
    assert_eq!(
        DateTime::from_iso_format("2012-12-19T23:21:28.512400+00:00")?,
        datetime!(2012-12-19 23:21:28+00:00).with_microsecond(512400).unwrap()
    );
    assert_eq!(
        DateTime::from_iso_format("2006-10-20T15:34:56.123+02:30")?,
        datetime!(2006-10-20 15:34:56+02:30).with_microsecond(123000).unwrap()
    );
    assert_eq!(
        DateTime::from_iso_format("2010-07-20T15:25:52.520701+00:00")?,
        datetime!(2010-7-20 15:25:52+00:00).with_microsecond(520701).unwrap()
    );
    assert_eq!(
        DateTime::from_iso_format("1985-04-12T23:20:50.52-05:30")?,
        datetime!(1985-4-12 23:20:50-05:30).with_microsecond(520000).unwrap()
    );
    assert_eq!(
        DateTime::from_iso_format("1997-08-29T06:14:00.000123Z")?,
        datetime!(1997-8-29 6:14:00+00:00).with_microsecond(123).unwrap()
    );
    assert_eq!(
        DateTime::from_iso_format("1997-08-29T06:14:00,000123Z")?,
        datetime!(1997-8-29 6:14:00+00:00).with_microsecond(123).unwrap()
    );
    assert_eq!(
        DateTime::from_iso_format("2009-12T12:34")?,
        datetime!(2009-12-1 12:34+00:00)
    );
    assert_eq!(
        DateTime::from_iso_format("2009-W21-2T01:22")?,
        datetime!(2009-5-19 1:22+00:00)
    );
    assert_eq!(
        DateTime::from_iso_format("2009-139T00:00+01:00")?,
        datetime!(2009-5-19 00:00+01:00)
    );
    Ok(())
}

#[test]
fn test_isoformat_roundtrip() {
    let dates = [
        date!(0001 - 01 - 01),
        date!(1990 - 01 - 01),
        date!(2005 - 11 - 12),
        date!(2012 - 02 - 29),
        date!(2022 - 01 - 25),
    ];

    let times = [
        time!(00:00:00),
        time!(05:06:07),
        time!(05:06:07).with_microsecond(123456).unwrap(),
        time!(12:30:40),
        time!(12:34:56).with_microsecond(789101).unwrap(),
    ];

    let offsets = [
        utc_offset!(00:00),
        utc_offset!(-05:00),
        utc_offset!(04:00),
        utc_offset!(12:45),
        utc_offset!(-10:00),
    ];

    for date in dates {
        let s = date.to_iso_format();
        assert_eq!(Date::from_iso_format(&s).unwrap(), date);
        for time in times {
            let s = time.to_iso_format();
            assert_eq!(Time::from_iso_format(&s).unwrap(), time);
            for offset in offsets {
                let dt = date.at(time).with_timezone(offset);
                let out = dt.to_iso_format().to_string();
                assert_eq!(DateTime::from_iso_format(&out).unwrap(), dt);
            }
        }
    }
}

#[test]
fn test_valid_interval() -> Result<(), eos::fmt::ParseError> {
    assert_eq!(Interval::from_iso_format("PT15M")?, 15.minutes());
    assert_eq!(
        Interval::from_iso_format("PT20.5S")?,
        (20.seconds() + 500.milliseconds())
    );
    assert_eq!(
        Interval::from_iso_format("P10Y2M3DT10S")?,
        (10.years() + 2.months() + 3.days() + 10.seconds())
    );
    assert_eq!(Interval::from_iso_format("-P30D")?, (-30).days());
    assert_eq!(Interval::from_iso_format("P-30D")?, (-30).days());
    assert_eq!(Interval::from_iso_format("-P-30DT30S")?, (30.days() - 30.seconds()));
    Ok(())
}

#[test]
fn test_valid_duration() -> Result<(), eos::fmt::ParseError> {
    assert_eq!(Duration::from_iso_format("PT15M")?, Duration::from_secs(15 * 60));
    assert_eq!(Duration::from_iso_format("PT20.5S")?, Duration::from_secs_f64(20.5));
    assert_eq!(Duration::from_iso_format("PT10H")?, Duration::from_secs(10 * 3600));
    assert_eq!(
        Duration::from_iso_format("PT6H30M20.5S")?,
        Duration::from_secs(6 * 3600 + 30 * 60 + 20).saturating_add(Duration::from_millis(500))
    );
    Ok(())
}
