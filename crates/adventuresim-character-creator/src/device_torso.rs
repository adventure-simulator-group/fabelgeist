//! The breastplate fitted on the device: the front torso measured on the
//! wearer, then the breastplate fitted to it.
//!
//! One invocation orients the torso from its rig; the torso's half width is
//! the ninetieth percentile of the front's lateral extent, which one
//! invocation per candidate vertex finds by ranking it; one invocation per
//! body vertex then places it in the torso's semantic chart and decides
//! whether the front torso supports it. The shared anatomical selection
//! splits the supported faces at atlas seams, and the breastplate is fitted
//! to what it selected. Which faces count as torso for fitting depends only
//! on the rig's skin weights, so the host lists them.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, bail, ensure};
use fabelgeist_armor::gpu::anatomy::{
    DeviceSeams, DeviceSurface, STATUS_EMPTY_SELECTION, SeamTopology,
};
use fabelgeist_armor::gpu::body::{BodySurface, GpuBody};
use fabelgeist_armor::gpu::breastplate::{DeviceBreastplate, TORSO_RIG_WORDS, TorsoBody};
use fabelgeist_armor::gpu::device_error;
use fabelgeist_armor::{ArmorGpu, BreastplateDesign, GeneratedArmor};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use crate::bracer::ForearmMorphSample;
use crate::device_torso_wgsl::{FRAME, HALF_WIDTH, RANK, SUPPORT, WIDTHS, torso_source};

/// The wearer's body, as the breastplate is fitted to it.
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

/// Floats of the torso frame: origin at the lower spine, the vertical axis
/// and its extent to the neck, the lateral and anterior axes, and the front
/// half width.
const TORSO_FRAME_WORDS: u64 = 16;
/// Set when the rig's landmarks coincide.
pub(crate) const STATUS_COINCIDENT_LANDMARKS: u32 = 32;
/// Set when no front vertex measures the torso's width.
pub(crate) const STATUS_NO_WIDTH: u32 = 64;
/// Set when the torso's measured width is degenerate.
pub(crate) const STATUS_DEGENERATE_WIDTH: u32 = 128;

/// Rig landmarks the torso frame reads, in the frame kernel's order.
const LANDMARKS: [&str; 9] = [
    "c_spine0",
    "c_neck",
    "l_clavicle",
    "r_clavicle",
    "l_uparm",
    "r_uparm",
    "c_head",
    "l_eye",
    "r_eye",
];

/// Joints whose skin supports the front torso.
const SUPPORT_JOINTS: [&str; 12] = [
    "c_spine0",
    "root",
    "c_spine1",
    "c_spine2",
    "c_spine3",
    "c_neck",
    "l_clavicle",
    "r_clavicle",
    "l_uparm",
    "r_uparm",
    "l_upleg",
    "r_upleg",
];

/// Joints of the torso proper, for the faces plates are fitted against.
const TORSO_JOINTS: [&str; 8] = [
    "root",
    "c_spine0",
    "c_spine1",
    "c_spine2",
    "c_spine3",
    "c_neck",
    "l_clavicle",
    "r_clavicle",
];
/// Name fragments of the limb joints whose skin is not torso.
const LIMB_JOINTS: [&str; 6] = ["uparm", "loarm", "hand", "upleg", "loleg", "foot"];
/// A fitting face's least mean torso weight, and its greatest mean limb
/// weight.
const TORSO_FACE_WEIGHT: f32 = 0.08;
const LIMB_FACE_WEIGHT: f32 = 0.35;

