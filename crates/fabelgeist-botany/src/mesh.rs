//! Geometry out: branches as generalized cylinders (Sec. 5.3) and foliage as
//! leaf cards (Sec. 7).

use fabelgeist_math::Vec3;
use fabelgeist_mesh::MeshData;
use fabelgeist_tube::TubeSegment;
use rayon::prelude::*;

use crate::ecosystem::Ecosystem;
use crate::plant::Plant;
use crate::prototype::{NONE, PrototypeLibrary};
use crate::species::PlantType;

/// Segments shorter than this are not swept.
const MIN_LENGTH: f32 = 1e-4;

impl Plant {
    /// Every grown segment as a truncated cone, tapering from its own
    /// pipe-model diameter to its thickest continuation.
    pub fn branch_segments(&self, prototypes: &PrototypeLibrary, out: &mut Vec<TubeSegment>) {
        for module in &self.modules {
            let prototype = prototypes.get(module.prototype);
            let offset = module.node_offset as usize;
            for (n, node) in prototype.nodes.iter().enumerate().skip(1) {
                let a = self.nodes[offset + node.parent as usize].position;
                let b = &self.nodes[offset + n];
                if (b.position - a).length() < MIN_LENGTH || b.diameter <= 0.0 {
                    continue;
                }
                let end = if b.end_diameter > 0.0 {
                    b.end_diameter
                } else {
                    b.diameter * 0.5
                };
                // Start a little behind the joint, so that the cone overlaps
                // its parent on the outside of a bend instead of leaving a
                // notch.
                let axis = b.position - a;
                let length = axis.length();
                let start = a - axis * ((b.diameter * 0.5).min(0.3 * length) / length);
                out.push(TubeSegment::new(
                    start.to_array(),
                    b.diameter * 0.5,
                    b.position.to_array(),
                    end.min(b.diameter) * 0.5,
                ));
            }
        }
    }

    /// Crossed quads at every branch tip, scaled in with the module's age.
    pub fn leaf_cards(
        &self,
        species: &PlantType,
        prototypes: &PrototypeLibrary,
        out: &mut MeshData,
    ) {
        if species.leaf_size <= 0.0 {
            return;
        }
        for module in self.modules.iter().filter(|m| self.has_leaves(m, species)) {
            let prototype = prototypes.get(module.prototype);
            let offset = module.node_offset as usize;
            for (n, node) in prototype.nodes.iter().enumerate().skip(1) {
                if !prototype.children[n].is_empty() {
                    continue;
                }
                let slot = prototype.terminal_slot[n];
                if slot != NONE && module.children[slot as usize] != NONE {
                    continue;
                }
                let tip = self.nodes[offset + n];
                let base = self.nodes[offset + node.parent as usize].position;
                let axis = tip.position - base;
                let length = axis.length();
                if length < MIN_LENGTH {
                    continue;
                }
                let size = species.leaf_size * (0.4 + 0.6 * module.age.min(1.0));
                push_card(out, tip.position, axis / length, size);
            }
        }
    }
}

impl Plant {
    /// Whether `module` still carries leaves (see [`PlantType::leaf_years`]).
    pub fn has_leaves(&self, module: &crate::Module, species: &PlantType) -> bool {
        species
            .leaf_years
            .is_none_or(|years| self.age - module.born <= years)
    }
}

/// Two quads crossed along `axis`, centred on `center`.
fn push_card(out: &mut MeshData, center: Vec3, axis: Vec3, size: f32) {
    let (b1, b2) = fabelgeist_tube::orthonormal_basis(axis.to_array());
    let (b1, b2) = (Vec3::from_array(b1), Vec3::from_array(b2));
    for side in [b1, b2] {
        let normal = side.cross(axis).normalize();
        let half = size * 0.5;
        let corners = [
            center - side * half - axis * half,
            center + side * half - axis * half,
            center + side * half + axis * half,
            center - side * half + axis * half,
        ];
        // Both faces, each with its own normal, so a card is lit from
        // either side.
        for facing in [normal, -normal] {
            let base = out.positions.len() as u32;
            for (k, c) in corners.iter().enumerate() {
                out.positions.push(c.to_array());
                out.normals.push(facing.to_array());
                out.tex_coords
                    .push([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]][k]);
            }
            let indices = out.indices.get_or_insert_with(Vec::new);
            if facing == normal {
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            } else {
                indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
            }
        }
    }
}

impl Ecosystem {
    /// All branch segments, grouped by species.
    pub fn branch_segments(&self) -> Vec<Vec<TubeSegment>> {
        (0..self.species.len())
            .into_par_iter()
            .map(|s| {
                let mut out = Vec::new();
                for plant in self.plants.iter().filter(|p| p.species as usize == s) {
                    plant.branch_segments(&self.prototypes, &mut out);
                }
                out
            })
            .collect()
    }

    /// Leaf cards, one mesh per species.
    pub fn leaf_meshes(&self) -> Vec<MeshData> {
        (0..self.species.len())
            .into_par_iter()
            .map(|s| {
                let mut out = MeshData {
                    indices: Some(Vec::new()),
                    ..MeshData::default()
                };
                for plant in self.plants.iter().filter(|p| p.species as usize == s) {
                    plant.leaf_cards(&self.species[s], &self.prototypes, &mut out);
                }
                out
            })
            .collect()
    }

    /// Branch meshes on the host, one per species.
    pub fn branch_meshes(&self, sides: u32) -> Vec<MeshData> {
        self.branch_segments()
            .par_iter()
            .map(|segments| fabelgeist_tube::mesh(segments, sides))
            .collect()
    }
}
