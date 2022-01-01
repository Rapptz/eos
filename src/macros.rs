#[doc(hidden)]
#[macro_export]
macro_rules! const_assert {
    ($cond:expr) => {
        $crate::const_assert!($cond, concat!("compile time assertion failed: ", stringify!($cond)));
    };
    ($cond:expr, $($t:tt)+) => {
        #[forbid(const_err)]
        const _: () = {
            if !$cond {
                core::panic!($($t)+)
            }
        };
    };
}

#[doc(hidden)]
pub use const_assert;

#[doc(hidden)]
#[macro_export]
#[rustfmt::skip]
#[cfg(feature = "macros")]
macro_rules! __meridian_parser {
    (am) => { true };
    (AM) => { true };
    (Am) => { true };
    (pm) => { false };
    (PM) => { false };
    (Pm) => { false };
    ($t:tt+) => {{
        core::panic!("meridian must be one of `am`, `AM`, `Am`, `pm`, `PM`, or `Pm`")
    }};
}

#[doc(hidden)]
#[cfg(feature = "macros")]
pub use __meridian_parser;

#[doc(hidden)]
#[macro_export]
#[rustfmt::skip]
#[cfg(feature = "macros")]
macro_rules! __expand_or_zero {
    ($l:literal) => { $l };
    () => { 0 };
}

#[doc(hidden)]
#[cfg(feature = "macros")]
pub use __expand_or_zero;

/// Creates a [`Time`] with compile-time validation and values.
///
/// The resulting type can be used in both `static` and `const` contexts.
/// All units passed are validated at compile-time. A compile time
/// error will trigger if any of the units are invalid.
///
/// Currently there is no way to denote sub-second precision.
///
/// The syntax supported is `HH:MM:SS (am|pm)` with the AM/PM component
/// and the seconds components being optional.
///
/// # Examples
///
/// ```rust
/// use eos::{time, Time};
/// assert_eq!(time!(12:00), Time::new(12, 0, 0)?);
/// assert_eq!(time!(12:23:05), Time::new(12, 23, 05)?);
///
/// // AM and PM are supported too
/// assert_eq!(time!(12:00 am), Time::new(0, 0, 0)?);
/// assert_eq!(time!(1:12:23 pm), Time::new(13, 12, 23)?);
/// # Ok::<_, eos::Error>(())
/// ```
///
/// [`Time`]: crate::Time
#[macro_export]
#[cfg(feature = "macros")]
macro_rules! time {
    ($hours:literal:$minutes:literal$(:$seconds:literal)?) => {{
        const HOURS: u8 = $hours;
        const MINUTES: u8 = $minutes;
        const SECONDS: u8 = $crate::macros::__expand_or_zero!($($seconds)?);
        $crate::macros::const_assert!(HOURS <= 23, "hours must be less than 24");
        $crate::macros::const_assert!(MINUTES <= 59, "minutes must be less than 60");
        $crate::macros::const_assert!(SECONDS <= 59, "seconds must be less than 60");
        $crate::Time::__new_unchecked_from_macro(HOURS, MINUTES, SECONDS)
    }};

    ($hours:literal:$minutes:literal$(:$seconds:literal)? $meridian:ident) => {{
        const HOURS: u8 = $hours;
        const MINUTES: u8 = $minutes;
        const SECONDS: u8 = $crate::macros::__expand_or_zero!($($seconds)?);
        $crate::macros::const_assert!(HOURS <= 12, "hours must be less than 13");
        $crate::macros::const_assert!(MINUTES <= 59, "minutes must be less than 60");
        $crate::macros::const_assert!(SECONDS <= 59, "seconds must be less than 60");
        const MERIDIAN: bool = $crate::macros::__meridian_parser!($meridian);
        if MERIDIAN {
            $crate::Time::__new_unchecked_from_macro(if HOURS == 12 { 0 } else { HOURS }, MINUTES, SECONDS)
        } else {
            $crate::Time::__new_unchecked_from_macro(if HOURS == 12 { 12 } else { HOURS + 12 }, MINUTES, SECONDS)
        }
    }};
}