impl TorsoSurfaceInput<'_> {
    fn check(&self) -> Result<()> {
        let count = self.positions.len();
        ensure!(
            !self.domain.is_empty()
                && self.normals.len() == count
                && self.joint_indices.len() == count
                && self.joint_weights.len() == count
                && self.faces.len() == self.texcoord_faces.len()
                && self.joint_names.len() == self.global_joint_states.len()
                && self.morphs.iter().all(|morph| {
                    morph.positions.len() == count
                        && morph.normals.len() == count
                        && morph.global_joint_states.len() == self.joint_names.len()
                }),
            "front torso surface inputs are inconsistent"
        );
        Ok(())
    }

    fn joints_named(&self, keep: impl Fn(&str) -> bool) -> BTreeSet<usize> {
        self.joint_names
            .iter()
            .enumerate()
            .filter_map(|(index, name)| keep(name).then_some(index))
            .collect()
    }

    fn set_weight(&self, vertex: usize, joints: &BTreeSet<usize>) -> f32 {
        self.joint_indices[vertex]
            .iter()
            .zip(self.joint_weights[vertex])
            .filter(|(joint, _)| joints.contains(&(**joint as usize)))
            .map(|(_, weight)| weight)
            .sum::<f32>()
    }

    /// The rig joints of [`LANDMARKS`].
    fn landmarks(&self) -> Result<Vec<u32>> {
        LANDMARKS
            .iter()
            .map(|name| {
                self.joint_names
                    .iter()
                    .position(|candidate| candidate == name)
                    .map(|index| index as u32)
                    .with_context(|| format!("MHR rig is missing {name}"))
            })
            .collect()
    }

    /// The body face each torso face samples skin from: the last body face
    /// with its corners.
    fn eligible(&self, torso_faces: &[[u32; 3]]) -> Vec<u32> {
        let face_index = self
            .faces
            .iter()
            .enumerate()
            .map(|(index, face)| (*face, index as u32))
            .collect::<BTreeMap<_, _>>();
        torso_faces.iter().map(|face| face_index[face]).collect()
    }

    /// Faces whose skin is mostly torso and little limb.
    fn torso_faces(&self) -> Result<Vec<[u32; 3]>> {
        let torso = self.joints_named(|name| TORSO_JOINTS.contains(&name));
        let limbs = self.joints_named(|name| LIMB_JOINTS.iter().any(|limb| name.contains(limb)));
        let mean = |face: &[u32; 3], joints: &BTreeSet<usize>| {
            face.iter()
                .map(|vertex| self.set_weight(*vertex as usize, joints))
                .sum::<f32>()
                / 3.0
        };
        let faces = self
            .faces
            .iter()
            .copied()
            .filter(|face| {
                mean(face, &torso) >= TORSO_FACE_WEIGHT && mean(face, &limbs) <= LIMB_FACE_WEIGHT
            })
            .collect::<Vec<_>>();
        ensure!(!faces.is_empty(), "torso section face selection is empty");
        Ok(faces)
    }
}

/// Fit `design` to the wearer and each morph sample of `input` on `gpu`.
pub fn generate_breastplate_on_device(
    gpu: &ArmorGpu,
    design: &BreastplateDesign,
    input: TorsoSurfaceInput<'_>,
) -> Result<GeneratedArmor> {
    pollster::block_on(generate_breastplate_on_device_async(gpu, design, input))
}

pub async fn generate_breastplate_on_device_async(
    gpu: &ArmorGpu,
    design: &BreastplateDesign,
    input: TorsoSurfaceInput<'_>,
) -> Result<GeneratedArmor> {
    input.check()?;
    let landmarks = input.landmarks()?;
    let support_joints = input.joints_named(|name| SUPPORT_JOINTS.contains(&name));
    let supports = (0..input.joint_names.len())
        .map(|joint| u32::from(support_joints.contains(&joint)))
        .collect::<Vec<_>>();
    let torso_faces = input.torso_faces()?;
    let eligible = input.eligible(&torso_faces);

    let vertex_count = input.positions.len();
    let unused_texcoords = vec![[0.0f32; 2]; vertex_count];
    let body = GpuBody::new(
        gpu,
        BodySurface {
            positions: input.positions,
            normals: input.normals,
            faces: input.faces,
            texcoords: &unused_texcoords,
            joint_indices: input.joint_indices,
            joint_weights: input.joint_weights,
            joints: input.global_joint_states,
        },
    )?;
    let seams = DeviceSeams::new(gpu, &SeamTopology::new(input.faces, input.texcoord_faces)?)?;
    let atlas = gpu.upload(input.texcoords)?;
    let atlas_faces = gpu.upload(input.texcoord_faces)?;
    let status = gpu.scratch(4, "breastplate status")?;

    let mut batch = gpu.batch("breastplate");
    let rig = gpu.scratch(TORSO_RIG_WORDS as u64 * 4, "torso rig")?;
    let semantic = gpu.scratch(vertex_count as u64 * 8, "torso coordinates")?;
    let support = gpu.scratch(vertex_count as u64 * 4, "front torso support")?;
    record_torso(
        gpu,
        &mut batch,
        &body,
        &landmarks,
        &gpu.upload(&supports)?,
        TorsoOutputs {
            rig: &rig,
            semantic: &semantic,
            support: &support,
            status: &status,
        },
    )?;
    let surface = DeviceSurface::record(gpu, &mut batch, &seams, &body.faces, &support, &status)?;
    let mut breastplate = DeviceBreastplate::record(
        gpu,
        &mut batch,
        design,
        TorsoBody {
            body: &body,
            surface: &surface,
            semantic: &semantic,
            rig: &rig,
            atlas: &atlas,
            atlas_faces: &atlas_faces,
            torso_faces: &torso_faces,
            eligible: &eligible,
        },
        &status,
    )?;
    batch.submit();
    for morph in input.morphs {
        let positions = gpu.upload(&morph.positions)?;
        let mut batch = gpu.batch("breastplate morph");
        breastplate.record_morph(gpu, &mut batch, &positions)?;
        batch.submit();
    }

    torso_failure(gpu.read_async::<u32>(&status).await?[0])?;
    let names = input
        .morphs
        .iter()
        .map(|morph| morph.name.clone())
        .collect::<Vec<_>>();
    let mut armor = breastplate.read_async(gpu, input.domain, &names).await?;
    crate::skin_rules::breastplate(input.joint_names, input.global_joint_states, &mut armor)?;
    Ok(armor)
}

