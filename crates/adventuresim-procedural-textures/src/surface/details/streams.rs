//! Named random purposes owned by this generator. Names are part of its replay contract.
use fabelgeist_determinism::StreamId;

pub(super) const BRANCH_DEPTH: StreamId = StreamId::new("texture.surface.details.branch-depth");
pub(super) const BRANCH_END: StreamId = StreamId::new("texture.surface.details.branch-end");
pub(super) const BRANCH_PRESENCE: StreamId =
    StreamId::new("texture.surface.details.branch-presence");
pub(super) const BRANCH_REACH: StreamId = StreamId::new("texture.surface.details.branch-reach");
pub(super) const BRANCH_SIDE: StreamId = StreamId::new("texture.surface.details.branch-side");
pub(super) const BRANCH_START: StreamId = StreamId::new("texture.surface.details.branch-start");
pub(super) const BRANCH_WIDTH: StreamId = StreamId::new("texture.surface.details.branch-width");
pub(super) const CHIP_DEPTH: StreamId = StreamId::new("texture.surface.details.chip-depth");
pub(super) const CHIP_X: StreamId = StreamId::new("texture.surface.details.chip-x");
pub(super) const CHIP_Y: StreamId = StreamId::new("texture.surface.details.chip-y");
pub(super) const NOTCH_DEPTH: StreamId = StreamId::new("texture.surface.details.notch-depth");
pub(super) const NOTCH_Y: StreamId = StreamId::new("texture.surface.details.notch-y");
pub(super) const PLATE_CRACK_DEPTH: StreamId =
    StreamId::new("texture.surface.details.plate-crack-depth");
pub(super) const PLATE_CRACK_PHASE: StreamId =
    StreamId::new("texture.surface.details.plate-crack-phase");
pub(super) const PLATE_CRACK_WIDTH: StreamId =
    StreamId::new("texture.surface.details.plate-crack-width");
