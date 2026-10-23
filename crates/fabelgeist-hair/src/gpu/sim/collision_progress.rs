//! The two native collision-sample buffers and their accepted history.

/// Copied while recording; the simulation retains it only after submission.
#[derive(Clone, Copy, Default)]
pub(super) struct CollisionProgress {
    current: SampleSlot,
    history: SampleHistory,
}

#[derive(Clone, Copy, Default)]
enum SampleSlot {
    #[default]
    First,
    Second,
}

#[derive(Clone, Copy, Default)]
enum SampleHistory {
    #[default]
    Missing,
    Available,
}

impl CollisionProgress {
    pub(super) fn advance(&mut self) {
        self.current = match self.current {
            SampleSlot::First => SampleSlot::Second,
            SampleSlot::Second => SampleSlot::First,
        };
    }

    /// Native indices into the simulation's fixed pair of sample buffers.
    pub(super) fn current_index(self) -> usize {
        match self.current {
            SampleSlot::First => 0,
            SampleSlot::Second => 1,
        }
    }

    pub(super) fn previous_index(self) -> usize {
        match self.current {
            SampleSlot::First => 1,
            SampleSlot::Second => 0,
        }
    }

    pub(super) fn has_previous(self) -> bool {
        matches!(self.history, SampleHistory::Available)
    }

    pub(super) fn mark_available(&mut self) {
        self.history = SampleHistory::Available;
    }

    pub(super) fn forget_previous(&mut self) {
        self.history = SampleHistory::Missing;
    }
}
