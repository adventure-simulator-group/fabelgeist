//! Adapts stable MHR forearm topology and anatomical UVs to the renderer-
//! independent parametric armor generator.

use std::collections::{BTreeMap, BTreeSet};

use adventuresim_armor_model::{AnatomicalSurface, SurfaceMorph, SurfaceVertex};

const FOREARM_WEIGHT_THRESHOLD: f32 = 0.35;
const AXIAL_SUPPORT_MARGIN: f32 = 0.1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForearmSide {
    Left,
    Right,
}

impl ForearmSide {
    fn lowarm(self) -> &'static str {
        match self {
            Self::Left => "l_lowarm",
            Self::Right => "r_lowarm",
        }
    }

    fn wrist(self) -> &'static str {
        match self {
            Self::Left => "l_wrist",
            Self::Right => "r_wrist",
        }
    }

    fn upperarm(self) -> &'static str {
        match self {
            Self::Left => "l_upperarm",
            Self::Right => "r_upperarm",
        }
    }

    fn owns_forearm_joint(self, name: &str) -> bool {
        name == self.lowarm() || name.starts_with(&format!("{}_twist", self.lowarm()))
    }

    fn supports_forearm_boundary(self, name: &str) -> bool {
        self.owns_forearm_joint(name)
            || name == self.wrist()
            || name == self.upperarm()
            || name.starts_with(&format!("{}_twist", self.upperarm()))
    }
}

pub struct ForearmSurfaceInput<'a> {
    pub domain: &'a str,
    pub side: ForearmSide,
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
    pub faces: &'a [[u32; 3]],
    pub texcoords: &'a [[f32; 2]],
    pub texcoord_faces: &'a [[u32; 3]],
    pub joint_indices: &'a [[u32; 8]],
    pub joint_weights: &'a [[f32; 8]],
    pub joint_names: &'a [String],
    pub global_joint_states: &'a [[f32; 8]],
    pub morphs: &'a [ForearmMorphSample],
}

pub struct ForearmMorphSample {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    /// Global rig state fitted for this exact body realization.  Garment
    /// semantic curves must be evaluated from these landmarks rather than
    /// translating neutral landmarks by a nearest-envelope approximation.
    pub global_joint_states: Vec<[f32; 8]>,
}

fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

impl ForearmSurfaceInput<'_> {
    fn validate(&self) -> Result<(), String> {
        let vertices = self.positions.len();
        if self.domain.trim().is_empty()
            || self.normals.len() != vertices
            || self.joint_indices.len() != vertices
            || self.joint_weights.len() != vertices
            || self.faces.len() != self.texcoord_faces.len()
            || self.joint_names.len() != self.global_joint_states.len()
            || self
                .morphs
                .iter()
                .any(|morph| morph.positions.len() != vertices || morph.normals.len() != vertices)
        {
            return Err("forearm surface inputs are inconsistent".into());
        }
        let forearm_joints = self
            .joint_names
            .iter()
            .enumerate()
            .filter_map(|(index, name)| self.side.owns_forearm_joint(name).then_some(index))
            .collect::<BTreeSet<_>>();
        if forearm_joints.is_empty() {
            return Err(format!("MHR rig has no {} skin joints", self.side.lowarm()));
        }
        Ok(())
    }
}

struct ForearmFrame {
    proximal: [f32; 3],
    axis: [f32; 3],
    axis_length_squared: f32,
}

impl ForearmFrame {
    fn new(input: &ForearmSurfaceInput<'_>) -> Result<Self, String> {
        let joint_position = |name: &str| {
            input
                .joint_names
                .iter()
                .position(|candidate| candidate == name)
                .map(|index| {
                    let state = input.global_joint_states[index];
                    [state[0], state[1], state[2]]
                })
                .ok_or_else(|| format!("MHR rig is missing {name}"))
        };
        let proximal = joint_position(input.side.lowarm())?;
        let distal = joint_position(input.side.wrist())?;
        let axis = subtract(distal, proximal);
        let axis_length_squared = dot(axis, axis);
        if axis_length_squared <= f32::EPSILON {
            return Err("MHR forearm landmarks coincide".into());
        }
        Ok(Self {
            proximal,
            axis,
            axis_length_squared,
        })
    }
    fn axial(&self, position: [f32; 3]) -> f32 {
        dot(subtract(position, self.proximal), self.axis) / self.axis_length_squared
    }
}

