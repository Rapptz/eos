use eos::{
    date, datetime,
    fmt::{FormatSpec, FormatSpecKind},
    format_dt, time, Utc,
};
#[test]
fn parse_invalid_format_spec() {
    assert!(eos::fmt::parse_spec("%d%").is_err());
    // %L does not exist
    assert!(eos::fmt::parse_spec("hello %L world").is_err());
    assert!(eos::fmt::parse_spec("this is incomplete%_").is_err());
    assert!(eos::fmt::parse_spec("this is also incomplete %#").is_err());
    assert!(eos::fmt::parse_spec("this is invalid %#L").is_err());
}

#[test]
fn parse_valid_format_spec() {
    assert_eq!(
        eos::fmt::parse_spec("hello world"),
        Ok(vec![FormatSpec::raw("hello world")])
    );
    assert_eq!(
        eos::fmt::parse_spec("%Y-%m-%d"),
        Ok(vec![
            FormatSpec::new(FormatSpecKind::Year),
            FormatSpec::raw("-"),
            FormatSpec::new(FormatSpecKind::Month),
            FormatSpec::raw("-"),
            FormatSpec::new(FormatSpecKind::Day),
        ])
    );
    assert_eq!(
        eos::fmt::parse_spec("%Y-%#m-%#d"),
        Ok(vec![
            FormatSpec::new(FormatSpecKind::Year),
            FormatSpec::raw("-"),
            FormatSpec::new(FormatSpecKind::Month).with_no_padding(),
            FormatSpec::raw("-"),
            FormatSpec::new(FormatSpecKind::Day).with_no_padding(),
        ])
    );
}

#[test]
fn test_date_format() {
    assert_eq!(format_dt!("%Y-%m-%d", date!(2021 - 02 - 28)).to_string(), "2021-02-28");
    assert_eq!(
        format_dt!("Hello, today is %Y-%m-%d!", date!(2021 - 02 - 28)).to_string(),
        "Hello, today is 2021-02-28!"
    );
}

#[test]
fn test_time_format() {
    assert_eq!(format_dt!("%H:%M:%S", time!(12:23:45)).to_string(), "12:23:45");
    assert_eq!(format_dt!("%I:%M:%S %p", time!(13:45:59)).to_string(), "01:45:59 PM");
    assert_eq!(format_dt!("%#I:%M:%S %p", time!(13:45:59)).to_string(), "1:45:59 PM");
}

#[test]
fn test_datetime_format() {
    let dt = datetime!(2022-01-23 18:20:30 -05:00);
    let utc = dt.with_timezone(Utc);

    assert_eq!(format_dt!("%Y-%m-%d", dt).to_string(), "2022-01-23");
    assert_eq!(
        format_dt!("%Y-%m-%d %H:%M:%S %Z", dt).to_string(),
        "2022-01-23 18:20:30 "
    );
    assert_eq!(
        format_dt!("%Y-%m-%d %H:%M:%S %Z", utc).to_string(),
        "2022-01-23 18:20:30 UTC"
    );
    assert_eq!(
        format_dt!("%a %b %d, %Y %#I:%M:%S %p UTC%o", dt).to_string(),
        "Sun Jan 23, 2022 6:20:30 PM UTC-05:00"
    );
    assert_eq!(format_dt!("%A %Y-%j", dt).to_string(), "Sunday 2022-023");
    assert_eq!(format_dt!("%G-W%V-%u", dt).to_string(), "2022-W03-7");
}
