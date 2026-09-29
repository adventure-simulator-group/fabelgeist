//! A mesh whose every vertex is a place on one of a piece's grids, so it can
//! be realized wherever the piece is: its base fit or any of its morphs.

use fabelgeist_math::vector::Vec3;

use super::surface::{GridPoint, GridSurface};
use crate::{ArmorMorph, GeneratedArmor, PlateFace, skin};

/// A vertex, as a place on a grid.
#[derive(Clone, Copy, Debug)]
pub(super) struct Embedded {
    pub grid: usize,
    pub at: GridPoint,
    /// Metres out of the surface.
    pub height: f32,
    /// The shading normal in the surface's frame at `at`: across, up, out.
    pub normal: [f32; 3],
    pub texcoord: [f32; 2],
}

/// Vertices on a piece's grids and the triangles joining them.
#[derive(Clone, Debug, Default)]
pub(super) struct EmbeddedMesh {
    pub vertices: Vec<Embedded>,
    pub indices: Vec<u32>,
    /// The plate face of each triangle; empty for a mesh that is no plate.
    pub faces: Vec<PlateFace>,
}

impl EmbeddedMesh {
    /// Positions and normals on one realization of every grid.
    fn realize(&self, surfaces: &[GridSurface]) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
        self.vertices
            .iter()
            .map(|vertex| {
                let frame = surfaces[vertex.grid].frame(vertex.at);
                let position = frame.origin + frame.out * vertex.height;
                (
                    position.to_array(),
                    frame.direction(vertex.normal).to_array(),
                )
            })
            .unzip()
    }

    /// The mesh as a piece of `source`'s: skinned and morphed as the
    /// surface it lies on.
    pub(super) fn armor(
        &self,
        source: &GeneratedArmor,
        surfaces: &[GridSurface],
        design_hash: [u8; 32],
    ) -> GeneratedArmor {
        let (positions, normals) = self.realize(surfaces);
        let (joint_indices, joint_weights) = self
            .vertices
            .iter()
            .map(|vertex| {
                let corners = surfaces[vertex.grid]
                    .corners(vertex.at)
                    .map(|(corner, share)| {
                        let corner = corner as usize;
                        (
                            (source.joint_indices[corner], source.joint_weights[corner]),
                            share,
                        )
                    });
                skin::blend(&corners)
            })
            .unzip();
        let morphs = source
            .morphs
            .iter()
            .map(|morph| {
                let realized = surfaces
                    .iter()
                    .map(|surface| surface.realized(&morph.direct_positions))
                    .collect::<Vec<_>>();
                let (direct, morph_normals) = self.realize(&realized);
                ArmorMorph {
                    name: morph.name.clone(),
                    position_deltas: deltas(&positions, &direct),
                    normal_deltas: deltas(&normals, &morph_normals),
                    direct_positions: direct,
                }
            })
            .collect();
        GeneratedArmor {
            components: Vec::new(),
            design_hash,
            surface_domain: source.surface_domain.clone(),
            positions,
            normals,
            texcoords: self.vertices.iter().map(|vertex| vertex.texcoord).collect(),
            joint_indices,
            joint_weights,
            indices: self.indices.clone(),
            faces: self.faces.clone(),
            trim: None,
            grids: Vec::new(),
            morphs,
        }
    }
}

fn deltas(base: &[[f32; 3]], target: &[[f32; 3]]) -> Vec<[f32; 3]> {
    base.iter()
        .zip(target)
        .map(|(a, b)| (Vec3::from_array(*b) - Vec3::from_array(*a)).to_array())
        .collect()
}
