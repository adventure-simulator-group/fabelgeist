//! Shared native projection and bounded indirect-publication programs.

use super::EmissionKind;
use crate::prelude::*;

const DEINTERLEAVE: &str = r#"
            struct Vertex {
                position: vec4<f32>,
                normal: vec4<f32>,
            }

            @group(0) @binding(0) var<storage, read> vertices: array<Vertex>;
            @group(0) @binding(1) var<storage, read_write> out_positions: array<f32>;
            @group(0) @binding(2) var<storage, read_write> out_normals: array<f32>;

            @compute @workgroup_size(64)
            fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
                let idx = global_id.x;
                if (idx >= arrayLength(&vertices)) {
                    return;
                }
                let v = vertices[idx];
                let pos_idx = idx * 3u;
                out_positions[pos_idx] = v.position.x;
                out_positions[pos_idx + 1u] = v.position.y;
                out_positions[pos_idx + 2u] = v.position.z;

                out_normals[pos_idx] = v.normal.x;
                out_normals[pos_idx + 1u] = v.normal.y;
                out_normals[pos_idx + 2u] = v.normal.z;
            }
        "#;

pub(crate) fn deinterleave(context: &WgpuContext) -> Result<ResourcePipeline> {
    let shader = ComputeShader::new(context, DEINTERLEAVE)?;
    let pipeline = ComputePipeline::new(context, shader)?;
    ResourcePipeline::new(
        pipeline,
        DispatchResource::Vertices,
        TextureAddressing::Coordinates,
    )
}

pub(crate) fn indirect_source(kind: EmissionKind) -> String {
    let width = kind.records_per_primitive();
    let indexed_tail = match kind {
        EmissionKind::TriangleVertices => "",
        EmissionKind::QuadIndices => "    indirect[4] = 0u;\n",
    };
    format!(
        r#"
struct OutputCapacity {{
    capacity: u32,
}}
@group(0) @binding(2) var<storage, read> inclusive_offsets: array<u32>;
@group(0) @binding(3) var<storage, read_write> indirect: array<u32>;
@group(0) @binding(4) var<uniform> limit: OutputCapacity;

fn map(val: u32) -> u32 {{
    let last_index = arrayLength(&inclusive_offsets) - 1u;
    let complete_capacity = (limit.capacity / {width}u) * {width}u;
    indirect[0] = min(inclusive_offsets[last_index], complete_capacity);
    indirect[1] = 1u;
    indirect[2] = 0u;
    indirect[3] = 0u;
{indexed_tail}    return val;
}}
"#
    )
}
