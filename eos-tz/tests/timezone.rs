#![cfg(feature = "bundled")]

// Tests shamelessly and gracefully adapted from Python's zoneinfo package.
// Due to how tests need to be "frozen" in time, a lot of these are snap shots of pre-existing
// yet valid data.

// This was last updated for 2021e

use eos::{datetime, ext::IntervalLiteral, utc_offset, DateTime, TimeZone, Utc, UtcOffset};
use eos_tz::zone;

const TEST_DATA: [(&str, &[u8]); 12] = [
    ("Africa/Abidjan", include_bytes!("Africa/Abidjan")),
    ("Africa/Casablanca", include_bytes!("Africa/Casablanca")),
    ("America/Los_Angeles", include_bytes!("America/Los_Angeles")),
    ("America/Santiago", include_bytes!("America/Santiago")),
    ("Asia/Tokyo", include_bytes!("Asia/Tokyo")),
    ("Australia/Sydney", include_bytes!("Australia/Sydney")),
    ("Europe/Dublin", include_bytes!("Europe/Dublin")),
    ("Europe/Lisbon", include_bytes!("Europe/Lisbon")),
    ("Europe/London", include_bytes!("Europe/London")),
    ("Europe/Prague", include_bytes!("Europe/Prague")),
    ("Pacific/Kiritimati", include_bytes!("Pacific/Kiritimati")),
    ("UTC", include_bytes!("UTC")),
];

fn get_zone(key: &str) -> eos_tz::TimeZone {
    let idx = TEST_DATA.binary_search_by_key(&key, |x| x.0).expect("not found");
    let (_, bytes) = TEST_DATA.get(idx).expect("out of bounds");
    let cursor = std::io::Cursor::new(bytes);
    eos_tz::TimeZone::load(cursor, key.to_owned()).expect("parsing failed")
}

