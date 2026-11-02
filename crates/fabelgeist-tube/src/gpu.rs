//! The sweep as one compute pass, writing straight into mesh buffers.

use anyhow::{Result, anyhow};
use fabelgeist_compute::{Kernel, KernelBatch, KernelCache};
use fabelgeist_gpu::data::gpu::buffer::{Buffer, BufferDefinition};
use fabelgeist_gpu::data::gpu::parameters::PassParameters;
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_mesh::{FrontFace, GpuMesh, PrimitiveTopology};
use std::sync::Arc;

use crate::{TubeSegment, indices_per_segment, vertices_per_segment};
use fabelgeist_gpu::prelude::{BufferUpload, BufferUse};

/// One invocation per `(segment, ring column)`: it writes that column's vertex
/// on both rings, and the two triangles of the quad to its right.
///
/// Positions and normals are packed three floats a vertex, texture
/// coordinates two, which is the layout `GpuMesh` draws from.
pub const SWEEP_WGSL: &str = r#"
@group(0) @binding(0) var<storage, read> segments: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> positions: array<f32>;
@group(0) @binding(2) var<storage, read_write> normals: array<f32>;
@group(0) @binding(3) var<storage, read_write> tex_coords: array<f32>;
@group(0) @binding(4) var<storage, read_write> indices: array<u32>;

struct SweepParams {
    count: u32,
    sides: u32,
    pad0: u32,
    pad1: u32,
};
@group(0) @binding(5) var<uniform> params: SweepParams;

fn write_vertex(v: u32, p: vec3<f32>, n: vec3<f32>, uv: vec2<f32>) {
    positions[v * 3u] = p.x;
    positions[v * 3u + 1u] = p.y;
    positions[v * 3u + 2u] = p.z;
    normals[v * 3u] = n.x;
    normals[v * 3u + 1u] = n.y;
    normals[v * 3u + 2u] = n.z;
    tex_coords[v * 2u] = uv.x;
    tex_coords[v * 2u + 1u] = uv.y;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let item = id.y * groups.x * 64u + id.x;
    let ring = params.sides + 1u;
    let segment = item / ring;
    let k = item % ring;
    if (segment >= params.count) {
        return;
    }

    let a = segments[segment * 2u];
    let b = segments[segment * 2u + 1u];
    let axis = b.xyz - a.xyz;
    let len = length(axis);
    var t = vec3<f32>(0.0, 1.0, 0.0);
    var slope = 0.0;
    if (len > 1e-12) {
        t = axis / len;
        slope = (a.w - b.w) / len;
    }

    // Duff et al., branchless.
    let sign = select(-1.0, 1.0, t.z >= 0.0);
    let q = -1.0 / (sign + t.z);
    let w = t.x * t.y * q;
    let b1 = vec3<f32>(1.0 + sign * t.x * t.x * q, sign * w, -sign * t.x);
    let b2 = vec3<f32>(w, sign + t.y * t.y * q, -t.y);

    let theta = 6.28318530718 * f32(k) / f32(params.sides);
    let n = cos(theta) * b1 + sin(theta) * b2;
    let normal = normalize(n + slope * t);
    let u = f32(k) / f32(params.sides);

    let base = segment * 2u * ring;
    write_vertex(base + k, a.xyz + a.w * n, normal, vec2<f32>(u, 0.0));
    write_vertex(base + ring + k, b.xyz + b.w * n, normal, vec2<f32>(u, len));

    if (k < params.sides) {
        let i = (segment * params.sides + k) * 6u;
        let v0 = base + k;
        let v1 = v0 + 1u;
        let v2 = v0 + ring;
        let v3 = v2 + 1u;
        indices[i] = v0;
        indices[i + 1u] = v1;
        indices[i + 2u] = v2;
        indices[i + 3u] = v1;
        indices[i + 4u] = v3;
        indices[i + 5u] = v2;
    }
}
"#;

/// Output buffers sized for a number of segments, grown on demand and reused.
struct Outputs {
    positions: Buffer,
    normals: Buffer,
    tex_coords: Buffer,
    indices: Buffer,
    capacity: u32,
}

impl Outputs {
    fn new(context: &WgpuContext, capacity: u32, sides: u32) -> Result<Self> {
        let vertices = capacity as u64 * vertices_per_segment(sides) as u64;
        let indices = capacity as u64 * indices_per_segment(sides) as u64;
        let vertex = BufferDefinition::storage().with_usage(BufferUse::Vertex);
        Ok(Self {
            positions: Buffer::new(
                context,
                (vertices * 12).into(),
                vertex.clone().with_label(("tube positions").into()),
            )?,
            normals: Buffer::new(
                context,
                (vertices * 12).into(),
                vertex.clone().with_label(("tube normals").into()),
            )?,
            tex_coords: Buffer::new(
                context,
                (vertices * 8).into(),
                vertex.with_label(("tube uvs").into()),
            )?,
            indices: Buffer::new(
                context,
                (indices * 4).into(),
                BufferDefinition::storage()
                    .with_usage(BufferUse::Index)
                    .with_label(("tube indices").into()),
            )?,
            capacity,
        })
    }
}

