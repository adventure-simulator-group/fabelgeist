//! Split anatomical UV seams while preserving body-to-surface correspondence.

use super::*;

pub(super) struct SurfaceTopology {
    pub(super) vertices: Vec<TorsoVertex>,
    pub(super) faces: Vec<[u32; 3]>,
    pub(super) source: Vec<(usize, usize)>,
}
impl SurfaceTopology {
    pub(super) fn new(input: &TorsoSurfaceInput<'_>, frame: TorsoFrame) -> Result<Self, String> {
        let support_joints = input.support_joints();
        let supported = |vertex: usize| {
            let [lateral, vertical] = frame.coordinates(input.positions[vertex]);
            lateral.abs() <= 1.35
                && (-0.30..=1.08).contains(&vertical)
                && dot(input.normals[vertex], frame.front_axis) > 0.02
                && input.joint_weight(vertex, &support_joints) >= 0.2
        };
        let selected = input
            .faces
            .iter()
            .copied()
            .zip(input.texcoord_faces.iter().copied())
            .filter(|(face, _)| {
                face.iter()
                    .filter(|vertex| supported(**vertex as usize))
                    .count()
                    >= 2
            })
            .collect::<Vec<_>>();
        if selected.is_empty() {
            return Err("front torso selection is empty".into());
        }
        let mut split = BTreeMap::<(u32, u32), u32>::new();
        let mut source = Vec::<(usize, usize)>::new();
        let mut faces = Vec::with_capacity(selected.len());
        for (body_face, uv_face) in selected {
            let mut face = [0; 3];
            for corner in 0..3 {
                let key = (body_face[corner], uv_face[corner]);
                face[corner] = *split.entry(key).or_insert_with(|| {
                    source.push((key.0 as usize, key.1 as usize));
                    (source.len() - 1) as u32
                });
            }
            faces.push(face);
        }
        let vertices = source
            .iter()
            .map(|(body, uv)| {
                let [lateral, vertical] = frame.coordinates(input.positions[*body]);
                Ok(TorsoVertex {
                    uv: *input
                        .texcoords
                        .get(*uv)
                        .ok_or_else(|| "torso UV references a missing coordinate".to_owned())?,
                    lateral,
                    vertical,
                    position: input.positions[*body],
                    normal: input.normals[*body],
                    joint_indices: input.joint_indices[*body],
                    joint_weights: input.joint_weights[*body],
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self {
            vertices,
            faces,
            source,
        })
    }
}