/// Creates a [`Date`] with compile-time validation and values.
///
/// The resulting type can be used in both `static` and `const` contexts.
/// All units passed are validated at compile-time. A compile time
/// error will trigger if any of the units are invalid.
///
/// The syntax supported is `YYYY-MM-DD`.
///
/// # Examples
///
/// ```rust
/// use eos::{date, Date};
/// assert_eq!(date!(2012-2-29), Date::new(2012, 2, 29)?);
/// assert_eq!(date!(2000-01-25), Date::new(2000, 1, 25)?);
/// # Ok::<_, eos::Error>(())
/// ```
///
/// [`Date`]: crate::Date
#[macro_export]
#[cfg(feature = "macros")]
macro_rules! date {
    ($year:literal-$month:literal-$day:literal) => {{
        const YEAR: i16 = $year;
        const MONTH: u8 = $month;
        const DAY: u8 = $day;
        $crate::macros::const_assert!(MONTH >= 1 && MONTH <= 12, "months must be between [1, 12]");

        $crate::macros::const_assert!(
            DAY >= 1 && DAY <= $crate::utils::days_in_month(YEAR, MONTH),
            "day must be positive and within range of the month"
        );

        $crate::Date::__new_unchecked_from_macro(YEAR, MONTH, DAY)
    }};
}

/// Creates a [`UtcOffset`] with compile-time validation and values.
///
/// The resulting type can be used in both `static` and `const` contexts.
/// All units passed are validated at compile-time. A compile time
/// error will trigger if any of the units are invalid.
///
/// The syntax supported is `(+|-)HH:MM:SS` with the seconds, minutes, and +/- being optional.
/// If the sign is not given then it's assumed to be positive.
///
/// # Examples
///
/// ```rust
/// use eos::{utc_offset, UtcOffset};
/// assert_eq!(utc_offset!(5), UtcOffset::from_hms(5, 0, 0)?);
/// assert_eq!(utc_offset!(-5), UtcOffset::from_hms(-5, 0, 0)?);
/// assert_eq!(utc_offset!(-5:30), UtcOffset::from_hms(-5, -30, 0)?);
/// assert_eq!(utc_offset!(05:30), UtcOffset::from_hms(5, 30, 0)?);
/// assert_eq!(utc_offset!(01:02:03), UtcOffset::from_hms(1, 2, 3)?);
/// assert_eq!(utc_offset!(-01:02:03), UtcOffset::from_hms(-1, -2, -3)?);
/// # Ok::<_, eos::Error>(())
/// ```
///
/// [`UtcOffset`]: crate::UtcOffset
#[macro_export]
#[cfg(feature = "macros")]
macro_rules! utc_offset {
    ($(+)?$hours:literal$(:$minutes:literal$(:$seconds:literal)?)?) => {{
        const HOURS: i8 = $hours;
        const MINUTES: i8 = $crate::macros::__expand_or_zero!($($minutes)?);
        const SECONDS: i8 = $crate::macros::__expand_or_zero!($($($seconds)?)?);

        $crate::macros::const_assert!(HOURS <= 23 && HOURS >= -23, "hours must be between [-23, 23]");
        $crate::macros::const_assert!(MINUTES <= 59 && MINUTES >= 0, "minutes must be between [0, 59]");
        $crate::macros::const_assert!(SECONDS <= 59 && SECONDS >= 0, "seconds must be between [0, 59]");

        if HOURS >= 0 {
            $crate::UtcOffset::__new_unchecked_from_macro(HOURS, MINUTES, SECONDS)
        } else {
            $crate::UtcOffset::__new_unchecked_from_macro(HOURS, -MINUTES, -SECONDS)
        }
    }};
}

