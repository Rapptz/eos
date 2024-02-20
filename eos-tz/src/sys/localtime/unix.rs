use crate::TimeZone;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct LocalTime {
    inner: TimeZone,
}

impl LocalTime {
    pub(crate) fn new() -> Result<Self, crate::Error> {
        Ok(Self {
            inner: TimeZone::etc_localtime()?,
        })
    }

    #[inline]
    pub(crate) fn as_inner(&self) -> &TimeZone {
        &self.inner
    }
}
