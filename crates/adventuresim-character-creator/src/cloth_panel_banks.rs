//! Native anatomical ownership carried through coarse edge collapses. A dense
//! facet belongs to its surviving face, edge or vertex star. Conditioning
//! flips transfer only the union of the two replaced facets. Refits reuse these identities without spatial search.
use super::domain::PanelDomain;
use crate::armor_layer::ArmorLayerSurface;
use crate::surface_cut::SamplingAncestry;
use anyhow::{Context, Result, ensure};
use fabelgeist_bvh::{Aabb, TriangleBvh, closest_point_on_triangle};
use fabelgeist_math::Vec3;
use std::collections::BTreeSet;

/// Finite facet samples used for both correspondence and complementary rays.
pub(crate) const CONTACT_SAMPLE_WEIGHTS: [[f32; 3]; 7] = [
    [1., 0., 0.],
    [0., 1., 0.],
    [0., 0., 1.],
    [1. / 3.; 3],
    [0.5, 0.5, 0.],
    [0., 0.5, 0.5],
    [0.5, 0., 0.5],
];

#[derive(Clone)]
pub(crate) struct ContactBanks {
    target_faces: Vec<Vec<u32>>,
    source_faces: Vec<Vec<u32>>,
    target_face_count: usize,
}

impl ContactBanks {
    pub(super) fn new(
        surface: &PanelDomain,
        support: &PanelDomain,
        ancestry: &SamplingAncestry,
    ) -> Result<Self> {
        ensure!(
            ancestry.source_facets.len() == surface.cut.faces.len(),
            "anatomical collapse ancestry has different coarse facets"
        );
        let mut target_faces = Vec::with_capacity(surface.cut.faces.len());
        let mut reverse = vec![BTreeSet::new(); support.cut.points.len()];
        let mut covered = vec![false; support.cut.faces.len()];
        for (source, bank) in ancestry.source_facets.iter().enumerate() {
            for &target in bank {
                let face = support
                    .cut
                    .faces
                    .get(target as usize)
                    .context("anatomical bank has an invalid native facet")?;
                covered[target as usize] = true;
                for &vertex in face {
                    reverse[vertex as usize].insert(u32::try_from(source)?);
                }
            }
            target_faces.push(bank.iter().copied().collect::<Vec<_>>());
        }
        ensure!(
            covered.iter().all(|&v| v),
            "retained native facet has no coarse bank"
        );
        ensure!(
            target_faces.iter().all(|faces| !faces.is_empty()),
            "coarse anatomical facet has no retained dense support"
        );
        for face in &support.cut.faces {
            ensure!(
                face.iter().all(|&v| !reverse[v as usize].is_empty()),
                "retained anatomical vertex has no source bank"
            );
        }
        Ok(Self {
            target_faces,
            source_faces: reverse
                .into_iter()
                .map(|faces| faces.into_iter().collect())
                .collect(),
            target_face_count: support.cut.faces.len(),
        })
    }

    /// Freeze nearest finite source cells for each lower-facet sample before
    /// seating. Canonical metre geometry; ties include duplicate material cuts.
    /// Sampled correspondence is not a continuous facet-coverage certificate.
    pub(crate) fn for_layer(
        positions: &[[f32; 3]],
        faces: &[[u32; 3]],
        layer: &ArmorLayerSurface<'_>,
        eligible: &[bool],
    ) -> Result<Self> {
        let tree = TriangleBvh::new(
            positions.iter().copied().map(point).collect(),
            faces.to_vec(),
        );
        let mut targets = vec![BTreeSet::new(); faces.len()];
        let mut reverse = vec![BTreeSet::new(); layer.positions.len()];
        for (target, face) in layer.faces.iter().enumerate() {
            if !face.iter().all(|&v| eligible[v as usize]) {
                continue;
            }
            let vertices = face.map(|v| point(layer.positions[v as usize]));
            let mut sources = BTreeSet::new();
            for weights in CONTACT_SAMPLE_WEIGHTS {
                let sample = vertices
                    .into_iter()
                    .zip(weights)
                    .map(|(p, w)| p * w)
                    .fold(Vec3::new(0., 0., 0.), |sum, value| sum + value);
                let (_, _, distance) = tree
                    .closest_point(sample, f32::MAX)
                    .context("lower garment has no finite source correspondence")?;
                let limit = distance + crate::cloth_layer_seating::CLEARANCE_TOLERANCE_M;
                let bounds = Aabb::point(sample).expand(limit);
                tree.bvh.candidates_aabb(&bounds, |source| {
                    let [a, b, c] =
                        tree.triangles[source as usize].map(|v| tree.positions[v as usize]);
                    if (closest_point_on_triangle(sample, a, b, c) - sample).length() <= limit {
                        sources.insert(source);
                    }
                });
            }
            ensure!(
                !sources.is_empty(),
                "lower facet lost all frozen source correspondence"
            );
            for source in sources {
                targets[source as usize].insert(u32::try_from(target)?);
                for &vertex in face {
                    reverse[vertex as usize].insert(source);
                }
            }
        }
        Ok(Self {
            target_faces: targets
                .into_iter()
                .map(|v| v.into_iter().collect())
                .collect(),
            source_faces: reverse
                .into_iter()
                .map(|v| v.into_iter().collect())
                .collect(),
            target_face_count: layer.faces.len(),
        })
    }

    pub(crate) fn validate(
        &self,
        source_faces: usize,
        target_faces: usize,
        target_vertices: usize,
    ) -> Result<()> {
        ensure!(
            self.target_faces.len() == source_faces
                && self.target_face_count == target_faces
                && self.source_faces.len() == target_vertices,
            "retained anatomical bank topology mismatch"
        );
        Ok(())
    }

    pub(crate) fn targets(&self, source: usize) -> &[u32] {
        &self.target_faces[source]
    }

    pub(crate) fn sources(&self, target: usize) -> &[u32] {
        &self.source_faces[target]
    }
}

/// Native BVH boundary in canonical unposed metres.
fn point([x, y, z]: [f32; 3]) -> Vec3 {
    Vec3::new(x, y, z)
}
