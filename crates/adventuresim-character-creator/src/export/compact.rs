//! Retain only rendered shell vertices, with identical remapping for every channel.

use super::*;

struct Target {
    name: String,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
}

pub(super) struct CompactShell<'a> {
    source: &'a RiggedShell<'a>,
    plate_edges: Vec<[u32; 2]>,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    texcoords: Option<Vec<[f32; 2]>>,
    faces: Vec<[u32; 3]>,
    joints: Vec<[u32; 8]>,
    weights: Vec<[f32; 8]>,
    targets: Vec<Target>,
}

impl<'a> CompactShell<'a> {
    pub(super) fn new(body: &RiggedMesh<'_>, shell: &'a RiggedShell<'a>) -> Self {
        let used = shell
            .faces
            .iter()
            .flatten()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let mut remap = vec![0; shell.positions.len()];
        for (index, vertex) in used.iter().enumerate() {
            remap[*vertex as usize] = index as u32;
        }
        let pick = |values: &[[f32; 3]]| {
            used.iter()
                .map(|index| values[*index as usize])
                .collect::<Vec<_>>()
        };
        let joints = shell.joint_indices.unwrap_or(body.joint_indices);
        let weights = shell.joint_weights.unwrap_or(body.joint_weights);
        Self {
            source: shell,
            plate_edges: shell
                .plate_edges
                .iter()
                .filter(|edge| edge.iter().all(|i| used.contains(i)))
                .map(|edge| edge.map(|i| remap[i as usize]))
                .collect(),
            texcoords: shell
                .texcoords
                .map(|uv| used.iter().map(|i| uv[*i as usize]).collect()),
            positions: pick(shell.positions),
            normals: pick(shell.normals),
            faces: shell
                .faces
                .iter()
                .map(|face| face.map(|vertex| remap[vertex as usize]))
                .collect(),
            joints: used.iter().map(|index| joints[*index as usize]).collect(),
            weights: used.iter().map(|index| weights[*index as usize]).collect(),
            targets: shell
                .morph_targets
                .iter()
                .map(|target| Target {
                    name: target.name.to_owned(),
                    positions: pick(target.position_deltas),
                    normals: pick(target.normal_deltas),
                })
                .collect(),
        }
    }

    pub(super) fn targets(&self) -> Vec<RiggedMorphTarget<'_>> {
        self.targets
            .iter()
            .map(|target| RiggedMorphTarget {
                name: &target.name,
                position_deltas: &target.positions,
                normal_deltas: &target.normals,
            })
            .collect()
    }

    pub(super) fn rigged<'b>(&'b self, targets: &'b [RiggedMorphTarget<'b>]) -> RiggedShell<'b> {
        RiggedShell {
            plate_edges: &self.plate_edges,
            textures: self.source.textures,
            texcoords: self.texcoords.as_deref(),
            hinge: self.source.hinge,
            name: self.source.name,
            positions: &self.positions,
            normals: &self.normals,
            faces: &self.faces,
            joint_indices: Some(&self.joints),
            joint_weights: Some(&self.weights),
            morph_targets: targets,
            base_color: self.source.base_color,
            metallic: self.source.metallic,
            roughness: self.source.roughness,
        }
    }
}
