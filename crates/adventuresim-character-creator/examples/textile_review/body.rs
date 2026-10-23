//! Native unposed body inputs shared by the independent textile exporters.
use adventuresim_character_creator::{
    device_body::DeviceBody,
    device_fit::{FitBody, Realization},
};
use fabelgeist_rig::RigJointName;
use serde::Deserialize;

/// Native attributes at the review-file boundary, in canonical metres.
#[derive(Deserialize)]
#[serde(try_from = "NativeBody")]
pub struct Body {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    texcoords: Vec<[f32; 2]>,
    joint_indices: fabelgeist_rig::PackedRigSkin<8>,
    joint_weights: Vec<[f32; 8]>,
    pub joint_names: fabelgeist_rig::RigJointTable<RigJointName>,
    pub joints: fabelgeist_rig::RigJointTable<[f32; 8]>,
}

impl Body {
    pub fn fit_body(&self) -> FitBody<'_> {
        FitBody {
            faces: &self.faces,
            texcoords: &self.texcoords,
            joint_indices: &self.joint_indices,
            joint_weights: &self.joint_weights,
            joint_names: &self.joint_names,
        }
    }

    pub fn realization<'a>(&'a self, device: &'a DeviceBody) -> Realization<'a> {
        Realization {
            positions: &self.positions,
            normals: &self.normals,
            joints: &self.joints,
            device,
        }
    }
}

#[derive(Deserialize)]
struct NativeBody {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    texcoords: Vec<[f32; 2]>,
    joint_indices: Vec<[u32; 8]>,
    joint_weights: Vec<[f32; 8]>,
    pub joint_names: Vec<RigJointName>,
    pub joints: Vec<[f32; 8]>,
}

impl TryFrom<NativeBody> for Body {
    type Error = fabelgeist_rig::RigSkinError;
    fn try_from(raw: NativeBody) -> Result<Self, Self::Error> {
        let joint_names: fabelgeist_rig::RigJointTable<_> = raw.joint_names.into();
        let joint_indices = fabelgeist_rig::PackedRigSkin::from_native(
            raw.joint_indices,
            joint_names.joint_count(),
        )?;
        Ok(Self {
            positions: raw.positions,
            normals: raw.normals,
            faces: raw.faces,
            texcoords: raw.texcoords,
            joint_indices,
            joint_weights: raw.joint_weights,
            joint_names,
            joints: raw.joints.into(),
        })
    }
}
