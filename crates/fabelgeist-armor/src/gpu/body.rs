//! A wearer's body on the device, and what armor takes from it.

use fabelgeist_gpu::prelude::BufferUpload;
use std::sync::Arc;

use fabelgeist_compute::{Kernel, KernelBatch, PointTargets, QueryHits};
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use super::staging::{Staged, StagedResults, Staging};
use super::{ArmorGpu, device_error, wgsl};
use crate::GenerateError;

/// One realization of a wearer's body, on the host.
#[derive(Clone, Copy, Debug)]
pub struct BodySurface<'a> {
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
    pub faces: &'a [[u32; 3]],
    /// Each vertex's anatomical surface coordinate.
    pub texcoords: &'a [[f32; 2]],
    pub joint_indices: &'a [[u32; 8]],
    pub joint_weights: &'a [[f32; 8]],
    /// Global joint states: position, then rotation as a quaternion, then
    /// scale.
    pub joints: &'a [[f32; 8]],
}

/// A wearer's body uploaded for fitting.
#[derive(Clone, Debug)]
pub struct GpuBody {
    pub positions: Buffer,
    pub normals: Buffer,
    pub faces: Buffer,
    pub texcoords: Buffer,
    pub joint_indices: Buffer,
    pub joint_weights: Buffer,
    pub joints: Buffer,
    pub vertex_count: u32,
    pub face_count: u32,
    pub joint_count: u32,
}

impl GpuBody {
    pub fn new(gpu: &ArmorGpu, surface: BodySurface) -> Result<Self, GenerateError> {
        let vertices = surface.positions.len();
        if vertices == 0
            || surface.normals.len() != vertices
            || surface.texcoords.len() != vertices
            || surface.joint_indices.len() != vertices
            || surface.joint_weights.len() != vertices
            || surface
                .faces
                .iter()
                .flatten()
                .any(|i| *i as usize >= vertices)
        {
            return Err(GenerateError::InvalidSurface);
        }
        Ok(Self {
            positions: gpu.upload(BufferUpload::from_elements(surface.positions))?,
            normals: gpu.upload(BufferUpload::from_elements(surface.normals))?,
            faces: gpu.upload(BufferUpload::from_elements(surface.faces))?,
            texcoords: gpu.upload(BufferUpload::from_elements(surface.texcoords))?,
            joint_indices: gpu.upload(BufferUpload::from_elements(surface.joint_indices))?,
            joint_weights: gpu.upload(BufferUpload::from_elements(surface.joint_weights))?,
            joints: gpu.upload(BufferUpload::from_elements(surface.joints))?,
            vertex_count: vertices as u32,
            face_count: surface.faces.len() as u32,
            joint_count: surface.joints.len() as u32,
        })
    }
}

/// Skin and surface coordinates carried from the nearest body vertex.
#[derive(Clone, Debug)]
pub struct Correspondence {
    hits: QueryHits,
    pub texcoords: Buffer,
    pub joint_indices: Buffer,
    pub joint_weights: Buffer,
    count: u32,
}

/// Correspondence read back to the host.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Skin {
    pub texcoords: Vec<[f32; 2]>,
    pub joint_indices: Vec<[u32; 8]>,
    pub joint_weights: Vec<[f32; 8]>,
}

impl Skin {
    /// Add `other`'s points after this skin's, matching an appended part.
    pub fn append(&mut self, other: Skin) {
        self.texcoords.extend(other.texcoords);
        self.joint_indices.extend(other.joint_indices);
        self.joint_weights.extend(other.joint_weights);
    }
}

