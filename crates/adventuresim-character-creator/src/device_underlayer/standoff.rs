//! The layer stack compressed where converging body normals leave little
//! room.
//!
//! After the gap limit, every body edge whose offset directions converge
//! limits both its vertices -- a minimum, so one invocation per vertex over
//! its own triangles gives it exactly. The prism sweeps follow (see
//! [`super::sweep`]), and the room found lowers the frozen compression.

use anyhow::Result;

use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, PassParameters, ShaderSource};

use super::wgsl;
use super::workspace::Workspace;

/// The full clearance-plus-thickness offset a layer stack may reach.
pub const LAYER_STACK_ENVELOPE_M: f32 = 0.018;
/// Share of the distance to a converging edge's focal point a stack may take.
pub const FOCAL_DISTANCE_FRACTION: f32 = 0.25;
/// The least share of its own height an extruded prism may keep anywhere.
pub const PRISM_MARGIN: f32 = 0.25;
/// Halving sweeps over the triangles before the room is accepted.
pub const COMPRESSION_PASSES: u32 = 32;

impl Workspace<'_> {
    /// Record the room of a realization along `directions` and lower
    /// `compression` to it: gap, converging edges, then the prism sweeps.
    pub(super) fn record_standoff(
        &mut self,
        batch: &mut KernelBatch,
        positions: &Buffer,
        directions: &Buffer,
        links: &Buffer,
    ) -> Result<()> {
        self.record_gap(batch, positions, directions)?;
        let gpu = self.gpu;
        let kernel = |source: String| {
            gpu.cache()
                .get(gpu.context(), &ShaderSource::from(source))
                .map_err(fabelgeist_armor::GenerateError::from)
        };
        let mut parameters = PassParameters::new();
        parameters.insert("vertices".into(), (self.vertex_count).into());
        parameters.insert("faces".into(), (self.face_count).into());
        parameters.insert("pad0".into(), (0u32).into());
        parameters.insert("pad1".into(), (0u32).into());
        parameters.insert("positions".into(), (positions.clone()).into());
        parameters.insert("directions".into(), (directions.clone()).into());
        parameters.insert("triangles".into(), (self.faces.clone()).into());
        parameters.insert("incidence".into(), (self.incidence.clone()).into());
        parameters.insert("rooms".into(), (self.rooms.clone()).into());
        batch
            .dispatch_items(
                &*kernel(convergence_source())?,
                &parameters,
                (self.vertex_count).into(),
            )
            .map_err(fabelgeist_armor::GenerateError::from)?;
        self.record_sweeps(batch, positions, directions, links)?;
        let mut parameters = PassParameters::new();
        parameters.insert("count".into(), (self.vertex_count).into());
        for pad in ["pad0", "pad1", "pad2"] {
            parameters.insert(pad.into(), (0u32).into());
        }
        parameters.insert("rooms".into(), (self.rooms.clone()).into());
        parameters.insert("compression".into(), (self.compression.clone()).into());
        batch
            .dispatch_items(
                &*kernel(lower_source())?,
                &parameters,
                (self.vertex_count).into(),
            )
            .map_err(fabelgeist_armor::GenerateError::from)?;
        Ok(())
    }
}

pub(super) const BINDINGS: &str = r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> directions: array<f32>;
@group(0) @binding(2) var<storage, read> triangles: array<u32>;
@group(0) @binding(3) var<storage, read> incidence: array<u32>;
struct Params {
    vertices: u32,
    faces: u32,
    pad0: u32,
    pad1: u32,
};

fn point(v: u32) -> vec3<f32> {
    return vec3<f32>(positions[v * 3u], positions[v * 3u + 1u], positions[v * 3u + 2u]);
}

fn direction(v: u32) -> vec3<f32> {
    return vec3<f32>(directions[v * 3u], directions[v * 3u + 1u], directions[v * 3u + 2u]);
}
"#;

fn convergence_source() -> String {
    format!(
        r#"
{bindings}
@group(0) @binding(4) var<storage, read_write> rooms: array<f32>;
@group(0) @binding(5) var<uniform> params: Params;
{vector}
{fraction}
const EPSILON: f32 = {epsilon:?}f;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let v = id.x;
    if (v >= params.vertices) {{
        return;
    }}
    var room = rooms[v];
    for (var entry = incidence[v]; entry < incidence[v + 1u]; entry = entry + 1u) {{
        let f = incidence[entry];
        for (var side = 0u; side < 3u; side = side + 1u) {{
            let a = triangles[f * 3u + side];
            let b = triangles[f * 3u + (side + 1u) % 3u];
            if (a != v && b != v) {{
                continue;
            }}
            let edge = point(b) - point(a);
            let normal_change = direction(b) - direction(a);
            let convergence = -dot3(edge, normal_change);
            if (convergence > EPSILON) {{
                room = min(room, dot3(edge, edge) / convergence * FOCAL_DISTANCE_FRACTION);
            }}
        }}
    }}
    rooms[v] = room;
}}
"#,
        bindings = BINDINGS,
        vector = wgsl::VECTOR,
        fraction = wgsl::constant("FOCAL_DISTANCE_FRACTION", FOCAL_DISTANCE_FRACTION),
        epsilon = f32::EPSILON,
    )
}

fn lower_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> rooms: array<f32>;
@group(0) @binding(1) var<storage, read_write> compression: array<f32>;
struct Params {{
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(2) var<uniform> params: Params;
{envelope}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let v = id.x;
    if (v >= params.count) {{
        return;
    }}
    compression[v] = min(compression[v], rooms[v] / LAYER_STACK_ENVELOPE_M);
}}
"#,
        envelope = wgsl::constant("LAYER_STACK_ENVELOPE_M", LAYER_STACK_ENVELOPE_M),
    )
}