/// Creates a [`DateTime`] with compile-time validation and values.
///
/// The resulting type can be used in both `static` and `const` contexts.
/// All units passed are validated at compile-time. A compile time
/// error will trigger if any of the units are invalid.
///
/// The resulting [`DateTime`] will either be in [`Utc`] or with a [`UtcOffset`]
/// if one is provided in the syntax.
///
/// The syntax supported is a combination of the [`date`] and [`time`] macros with
/// an optional [`utc_offset`] component. For completeness, the syntax is:
///
/// ```ignore
/// YYYY-MM-DD HH:MM(:SS)? (am|pm)? (((+|-)?HH:MM(:SS)?))?
/// ```
///
/// Due to a limitation with suffixes on literals, these components must be space separated
/// to work.
///
/// With the time seconds and AM/PM being optional and the entire offset component
/// being optional. Unlike [`Datetime::new`], the macro requires a time to be passed.
///
/// # Examples
///
/// ```rust
/// use eos::{datetime, DateTime, Time, Utc, UtcOffset};
/// assert_eq!(
///     datetime!(2012-02-29 2:00),
///     DateTime::<Utc>::new(2012, 2, 29)?.with_time(Time::new(2, 0, 0)?)
/// );
/// assert_eq!(
///     datetime!(2001-02-18 3:12 pm),
///     DateTime::<Utc>::new(2001, 2, 18)?.with_time(Time::new(15, 12, 0)?)
/// );
/// assert_eq!(
///     datetime!(2001-02-18 3:12:32 am),
///     DateTime::<Utc>::new(2001, 2, 18)?.with_time(Time::new(3, 12, 32)?)
/// );
/// assert_eq!(
///     datetime!(2001-02-18 3:12:32 am),
///     DateTime::<Utc>::new(2001, 2, 18)?.with_time(Time::new(3, 12, 32)?)
/// );
/// let with_offset = datetime!(2001-02-18 20:12 5:00);
/// assert_eq!(
///     with_offset,
///     DateTime::<Utc>::new(2001, 2, 18)?
///       .with_time(Time::new(20, 12, 0)?)
///       .with_timezone(UtcOffset::from_hms(5, 0, 0)?)
/// );
/// assert_eq!(with_offset.timezone(), &UtcOffset::from_hms(5, 0, 0)?);
/// let with_neg_offset = datetime!(2001-02-18 20:12 -6:30);
/// assert_eq!(
///     with_neg_offset,
///     DateTime::<Utc>::new(2001, 2, 18)?
///       .with_time(Time::new(20, 12, 0)?)
///       .with_timezone(UtcOffset::from_hms(-6, -30, 0)?)
/// );
/// # Ok::<_, eos::Error>(())
/// ```
///
/// [`DateTime`]: crate::DateTime
/// [`DateTime::new]: crate::DateTime::new
/// [`UtcOffset`]: crate::UtcOffset
/// [`Utc`]: crate::Utc
#[macro_export]
#[cfg(feature = "macros")]
macro_rules! datetime {
    (
        $year:tt-$month:tt-$day:tt
        $hours:tt:$minutes:tt$(:$seconds:tt)? $($meridian:ident)?
        $(+)?$off_hours:literal$(:$off_minutes:literal$(:$off_seconds:literal)?)?
    ) => {{
        const DATE: $crate::Date = $crate::date!($year-$month-$day);
        const TIME: $crate::Time = $crate::time!($hours:$minutes$(:$seconds)? $($meridian)?);
        const OFFSET: $crate::UtcOffset = $crate::utc_offset!($off_hours$(:$off_minutes$(:$off_seconds)?)?);
        $crate::__create_offset_datetime_from_macro(DATE, TIME, OFFSET)
    }};

    (
        $year:tt-$month:tt-$day:tt
        $hours:tt:$minutes:tt$(:$seconds:tt)? $($meridian:ident)?
    ) => {{
        const DATE: $crate::Date = $crate::date!($year-$month-$day);
        const TIME: $crate::Time = $crate::time!($hours:$minutes$(:$seconds)? $($meridian)?);
        $crate::DateTime::__new_utc_unchecked_from_macro(DATE, TIME)
    }};
}
