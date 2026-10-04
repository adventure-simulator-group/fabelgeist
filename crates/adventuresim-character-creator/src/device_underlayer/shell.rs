//! The two cut layers of an underlayer, offset from a body realization, and
//! the skin they carry.

use anyhow::Result;

use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, PassParameters, ShaderSource};

use super::plan::POINT_WORDS;
use super::wgsl;
use super::workspace::Workspace;

/// Skin influences a shell vertex keeps.
pub(super) const INFLUENCES: usize = 8;
/// Floats per shell vertex in the skin kernel's output: the surface
/// coordinate, then the influence weights.
pub(super) const SKIN_FLOATS: usize = 2 + INFLUENCES;

/// The offsets of the shell's outer and inner layers, before compression.
#[derive(Clone, Copy, Debug)]
pub(super) struct LayerOffsets {
    pub outer: f32,
    pub inner: f32,
}

impl Workspace<'_> {
    /// Record the shell of a realization into `shell`: each layer point
    /// interpolated from its source triangle's offset corners.
    pub(super) fn record_layers(
        &self,
        batch: &mut KernelBatch,
        positions: &Buffer,
        directions: &Buffer,
        offsets: LayerOffsets,
        shell: &Buffer,
    ) -> Result<()> {
        let gpu = self.gpu;
        let mut parameters = PassParameters::new();
        parameters.insert("count".into(), (self.plan.vertex_count()).into());
        parameters.insert("points".into(), (self.plan.point_count).into());
        parameters.insert("outer".into(), (offsets.outer).into());
        parameters.insert("inner".into(), (offsets.inner).into());
        parameters.insert("table".into(), (self.table.clone()).into());
        parameters.insert("sources".into(), (self.sources.clone()).into());
        parameters.insert("positions".into(), (positions.clone()).into());
        parameters.insert("directions".into(), (directions.clone()).into());
        parameters.insert("compression".into(), (self.compression.clone()).into());
        parameters.insert("shell".into(), (shell.clone()).into());
        let kernel = gpu
            .cache()
            .get(gpu.context(), &layers_source())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        batch
            .dispatch_items(&kernel, &parameters, (self.plan.vertex_count()).into())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        Ok(())
    }

    /// Record every shell vertex's surface coordinate and skin: its source
    /// triangle's, blended by the point's weights and cut to the strongest
    /// influences.
    pub(super) fn record_skin(
        &self,
        batch: &mut KernelBatch,
        skin: &SkinSources,
        joints: &Buffer,
        floats: &Buffer,
    ) -> Result<()> {
        let gpu = self.gpu;
        let mut parameters = PassParameters::new();
        parameters.insert("count".into(), (self.plan.vertex_count()).into());
        parameters.insert("points".into(), (self.plan.point_count).into());
        parameters.insert("pad0".into(), (0u32).into());
        parameters.insert("pad1".into(), (0u32).into());
        parameters.insert("table".into(), (self.table.clone()).into());
        parameters.insert("sources".into(), (self.sources.clone()).into());
        parameters.insert("texcoords".into(), (skin.texcoords.clone()).into());
        parameters.insert("joint_indices".into(), (skin.joint_indices.clone()).into());
        parameters.insert("joint_weights".into(), (skin.joint_weights.clone()).into());
        parameters.insert("joints".into(), (joints.clone()).into());
        parameters.insert("floats".into(), (floats.clone()).into());
        parameters.insert("status".into(), (self.status.clone()).into());
        let kernel = gpu
            .cache()
            .get(gpu.context(), &skin_source())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        batch
            .dispatch_items(&kernel, &parameters, (self.plan.vertex_count()).into())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        Ok(())
    }
}

/// The body's surface coordinates and skin, on the device.
pub(super) struct SkinSources {
    pub texcoords: Buffer,
    pub joint_indices: Buffer,
    pub joint_weights: Buffer,
}

const POINTS: &str = r#"
fn corner_vertex(point: u32, corner: u32) -> u32 {
    return table[point * POINT_WORDS + corner];
}

fn corner_texcoord(point: u32, corner: u32) -> u32 {
    return table[point * POINT_WORDS + 3u + corner];
}

fn corner_weight(point: u32, corner: u32) -> f32 {
    return bitcast<f32>(table[point * POINT_WORDS + 6u + corner]);
}
"#;

