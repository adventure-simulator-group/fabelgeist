//! Named random purposes owned by this generator. Names are part of its replay contract.
use fabelgeist_determinism::StreamId;

pub(super) const AGGREGATE: StreamId = StreamId::new("texture.lime-plaster.aggregate");
pub(super) const CAVITY: StreamId = StreamId::new("texture.lime-plaster.cavity");
pub(super) const CROSS_DIAGONAL: StreamId = StreamId::new("texture.lime-plaster.cross-diagonal");
pub(super) const DIAGONAL: StreamId = StreamId::new("texture.lime-plaster.diagonal");
pub(super) const FEATURE_IDENTITY: StreamId =
    StreamId::new("texture.lime-plaster.feature-identity");
pub(super) const FEATURE_PRESENCE: StreamId =
    StreamId::new("texture.lime-plaster.feature-presence");
pub(super) const FEATURE_X: StreamId = StreamId::new("texture.lime-plaster.feature-x");
pub(super) const FEATURE_Y: StreamId = StreamId::new("texture.lime-plaster.feature-y");
pub(super) const FINE_AGGREGATE: StreamId = StreamId::new("texture.lime-plaster.fine-aggregate");
pub(super) const FLOAT_TRACKS: StreamId = StreamId::new("texture.lime-plaster.float-tracks");
pub(super) const LATTICE: StreamId = StreamId::new("texture.lime-plaster.lattice");
pub(super) const MINERAL: StreamId = StreamId::new("texture.lime-plaster.mineral");
pub(super) const SAND: StreamId = StreamId::new("texture.lime-plaster.sand");
pub(super) const TROWEL_STROKES: StreamId = StreamId::new("texture.lime-plaster.trowel-strokes");
pub(super) const WARP_X: StreamId = StreamId::new("texture.lime-plaster.warp-x");
pub(super) const WARP_Y: StreamId = StreamId::new("texture.lime-plaster.warp-y");