impl Correspondence {
    /// Record, for each of `count` points, the nearest body vertex and its
    /// attributes.
    pub fn record(
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        body: &GpuBody,
        points: &Buffer,
        count: u32,
    ) -> Result<Self, GenerateError> {
        let hits = QueryHits::new(gpu.context(), count).map_err(device_error)?;
        gpu.query()
            .record_nearest_points(
                batch,
                points,
                count,
                PointTargets {
                    positions: &body.positions,
                    candidates: None,
                    count: body.vertex_count,
                },
                &hits,
            )
            .map_err(device_error)?;
        let correspondence = Self {
            texcoords: gpu.scratch(count as u64 * 8, "armor texcoords")?,
            joint_indices: gpu.scratch(count as u64 * 32, "armor joint indices")?,
            joint_weights: gpu.scratch(count as u64 * 32, "armor joint weights")?,
            hits,
            count,
        };
        let mut gather = PassParameters::new();
        gather.insert("count", count);
        for pad in ["pad0", "pad1", "pad2"] {
            gather.insert(pad, 0u32);
        }
        gather.insert("nearest", correspondence.hits.nearest.clone());
        gather.insert("body_texcoords", body.texcoords.clone());
        gather.insert("body_joint_indices", body.joint_indices.clone());
        gather.insert("body_joint_weights", body.joint_weights.clone());
        gather.insert("texcoords", correspondence.texcoords.clone());
        gather.insert("joint_indices", correspondence.joint_indices.clone());
        gather.insert("joint_weights", correspondence.joint_weights.clone());
        batch
            .dispatch_items(&*gather_kernel(gpu)?, &gather, (count).into())
            .map_err(device_error)?;
        Ok(correspondence)
    }

    /// Read back after the batch has been submitted.
    pub fn read(&self, gpu: &ArmorGpu) -> Result<Skin, GenerateError> {
        let mut staging = Staging::new();
        let slots = self.stage(&mut staging);
        Ok(self.finish(&gpu.read_staged(staging)?, slots))
    }

    /// Stage the skin for a shared readback.
    pub fn stage<'a>(&'a self, staging: &mut Staging<'a>) -> [Staged; 3] {
        [
            staging.stage(&self.texcoords),
            staging.stage(&self.joint_indices),
            staging.stage(&self.joint_weights),
        ]
    }

    /// The skin, from a shared readback.
    pub fn finish(&self, results: &StagedResults, slots: [Staged; 3]) -> Skin {
        let count = self.count as usize;
        let mut texcoords: Vec<[f32; 2]> = results.get(slots[0]);
        let mut joint_indices: Vec<[u32; 8]> = results.get(slots[1]);
        let mut joint_weights: Vec<[f32; 8]> = results.get(slots[2]);
        texcoords.truncate(count);
        joint_indices.truncate(count);
        joint_weights.truncate(count);
        Skin {
            texcoords,
            joint_indices,
            joint_weights,
        }
    }
}

fn gather_kernel(gpu: &ArmorGpu) -> Result<Arc<Kernel>, GenerateError> {
    let source = format!(
        r#"
@group(0) @binding(0) var<storage, read> nearest: array<u32>;
@group(0) @binding(1) var<storage, read> body_texcoords: array<f32>;
@group(0) @binding(2) var<storage, read> body_joint_indices: array<u32>;
@group(0) @binding(3) var<storage, read> body_joint_weights: array<f32>;
@group(0) @binding(4) var<storage, read_write> texcoords: array<f32>;
@group(0) @binding(5) var<storage, read_write> joint_indices: array<u32>;
@group(0) @binding(6) var<storage, read_write> joint_weights: array<f32>;
{counted}
@group(0) @binding(7) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let i = id.x;
    if (i >= params.count) {{
        return;
    }}
    let source = nearest[i];
    texcoords[i * 2u] = body_texcoords[source * 2u];
    texcoords[i * 2u + 1u] = body_texcoords[source * 2u + 1u];
    for (var k = 0u; k < 8u; k = k + 1u) {{
        joint_indices[i * 8u + k] = body_joint_indices[source * 8u + k];
        joint_weights[i * 8u + k] = body_joint_weights[source * 8u + k];
    }}
}}
"#,
        counted = wgsl::COUNTED,
    );
    gpu.cache()
        .get(gpu.context(), &source)
        .map_err(device_error)
}