fn layers_source() -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> table: array<u32>;
@group(0) @binding(1) var<storage, read> sources: array<u32>;
@group(0) @binding(2) var<storage, read> positions: array<f32>;
@group(0) @binding(3) var<storage, read> directions: array<f32>;
@group(0) @binding(4) var<storage, read> compression: array<f32>;
@group(0) @binding(5) var<storage, read_write> shell: array<f32>;
struct Params {{
    count: u32,
    points: u32,
    outer: f32,
    inner: f32,
}};
@group(0) @binding(6) var<uniform> params: Params;
const POINT_WORDS: u32 = {words}u;
{points}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let k = id.x;
    if (k >= params.count) {{
        return;
    }}
    let layer_point = sources[k];
    let point = layer_point % params.points;
    let offset = select(params.outer, params.inner, layer_point >= params.points);
    var sum = vec3<f32>(0.0);
    for (var corner = 0u; corner < 3u; corner = corner + 1u) {{
        let v = corner_vertex(point, corner);
        let body = vec3<f32>(positions[v * 3u], positions[v * 3u + 1u], positions[v * 3u + 2u]);
        let along = vec3<f32>(directions[v * 3u], directions[v * 3u + 1u], directions[v * 3u + 2u]);
        // Cut vertices lie on the already offset source triangle.
        let offset_corner = body + along * (offset * compression[v]);
        sum = sum + offset_corner * corner_weight(point, corner);
    }}
    shell[k * 3u] = sum.x;
    shell[k * 3u + 1u] = sum.y;
    shell[k * 3u + 2u] = sum.z;
}}
"#,
        words = POINT_WORDS,
        points = POINTS,
    ))
}

/// One invocation per shell vertex: its surface coordinate and strongest skin influences.
const SKIN_POINTS: &str = r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let k = id.x;
    if (k >= params.count) {
        return;
    }
    let point = sources[k] % params.points;
    var uv = vec2<f32>(0.0);
    // Joints in the order they are first met, with their summed weights.
    var ids: array<u32, GATHERED>;
    var sums: array<f32, GATHERED>;
    var gathered = 0u;
    for (var corner = 0u; corner < 3u; corner = corner + 1u) {
        let weight = corner_weight(point, corner);
        let t = corner_texcoord(point, corner);
        uv = uv + vec2<f32>(texcoords[t * 2u], texcoords[t * 2u + 1u]) * weight;
        let v = corner_vertex(point, corner);
        for (var i = 0u; i < INFLUENCES; i = i + 1u) {
            let joint = joint_indices[v * INFLUENCES + i];
            let share = weight * joint_weights[v * INFLUENCES + i];
            var found = gathered;
            for (var j = 0u; j < gathered; j = j + 1u) {
                if (ids[j] == joint) {
                    found = j;
                    break;
                }
            }
            if (found == gathered) {
                ids[found] = joint;
                sums[found] = 0.0;
                gathered = gathered + 1u;
            }
            sums[found] = sums[found] + share;
        }
    }
    // The strongest influences, ties to the lower joint.
    var kept: array<u32, INFLUENCES>;
    var kept_weights: array<f32, INFLUENCES>;
    let count = min(gathered, INFLUENCES);
    var total = 0.0;
    for (var slot = 0u; slot < count; slot = slot + 1u) {
        var best = slot;
        for (var j = slot + 1u; j < gathered; j = j + 1u) {
            if (sums[j] > sums[best] || (sums[j] == sums[best] && ids[j] < ids[best])) {
                best = j;
            }
        }
        let joint = ids[best];
        let weight = sums[best];
        ids[best] = ids[slot];
        sums[best] = sums[slot];
        ids[slot] = joint;
        sums[slot] = weight;
        kept[slot] = joint;
        kept_weights[slot] = weight;
        total = total + weight;
    }
    if (!(total > 0.0)) {
        fail(STATUS_NO_INFLUENCE);
    }
    floats[k * SKIN_FLOATS] = uv.x;
    floats[k * SKIN_FLOATS + 1u] = uv.y;
    for (var slot = 0u; slot < INFLUENCES; slot = slot + 1u) {
        var joint = 0u;
        var weight = 0.0;
        if (slot < count) {
            joint = kept[slot];
            weight = kept_weights[slot] / total;
        }
        joints[k * INFLUENCES + slot] = joint;
        floats[k * SKIN_FLOATS + 2u + slot] = weight;
    }
}
"#;

fn skin_source() -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> table: array<u32>;
@group(0) @binding(1) var<storage, read> sources: array<u32>;
@group(0) @binding(2) var<storage, read> texcoords: array<f32>;
@group(0) @binding(3) var<storage, read> joint_indices: array<u32>;
@group(0) @binding(4) var<storage, read> joint_weights: array<f32>;
@group(0) @binding(5) var<storage, read_write> joints: array<u32>;
@group(0) @binding(6) var<storage, read_write> floats: array<f32>;
@group(0) @binding(7) var<storage, read_write> status: array<atomic<u32>>;
struct Params {{
    count: u32,
    points: u32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(8) var<uniform> params: Params;
const POINT_WORDS: u32 = {words}u;
const INFLUENCES: u32 = {influences}u;
const SKIN_FLOATS: u32 = {skin_floats}u;
// Every influence of three body vertices.
const GATHERED: u32 = 3u * INFLUENCES;
{status}
{points}

{skin_points}
"#,
        skin_points = SKIN_POINTS,
        words = POINT_WORDS,
        influences = INFLUENCES,
        skin_floats = SKIN_FLOATS,
        status = wgsl::status(),
        points = POINTS,
    ))
}
