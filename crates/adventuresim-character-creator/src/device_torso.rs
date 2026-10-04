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

use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_gpu::prelude::PassParameterName;
use fabelgeist_rig::{
    RigJointLookupError, RigJointMembership, RigJointName, RigJointOrdinal, RigJointPart,
};
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, bail, ensure};
use fabelgeist_armor::gpu::anatomy::{
    DeviceSeams, DeviceSurface, STATUS_EMPTY_SELECTION, SeamTopology,
};
use fabelgeist_armor::gpu::body::{BodySurface, GpuBody};
use fabelgeist_armor::gpu::breastplate::{DeviceBreastplate, TORSO_RIG_WORDS, TorsoBody};

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
    pub joint_names: &'a [RigJointName],
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
const LANDMARKS: [RigJointName; 9] = [
    RigJointName::C_SPINE0,
    RigJointName::C_NECK,
    RigJointName::L_CLAVICLE,
    RigJointName::R_CLAVICLE,
    RigJointName::L_UPARM,
    RigJointName::R_UPARM,
    RigJointName::C_HEAD,
    RigJointName::L_EYE,
    RigJointName::R_EYE,
];

/// Joints whose skin supports the front torso.
const SUPPORT_JOINTS: [RigJointName; 12] = [
    RigJointName::C_SPINE0,
    RigJointName::ROOT,
    RigJointName::C_SPINE1,
    RigJointName::C_SPINE2,
    RigJointName::C_SPINE3,
    RigJointName::C_NECK,
    RigJointName::L_CLAVICLE,
    RigJointName::R_CLAVICLE,
    RigJointName::L_UPARM,
    RigJointName::R_UPARM,
    RigJointName::L_UPLEG,
    RigJointName::R_UPLEG,
];

