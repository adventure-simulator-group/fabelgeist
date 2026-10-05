//! Recorded dispatches, independent of compilations, items or workgroups.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordedDispatchCount(usize);
impl RecordedDispatchCount {
    pub(super) fn record(&mut self) {
        self.0 += 1;
    }
}
impl std::fmt::Display for RecordedDispatchCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
