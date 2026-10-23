//! Taut lacing follows final visible backing, with frozen triangle ownership.
use crate::cloth_fastener_mesh::ClosureFrame;
use crate::device_fit::DevicePiece;
use anyhow::{Result, ensure};
use fabelgeist_armor::{BuiltPart, PlateFace};
use fabelgeist_bvh::{Ray, TriangleBvh};
use fabelgeist_math::Vec3;
use std::ops::Range;

/// Native final triangle IDs, shared by the base and every fitted realization.
pub(crate) struct LacingSupport {
    faces: Vec<[u32; 3]>,
}
impl LacingSupport {
    /// The unpartitioned first shell still has its generic material role.
    /// Native source aliases identify just its torso, excluding sleeves/skirt.
    pub fn new(
        piece: &DevicePiece,
        torso_sources: Range<usize>,
        opening: Option<&fabelgeist_armor::gpu::SkirtDressingOpening>,
    ) -> Self {
        let faces = piece
            .base
            .components
            .iter()
            .enumerate()
            .filter(|(index, _)| *index == 0)
            .flat_map(|(_, component)| {
                let torso_sources = &torso_sources;
                (component.indices.start..component.indices.end)
                    .step_by(3)
                    .filter_map(move |i| {
                        let face = [
                            piece.base.indices[i],
                            piece.base.indices[i + 1],
                            piece.base.indices[i + 2],
                        ];
                        let torso = face.iter().all(|&v| {
                            let source = piece.skin_aliases[v as usize];
                            let source = opening.map_or(source, |o| o.source_vertex(source));
                            torso_sources.contains(&source)
                        });
                        (torso && piece.base.faces[i / 3] == PlateFace::Outer).then_some(face)
                    })
            })
            .collect();
        Self { faces }
    }
    /// Native closure adapter: outward metre height above the ring-center
    /// chord at its midpoint. One sample does not certify all cord facets.
    pub fn height_metres(&self, frame: &ClosureFrame, part: &BuiltPart) -> Result<f32> {
        if self.faces.is_empty() {
            return Ok(0.);
        }
        let points = part
            .positions
            .iter()
            .map(|p| Vec3::new(p[0], p[1], p[2]))
            .collect();
        let tree = TriangleBvh::new(points, self.faces.clone());
        let origin = frame.point(Vec3::new(frame.gap * 0.5, 0., 0.));
        let direction = frame.vector(Vec3::forward());
        let ray = Ray::new(
            Vec3::new(origin[0], origin[1], origin[2]),
            Vec3::new(direction[0], direction[1], direction[2]),
        );
        let height = tree
            .raycast(&ray, f32::MAX)
            .map_or(0., |(_, distance)| distance);
        ensure!(
            height.is_finite() && height >= 0.,
            "lacing backing support is invalid"
        );
        Ok(height)
    }
}