/// The torso's failure, if its status bits hold one.
fn torso_failure(bits: u32) -> Result<()> {
    for (bit, message) in [
        (STATUS_COINCIDENT_LANDMARKS, "torso landmarks coincide"),
        (STATUS_NO_WIDTH, "front torso has no width samples"),
        (STATUS_DEGENERATE_WIDTH, "front torso width is degenerate"),
        (STATUS_EMPTY_SELECTION, "front torso selection is empty"),
    ] {
        if bits & bit != 0 {
            bail!(message);
        }
    }
    Ok(())
}

/// Where the torso's measurement goes.
struct TorsoOutputs<'a> {
    rig: &'a Buffer,
    semantic: &'a Buffer,
    support: &'a Buffer,
    status: &'a Buffer,
}

/// Record the torso frame, its half width, and each body vertex's
/// semantic coordinates and support.
fn record_torso(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    body: &GpuBody,
    landmarks: &[u32],
    supports: &Buffer,
    outputs: TorsoOutputs,
) -> Result<()> {
    let frame = gpu.scratch(TORSO_FRAME_WORDS * 4, "torso frame")?;
    let widths = gpu.scratch(body.vertex_count as u64 * 4, "front widths")?;
    let counter = gpu.scratch(4, "front width count")?;
    let mut parameters = PassParameters::new();
    parameters.insert("count", body.vertex_count);
    for (slot, joint) in landmarks.iter().enumerate() {
        parameters.insert(format!("joint{slot}"), *joint);
    }
    for pad in 0..6 {
        parameters.insert(format!("pad{pad}"), 0u32);
    }
    let buffers = [
        ("positions", &body.positions),
        ("normals", &body.normals),
        ("joints", &body.joints),
        ("joint_indices", &body.joint_indices),
        ("joint_weights", &body.joint_weights),
        ("supports", supports),
        ("frame", &frame),
        ("rig", outputs.rig),
        ("widths", &widths),
        ("counter", &counter),
        ("semantic", outputs.semantic),
        ("support", outputs.support),
        ("status", outputs.status),
    ];
    for (entry, groups) in [
        (FRAME, Some(1)),
        (WIDTHS, None),
        (RANK, None),
        (HALF_WIDTH, Some(1)),
        (SUPPORT, None),
    ] {
        let source = torso_source(entry);
        let kernel = gpu
            .cache()
            .get(gpu.context(), &source)
            .map_err(device_error)?;
        let mut bound = parameters.clone();
        for (name, buffer) in buffers {
            if source.contains(&format!("> {name}:")) {
                bound.insert(name, buffer.clone());
            }
        }
        match groups {
            Some(groups) => batch.dispatch(&kernel, &bound, [groups, 1, 1]),
            None => batch.dispatch_items(&kernel, &bound, body.vertex_count),
        }
        .map_err(device_error)?;
    }
    Ok(())
}
