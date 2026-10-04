//! Part frames fitted on the device.
//!
//! A frame is oriented by rig landmarks and sized by the skin those
//! landmarks own. The device does both: a reduction finds the body's top and
//! floor where a region needs them, one invocation orients the frame, an
//! atomic reduction bounds the owned skin in the frame, and one invocation
//! finishes the frame for its region. The frame never leaves the device; the
//! chart kernels read it where it was written.

use anyhow::Result;
use fabelgeist_armor::ArmorGpu;
use fabelgeist_armor::gpu::Staging;
use fabelgeist_armor::gpu::body::{BodySurface, GpuBody};
use fabelgeist_gpu::prelude::BufferUpload;

use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::PassParameterName;
use fabelgeist_gpu::prelude::{Buffer, PassParameters, ReadbackSlot, ReadbackStatusWord};
use fabelgeist_rig::{RigJointMembership, RigJointName};

use crate::armor_frames::{FitRegion, Side, Wearer};

mod landmarks;
mod passes;
#[cfg(test)]
mod tests;
use landmarks::Landmarks;
use passes::{ORDERED_NEGATIVE_INFINITY, ORDERED_POSITIVE_INFINITY, kernels};

/// Floats in a device frame: origin, three axes, half extents, then the
/// landmark span its extents were measured along.
pub const FRAME_WORDS: u64 = 16;

/// A frame on the device.
#[derive(Clone, Debug)]
pub struct DeviceFrame {
    /// [`FRAME_WORDS`] floats; the first fifteen are a part frame.
    pub frame: Buffer,
    /// Frame validity; nonzero is a [`FrameFailure`] bit set.
    pub status: Buffer,
}

/// Why a frame could not be fitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameFailure {
    CoincidentLandmarks = 1,
    NoEnvelope = 2,
    Invalid = 4,
}

impl DeviceFrame {
    /// Read the fitted frame's status back. Stalls on the device.
    pub fn check(&self, gpu: &ArmorGpu, region: FitRegion) -> Result<()> {
        let word = gpu
            .read::<u32>(&self.status)?
            .first()
            .copied()
            .ok_or(fabelgeist_gpu::prelude::ReadbackError::EmptyStatus)?;
        Self::check_status(ReadbackStatusWord::from(word), region)
    }

    /// Stage the frame's status for a shared readback.
    pub fn stage<'a>(&'a self, staging: &mut Staging<'a>) -> ReadbackSlot {
        staging.stage(&self.status)
    }

    /// Turn a frame's status into the failure it records.
    pub fn check_status(status: ReadbackStatusWord, region: FitRegion) -> Result<()> {
        for failure in [
            FrameFailure::CoincidentLandmarks,
            FrameFailure::NoEnvelope,
            FrameFailure::Invalid,
        ] {
            if u32::from(status) & failure as u32 != 0 {
                anyhow::bail!("{region:?} frame: {}", failure.message());
            }
        }
        Ok(())
    }

    /// Read the frame back, for the stages that use it on the host.
    pub fn read(&self, gpu: &ArmorGpu) -> Result<fabelgeist_armor::PartFrame> {
        let words: Vec<f32> = gpu.read(&self.frame)?;
        Ok(fabelgeist_armor::PartFrame {
            origin: [words[0], words[1], words[2]],
            axes: [
                [words[3], words[4], words[5]],
                [words[6], words[7], words[8]],
                [words[9], words[10], words[11]],
            ],
            half_extents: [words[12], words[13], words[14]],
        })
    }
}

/// WGSL constants every frame pass shares: the failure bits, the skin a
/// region's joints must own and the smallest half extent.
pub(crate) fn frame_constants() -> String {
    use crate::armor_frames::{MINIMUM_REGION_RADIUS_M, skin_support_wgsl};
    format!(
        "const COINCIDENT: u32 = {}u;\nconst NO_ENVELOPE: u32 = {}u;\nconst INVALID: u32 = {}u;\n\
         {}const MINIMUM_REGION_RADIUS_M: f32 = {MINIMUM_REGION_RADIUS_M:?};\n",
        FrameFailure::CoincidentLandmarks as u32,
        FrameFailure::NoEnvelope as u32,
        FrameFailure::Invalid as u32,
        skin_support_wgsl(),
    )
}

impl FrameFailure {
    fn message(self) -> &'static str {
        match self {
            Self::CoincidentLandmarks => "coincident armor landmarks",
            Self::NoEnvelope => "no anatomical envelope",
            Self::Invalid => "frame is not a finite orthonormal frame",
        }
    }
}

/// A wearer with its body on the device.
pub struct DeviceWearer<'a> {
    pub gpu: &'a ArmorGpu,
    pub body: &'a GpuBody,
    pub host: &'a Wearer<'a>,
}

