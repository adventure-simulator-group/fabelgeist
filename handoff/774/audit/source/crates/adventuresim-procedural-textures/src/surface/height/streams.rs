//! Named random purposes owned by this generator. Names are part of its replay contract.
use fabelgeist_determinism::StreamId;

pub(super) const CORE_WIDTH: StreamId = StreamId::new("texture.surface.height.core-width");
pub(super) const CROWN_HEIGHT: StreamId = StreamId::new("texture.surface.height.crown-height");
pub(super) const PLATE_VARIATION: StreamId =
    StreamId::new("texture.surface.height.plate-variation");
pub(super) const ROW_OFFSET: StreamId = StreamId::new("texture.surface.height.row-offset");
pub(super) const SHOULDER_HEIGHT: StreamId =
    StreamId::new("texture.surface.height.shoulder-height");
pub(super) const VALLEY_WIDTH: StreamId = StreamId::new("texture.surface.height.valley-width");
pub(super) const VERTICAL_BULGE: StreamId = StreamId::new("texture.surface.height.vertical-bulge");
