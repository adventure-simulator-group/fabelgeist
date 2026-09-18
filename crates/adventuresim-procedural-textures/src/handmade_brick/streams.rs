//! Named random purposes owned by this generator. Names are part of its replay contract.
use fabelgeist_determinism::StreamId;

pub(super) const BOW_AMPLITUDE: StreamId = StreamId::new("texture.handmade-brick.bow-amplitude");
pub(super) const BOW_PHASE: StreamId = StreamId::new("texture.handmade-brick.bow-phase");
pub(super) const BRICK: StreamId = StreamId::new("texture.handmade-brick.brick");
pub(super) const CHIP_CENTER: StreamId = StreamId::new("texture.handmade-brick.chip-center");
pub(super) const CHIP_DEPTH: StreamId = StreamId::new("texture.handmade-brick.chip-depth");
pub(super) const CHIP_PRESENCE: StreamId = StreamId::new("texture.handmade-brick.chip-presence");
pub(super) const CHIP_WIDTH: StreamId = StreamId::new("texture.handmade-brick.chip-width");
pub(super) const FACE_BROAD_PHASE: StreamId =
    StreamId::new("texture.handmade-brick.face-broad-phase");
pub(super) const FACE_FINE_PHASE: StreamId =
    StreamId::new("texture.handmade-brick.face-fine-phase");
pub(super) const HEIGHT: StreamId = StreamId::new("texture.handmade-brick.height");
pub(super) const HORIZONTAL_JITTER: StreamId =
    StreamId::new("texture.handmade-brick.horizontal-jitter");
pub(super) const PALETTE: StreamId = StreamId::new("texture.handmade-brick.palette");
pub(super) const VERTICAL_JITTER: StreamId =
    StreamId::new("texture.handmade-brick.vertical-jitter");
pub(super) const WIDTH: StreamId = StreamId::new("texture.handmade-brick.width");