impl Wearer<'_> {
    /// This body on the device, for fits that take nothing from its surface
    /// coordinates.
    pub fn upload(&self, gpu: &ArmorGpu) -> Result<GpuBody> {
        let texcoords = vec![[0.0; 2]; self.positions.len()];
        Ok(GpuBody::new(
            gpu,
            BodySurface {
                positions: self.positions,
                normals: self.normals,
                faces: self.faces,
                texcoords: &texcoords,
                joint_indices: self.joint_indices,
                joint_weights: self.joint_weights,
                joints: self.joints,
            },
        )?)
    }
}

impl DeviceWearer<'_> {
    /// Fit `region`'s frame and read it back, for the stages that use it on
    /// the host. Stalls on the device.
    pub fn read_frame(&self, region: FitRegion) -> Result<fabelgeist_armor::PartFrame> {
        let mut batch = self.gpu.batch(("armor frame").into());
        let frame = self.record_frame(&mut batch, region)?;
        batch.submit();
        frame.check(self.gpu, region)?;
        frame.read(self.gpu)
    }

    /// Record the fitting of `region`'s frame.
    pub fn record_frame(&self, batch: &mut KernelBatch, region: FitRegion) -> Result<DeviceFrame> {
        let landmarks = self.host.device_landmarks(region)?;
        let owned = self.host.owned_joints(&landmarks.owners);
        self.record(batch, landmarks, owned)
    }

    /// Record the fitting of a mitten's separate thumb frame.
    pub fn record_thumb_frame(&self, batch: &mut KernelBatch, side: Side) -> Result<DeviceFrame> {
        let landmarks = self.host.thumb_landmarks(side)?;
        let prefix = if matches!(side, Side::Left) {
            RigJointName::L_THUMB
        } else {
            RigJointName::R_THUMB
        };
        let owned = self
            .host
            .joint_names
            .iter()
            .map(|n: &RigJointName| -> RigJointMembership { n.family_prefix(&prefix) })
            .collect();
        self.record(batch, landmarks, owned)
    }

    fn record(
        &self,
        batch: &mut KernelBatch,
        landmarks: Landmarks,
        owned: Vec<RigJointMembership>,
    ) -> Result<DeviceFrame> {
        let gpu = self.gpu;
        let frame = DeviceFrame {
            frame: gpu.scratch((FRAME_WORDS * 4).into(), ("armor frame").into())?,
            status: gpu.scratch((4u64).into(), ("armor frame status").into())?,
        };
        // Top, left floor, right floor, then six bounds.
        let reductions = gpu.upload(BufferUpload::from_elements(&[
            ORDERED_NEGATIVE_INFINITY,
            ORDERED_POSITIVE_INFINITY,
            ORDERED_POSITIVE_INFINITY,
            ORDERED_POSITIVE_INFINITY,
            ORDERED_POSITIVE_INFINITY,
            ORDERED_POSITIVE_INFINITY,
            ORDERED_NEGATIVE_INFINITY,
            ORDERED_NEGATIVE_INFINITY,
            ORDERED_NEGATIVE_INFINITY,
        ]))?;
        let owned = gpu.upload(BufferUpload::from_elements(
            &owned.into_iter().map(u32::from).collect::<Vec<_>>(),
        ))?;
        let mut parameters = PassParameters::new();
        parameters.insert("count".into(), (self.body.vertex_count).into());
        parameters.insert("rule".into(), (landmarks.rule as u32).into());
        for (i, joint) in landmarks.joints.iter().enumerate() {
            parameters.insert(
                PassParameterName::from(format!("joint{i}")),
                (joint.device_word()).into(),
            );
        }
        parameters.insert("side".into(), (landmarks.side).into());
        parameters.insert("pad0".into(), (0.0f32).into());
        parameters.insert("positions".into(), (self.body.positions.clone()).into());
        parameters.insert(
            "joint_indices".into(),
            (self.body.joint_indices.clone()).into(),
        );
        parameters.insert(
            "joint_weights".into(),
            (self.body.joint_weights.clone()).into(),
        );
        parameters.insert("joints".into(), (self.body.joints.clone()).into());
        parameters.insert("owned".into(), (owned).into());
        parameters.insert("reductions".into(), (reductions).into());
        parameters.insert("frame".into(), (frame.frame.clone()).into());
        parameters.insert("status".into(), (frame.status.clone()).into());
        let kernels = kernels(gpu)?;
        let vertices = self.body.vertex_count;
        batch
            .dispatch_items(&kernels[0], &parameters, (vertices).into())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        batch
            .dispatch(&kernels[1], &parameters, ([1, 1, 1]).into())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        batch
            .dispatch_items(&kernels[2], &parameters, (vertices).into())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        batch
            .dispatch(&kernels[3], &parameters, ([1, 1, 1]).into())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        Ok(frame)
    }
}
