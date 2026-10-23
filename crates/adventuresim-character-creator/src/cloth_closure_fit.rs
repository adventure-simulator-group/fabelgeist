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
            self.attach_edge_solid(&anchors, |part| {
                FastenerMesh::new(&design.fastener, edge_frame(part, &anchors)?)
            })?;
        }
        Ok(())
    }
    pub(crate) fn attach_edge_solid(
        &mut self,
        anchors: &ClosureAnchors,
        fit: impl Fn(&BuiltPart) -> Result<FastenerMesh>,
    ) -> Result<()> {
        let mesh = fit(&self.base).context("fitting base cloth fastener")?;
        let endpoints = self
            .endpoints
            .iter()
            .enumerate()
            .map(|(endpoint, part)| {
                fit(part)
                    .map(|mesh| mesh.part)
                    .with_context(|| format!("fitting cloth fastener endpoint {endpoint}"))
            })
            .collect::<Result<Vec<_>>>()?;
        let mut skin = Skin {
            joint_indices: fabelgeist_rig::PackedRigSkin::empty(
                self.skin.joint_indices.joint_count(),
            ),
            ..Skin::default()
        };
        for side in mesh.owners {
            let source = anchors.side(side).skin_vertex();
            skin.texcoords.push(
                *self
                    .skin
                    .texcoords
                    .get(source)
                    .context("closure has no cloth skin chart")?,
            );
            anyhow::ensure!(
                source < self.skin.joint_indices.len(),
                "closure has no cloth joints"
            );
            skin.joint_indices
                .push(self.skin.joint_indices.vertex(source))?;
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
        Ok(())
    }
}

/// Native actual cloth samples in canonical metres, used for every realization.
pub(crate) fn edge_frame(part: &BuiltPart, anchors: &ClosureAnchors) -> Result<FastenerFrame> {
    Ok(FastenerFrame {
        left: anchors.left.position(part)?,
        right: anchors.right.position(part)?,
        normal: (anchors.left.normal(part)? + anchors.right.normal(part)?).normalize(),
    })
}