/// Joints of the torso proper, for the faces plates are fitted against.
const TORSO_JOINTS: [RigJointName; 8] = [
    RigJointName::ROOT,
    RigJointName::C_SPINE0,
    RigJointName::C_SPINE1,
    RigJointName::C_SPINE2,
    RigJointName::C_SPINE3,
    RigJointName::C_NECK,
    RigJointName::L_CLAVICLE,
    RigJointName::R_CLAVICLE,
];
/// Name fragments of the limb joints whose skin is not torso.
const LIMB_JOINTS: [RigJointPart; 6] = [
    RigJointPart::Uparm,
    RigJointPart::Loarm,
    RigJointPart::Hand,
    RigJointPart::Upleg,
    RigJointPart::Loleg,
    RigJointPart::Foot,
];
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

    fn joints_named(
        &self,
        keep: impl Fn(&RigJointName) -> RigJointMembership,
    ) -> BTreeSet<RigJointOrdinal> {
        self.joint_names
            .iter()
            .enumerate()
            .filter_map(
                |(index, name): (usize, &RigJointName)| -> Option<RigJointOrdinal> {
                    (keep(name) == RigJointMembership::Included)
                        .then_some(RigJointOrdinal::from(index))
                },
            )
            .collect()
    }

    fn set_weight(&self, vertex: usize, joints: &BTreeSet<RigJointOrdinal>) -> f32 {
        self.joint_indices[vertex]
            .iter()
            .zip(self.joint_weights[vertex])
            .filter(|(joint, _)| joints.contains(&RigJointOrdinal::from(**joint)))
            .map(|(_, weight)| weight)
            .sum::<f32>()
    }

    /// The rig joints of [`LANDMARKS`].
    fn landmarks(&self) -> std::result::Result<Vec<RigJointOrdinal>, RigJointLookupError> {
        LANDMARKS
            .iter()
            .map(
                |name: &RigJointName| -> std::result::Result<RigJointOrdinal, RigJointLookupError> {
                    name.require_in(self.joint_names)
                },
            )
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
        let torso = self.joints_named(|name: &RigJointName| -> RigJointMembership {
            RigJointMembership::from(TORSO_JOINTS.contains(name))
        });
        let limbs = self.joints_named(|name: &RigJointName| -> RigJointMembership {
            RigJointMembership::from(LIMB_JOINTS.iter().any(|limb: &RigJointPart| -> bool {
                name.contains_part(*limb) == RigJointMembership::Included
            }))
        });
        let mean = |face: &[u32; 3], joints: &BTreeSet<RigJointOrdinal>| {
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
    if let Some(option) = design.device_unsupported() {
        return Err(fabelgeist_armor::GenerateError::NotOnDevice(option).into());
    }
    input.check()?;
    let landmarks = input.landmarks()?;
    let support_joints = input.joints_named(|name: &RigJointName| -> RigJointMembership {
        RigJointMembership::from(SUPPORT_JOINTS.contains(name))
    });
    let supports = (0..input.joint_names.len())
        .map(|joint: usize| -> RigJointMembership {
            RigJointMembership::from(support_joints.contains(&RigJointOrdinal::from(joint)))
        })
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
    let atlas = gpu.upload(BufferUpload::from_elements(input.texcoords))?;
    let atlas_faces = gpu.upload(BufferUpload::from_elements(input.texcoord_faces))?;
    let status = gpu.scratch((4u64).into(), ("breastplate status").into())?;

    let mut batch = gpu.batch(("breastplate").into());
    let rig = gpu.scratch((TORSO_RIG_WORDS as u64 * 4).into(), ("torso rig").into())?;
    let semantic = gpu.scratch(
        (vertex_count as u64 * 8).into(),
        ("torso coordinates").into(),
    )?;
    let support = gpu.scratch(
        (vertex_count as u64 * 4).into(),
        ("front torso support").into(),
    )?;
    record_torso(
        gpu,
        &mut batch,
        &body,
        &landmarks,
        &gpu.upload(BufferUpload::from_elements(
            &supports.into_iter().map(u32::from).collect::<Vec<_>>(),
        ))?,
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
        let positions = gpu.upload(BufferUpload::from_elements(&morph.positions))?;
        let mut batch = gpu.batch(("breastplate morph").into());
        breastplate.record_morph(gpu, &mut batch, &positions)?;
        batch.submit();
    }

    torso_failure(gpu.read::<u32>(&status)?[0])?;
    let names = input
        .morphs
        .iter()
        .map(|morph| morph.name.clone())
        .collect::<Vec<_>>();
    Ok(breastplate.read(gpu, input.domain, &names)?)
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
    landmarks: &[RigJointOrdinal],
    supports: &Buffer,
    outputs: TorsoOutputs,
) -> Result<()> {
    let frame = gpu.scratch((TORSO_FRAME_WORDS * 4).into(), ("torso frame").into())?;
    let widths = gpu.scratch(
        (body.vertex_count as u64 * 4).into(),
        ("front widths").into(),
    )?;
    let counter = gpu.scratch((4u64).into(), ("front width count").into())?;
    let mut parameters = PassParameters::new();
    parameters.insert("count".into(), (body.vertex_count).into());
    for (slot, joint) in landmarks.iter().enumerate() {
        parameters.insert(
            PassParameterName::from(format!("joint{slot}")),
            (usize::from(*joint) as u32).into(),
        );
    }
    for pad in 0..6 {
        parameters.insert(PassParameterName::from(format!("pad{pad}")), (0u32).into());
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
            .map_err(fabelgeist_armor::GenerateError::from)?;
        let mut bound = parameters.clone();
        for (name, buffer) in buffers {
            if matches!(
                source.binding_marker(&fabelgeist_gpu::prelude::ShaderBindingName::from(name)),
                fabelgeist_gpu::prelude::ShaderBindingMarker::Present
            ) {
                bound.insert(name.into(), buffer.clone().into());
            }
        }
        match groups {
            Some(groups) => batch.dispatch(&kernel, &bound, ([groups, 1, 1]).into()),
            None => batch.dispatch_items(&kernel, &bound, (body.vertex_count).into()),
        }
        .map_err(fabelgeist_armor::GenerateError::from)?;
    }
    Ok(())
}
