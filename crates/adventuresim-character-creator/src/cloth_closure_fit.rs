//! Closure solids follow frozen garment edges, after the cloth itself is fitted.
use crate::cloth_fastener_mesh::{FastenerFrame, FastenerMesh};
use crate::cloth_opening::{ClosureAnchors, ClothOpening};
use crate::device_fit::DevicePiece;
use anyhow::{Context, Result};
use fabelgeist_armor::gpu::body::Skin;
use fabelgeist_armor::{BuiltPart, ClothClosures};

pub(crate) struct PlannedClothClosures {
    pub design: ClothClosures,
    pub opening: ClothOpening,
}

impl DevicePiece {
    /// Native outer-carrier offset converts construction IDs to final vertices.
    pub(crate) fn attach_closures(
        &mut self,
        design: &ClothClosures,
        opening: &ClothOpening,
        first: usize,
    ) -> Result<()> {
        for &position in &design.positions {
            let anchors = opening.sample(position, first)?;
            let mesh = fit(&self.base, design, &anchors)?;
            let endpoints = self
                .endpoints
                .iter()
                .map(|p| fit(p, design, &anchors).map(|m| m.part))
                .collect::<Result<Vec<_>>>()?;
            let mut skin = Skin::default();
            for side in mesh.owners {
                let source = anchors.side(side).skin_vertex();
                skin.texcoords.push(
                    *self
                        .skin
                        .texcoords
                        .get(source)
                        .context("closure has no cloth skin chart")?,
                );
                skin.joint_indices.push(
                    *self
                        .skin
                        .joint_indices
                        .get(source)
                        .context("closure has no cloth joints")?,
                );
                skin.joint_weights.push(
                    *self
                        .skin
                        .joint_weights
                        .get(source)
                        .context("closure has no cloth weights")?,
                );
            }
            let aliases = (0..mesh.part.positions.len()).collect();
            self.append(Self {
                base: mesh.part,
                skin,
                endpoints,
                skin_aliases: aliases,
            })?;
        }
        Ok(())
    }
}
fn fit(part: &BuiltPart, design: &ClothClosures, anchors: &ClosureAnchors) -> Result<FastenerMesh> {
    FastenerMesh::new(
        &design.fastener,
        FastenerFrame {
            left: anchors.left.position(part)?,
            right: anchors.right.position(part)?,
            normal: (anchors.left.normal(part)? + anchors.right.normal(part)?).normalize(),
        },
    )
}
