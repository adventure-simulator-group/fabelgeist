//! Offset directions that point outside every incident body face.
//!
//! Each physical vertex's shading normal is projected onto the outward
//! half-space of every face constraint touching it, pass after pass, until
//! nothing moves. A constraint only ever changes the vertex group it names,
//! and its normal is fixed, so every group's sequence of projections is
//! independent of every other's: one invocation per vertex replays its own
//! group's sequence, realization by realization in triangle order, so every
//! member of a group reaches the same direction.

use anyhow::Result;
use fabelgeist_armor::gpu::device_error;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use super::wgsl;
use super::workspace::Workspace;

/// How far inside each face's outward half-space a direction must stay,
/// leaving room for numerical roundoff.
pub const OUTWARD_MARGIN: f32 = 0.05;
/// Projection sweeps over the constraints before giving up on settling.
pub const PROJECTION_PASSES: u32 = 32;
/// Floats per face constraint: the unit face normal, then whether the face
/// has one.
const CONSTRAINT_WORDS: u32 = 4;

impl Workspace<'_> {
    /// Record the unit face normals of a realization as constraint set
    /// `set`.
    pub(super) fn record_constraints(
        &self,
        batch: &mut KernelBatch,
        positions: &Buffer,
        set: u32,
    ) -> Result<()> {
        let gpu = self.gpu;
        let mut parameters = PassParameters::new();
        parameters.insert("count", self.face_count);
        parameters.insert("first", set * self.face_count * CONSTRAINT_WORDS);
        parameters.insert("pad0", 0u32);
        parameters.insert("pad1", 0u32);
        parameters.insert("positions", positions.clone());
        parameters.insert("faces", self.faces.clone());
        parameters.insert("constraints", self.constraints.clone());
        let kernel = gpu
            .cache()
            .get(gpu.context(), &constraint_source().into())
            .map_err(device_error)?;
        batch
            .dispatch_items(&kernel, &parameters, self.face_count)
            .map_err(device_error)?;
        Ok(())
    }

    /// Record the constrained offset directions of a realization whose own
    /// face normals are constraint set 0, followed by `sets - 1` sets of
    /// other realizations.
    pub(super) fn record_directions(
        &self,
        batch: &mut KernelBatch,
        normals: &Buffer,
        links: &Buffer,
        sets: u32,
        directions: &Buffer,
    ) -> Result<()> {
        let gpu = self.gpu;
        let mut parameters = PassParameters::new();
        parameters.insert("count", self.vertex_count);
        parameters.insert("faces", self.face_count);
        parameters.insert("sets", sets);
        parameters.insert("pad0", 0u32);
        parameters.insert("normals", normals.clone());
        parameters.insert("incidence", self.incidence.clone());
        parameters.insert("links", links.clone());
        parameters.insert("constraints", self.constraints.clone());
        parameters.insert("directions", directions.clone());
        parameters.insert("status", self.status.clone());
        let kernel = gpu
            .cache()
            .get(gpu.context(), &projection_source().into())
            .map_err(device_error)?;
        batch
            .dispatch_items(&kernel, &parameters, self.vertex_count)
            .map_err(device_error)?;
        Ok(())
    }
}

fn constraint_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> faces: array<u32>;
@group(0) @binding(2) var<storage, read_write> constraints: array<f32>;
struct Params {{
    count: u32,
    first: u32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(3) var<uniform> params: Params;
{vector}

fn point(v: u32) -> vec3<f32> {{
    return vec3<f32>(positions[v * 3u], positions[v * 3u + 1u], positions[v * 3u + 2u]);
}}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let f = id.x;
    if (f >= params.count) {{
        return;
    }}
    let a = point(faces[f * 3u]);
    let u = point(faces[f * 3u + 1u]) - a;
    let v = point(faces[f * 3u + 2u]) - a;
    let n = vec3<f32>(u.y * v.z - u.z * v.y, u.z * v.x - u.x * v.z, u.x * v.y - u.y * v.x);
    let length = sqrt(dot3(n, n));
    let slot = params.first + f * {words}u;
    if (length > 0.0) {{
        constraints[slot] = n.x / length;
        constraints[slot + 1u] = n.y / length;
        constraints[slot + 2u] = n.z / length;
        constraints[slot + 3u] = 1.0;
    }} else {{
        constraints[slot + 3u] = 0.0;
    }}
}}
"#,
        vector = wgsl::VECTOR,
        words = CONSTRAINT_WORDS,
    )
}

