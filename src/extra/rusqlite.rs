//! Convert most of the [Time Strings](http://sqlite.org/lang_datefunc.html) to our types.

use crate::{
    fmt::{FromIsoFormat, ParseError, Parser, ToIsoFormat},
    Date, DateTime, Time, Utc, UtcOffset,
};
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

/// This is basically RFC3339 except the offset is optional. SQLite internally
/// stores all datetimes as UTC time, so an omitted offset is equivalent to UTC
/// time.
///
/// Note this is meant for the DateTime parsing procedures
fn parse_sqlite3_format(s: &str) -> Result<DateTime<UtcOffset>, ParseError> {
    let mut parser = Parser::new(s);
    let year = parser.parse_year()?;
    parser.expect(b'-')?;
    let month = parser.parse_month()?;
    parser.expect(b'-')?;
    let day = parser.parse_two_digits()?;
    let date = Date::new(year, month, day).ok_or(ParseError::OutOfBounds)?;
    match parser.advance() {
        Some(b' ' | b'T') => {}
        Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
        None => return Err(ParseError::UnexpectedEnd),
    }
    let time = parser.parse_time()?;
    let offset = match parser.advance() {
        Some(b'Z') => UtcOffset::UTC,
        Some(x @ b'+' | x @ b'-') => {
            let negative = x == b'-';
            let hours = parser.parse_two_digits()? as i8;
            parser.expect(b':')?;
            let minutes = parser.parse_two_digits()? as i8;
            if hours > 23 || minutes > 59 {
                return Err(ParseError::OutOfBounds);
            }
            if negative {
                UtcOffset {
                    hours: -hours,
                    minutes: -minutes,
                    seconds: 0,
                }
            } else {
                UtcOffset {
                    hours,
                    minutes,
                    seconds: 0,
                }
            }
        }
        Some(c) => return Err(ParseError::UnexpectedChar(c as char)),
        None => UtcOffset::UTC,
    };

    Ok(DateTime {
        date,
        time,
        offset,
        timezone: offset,
    })
}

/// Converts to an ISO-8601 calendar date without timezone (i.e. `"YYYY-MM-DD"`)
impl ToSql for Date {
    #[inline]
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_iso_format()))
    }
}

/// Converts from `"YYYY-MM-DD"` (i.e. a ISO-8601 calendar date without timezone).
impl FromSql for Date {
    #[inline]
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| match Self::from_iso_format(s) {
            Ok(dt) => Ok(dt),
            Err(err) => Err(FromSqlError::Other(Box::new(err))),
        })
    }
}

/// Converts to an ISO-8601 time without timezone (i.e. `"HH:MM:SS.SSS"`)
impl ToSql for Time {
    #[inline]
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_iso_format()))
    }
}

/// Converts from an ISO-8601 time without timezone.
impl FromSql for Time {
    #[inline]
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| match Self::from_iso_format(s) {
            Ok(dt) => Ok(dt),
            Err(err) => Err(FromSqlError::Other(Box::new(err))),
        })
    }
}

/// Converts to an RFC3339 timestamp (i.e. `"YYYY-MM-DD HH:MM:SS.SSSSSS+00:00"`).
impl ToSql for DateTime<Utc> {
    #[inline]
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_rfc3339().to_string()))
    }
}

/// Converts from an RFC3339 timestamp (e.g. `"YYYY-MM-DD HH:MM:SS.SSSSSS[+-]HH:MM"`) into `DateTime<Utc>`.
///
/// This also supports formats that place a T between the date and time components.
impl FromSql for DateTime<Utc> {
    #[inline]
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| match parse_sqlite3_format(s) {
            Ok(dt) => Ok(dt.into_utc()),
            Err(err) => Err(FromSqlError::Other(Box::new(err))),
        })
    }
}

/// Converts to an RFC3339 timestamp with timezone (e.g. `"YYYY-MM-DD HH:MM:SS.SSSSSS[+-]HH:MM"`).
impl ToSql for DateTime<UtcOffset> {
    #[inline]
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_rfc3339().to_string()))
    }
}

/// Converts from an RFC3339 timestamp (e.g. `"YYYY-MM-DD HH:MM:SS.SSSSSS[+-]HH:MM"`) into `DateTime<UtcOffset>`.
///
/// This also supports formats that place a T between the date and time components.
impl FromSql for DateTime<UtcOffset> {
    #[inline]
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| match parse_sqlite3_format(s) {
            Ok(dt) => Ok(dt),
            Err(err) => Err(FromSqlError::Other(Box::new(err))),
        })
    }
}

#[cfg(test)]
mod test {
    // Most of these tests are adapted from rusqlite directly
    use rusqlite::{
        types::{FromSql, ValueRef},
        Connection, Result,
    };

    use crate::{date, datetime, time, Date, DateTime, Interval, Time, Utc, UtcOffset};

    fn checked_memory_handle() -> Result<Connection> {
        let db = Connection::open_in_memory()?;
        db.execute_batch("CREATE TABLE foo (t TEXT, i INTEGER, f FLOAT, b BLOB)")?;
        Ok(db)
    }

