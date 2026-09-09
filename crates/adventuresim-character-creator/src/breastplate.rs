//! Maps the front torso into a stable semantic parameter domain for armor.

use std::collections::{BTreeMap, BTreeSet};

use adventuresim_armor_model::{
    SurfaceMorph, TORSO_SHOULDER_ENVELOPE_SAMPLES, TorsoClearanceMesh, TorsoClearancePose,
    TorsoCoronalAnchor, TorsoShoulderSample, TorsoSurface, TorsoUpperRigAnchors, TorsoVertex,
};

use crate::bracer::ForearmMorphSample;
mod clearance;
mod frame;
mod sections;
mod shoulders;
mod topology;
use frame::TorsoFrame;
use sections::triangle_lateral_crossings;

pub struct TorsoSurfaceInput<'a> {
    pub domain: &'a str,
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

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn length(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}
fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}
fn normalized(a: [f32; 3]) -> Result<[f32; 3], String> {
    let length = length(a);
    (length > 1e-8)
        .then(|| scale(a, length.recip()))
        .ok_or_else(|| "torso landmarks coincide".to_owned())
}

impl TorsoSurfaceInput<'_> {
    fn validate(&self) -> Result<(), String> {
        let count = self.positions.len();
        if self.domain.is_empty()
            || self.normals.len() != count
            || self.joint_indices.len() != count
            || self.joint_weights.len() != count
            || self.faces.len() != self.texcoord_faces.len()
            || self.joint_names.len() != self.global_joint_states.len()
            || self.morphs.iter().any(|morph| {
                morph.positions.len() != count
                    || morph.normals.len() != count
                    || morph.global_joint_states.len() != self.joint_names.len()
            })
        {
            return Err("front torso surface inputs are inconsistent".into());
        }
        Ok(())
    }
    fn support_joints(&self) -> BTreeSet<usize> {
        self.joint_names
            .iter()
            .enumerate()
            .filter_map(|(index, name)| {
                matches!(
                    name.as_str(),
                    "c_spine0"
                        | "root"
                        | "c_spine1"
                        | "c_spine2"
                        | "c_spine3"
                        | "c_neck"
                        | "l_clavicle"
                        | "r_clavicle"
                        | "l_uparm"
                        | "r_uparm"
                        | "l_upleg"
                        | "r_upleg"
                )
                .then_some(index)
            })
            .collect::<BTreeSet<_>>()
    }
    fn joint_weight(&self, vertex: usize, joints: &BTreeSet<usize>) -> f32 {
        self.joint_indices[vertex]
            .iter()
            .zip(self.joint_weights[vertex])
            .filter(|(joint, _)| joints.contains(&(**joint as usize)))
            .map(|(_, weight)| weight)
            .sum()
    }
}
pub fn build_front_torso_surface(input: TorsoSurfaceInput<'_>) -> Result<TorsoSurface, String> {
    input.validate()?;
    let frame = TorsoFrame::new(
        &input,
        input.positions,
        input.normals,
        input.global_joint_states,
    )?;
    let morph_frames = input
        .morphs
        .iter()
        .map(|morph| {
            TorsoFrame::new(
                &input,
                &morph.positions,
                &morph.normals,
                &morph.global_joint_states,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let section_faces = sections::section_faces(&input)?;
    let coronal_anchors = sections::CORONAL_LEVELS
        .into_iter()
        .zip(sections::coronal_depths(
            input.positions,
            &section_faces,
            frame,
        )?)
        .map(|(vertical, depth)| TorsoCoronalAnchor { vertical, depth })
        .collect();
    let morph_coronal_depths = input
        .morphs
        .iter()
        .zip(&morph_frames)
        .map(|(morph, frame)| sections::coronal_depths(&morph.positions, &section_faces, *frame))
        .collect::<Result<Vec<_>, _>>()?;
    let shoulder_faces = shoulders::shoulder_faces(&input, frame);
    let contour = shoulders::ShoulderContour {
        frame,
        faces: &shoulder_faces,
    };
    let (shoulder_envelope, signature) = contour.envelope(input.positions, input.normals, None)?;
    let morph_shoulder_envelopes = input
        .morphs
        .iter()
        .map(|morph| {
            contour
                .envelope(&morph.positions, &morph.normals, Some(&signature))
                .map(|(envelope, _)| envelope)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let clearance_mesh = clearance::build(&input, &shoulder_faces, &section_faces)?;
    let topology = topology::SurfaceTopology::new(&input, frame)?;
    let morphs = input
        .morphs
        .iter()
        .map(|morph| SurfaceMorph {
            name: morph.name.clone(),
            positions: topology
                .source
                .iter()
                .map(|(body, _)| morph.positions[*body])
                .collect(),
            normals: topology
                .source
                .iter()
                .map(|(body, _)| morph.normals[*body])
                .collect(),
        })
        .collect();
    let morph_semantic_coordinates = input
        .morphs
        .iter()
        .zip(&morph_frames)
        .map(|(morph, frame)| {
            topology
                .source
                .iter()
                .map(|(body, _)| frame.coordinates(morph.positions[*body]))
                .collect()
        })
        .collect();
    Ok(TorsoSurface {
        domain: input.domain.to_owned(),
        front: frame.front_axis,
        morph_fronts: morph_frames.iter().map(|frame| frame.front_axis).collect(),
        upper_rig_anchors: frame.anchors,
        morph_upper_rig_anchors: morph_frames.iter().map(|frame| frame.anchors).collect(),
        morph_semantic_coordinates,
        vertices: topology.vertices,
        faces: topology.faces,
        coronal_anchors,
        morph_coronal_depths,
        shoulder_envelope,
        morph_shoulder_envelopes,
        clearance_mesh,
        morphs,
    })
}