macro_rules! trace_variables {
    ($($name:ident,)* $block:block) => {
        let variables = [
            $((stringify!($name), &$name as &dyn ::std::fmt::Debug),)*
        ];
        if let Err(e) = std::panic::catch_unwind(|| $block) {
            eprintln!("----- TRACED ARGUMENTS -----");
            for (name, item) in variables {
                eprintln!("{} = {:#?}", name, item);
            }
            std::panic::resume_unwind(e);
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct NamedOffset {
    name: &'static str,
    offset: UtcOffset,
    dst: UtcOffset,
}

impl NamedOffset {
    const fn new(name: &'static str, offset: UtcOffset) -> Self {
        Self {
            name,
            offset,
            dst: UtcOffset::UTC,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ZoneTransition {
    transition: DateTime<Utc>,
    offset_before: NamedOffset,
    offset_after: NamedOffset,
}

impl ZoneTransition {
    const fn new(transition: DateTime<Utc>, offset_before: NamedOffset, offset_after: NamedOffset) -> Self {
        Self {
            transition,
            offset_before,
            offset_after,
        }
    }

    fn is_fold(&self) -> bool {
        self.offset_before.offset > self.offset_after.offset
    }

    fn is_gap(&self) -> bool {
        self.offset_before.offset < self.offset_after.offset
    }

    fn delta_offset(&self) -> UtcOffset {
        self.offset_after.offset.saturating_sub(self.offset_before.offset)
    }

    fn anomaly_start(&self) -> DateTime<Utc> {
        if self.is_fold() {
            let mut copy = self.transition;
            copy.shift(self.delta_offset());
            copy
        } else {
            self.transition
        }
    }

    fn anomaly_end(&self) -> DateTime<Utc> {
        if !self.is_fold() {
            let mut copy = self.transition;
            copy.shift(self.delta_offset());
            copy
        } else {
            self.transition
        }
    }
}

const ONE_HOUR: UtcOffset = utc_offset!(+01:00);

fn get_zonedump_data() -> [(&'static str, Vec<ZoneTransition>); 10] {
    #![allow(non_snake_case)]

    fn africa_abidjan() -> Vec<ZoneTransition> {
        let LMT = NamedOffset::new("LMT", utc_offset!(-00:16:08));
        let GMT = NamedOffset::new("GMT", UtcOffset::UTC);

        vec![ZoneTransition {
            transition: datetime!(1912-01-01 00:00),
            offset_before: LMT,
            offset_after: GMT,
        }]
    }

    fn africa_casablanca() -> Vec<ZoneTransition> {
        let p00_std = NamedOffset::new("+00", UtcOffset::UTC);
        let p01_dst = NamedOffset {
            name: "+01",
            offset: ONE_HOUR,
            dst: ONE_HOUR,
        };
        let p00_dst = NamedOffset {
            name: "+00",
            offset: UtcOffset::UTC,
            dst: -ONE_HOUR,
        };
        let p01_std = NamedOffset::new("+01", ONE_HOUR);
        vec![
            // Morocco sometimes pauses DST during Ramadan
            ZoneTransition::new(datetime!(2018-3-25 2:00), p00_std, p01_dst),
            ZoneTransition::new(datetime!(2018-5-13 3:00), p01_dst, p00_std),
            ZoneTransition::new(datetime!(2018-6-17 2:00), p00_std, p01_dst),
            // On October 28th Morocco set standard time to +01,
            // with negative DST only during Ramadan
            ZoneTransition::new(datetime!(2018-10-28 3:00), p01_dst, p01_std),
            ZoneTransition::new(datetime!(2019-5-5 3:00), p01_std, p00_dst),
            ZoneTransition::new(datetime!(2019-6-9 2:00), p00_dst, p01_std),
        ]
    }

    fn america_los_angeles() -> Vec<ZoneTransition> {
        let LMT = NamedOffset {
            name: "LMT",
            offset: utc_offset!(-07:52:58),
            dst: UtcOffset::UTC,
        };
        let PST = NamedOffset {
            name: "PST",
            offset: utc_offset!(-08:00),
            dst: UtcOffset::UTC,
        };
        let PDT = NamedOffset {
            name: "PDT",
            offset: utc_offset!(-07:00),
            dst: ONE_HOUR,
        };
        let PWT = NamedOffset {
            name: "PWT",
            offset: utc_offset!(-07:00),
            dst: ONE_HOUR,
        };
        let PPT = NamedOffset {
            name: "PPT",
            offset: utc_offset!(-07:00),
            dst: ONE_HOUR,
        };

        vec![
            ZoneTransition::new(datetime!(1883-11-18 12:07:02), LMT, PST),
            ZoneTransition::new(datetime!(1918-3-31 2:00:00), PST, PDT),
            ZoneTransition::new(datetime!(1918-3-31 2:00:00), PST, PDT),
            ZoneTransition::new(datetime!(1918-10-27 2:00:00), PDT, PST),
            // Transition to Pacific War Time
            ZoneTransition::new(datetime!(1942-2-9 2:00:00), PST, PWT),
            // Transition from Pacific War Time to Pacific Peace Time
            ZoneTransition::new(datetime!(1945-8-14 16:00:00), PWT, PPT),
            ZoneTransition::new(datetime!(1945-9-30 2:00:00), PPT, PST),
            ZoneTransition::new(datetime!(2015-3-8 2:00:00), PST, PDT),
            ZoneTransition::new(datetime!(2015-11-1 2:00:00), PDT, PST),
            // After 2038: Rules continue indefinitely
            ZoneTransition::new(datetime!(2450-3-13 2:00:00), PST, PDT),
            ZoneTransition::new(datetime!(2450-11-6 2:00:00), PDT, PST),
        ]
    }

    fn america_santiago() -> Vec<ZoneTransition> {
        let LMT = NamedOffset {
            name: "LMT",
            offset: utc_offset!(-04:42:46),
            dst: UtcOffset::UTC,
        };
        let SMT = NamedOffset {
            name: "SMT",
            offset: utc_offset!(-04:42:46),
            dst: UtcOffset::UTC,
        };
        let N05 = NamedOffset {
            name: "-05",
            offset: utc_offset!(-05:00:00),
            dst: UtcOffset::UTC,
        };
        let N04 = NamedOffset {
            name: "-04",
            offset: utc_offset!(-04:00:00),
            dst: UtcOffset::UTC,
        };
        let N03 = NamedOffset {
            name: "-03",
            offset: utc_offset!(-03:00:00),
            dst: ONE_HOUR,
        };

        vec![
            ZoneTransition::new(datetime!(1890-01-01 00:00), LMT, SMT),
            ZoneTransition::new(datetime!(1910-01-10 00:00), SMT, N05),
            ZoneTransition::new(datetime!(1916-07-01 00:00), N05, SMT),
            ZoneTransition::new(datetime!(2008-03-30 00:00), N03, N04),
            ZoneTransition::new(datetime!(2008-10-12 00:00), N04, N03),
            ZoneTransition::new(datetime!(2040-04-08 00:00), N03, N04),
            ZoneTransition::new(datetime!(2040-09-02 00:00), N04, N03),
        ]
    }

    fn asia_tokyo() -> Vec<ZoneTransition> {
        let JST = NamedOffset {
            name: "JST",
            offset: utc_offset!(09:00),
            dst: UtcOffset::UTC,
        };
        let JDT = NamedOffset {
            name: "JDT",
            offset: utc_offset!(10:00),
            dst: ONE_HOUR,
        };

        // Japan had DST from 1948 to 1951, and it was unusual in that
        // the transition from DST to STD occurred at 25:00, and is
        // denominated as such in the time zone database
        vec![
            ZoneTransition::new(datetime!(1948-05-02 00:00), JST, JDT),
            ZoneTransition::new(datetime!(1948-09-12 01:00), JDT, JST),
            ZoneTransition::new(datetime!(1951-09-09 01:00), JDT, JST),
        ]
    }

    fn australia_sydney() -> Vec<ZoneTransition> {
        let LMT = NamedOffset {
            name: "LMT",
            offset: utc_offset!(10:04:52),
            dst: UtcOffset::UTC,
        };
        let AEST = NamedOffset {
            name: "AEST",
            offset: utc_offset!(10:00:00),
            dst: UtcOffset::UTC,
        };
        let AEDT = NamedOffset {
            name: "AEDT",
            offset: utc_offset!(11:00:00),
            dst: ONE_HOUR,
        };

        vec![
            ZoneTransition::new(datetime!(1895-02-01 00:00), LMT, AEST),
            ZoneTransition::new(datetime!(1917-01-01 02:00), AEST, AEDT),
            ZoneTransition::new(datetime!(1917-03-25 03:00), AEDT, AEST),
            ZoneTransition::new(datetime!(2012-04-01 03:00), AEDT, AEST),
            ZoneTransition::new(datetime!(2012-10-07 02:00), AEST, AEDT),
            ZoneTransition::new(datetime!(2040-04-01 03:00), AEDT, AEST),
            ZoneTransition::new(datetime!(2040-10-07 02:00), AEST, AEDT),
        ]
    }

    fn europe_dublin() -> Vec<ZoneTransition> {
        let LMT = NamedOffset {
            name: "LMT",
            offset: utc_offset!(-00:25:00),
            dst: UtcOffset::UTC,
        };
        let DMT = NamedOffset {
            name: "DMT",
            offset: utc_offset!(-00:25:21),
            dst: UtcOffset::UTC,
        };
        let IST_0 = NamedOffset {
            name: "IST",
            offset: utc_offset!(00:34:39),
            dst: ONE_HOUR,
        };
        let GMT_0 = NamedOffset {
            name: "GMT",
            offset: UtcOffset::UTC,
            dst: UtcOffset::UTC,
        };
        let BST = NamedOffset {
            name: "BST",
            offset: ONE_HOUR,
            dst: ONE_HOUR,
        };
        let GMT_1 = NamedOffset {
            name: "GMT",
            offset: UtcOffset::UTC,
            dst: -ONE_HOUR,
        };
        let IST_1 = NamedOffset {
            name: "IST",
            offset: ONE_HOUR,
            dst: UtcOffset::UTC,
        };

        vec![
            ZoneTransition::new(datetime!(1880-08-02 00:00), LMT, DMT),
            ZoneTransition::new(datetime!(1916-05-21 02:00), DMT, IST_0),
            ZoneTransition::new(datetime!(1916-10-01 03:00), IST_0, GMT_0),
            ZoneTransition::new(datetime!(1917-04-08 02:00), GMT_0, BST),
            ZoneTransition::new(datetime!(2016-03-27 01:00), GMT_1, IST_1),
            ZoneTransition::new(datetime!(2016-10-30 02:00), IST_1, GMT_1),
            ZoneTransition::new(datetime!(2487-03-30 01:00), GMT_1, IST_1),
            ZoneTransition::new(datetime!(2487-10-26 02:00), IST_1, GMT_1),
        ]
    }

    fn europe_lisbon() -> Vec<ZoneTransition> {
        let WET = NamedOffset {
            name: "WET",
            offset: UtcOffset::UTC,
            dst: UtcOffset::UTC,
        };
        let WEST = NamedOffset {
            name: "WEST",
            offset: ONE_HOUR,
            dst: ONE_HOUR,
        };
        let CET = NamedOffset {
            name: "CET",
            offset: ONE_HOUR,
            dst: UtcOffset::UTC,
        };
        let CEST = NamedOffset {
            name: "CEST",
            offset: utc_offset!(02:00),
            dst: ONE_HOUR,
        };

        vec![
            ZoneTransition::new(datetime!(1992-03-29 01:00), WET, WEST),
            ZoneTransition::new(datetime!(1992-09-27 02:00), WEST, CET),
            ZoneTransition::new(datetime!(1993-03-28 02:00), CET, CEST),
            ZoneTransition::new(datetime!(1993-09-26 03:00), CEST, CET),
            ZoneTransition::new(datetime!(1996-03-31 02:00), CET, WEST),
            ZoneTransition::new(datetime!(1996-10-27 02:00), WEST, WET),
        ]
    }

    fn europe_london() -> Vec<ZoneTransition> {
        let LMT = NamedOffset {
            name: "LMT",
            offset: utc_offset!(-00:01:15),
            dst: UtcOffset::UTC,
        };
        let GMT = NamedOffset {
            name: "GMT",
            offset: UtcOffset::UTC,
            dst: UtcOffset::UTC,
        };
        let BST = NamedOffset {
            name: "BST",
            offset: ONE_HOUR,
            dst: ONE_HOUR,
        };

        vec![
            ZoneTransition::new(datetime!(1847-12-01 00:00), LMT, GMT),
            ZoneTransition::new(datetime!(2005-03-27 01:00), GMT, BST),
            ZoneTransition::new(datetime!(2005-10-30 02:00), BST, GMT),
            ZoneTransition::new(datetime!(2043-03-29 01:00), GMT, BST),
            ZoneTransition::new(datetime!(2043-10-25 02:00), BST, GMT),
        ]
    }

    fn pacific_kiritimati() -> Vec<ZoneTransition> {
        let LMT = NamedOffset::new("LMT", utc_offset!(-10:29:20));
        let N1040 = NamedOffset::new("-1040", utc_offset!(-10:40:00));
        let N10 = NamedOffset::new("-10", utc_offset!(-10:00:00));
        let P14 = NamedOffset::new("+14", utc_offset!(+14:00:00));

        vec![
            ZoneTransition::new(datetime!(1901-01-01 00:00), LMT, N1040),
            ZoneTransition::new(datetime!(1979-10-01 00:00), N1040, N10),
            ZoneTransition::new(datetime!(1994-12-31 00:00), N10, P14),
        ]
    }

    [
        ("Africa/Abidjan", africa_abidjan()),
        ("Africa/Casablanca", africa_casablanca()),
        ("America/Los_Angeles", america_los_angeles()),
        ("America/Santiago", america_santiago()),
        ("Australia/Sydney", australia_sydney()),
        ("Asia/Tokyo", asia_tokyo()),
        ("Europe/Dublin", europe_dublin()),
        ("Europe/Lisbon", europe_lisbon()),
        ("Europe/London", europe_london()),
        ("Pacific/Kiritimati", pacific_kiritimati()),
    ]
}

#[test]
fn test_timezone_id_retrieval() {
    // This test also doubles as a parsing test for these time zones.
    assert_eq!(zone!("Africa/Abidjan").id(), "Africa/Abidjan");
    assert_eq!(zone!("Africa/Casablanca").id(), "Africa/Casablanca");
    assert_eq!(zone!("America/Los_Angeles").id(), "America/Los_Angeles");
    assert_eq!(zone!("America/Santiago").id(), "America/Santiago");
    assert_eq!(zone!("Australia/Sydney").id(), "Australia/Sydney");
    assert_eq!(zone!("Asia/Tokyo").id(), "Asia/Tokyo");
    assert_eq!(zone!("Europe/Dublin").id(), "Europe/Dublin");
    assert_eq!(zone!("Europe/Prague").id(), "Europe/Prague");
    assert_eq!(zone!("Europe/Lisbon").id(), "Europe/Lisbon");
    assert_eq!(zone!("Europe/London").id(), "Europe/London");
    assert_eq!(zone!("Pacific/Kiritimati").id(), "Pacific/Kiritimati");
}

#[test]
fn test_utc() {
    let utc = zone!("UTC");
    let dt = datetime!(2022-01-29 10:30);

    assert_eq!(utc.name(dt.timestamp()), Some("UTC"));
    assert_eq!(utc.offset(dt.timestamp()), eos::UtcOffset::UTC);
    assert!(utc.is_fixed());
}

#[test]
fn test_unambiguous() {
    for (key, transitions) in get_zonedump_data() {
        let zone = get_zone(key);
        for transition in transitions {
            let before = transition.transition - 2.days();
            let after = transition.transition + 2.days();
            trace_variables!(key, before, after, transition, {
                assert_eq!(zone.name(before.timestamp()), Some(transition.offset_before.name));
                assert_eq!(zone.offset(before.timestamp()), transition.offset_before.offset);

                assert_eq!(zone.name(after.timestamp()), Some(transition.offset_after.name));
                assert_eq!(zone.offset(after.timestamp()), transition.offset_after.offset);
            });
        }
    }
}

#[test]
fn test_ambiguous_times() {
    for (key, transitions) in get_zonedump_data() {
        let zone = get_zone(key);
        for transition in transitions {
            if !transition.is_fold() {
                continue;
            }

            // Before the fold is unambiguous
            let dt = transition.anomaly_start() - 1.seconds();
            trace_variables!(key, dt, transition, {
                let resolve = zone.clone().resolve(dt.date(), dt.time());
                assert!(resolve.is_unambiguous());
                let resolved = resolve.earlier().unwrap();
                assert_eq!(resolved.offset(), transition.offset_before.offset);
                assert_eq!(resolved.tzname(), Some(transition.offset_before.name));
            });

            // At the fold is ambiguous
            let dt = transition.anomaly_start();
            trace_variables!(key, dt, transition, {
                let resolve = zone.clone().resolve(dt.date(), dt.time());
                assert!(resolve.is_ambiguous());
                let (before, after) = resolve.into_pair();
                assert_eq!(before.offset(), transition.offset_before.offset);
                assert_eq!(before.tzname(), Some(transition.offset_before.name));
                assert_eq!(after.offset(), transition.offset_after.offset);
                assert_eq!(after.tzname(), Some(transition.offset_after.name));
            });

            // During the fold is ambiguous
            let dt = transition.anomaly_start() + 1.seconds();
            trace_variables!(key, dt, transition, {
                let resolve = zone.clone().resolve(dt.date(), dt.time());
                assert!(resolve.is_ambiguous());
                let (before, after) = resolve.into_pair();
                assert_eq!(before.offset(), transition.offset_before.offset);
                assert_eq!(before.tzname(), Some(transition.offset_before.name));
                assert_eq!(after.offset(), transition.offset_after.offset);
                assert_eq!(after.tzname(), Some(transition.offset_after.name));
            });

            // Before the fold ends is ambiguous
            let dt = transition.anomaly_end() - 1.seconds();
            trace_variables!(key, dt, transition, {
                let resolve = zone.clone().resolve(dt.date(), dt.time());
                assert!(resolve.is_ambiguous());
                let (before, after) = resolve.into_pair();
                assert_eq!(before.offset(), transition.offset_before.offset);
                assert_eq!(before.tzname(), Some(transition.offset_before.name));
                assert_eq!(after.offset(), transition.offset_after.offset);
                assert_eq!(after.tzname(), Some(transition.offset_after.name));
            });

            // When the fold ends it's unambiguous
            let dt = transition.anomaly_end();
            trace_variables!(key, dt, transition, {
                let resolve = zone.clone().resolve(dt.date(), dt.time());
                assert!(resolve.is_unambiguous());
                let resolved = resolve.earlier().unwrap();
                assert_eq!(resolved.offset(), transition.offset_after.offset);
                assert_eq!(resolved.tzname(), Some(transition.offset_after.name));
            });

            // After the fold ends it's still unambiguous
            let dt = transition.anomaly_end() + 1.seconds();
            trace_variables!(key, dt, transition, {
                let resolve = zone.clone().resolve(dt.date(), dt.time());
                assert!(resolve.is_unambiguous());
                let resolved = resolve.earlier().unwrap();
                assert_eq!(resolved.offset(), transition.offset_after.offset);
                assert_eq!(resolved.tzname(), Some(transition.offset_after.name));
            });
        }
    }
}

#[test]
fn test_missing_times() {
    for (key, transitions) in get_zonedump_data() {
        let zone = get_zone(key);
        for transition in transitions {
            if !transition.is_gap() {
                continue;
            }

            // Before the gap is unambiguous
            let dt = transition.anomaly_start() - 1.seconds();
            trace_variables!(key, dt, transition, {
                let resolve = zone.clone().resolve(dt.date(), dt.time());
                assert!(resolve.is_unambiguous());
                let resolved = resolve.earlier().unwrap();
                assert_eq!(resolved.offset(), transition.offset_before.offset);
                assert_eq!(resolved.tzname(), Some(transition.offset_before.name));
            });

            // At the gap is missing
            let dt = transition.anomaly_start();
            trace_variables!(key, dt, transition, {
                let resolve = zone.clone().resolve(dt.date(), dt.time());
                assert!(resolve.is_missing());
                let after = resolve.lenient();
                assert_eq!(after.offset(), transition.offset_after.offset);
                assert_eq!(after.tzname(), Some(transition.offset_after.name));
            });

            // During the gap is missing
            let dt = transition.anomaly_start() + 1.seconds();
            trace_variables!(key, dt, transition, {
                let resolve = zone.clone().resolve(dt.date(), dt.time());
                assert!(resolve.is_missing());
                let after = resolve.lenient();
                assert_eq!(after.offset(), transition.offset_after.offset);
                assert_eq!(after.tzname(), Some(transition.offset_after.name));
            });

            // Before the gap ends is missing
            let dt = transition.anomaly_end() - 1.seconds();
            trace_variables!(key, dt, transition, {
                let resolve = zone.clone().resolve(dt.date(), dt.time());
                assert!(resolve.is_missing());
                let after = resolve.lenient();
                assert_eq!(after.offset(), transition.offset_after.offset);
                assert_eq!(after.tzname(), Some(transition.offset_after.name));
            });

            // When the gap ends it's unambiguous
            let dt = transition.anomaly_end();
            trace_variables!(key, dt, transition, {
                let resolve = zone.clone().resolve(dt.date(), dt.time());
                assert!(resolve.is_unambiguous());
                let resolved = resolve.earlier().unwrap();
                assert_eq!(resolved.offset(), transition.offset_after.offset);
                assert_eq!(resolved.tzname(), Some(transition.offset_after.name));
            });

            // After the gap ends it's still unambiguous
            let dt = transition.anomaly_end() + 1.seconds();
            trace_variables!(key, dt, transition, {
                let resolve = zone.clone().resolve(dt.date(), dt.time());
                assert!(resolve.is_unambiguous());
                let resolved = resolve.earlier().unwrap();
                assert_eq!(resolved.offset(), transition.offset_after.offset);
                assert_eq!(resolved.tzname(), Some(transition.offset_after.name));
            });
        }
    }
}

#[test]
fn test_europe_prague_ambiguity() {
    let zone = get_zone("Europe/Prague");

    // DST start: 1946-05-06 2AM UTC+1 -> +1 hour (CET -> CEST)
    // DST end: 1946-10-06 3AM UTC+2 -> -1 hour (CEST -> CET)
    // DST start: 1946-12-01 3AM UTC+1 -> -1 hour (CET -> GMT)
    // DST end: 1947-02-23 2AM UTC+0 -> +1 hour (GMT -> CET)

    let names = [
        (datetime!(1946-12-01 2:30 +01:00), "CET"),
        (datetime!(1946-12-01 2:30 +00:00), "GMT"),
        (datetime!(1946-10-06 2:30 +02:00), "CEST"),
        (datetime!(1946-10-06 2:30 +01:00), "CET"),
        (datetime!(1947-02-23 3:30 am +01:00), "CET"),
        (datetime!(1946-05-06 3:30 am +02:00), "CEST"),
        (datetime!(1946-10-06 4:00 +01:00), "CET"),
    ];

    for (dt, name) in names {
        assert_eq!(zone.name(dt.timestamp()), Some(name));
    }

    // Ambiguous
    let local = datetime!(1946-12-01 2:30);
    let resolve = zone.clone().resolve(local.date(), local.time());
    assert!(resolve.is_ambiguous());
    assert_eq!(resolve.clone().earlier().unwrap(), datetime!(1946-12-01 2:30 +01:00));
    assert_eq!(resolve.clone().later().unwrap(), datetime!(1946-12-01 2:30 +00:00));
    assert_eq!(resolve.lenient(), datetime!(1946-12-01 2:30 +01:00));

    // Ambiguous again
    let local = datetime!(1946-10-06 2:30);
    let resolve = zone.clone().resolve(local.date(), local.time());
    assert!(resolve.is_ambiguous());
    assert_eq!(resolve.clone().earlier().unwrap(), datetime!(1946-10-06 2:30 +02:00));
    assert_eq!(resolve.clone().later().unwrap(), datetime!(1946-10-06 2:30 +01:00));
    assert_eq!(resolve.lenient(), datetime!(1946-10-06 2:30 +02:00));

    // Missing
    let local = datetime!(1947-02-23 2:30);
    let resolve = zone.clone().resolve(local.date(), local.time());
    assert!(resolve.is_missing());
    assert!(resolve.clone().earlier().is_err());
    assert!(resolve.clone().later().is_err());
    assert_eq!(resolve.lenient(), datetime!(1947-02-23 3:30 am +01:00));

    let local = datetime!(1946-05-06 2:30);
    let resolve = zone.clone().resolve(local.date(), local.time());
    assert!(resolve.is_missing());
    assert!(resolve.clone().earlier().is_err());
    assert!(resolve.clone().later().is_err());
    assert_eq!(resolve.lenient(), datetime!(1946-05-06 3:30 am +02:00));

    // Unambiguous
    let local = datetime!(1946-10-06 4:00);
    let resolve = zone.resolve(local.date(), local.time());
    assert!(resolve.is_unambiguous());
    assert_eq!(resolve.clone().earlier().unwrap(), datetime!(1946-10-06 4:00 +01:00));
    assert_eq!(resolve.clone().later().unwrap(), datetime!(1946-10-06 4:00 +01:00));
    assert_eq!(resolve.lenient(), datetime!(1946-10-06 4:00 +01:00));
}

#[test]
fn test_america_los_angeles_historical() {
    let zone = get_zone("America/Los_Angeles");

    let names = [
        (datetime!(1991-10-27 1:30 am -07:00), "PDT"),
        (datetime!(1991-10-27 1:30 am -08:00), "PST"),
        (datetime!(1991-10-27 12:30 am -07:00), "PDT"),
        (datetime!(1991-04-07 03:30 am -07:00), "PDT"),
    ];

    for (dt, name) in names {
        assert_eq!(zone.name(dt.timestamp()), Some(name));
    }

    // Ambiguous
    let local = datetime!(1991-10-27 1:30 am);
    let resolve = zone.clone().resolve(local.date(), local.time());
    assert!(resolve.is_ambiguous());
    assert_eq!(resolve.clone().earlier().unwrap(), datetime!(1991-10-27 1:30 am -07:00));
    assert_eq!(resolve.clone().later().unwrap(), datetime!(1991-10-27 1:30 am -08:00));
    assert_eq!(resolve.lenient(), datetime!(1991-10-27 1:30 am -07:00));

    // This is not ambiguous
    let local = datetime!(1991-10-27 12:30 am);
    let resolve = zone.clone().resolve(local.date(), local.time());
    assert!(resolve.is_unambiguous());
    assert_eq!(
        resolve.clone().earlier().unwrap(),
        datetime!(1991-10-27 12:30 am -07:00)
    );
    assert_eq!(resolve.lenient(), datetime!(1991-10-27 12:30 am -07:00));

    // This is missing
    let local = datetime!(1991-04-07 02:30 am);
    let resolve = zone.resolve(local.date(), local.time());
    assert!(resolve.is_missing());
    assert!(resolve.clone().earlier().is_err());
    assert!(resolve.clone().later().is_err());
    assert_eq!(resolve.lenient(), datetime!(1991-04-07 03:30 am -07:00));
}