pub fn build_forearm_surface(input: ForearmSurfaceInput<'_>) -> Result<AnatomicalSurface, String> {
    input.validate()?;
    let support_joints = input
        .joint_names
        .iter()
        .enumerate()
        .filter_map(|(index, name)| input.side.supports_forearm_boundary(name).then_some(index))
        .collect::<BTreeSet<_>>();
    let frame = ForearmFrame::new(&input)?;
    let joint_weight = |vertex: usize, joints: &BTreeSet<usize>| {
        input.joint_indices[vertex]
            .iter()
            .zip(&input.joint_weights[vertex])
            .filter(|(joint, _)| joints.contains(&(**joint as usize)))
            .map(|(_, weight)| *weight)
            .sum::<f32>()
    };
    let supports_surface = |vertex: usize| {
        let axial = frame.axial(input.positions[vertex]);
        (-AXIAL_SUPPORT_MARGIN..=1.0 + AXIAL_SUPPORT_MARGIN).contains(&axial)
            && joint_weight(vertex, &support_joints) >= FOREARM_WEIGHT_THRESHOLD
    };

    let selected_faces = input
        .faces
        .iter()
        .copied()
        .zip(input.texcoord_faces.iter().copied())
        .filter(|(face, _)| {
            face.iter()
                .filter(|vertex| supports_surface(**vertex as usize))
                .count()
                >= 2
        })
        .collect::<Vec<_>>();
    if selected_faces.is_empty() {
        return Err("MHR forearm skin selected no faces".into());
    }

    // MHR positions may share a vertex across an atlas seam. Split by the
    // stable (body vertex, UV vertex) pair so every armor vertex has exactly
    // one canonical coordinate without changing the anatomical topology.
    let mut split = BTreeMap::<(u32, u32), u32>::new();
    let mut source_vertices = Vec::<(usize, usize)>::new();
    let mut faces = Vec::with_capacity(selected_faces.len());
    for (body_face, uv_face) in selected_faces {
        let mut face = [0; 3];
        for corner in 0..3 {
            let key = (body_face[corner], uv_face[corner]);
            face[corner] = *split.entry(key).or_insert_with(|| {
                source_vertices.push((key.0 as usize, key.1 as usize));
                (source_vertices.len() - 1) as u32
            });
        }
        faces.push(face);
    }
    let surface_vertices = source_vertices
        .iter()
        .map(|(body, uv)| {
            let texcoord =
                input.texcoords.get(*uv).copied().ok_or_else(|| {
                    "MHR anatomical UV references a missing coordinate".to_owned()
                })?;
            Ok(SurfaceVertex {
                uv: texcoord,
                // The joint landmarks define the parameter domain. Boundary
                // support geometry may extend past them, but is clamped so an
                // endpoint contour crosses a complete elbow or wrist section
                // instead of converging on a skin-weight extremum.
                axial: frame.axial(input.positions[*body]).clamp(0.0, 1.0),
                position: input.positions[*body],
                normal: input.normals[*body],
                joint_indices: input.joint_indices[*body],
                joint_weights: input.joint_weights[*body],
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let morphs = input
        .morphs
        .iter()
        .map(|morph| SurfaceMorph {
            name: morph.name.clone(),
            positions: source_vertices
                .iter()
                .map(|(body, _)| morph.positions[*body])
                .collect(),
            normals: source_vertices
                .iter()
                .map(|(body, _)| morph.normals[*body])
                .collect(),
        })
        .collect();
    Ok(AnatomicalSurface {
        domain: input.domain.to_owned(),
        vertices: surface_vertices,
        faces,
        morphs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_seams_split_shared_body_vertices() {
        let positions = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let normals = [[0.0, 0.0, 1.0]; 4];
        let faces = [[0, 1, 2], [1, 3, 2]];
        let texcoords = [
            [0.0, 0.0],
            [0.5, 0.0],
            [0.0, 1.0],
            [0.6, 0.0],
            [0.5, 1.0],
            [1.0, 1.0],
        ];
        let texcoord_faces = [[0, 1, 2], [3, 5, 4]];
        let joint_indices = [[0; 8]; 4];
        let joint_weights = [[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 4];
        let joint_names = vec!["l_lowarm".into(), "l_wrist".into()];
        let states = [
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0],
            [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0],
        ];
        let surface = build_forearm_surface(ForearmSurfaceInput {
            domain: "test",
            side: ForearmSide::Left,
            positions: &positions,
            normals: &normals,
            faces: &faces,
            texcoords: &texcoords,
            texcoord_faces: &texcoord_faces,
            joint_indices: &joint_indices,
            joint_weights: &joint_weights,
            joint_names: &joint_names,
            global_joint_states: &states,
            morphs: &[],
        })
        .unwrap();
        assert_eq!(surface.faces.len(), 2);
        assert_eq!(surface.vertices.len(), 6);
    }

    #[test]
    fn wrist_weights_complete_the_distal_forearm_boundary() {
        let positions = [
            [-1.0, 0.8, 0.0],
            [1.0, 0.8, 0.0],
            [-1.0, 1.05, 0.0],
            [1.0, 1.05, 0.0],
        ];
        let normals = [[0.0, 0.0, 1.0]; 4];
        let faces = [[0, 2, 3], [0, 3, 1]];
        let texcoords = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
        let joint_indices = [[0; 8], [0; 8], [1; 8], [1; 8]];
        let joint_weights = [[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 4];
        let joint_names = vec!["l_lowarm".into(), "l_wrist".into()];
        let states = [
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0],
            [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0],
        ];
        let surface = build_forearm_surface(ForearmSurfaceInput {
            domain: "test",
            side: ForearmSide::Left,
            positions: &positions,
            normals: &normals,
            faces: &faces,
            texcoords: &texcoords,
            texcoord_faces: &faces,
            joint_indices: &joint_indices,
            joint_weights: &joint_weights,
            joint_names: &joint_names,
            global_joint_states: &states,
            morphs: &[],
        })
        .unwrap();

        assert_eq!(surface.faces.len(), 2);
        assert_eq!(
            surface
                .vertices
                .iter()
                .filter(|vertex| vertex.axial == 1.0)
                .count(),
            2
        );
    }
}