/// Sweeps segment buffers into [`GpuMesh`]es.
///
/// The mesher owns its output buffers and reuses them, so the mesh it returns
/// shares storage with the next call's: clone the buffers out, or draw before
/// meshing again. That is the right trade for a skeleton that is re-meshed
/// every simulation step.
pub struct TubeMesher {
    kernel: Arc<Kernel>,
    sides: u32,
    outputs: Option<Outputs>,
}

impl TubeMesher {
    pub fn new(context: &WgpuContext, cache: &KernelCache, sides: u32) -> Result<Self> {
        Ok(Self {
            kernel: cache.get(context, SWEEP_WGSL)?,
            sides: sides.max(3),
            outputs: None,
        })
    }

    pub fn sides(&self) -> u32 {
        self.sides
    }

    /// Record the sweep of the first `count` segments of `segments` (two
    /// `vec4<f32>` each, [`TubeSegment`]'s layout) and return the mesh it
    /// will have written once the batch is submitted.
    pub fn record(
        &mut self,
        context: &WgpuContext,
        batch: &mut KernelBatch,
        segments: &Buffer,
        count: u32,
    ) -> Result<GpuMesh> {
        if segments.size < (count as u64 * std::mem::size_of::<TubeSegment>() as u64).into() {
            return Err(anyhow!(
                "TubeMesher: segment buffer holds fewer than {count} segments"
            ));
        }
        let fits = self.outputs.as_ref().is_some_and(|o| o.capacity >= count);
        if !fits {
            // Half again, so a growing skeleton does not reallocate every step.
            let capacity = count
                .max(1)
                .max(self.outputs.as_ref().map_or(0, |o| o.capacity * 3 / 2));
            self.outputs = Some(Outputs::new(context, capacity, self.sides)?);
        }
        let outputs = self.outputs.as_ref().expect("allocated above");

        if count > 0 {
            let mut parameters = PassParameters::new();
            parameters.insert("segments", segments.clone());
            parameters.insert("positions", outputs.positions.clone());
            parameters.insert("normals", outputs.normals.clone());
            parameters.insert("tex_coords", outputs.tex_coords.clone());
            parameters.insert("indices", outputs.indices.clone());
            parameters.insert("count", count);
            parameters.insert("sides", self.sides);
            parameters.insert("pad0", 0u32);
            parameters.insert("pad1", 0u32);
            batch.dispatch_folded(&self.kernel, &parameters, count * (self.sides + 1))?;
        }

        let vertices = count * vertices_per_segment(self.sides);
        let sized = |buffer: &Buffer, bytes: u64| {
            let mut buffer = buffer.clone();
            buffer.size = (bytes).into();
            buffer
        };
        Ok(GpuMesh {
            positions: sized(&outputs.positions, vertices as u64 * 12),
            normals: sized(&outputs.normals, vertices as u64 * 12),
            tex_coords: sized(&outputs.tex_coords, vertices as u64 * 8),
            indices: Some(sized(
                &outputs.indices,
                count as u64 * indices_per_segment(self.sides) as u64 * 4,
            )),
            joints: None,
            weights: None,
            vertex_count: vertices,
            topology: PrimitiveTopology::TriangleList,
            front_face: FrontFace::Ccw,
        })
    }

    /// Upload `segments`, sweep them and submit. For a one-off.
    pub fn mesh(&mut self, context: &WgpuContext, segments: &[TubeSegment]) -> Result<GpuMesh> {
        let data: &[TubeSegment] = if segments.is_empty() {
            &[TubeSegment {
                start: [0.0; 3],
                start_radius: 0.0,
                end: [0.0; 3],
                end_radius: 0.0,
            }]
        } else {
            segments
        };
        let buffer = Buffer::from_upload(
            context,
            BufferUpload::from_elements(data),
            BufferDefinition::storage(),
        )?;
        let mut batch = KernelBatch::labelled(context, "TubeMesher");
        let mesh = self.record(context, &mut batch, &buffer, segments.len() as u32)?;
        batch.submit();
        Ok(mesh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn gpu_sweep_matches_the_host_sweep() -> Result<()> {
        let context = WgpuContext::new().await?;
        let cache = KernelCache::new();
        let mut mesher = TubeMesher::new(&context, &cache, 7)?;

        let segments = vec![
            TubeSegment::new([0.0, 0.0, 0.0], 0.3, [0.0, 1.0, 0.0], 0.2),
            TubeSegment::new([0.0, 1.0, 0.0], 0.2, [0.4, 1.8, -0.1], 0.1),
            TubeSegment::new([1.0, 0.0, 0.0], 0.1, [1.0, 0.0, -1.0], 0.05),
            TubeSegment::new([2.0, 0.0, 0.0], 0.1, [2.0, 0.0, 0.0], 0.1),
        ];
        let gpu = mesher.mesh(&context, &segments)?.readback(&context).await?;
        let host = crate::mesh(&segments, 7);

        assert_eq!(gpu.indices, host.indices);
        for (name, a, b) in [
            ("positions", &gpu.positions, &host.positions),
            ("normals", &gpu.normals, &host.normals),
        ] {
            assert_eq!(a.len(), b.len(), "{name}");
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                for k in 0..3 {
                    assert!((x[k] - y[k]).abs() < 1e-4, "{name}[{i}]: {x:?} vs {y:?}");
                }
            }
        }
        Ok(())
    }
}