    #[test]
    fn test_date() -> Result<()> {
        let db = checked_memory_handle()?;
        let date = date!(2022 - 06 - 21);
        db.execute("INSERT INTO foo (t) VALUES (?)", [date])?;

        let s: String = db.query_row("SELECT t FROM foo", [], |r| r.get(0))?;
        assert_eq!("2022-06-21", s);
        let t: Date = db.query_row("SELECT t FROM foo", [], |r| r.get(0))?;
        assert_eq!(date, t);
        Ok(())
    }

    #[test]
    fn test_time() -> Result<()> {
        let db = checked_memory_handle()?;
        let time = time!(23:56:04);
        db.execute("INSERT INTO foo (t) VALUES (?)", [time])?;

        let s: String = db.query_row("SELECT t FROM foo", [], |r| r.get(0))?;
        assert_eq!("23:56:04", s);
        let v: Time = db.query_row("SELECT t FROM foo", [], |r| r.get(0))?;
        assert_eq!(time, v);
        Ok(())
    }

    #[test]
    fn test_datetime_utc() -> Result<()> {
        let db = checked_memory_handle()?;
        let dt = datetime!(2022-06-21 11:52:04);
        db.execute("INSERT INTO foo (t) VALUES (?)", [dt])?;

        let s: String = db.query_row("SELECT t FROM foo", [], |r| r.get(0))?;
        assert_eq!("2022-06-21 11:52:04+00:00", s);
        let v: DateTime = db.query_row("SELECT t FROM foo", [], |r| r.get(0))?;
        assert_eq!(dt, v);

        db.execute("UPDATE foo set b = datetime(t)", [])?; // "YYYY-MM-DD HH:MM:SS"
        let s: String = db.query_row("SELECT b from foo", [], |r| r.get(0))?;
        // SQLite operates on UTC time implicitly so Z and +00:00 are stripped out
        assert_eq!("2022-06-21 11:52:04", s);

        Ok(())
    }

    #[test]
    fn test_datetime_utc_precision() -> Result<()> {
        let db = checked_memory_handle()?;
        let dt = datetime!(2022-06-21 09:34:01).with_millisecond(789).unwrap();

        db.execute("INSERT INTO foo (t) VALUES (?)", [dt])?;

        let s: String = db.query_row("SELECT t FROM foo", [], |r| r.get(0))?;
        assert_eq!("2022-06-21 09:34:01.789000+00:00", s);

        let v1: DateTime<Utc> = db.query_row("SELECT t FROM foo", [], |r| r.get(0))?;
        assert_eq!(dt, v1);

        let v2: DateTime<Utc> = db.query_row("SELECT '2022-06-21 09:34:01.789000+00:00'", [], |r| r.get(0))?;
        assert_eq!(dt, v2);

        let v3: DateTime<Utc> = db.query_row("SELECT '2022-06-21T09:34:01Z'", [], |r| r.get(0))?;
        assert_eq!(dt - Interval::from_milliseconds(789), v3);

        let v4: DateTime<Utc> = db.query_row("SELECT '2022-06-21 09:34:01.789000+00:00'", [], |r| r.get(0))?;
        assert_eq!(dt, v4);
        Ok(())
    }

    #[test]
    fn test_datetime_fixed() -> Result<()> {
        let db = checked_memory_handle()?;
        let time = DateTime::from_rfc3339("2022-06-21 09:34:01.789000-05:00").unwrap();

        db.execute("INSERT INTO foo (t) VALUES (?)", [time])?;

        // Stored string should preserve timezone offset
        let s: String = db.query_row("SELECT t FROM foo", [], |r| r.get(0))?;
        assert!(s.ends_with("-05:00"));

        let v: DateTime<UtcOffset> = db.query_row("SELECT t FROM foo", [], |r| r.get(0))?;
        assert_eq!(time.offset(), v.offset());
        assert_eq!(time, v);
        Ok(())
    }

    #[test]
    fn test_sqlite_functions() -> Result<()> {
        let db = checked_memory_handle()?;
        let result: Result<Time> = db.query_row("SELECT CURRENT_TIME", [], |r| r.get(0));
        assert!(result.is_ok());
        let result: Result<Date> = db.query_row("SELECT CURRENT_DATE", [], |r| r.get(0));
        assert!(result.is_ok());
        let result: Result<DateTime<Utc>> = db.query_row("SELECT CURRENT_TIMESTAMP", [], |r| r.get(0));
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_date_time_param() -> Result<()> {
        let db = checked_memory_handle()?;
        let result: Result<bool> = db.query_row(
            "SELECT 1 WHERE ? BETWEEN datetime('now', '-1 minute') AND datetime('now', '+1 minute')",
            [Utc::now()],
            |r| r.get(0),
        );
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_lenient_parse_timezone() {
        assert!(DateTime::<Utc>::column_result(ValueRef::Text(b"1970-01-01T00:00:00Z")).is_ok());
        assert!(DateTime::<Utc>::column_result(ValueRef::Text(b"1970-01-01T00:00:00+00:00")).is_ok());
    }
}
