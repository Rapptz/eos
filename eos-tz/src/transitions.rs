use eos::UtcOffset;

use crate::{timestamp::NaiveTimestamp, ParseError};

/// A transition type in the TZif data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TransitionType {
    pub(crate) offset: i32,
    pub(crate) is_dst: bool,
    pub(crate) abbr: String,
}

/// A transition in a timezone.
///
/// This includes data like a range of time when a time zone applies.
/// Along with a name and whether there's a DST correction being done.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Transition {
    /// The index that points to the name of the time zone if it falls at this interval.
    pub(crate) name_idx: usize,
    /// The *local* start time that the time zone will start at.
    ///
    /// This could be NaiveTimestamp::MIN to extend to the beginning of time.
    pub(crate) start: NaiveTimestamp,
    /// The UTC start time that the time zone will start.
    ///
    /// This could be NaiveTimestamp::MIN to extend to the beginning of time.
    pub(crate) utc_start: NaiveTimestamp,
    /// The *local* end time that the time zone will end.
    pub(crate) end: NaiveTimestamp,
    /// The UTC offset that applies to the timezone if it falls at this interval.
    pub(crate) offset: UtcOffset,
}

impl Transition {
    pub(crate) fn new(
        trans: i64,
        ttype: &TransitionType,
        ttype_idx: usize,
        previous: Option<&Self>,
    ) -> Result<Self, ParseError> {
        let utc_start = NaiveTimestamp::from_seconds(trans);
        let (local, fix) = match previous {
            Some(t) => {
                let prev_offset = t.offset.total_seconds() as i64;
                (trans.saturating_add(prev_offset), ttype.offset as i64 - prev_offset)
            }
            None => (trans.saturating_add(ttype.offset as i64), 0),
        };
        let start = NaiveTimestamp::from_seconds(local);
        let end = NaiveTimestamp::from_seconds(local + fix);
        Ok(Self {
            name_idx: ttype_idx,
            start,
            utc_start,
            end,
            offset: UtcOffset::from_seconds(ttype.offset).map_err(|_| ParseError::InvalidOffset)?,
        })
    }

    #[inline]
    pub(crate) fn is_ambiguous(&self, ts: NaiveTimestamp) -> bool {
        self.end <= ts && ts < self.start
    }

    #[inline]
    pub(crate) fn is_missing(&self, ts: NaiveTimestamp) -> bool {
        self.start <= ts && ts < self.end
    }
}
