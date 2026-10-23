//! Actual closed hardware follows frozen band and bodice cloth samples.
use crate::{
    cloth_closure_fit::edge_frame,
    cloth_fastener_mesh::{FastenerFrame, FastenerMesh},
    cloth_opening::{ClosureAnchors, ClosureSide},
    device_fit::DevicePiece,
    skirt_join::SkirtJoinPlan,
    waist_fastener_sources::WaistFastenerSources,
    waist_support_plan::WaistSupportPlan,
};
use anyhow::{Context, Result};
use fabelgeist_math::Vec3;
impl WaistSupportPlan {
    pub(crate) fn attach_fasteners(
        &self,
        piece: &mut DevicePiece,
        join: &SkirtJoinPlan,
    ) -> Result<()> {
        let Some(design) = &self.design.fasteners else {
            return Ok(());
        };
        let sources = WaistFastenerSources::new(self, piece, join)?;
        for anchors in sources.parallel {
            piece.attach_edge_solid(&anchors, |part| {
                FastenerMesh::new(&design.closures.fastener, edge_frame(part, &anchors)?)
            })?;
        }
        if let Some(sources) = sources.return_sources {
            let lacing = self.lacing.as_ref().context("waist return has no ring")?;
            let hook = design
                .return_hook
                .as_ref()
                .context("waist return has no hook")?;
            piece.attach_edge_solid(&sources.ring, |part| {
                FastenerMesh::single_lacing_ring(
                    lacing,
                    edge_frame(part, &sources.ring)?,
                    ClosureSide::Left,
                )
            })?;
            let anchors = ClosureAnchors {
                left: sources.hook,
                right: sources.ring.left,
            };
            piece.attach_edge_solid(&anchors, |part| {
                let frame = edge_frame(part, &sources.ring)?;
                let normal = -frame.normal;
                let ring = crate::cloth_lacing_mesh::ring_center(lacing, frame, ClosureSide::Left)?;
                let hook_frame = FastenerFrame {
                    left: sources.hook.position(part)?,
                    right: ring - Vec3::up() * lacing.ring_radius.metres(),
                    normal,
                };
                crate::waist_fastener_sources::validate_return_tail(part, hook, hook_frame)?;
                FastenerMesh::single_hook(hook, hook_frame).with_context(|| {
                    format!(
                        "waist return hook left={:?} right={:?} normal={:?}",
                        hook_frame.left, hook_frame.right, hook_frame.normal
                    )
                })
            })?;
        }
        Ok(())
    }
}
