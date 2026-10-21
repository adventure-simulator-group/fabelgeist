//! Garment armor fitted on the device.
//!
//! Every garment keeps its measurements in one device buffer, its *fit*: the
//! part frame first (fifteen floats, then a failure flag), then whatever
//! sections, cages and sewing rings its fitter measured from the wearer. Shape
//! kernels read the frame where the fit was written; fitting kernels read the
//! rest. Nothing is read back until the finished part is.

use crate::armor_frames::FittingPlacement;
use crate::armor_frames::FitRegion;
use crate::armor_layer::ArmorLayerSurface;
use crate::device_frames::{DeviceFrame, DeviceWearer};
use crate::device_garment_kernel::{
    Access, Bound, Grid, Word, atomic, dispatch, read, read_u32, write,
};
use crate::device_piece::DeviceRecording;
use crate::rig_parameters::RigJointParameter;
use anyhow::Result;
use fabelgeist_armor::{DevicePart, GarmentArmorDesign, GarmentArmorKind as Kind};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;
use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_rig::{RigJointLookupError, RigJointName, RigJointOrdinal};
use fabelgeist_geometry::SignedLength;
use fabelgeist_math::Ratio;

pub(crate) use candidates::BAND;

mod candidates;

/// Words of the part frame, failure flag included, at the start of a fit.
pub(crate) const FRAME_WORDS: u32 = 16;

/// A joint's position, read from the rig.
pub(crate) const JOINTS: &str = r#"
fn joint(index: u32) -> vec3<f32> {
    return vec3<f32>(joints[index * 8u], joints[index * 8u + 1u], joints[index * 8u + 2u]);
}
"#;

const SUPPORT: &str = r#"

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    var bits = 0u;
    for (var bit = 0u; bit < params.regions; bit = bit + 1u) {
        var weight = 0.0;
        for (var k = 0u; k < 8u; k = k + 1u) {
            if ((owned[joint_indices[i * 8u + k]] & (1u << bit)) != 0u) {
                weight = weight + joint_weights[i * 8u + k];
            }
        }
        if (weight >= SKIN_SUPPORT_THRESHOLD) {
            bits = bits | (1u << bit);
        }
    }
    support[i] = bits;
}
"#;

/// An upright garment frame: centred on the region frame between the span's
/// top and bottom, turned to the head's heading with a vertical up axis, and
/// as broad and deep as the region frame. The fit fails unless the frame's
/// words are finite, its axes orthonormal and its half extents positive.
const UPRIGHT: &str = r#"
fn vector(source: u32, at: u32) -> vec3<f32> {
    if (source == 0u) {
        return vec3<f32>(region[at], region[at + 1u], region[at + 2u]);
    }
    return vec3<f32>(head[at], head[at + 1u], head[at + 2u]);
}

fn joint(index: u32) -> vec3<f32> {
    return vec3<f32>(joints[index * 8u], joints[index * 8u + 1u], joints[index * 8u + 2u]);
}

@compute @workgroup_size(1)
fn main() {
    let top = joint(params.top_joint).y + params.top_offset;
    let hip = joint(params.bottom_joint).y;
    let bottom = hip - (hip - joint(params.reach_joint).y) * params.reach - params.bottom_offset;
    let origin = vector(0u, 0u);
    let words = array<f32, 15>(
        origin.x, (top + bottom) * 0.5, origin.z,
        head[3], head[4], head[5],
        0.0, 1.0, 0.0,
        head[9], head[10], head[11],
        region[12], (top - bottom) * 0.5, region[14],
    );
    var valid = true;
    for (var w = 0u; w < 15u; w = w + 1u) {
        fit[w] = words[w];
        if (!(abs(words[w]) <= INFINITY)) {
            valid = false;
        }
    }
    let f = frame_at(0u);
    let axes = array<vec3<f32>, 3>(f.x, f.y, f.z);
    for (var a = 0u; a < 3u; a = a + 1u) {
        for (var b = 0u; b < 3u; b = b + 1u) {
            let expected = select(0.0, 1.0, a == b);
            if (!(abs(host_dot(axes[a], axes[b]) - expected) < 0.001)) {
                valid = false;
            }
        }
        if (!(f.half_extents[a] > 0.0)) {
            valid = false;
        }
    }
    fit[FIT_FAILED] = select(1.0, 0.0, valid);
}
"#;

const FIT_STATUS: &str = r#"
@compute @workgroup_size(1)
fn main() {
    if (fit[FIT_FAILED] != 0.0) {
        atomicOr(&status[0], 1u);
    }
}
"#;

/// Where an upright frame's top and bottom lie: the top is a joint's height
/// plus an offset; the bottom lies `reach` of the way from a joint down to a
/// second joint's height, less an offset.
#[derive(Clone)]
pub(crate) struct UprightSpan {
    pub top: JointHeightAnchor,
    pub bottom: JointHeightAnchor,
    pub reach: JointReachTarget,
}

/// A model-space joint height and its signed continuous metre offset.
#[derive(Clone)]
pub(crate) struct JointHeightAnchor {
    pub joint: RigJointName,
    pub offset: SignedLength,
}

