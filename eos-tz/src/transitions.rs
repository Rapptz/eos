use eos::UtcOffset;

use crate::{posix::PosixTimeZone, timestamp::NaiveTimestamp};

/// Represents a transition type in the TZif data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TransitionType {
    pub(crate) offset: i32,
    pub(crate) is_dst: bool,
    pub(crate) abbr: String,
}

/// Represents a transition in a timezone.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Transition {
    pub(crate) at: NaiveTimestamp,
    pub(crate) type_idx: usize,
    pub(crate) local: NaiveTimestamp,
    pub(crate) to: NaiveTimestamp,
    pub(crate) fix: i32,
    pub(crate) offset: UtcOffset,
}

impl Transition {
    pub(crate) fn new(
        at: NaiveTimestamp,
        type_idx: usize,
        ttype: &TransitionType,
        prev: Option<&TransitionType>,
    ) -> Self {
        let (local, fix) = match prev {
            Some(prev) => (NaiveTimestamp(at.0 + prev.offset as i64), ttype.offset - prev.offset),
            None => (NaiveTimestamp(at.0 + ttype.offset as i64), 0),
        };
        Self {
            at,
            type_idx,
            local,
            to: NaiveTimestamp(local.0 + fix as i64),
            fix,
            // TZif guarantees that the range is valid
            offset: UtcOffset::from_seconds(ttype.offset).unwrap(),
        }
    }

    pub(crate) fn is_ambiguous(&self, ts: NaiveTimestamp) -> bool {
        self.to <= ts && ts < self.local
    }

    pub(crate) fn is_imaginary(&self, ts: NaiveTimestamp) -> bool {
        self.local <= ts && ts < self.to
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transitions {
    pub(crate) data: Vec<Transition>,
    pub(crate) types: Vec<TransitionType>,
    pub(crate) posix: Option<PosixTimeZone>,
}