/// One invocation per weld group: its offset direction pushed outside every face constraint.
const PROJECT_GROUPS: &str = r#"
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let v = id.x;
    if (v >= params.count) {
        return;
    }
    var members: array<u32, GROUP_CAPACITY>;
    var size = 0u;
    var member = links[v * 2u];
    loop {
        if (member == NO_VERTEX) {
            break;
        }
        if (size == GROUP_CAPACITY) {
            fail(STATUS_LARGE_GROUP);
            break;
        }
        members[size] = member;
        size = size + 1u;
        member = links[member * 2u + 1u];
    }
    // A group is seeded with the normal of its last vertex.
    let seed = members[size - 1u];
    direction = vec3<f32>(normals[seed * 3u], normals[seed * 3u + 1u], normals[seed * 3u + 2u]);
    for (var pass_index = 0u; pass_index < PROJECTION_PASSES; pass_index = pass_index + 1u) {
        moved = false;
        for (var origin = 0u; origin < params.sets; origin = origin + 1u) {
            // Merge the members' triangle lists into triangle order.
            var cursor: array<u32, GROUP_CAPACITY>;
            for (var k = 0u; k < size; k = k + 1u) {
                cursor[k] = incidence[members[k]];
            }
            loop {
                var best = NO_VERTEX;
                var best_face = NO_VERTEX;
                for (var k = 0u; k < size; k = k + 1u) {
                    if (cursor[k] < incidence[members[k] + 1u]) {
                        let face = incidence[cursor[k]];
                        if (face < best_face) {
                            best_face = face;
                            best = k;
                        }
                    }
                }
                if (best == NO_VERTEX) {
                    break;
                }
                cursor[best] = cursor[best] + 1u;
                project(origin, best_face);
            }
        }
        if (!moved) {
            break;
        }
    }
    directions[v * 3u] = direction.x;
    directions[v * 3u + 1u] = direction.y;
    directions[v * 3u + 2u] = direction.z;
}
"#;

fn projection_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> normals: array<f32>;
@group(0) @binding(1) var<storage, read> incidence: array<u32>;
@group(0) @binding(2) var<storage, read> links: array<u32>;
@group(0) @binding(3) var<storage, read> constraints: array<f32>;
@group(0) @binding(4) var<storage, read_write> directions: array<f32>;
@group(0) @binding(5) var<storage, read_write> status: array<atomic<u32>>;
struct Params {{
    count: u32,
    faces: u32,
    sets: u32,
    pad0: u32,
}};
@group(0) @binding(6) var<uniform> params: Params;
{status}
{vector}
{groups}
{margin}
const PROJECTION_PASSES: u32 = {passes}u;

var<private> direction: vec3<f32>;
var<private> moved: bool;

fn project(realization: u32, face: u32) {{
    let slot = (realization * params.faces + face) * {words}u;
    if (constraints[slot + 3u] == 0.0) {{
        return;
    }}
    let normal = vec3<f32>(constraints[slot], constraints[slot + 1u], constraints[slot + 2u]);
    let deficit = OUTWARD_MARGIN - dot3(direction, normal);
    if (deficit > 0.0) {{
        let pushed = vec3<f32>(
            direction.x + deficit * normal.x,
            direction.y + deficit * normal.y,
            direction.z + deficit * normal.z,
        );
        let length = sqrt(dot3(pushed, pushed));
        direction = pushed / length;
        moved = true;
    }}
}}

{project_groups}
"#,
        project_groups = PROJECT_GROUPS,
        status = wgsl::status(),
        vector = wgsl::VECTOR,
        groups = wgsl::GROUPS,
        margin = wgsl::constant("OUTWARD_MARGIN", OUTWARD_MARGIN),
        passes = PROJECTION_PASSES,
        words = CONSTRAINT_WORDS,
    )
}