/// Dimensionless extension from the bottom joint toward an authored joint.
/// The ratio is unrestricted; the upright fit validates its resulting frame.
#[derive(Clone)]
pub(crate) struct JointReachTarget {
    pub joint: RigJointName,
    pub coefficient: Ratio,
}

impl DeviceWearer<'_> {
    /// A joint's index in the rig.
    pub(crate) fn joint_slot(
        &self,
        name: &RigJointName,
    ) -> std::result::Result<RigJointOrdinal, RigJointLookupError> {
        name.require_in(self.host.joint_names)
    }

    /// Record which regions' skin each body vertex supports: bit `i` is set
    /// when the vertex is in `support_indices(regions[i])`.
    pub(crate) fn record_region_support(
        &self,
        batch: &mut KernelBatch,
        regions: &[FitRegion],
    ) -> Result<Buffer> {
        let mut owned: fabelgeist_rig::RigJointTable<u32> =
            vec![0; self.host.joint_names.len()].into();
        for (bit, region) in regions.iter().enumerate() {
            let owners = region.owners();
            for (mask, owns) in owned.iter_mut().zip(self.host.owned_joints(&owners)) {
                *mask |= u32::from(owns) << bit;
            }
        }
        let gpu = self.gpu;
        let owned = gpu.upload(BufferUpload::from_elements(owned.as_native_slice()))?;
        let support = gpu.scratch(self.body.vertex_count as u64 * 4, "garment support")?;
        dispatch(
            self,
            batch,
            &format!("{}{SUPPORT}", crate::armor_frames::skin_support_wgsl()),
            &[
                read_u32("joint_indices", &self.body.joint_indices),
                read("joint_weights", &self.body.joint_weights),
                read_u32("owned", &owned),
                Bound {
                    name: "support",
                    kind: Access::WriteU32,
                    buffer: &support,
                },
            ],
            &[
                Word::U("count", self.body.vertex_count),
                Word::U("regions", regions.len() as u32),
            ],
            Grid::Items(self.body.vertex_count),
        )?;
        Ok(support)
    }

    /// Record an upright garment frame into the start of a new fit buffer of
    /// `words` floats: the region frame's centre and breadth, the head's
    /// heading, and a height between two landmarks.
    pub(crate) fn record_upright(
        &self,
        batch: &mut KernelBatch,
        words: u32,
        region: &DeviceFrame,
        head: &DeviceFrame,
        span: UprightSpan,
    ) -> Result<Buffer> {
        let fit = self.gpu.scratch(u64::from(words) * 4, "garment fit")?;
        dispatch(
            self,
            batch,
            UPRIGHT,
            &[
                read("joints", &self.body.joints),
                read("region", &region.frame),
                read("head", &head.frame),
                write("fit", &fit),
            ],
            &[
                Word::Joint(RigJointParameter::TopJoint, self.joint_slot(&span.top.joint)?),
                Word::Joint(
                    RigJointParameter::BottomJoint,
                    self.joint_slot(&span.bottom.joint)?,
                ),
                Word::Joint(
                    RigJointParameter::ReachJoint,
                    self.joint_slot(&span.reach.joint)?,
                ),
                Word::F("top_offset", span.top.offset.metres()),
                Word::F("bottom_offset", span.bottom.offset.metres()),
                Word::F("reach", span.reach.coefficient.ratio()),
            ],
            Grid::Singles(1),
        )?;
        Ok(fit)
    }

    /// Record a fitted garment, short of thickening.
    pub fn record_fitted_garment(
        &self,
        batch: &mut KernelBatch,
        design: &GarmentArmorDesign,
        placement: FittingPlacement,
        layers: &[ArmorLayerSurface<'_>],
    ) -> Result<DeviceRecording> {
        if matches!(
            design.plate_shape,
            fabelgeist_armor::GarmentPlateShape::WrappedTassets(_)
        ) {
            return self.record_wrapped_tassets(batch, design, layers, None);
        }
        match design.kind {
            Kind::Brigandine | Kind::JackOfPlates | Kind::MailShirt | Kind::ArmingDoublet => {
                self.record_fitted_torso(batch, design)
            }
            Kind::MailSleeve | Kind::QuiltedSleeve | Kind::MailChausses | Kind::PaddedChausses => {
                self.record_fitted_tube(batch, design, placement.side()?)
            }
            Kind::MailSkirt | Kind::PaddedSkirt | Kind::Fauld | Kind::Tassets => {
                self.record_fitted_skirt(batch, design)
            }
            Kind::Gorget => self.record_fitted_gorget(batch, design, layers),
        }
    }

    /// Record the propagation of a fit's failure into a part's status.
    pub(crate) fn record_fit_status(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        part: &DevicePart,
    ) -> Result<()> {
        dispatch(
            self,
            batch,
            FIT_STATUS,
            &[read("fit", fit), atomic("status", part.status())],
            &[],
            Grid::Singles(1),
        )
    }
}
