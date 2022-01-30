use eos::UtcOffset;

use crate::timestamp::NaiveTimestamp;

/// Represents a transition type in the TZif data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TransitionType {
    pub(crate) offset: i32,
    pub(crate) is_dst: bool,
    pub(crate) abbr: String,
}

/// Represents a transition of a timezone.
///
/// This includes data like a range of time when a time zone applies.
/// Along with a name and whether there's a DST correction being done.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Transition {
    /// The index that points to the name of the time zone if it falls at this interval.
    pub(crate) name_idx: usize,
    /// The UTC start time that the time zone will start.
    ///
    /// This could be NaiveTimestamp::MIN to extend to the beginning of time.
    pub(crate) start: NaiveTimestamp,
    /// The UTC end time that the time zone will end.
    pub(crate) end: NaiveTimestamp,
    /// The UTC offset that applies to the timezone if it falls at this interval.
    pub(crate) offset: UtcOffset,
}

impl Transition {
    /// Returns `true` if the *local* timestamp is within this zone interval.
    #[inline]
    pub(crate) fn contains(&self, ts: NaiveTimestamp) -> bool {
        self.start <= ts && ts <= self.end
    }

    /// Returns the start time the time zone starts in local time.
    #[inline]
    pub(crate) fn start_at_local(&self) -> NaiveTimestamp {
        let seconds = self.offset.total_seconds();
        NaiveTimestamp::from_seconds(self.start.0.saturating_sub(seconds as i64))
    }

    /// Returns the end time the time zone starts in local time.
    #[inline]
    pub(crate) fn end_at_local(&self) -> NaiveTimestamp {
        let seconds = self.offset.total_seconds();
        NaiveTimestamp::from_seconds(self.end.0.saturating_sub(seconds as i64))
    }

    #[inline]
    pub(crate) fn contains_in_local(&self, ts: NaiveTimestamp) -> bool {
        self.start_at_local() <= ts && ts <= self.end_at_local()
    }
}
