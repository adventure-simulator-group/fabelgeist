//! Prism sweeps: halve the room of every triangle whose extruded prism
//! would fold, in triangle order, pass after pass.
//!
//! Within a pass each triangle sees the halvings of the triangles before it,
//! which makes the sweep sequential. Its outcome is nevertheless a fixed
//! point: a triangle fails if and only if its prism fails with the room its
//! vertices keep after the earlier failures. Only a triangle that fails with
//! the pass's starting room, or that shares a vertex with a triangle that has
//! failed, can fail at all. So a pass tests every triangle once in parallel,
//! and one workgroup iterates the verdicts of just those candidates -- adding
//! the neighbours of each new failure -- until none changes. A pass in which
//! nothing fails ends the sweep, leaving every later pass idle.

use anyhow::Result;
use fabelgeist_armor::gpu::device_error;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use super::standoff::{BINDINGS, COMPRESSION_PASSES, PRISM_MARGIN};
use super::wgsl;
use super::workspace::Workspace;

/// Verdict iterations one pass may take to settle before the fit fails.
const SWEEP_ITERATIONS: u32 = 256;
/// Invocations of the one workgroup that settles a pass.
const SETTLE_WORKGROUP: u32 = 256;
/// Words of the worklist ahead of its per-triangle marks, its candidate list
/// and its two verdict sets: the list length, then whether a pass found
/// nothing to halve.
pub(super) const WORKLIST_HEADER: u32 = 2;

impl Workspace<'_> {
    /// Record the prism sweeps over the room in the first half of `rooms`,
    /// ending with coincident vertices sharing their least room.
    pub(super) fn record_sweeps(
        &self,
        batch: &mut KernelBatch,
        positions: &Buffer,
        directions: &Buffer,
        links: &Buffer,
    ) -> Result<()> {
        let gpu = self.gpu;
        let kernel = |source: String| {
            gpu.cache()
                .get(gpu.context(), &source)
                .map_err(device_error)
        };
        let synchronize = kernel(synchronize_source())?;
        let restart = kernel(restart_source())?;
        let test = kernel(test_source())?;
        let settle = kernel(settle_source())?;
        batch.clear_buffer(&self.worklist);
        let mut parameters = PassParameters::new();
        parameters.insert("vertices", self.vertex_count);
        parameters.insert("faces", self.face_count);
        parameters.insert("gated", 1u32);
        parameters.insert("pad0", 0u32);
        parameters.insert("positions", positions.clone());
        parameters.insert("directions", directions.clone());
        parameters.insert("triangles", self.faces.clone());
        parameters.insert("incidence", self.incidence.clone());
        parameters.insert("links", links.clone());
        parameters.insert("rooms", self.rooms.clone());
        parameters.insert("worklist", self.worklist.clone());
        parameters.insert("status", self.status.clone());
        let vertices = self.vertex_count;
        for _ in 0..COMPRESSION_PASSES {
            batch
                .dispatch_items(&synchronize, &parameters, vertices)
                .and_then(|b| b.dispatch_items(&restart, &parameters, vertices))
                .and_then(|b| b.dispatch_items(&test, &parameters, self.face_count))
                .and_then(|b| b.dispatch(&settle, &parameters, [1, 1, 1]))
                .map_err(device_error)?;
        }
        parameters.insert("gated", 0u32);
        batch
            .dispatch_items(&synchronize, &parameters, vertices)
            .and_then(|b| b.dispatch_items(&restart, &parameters, vertices))
            .map_err(device_error)?;
        Ok(())
    }
}

/// Bindings and helpers every sweep kernel shares. The second half of
/// `rooms` holds the pass's starting room.
fn common() -> String {
    format!(
        r#"
{bindings}
@group(0) @binding(4) var<storage, read> links: array<u32>;
@group(0) @binding(5) var<storage, read_write> rooms: array<f32>;
@group(0) @binding(6) var<storage, read_write> worklist: array<atomic<u32>>;
@group(0) @binding(7) var<storage, read_write> status: array<atomic<u32>>;
struct Sweep {{
    vertices: u32,
    faces: u32,
    gated: u32,
    pad0: u32,
}};
@group(0) @binding(8) var<uniform> params: Sweep;
{status}
{vector}
{groups}
{margin}
const LENGTH: u32 = 0u;
const QUIET: u32 = 1u;
const MARKS: u32 = {header}u;

fn quiet() -> bool {{
    return atomicLoad(&worklist[QUIET]) != 0u;
}}

// Two verdict sets follow the marks and the list: whether each triangle fails.
fn verdict(chosen: u32, f: u32) -> u32 {{
    return atomicLoad(&worklist[MARKS + (2u + chosen) * params.faces + f]);
}}

fn set_verdict(chosen: u32, f: u32, value: u32) {{
    atomicStore(&worklist[MARKS + (2u + chosen) * params.faces + f], value);
}}

fn start_room(v: u32) -> f32 {{
    return rooms[params.vertices + v];
}}

fn height(a: f32, b: f32, c: f32, t: f32) -> f32 {{
    return a * t * t + b * t + c;
}}

// Whether every offset keeps the extruded triangle's Jacobian positive
// throughout the stack.
fn valid_prism(p: array<vec3<f32>, 3>, o: array<vec3<f32>, 3>) -> bool {{
    let e = p[1] - p[0];
    let g = p[2] - p[0];
    let u = o[1] - o[0];
    let w = o[2] - o[0];
    let sheared = cross3(e, w) + cross3(u, g);
    let quadratic = cross3(u, w);
    let base_normal = cross3(e, g);
    for (var corner = 0u; corner < 3u; corner = corner + 1u) {{
        let a = dot3(quadratic, o[corner]);
        let b = dot3(sheared, o[corner]);
        let c = dot3(base_normal, o[corner]);
        var minimum: f32;
        if (a > 0.0) {{
            minimum = min(height(a, b, c, clamp(-b / (2.0 * a), 0.0, 1.0)), height(a, b, c, 1.0));
        }} else {{
            minimum = min(c, height(a, b, c, 1.0));
        }}
        if (!(c > 0.0 && minimum >= c * PRISM_MARGIN)) {{
            return false;
        }}
    }}
    return true;
}}

// Failed triangles among `v`'s, in verdict set `chosen`, before `before`.
fn halvings(v: u32, chosen: u32, before: u32) -> i32 {{
    var count = 0;
    for (var entry = incidence[v]; entry < incidence[v + 1u]; entry = entry + 1u) {{
        let f = incidence[entry];
        if (f < before && verdict(chosen, f) != 0u) {{
            count = count + 1;
        }}
    }}
    return count;
}}

// Whether triangle `f`'s prism holds with its corners' starting room, halved
// once per failure among earlier triangles in verdict set `chosen` -- or not
// halved at all when `chosen` is `NO_VERTEX`.
fn holds(f: u32, chosen: u32) -> bool {{
    var p: array<vec3<f32>, 3>;
    var o: array<vec3<f32>, 3>;
    for (var corner = 0u; corner < 3u; corner = corner + 1u) {{
        let v = triangles[f * 3u + corner];
        var room = start_room(v);
        if (chosen != NO_VERTEX) {{
            room = ldexp(room, -halvings(v, chosen, f));
        }}
        p[corner] = point(v);
        o[corner] = direction(v) * room;
    }}
    return valid_prism(p, o);
}}

// Make `f` a candidate, once.
fn enlist(f: u32) {{
    if (atomicExchange(&worklist[MARKS + f], 1u) == 0u) {{
        let slot = atomicAdd(&worklist[LENGTH], 1u);
        atomicStore(&worklist[MARKS + params.faces + slot], f);
    }}
}}
"#,
        bindings = BINDINGS,
        status = wgsl::status(),
        vector = wgsl::VECTOR,
        groups = wgsl::GROUPS,
        margin = wgsl::constant("PRISM_MARGIN", PRISM_MARGIN),
        header = WORKLIST_HEADER,
    )
}

fn synchronize_source() -> String {
    format!(
        r#"
{common}

// Coincident vertices share the least room of their group.
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let v = id.x;
    if (v >= params.vertices || (params.gated != 0u && quiet())) {{
        return;
    }}
    var member = links[v * 2u];
    var room = rooms[member];
    loop {{
        member = links[member * 2u + 1u];
        if (member == NO_VERTEX) {{
            break;
        }}
        room = min(room, rooms[member]);
    }}
    rooms[params.vertices + v] = room;
}}
"#,
        common = common(),
    )
}

fn restart_source() -> String {
    format!(
        r#"
{common}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let v = id.x;
    if (v >= params.vertices || (params.gated != 0u && quiet())) {{
        return;
    }}
    rooms[v] = start_room(v);
}}
"#,
        common = common(),
    )
}

fn test_source() -> String {
    format!(
        r#"
{common}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let f = id.x;
    if (f >= params.faces || quiet()) {{
        return;
    }}
    if (!holds(f, NO_VERTEX)) {{
        enlist(f);
    }}
}}
"#,
        common = common(),
    )
}

/// The workgroup's shared worklist: reading its words alike, and gathering raised flags.
const WORKLIST: &str = r#"
var<workgroup> raised: atomic<u32>;
var<workgroup> published: u32;
var<private> lane: u32;

// A worklist word, read alike by every invocation.
fn shared_word(index: u32) -> u32 {
    storageBarrier();
    workgroupBarrier();
    if (lane == 0u) {
        published = atomicLoad(&worklist[index]);
    }
    return workgroupUniformLoad(&published);
}

// Whether any invocation raised the flag since the last call; clears it.
fn any_raised() -> bool {
    storageBarrier();
    workgroupBarrier();
    if (lane == 0u) {
        published = atomicLoad(&raised);
        atomicStore(&raised, 0u);
    }
    return workgroupUniformLoad(&published) != 0u;
}

fn candidate(slot: u32) -> u32 {
    return atomicLoad(&worklist[MARKS + params.faces + slot]);
}

fn enlist_neighbours(f: u32) {
    for (var corner = 0u; corner < 3u; corner = corner + 1u) {
        let v = triangles[f * 3u + corner];
        for (var entry = incidence[v]; entry < incidence[v + 1u]; entry = entry + 1u) {
            enlist(incidence[entry]);
        }
    }
}

"#;

fn settle_source() -> String {
    format!(
        r#"
{common}
const SWEEP_ITERATIONS: u32 = {iterations}u;
const WORKGROUP: u32 = {workgroup}u;

{worklist}
@compute @workgroup_size({workgroup})
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {{
    lane = local_id.x;
    if (shared_word(QUIET) != 0u) {{
        return;
    }}
    let failed = shared_word(LENGTH);
    if (failed == 0u) {{
        if (lane == 0u) {{
            atomicStore(&worklist[QUIET], 1u);
        }}
        return;
    }}
    for (var slot = lane; slot < failed; slot = slot + WORKGROUP) {{
        enlist_neighbours(candidate(slot));
    }}
    var current = 0u;
    var settled = false;
    for (var iteration = 0u; iteration < SWEEP_ITERATIONS; iteration = iteration + 1u) {{
        let count = shared_word(LENGTH);
        let next = 1u - current;
        for (var slot = lane; slot < count; slot = slot + WORKGROUP) {{
            let f = candidate(slot);
            let failure = select(1u, 0u, holds(f, current));
            set_verdict(next, f, failure);
            if (failure != verdict(current, f)) {{
                atomicOr(&raised, 1u);
                if (failure != 0u) {{
                    enlist_neighbours(f);
                }}
            }}
        }}
        current = next;
        if (!any_raised()) {{
            settled = true;
            break;
        }}
    }}
    if (!settled && lane == 0u) {{
        fail(STATUS_UNSETTLED);
    }}
    let count = shared_word(LENGTH);
    for (var slot = lane; slot < count; slot = slot + WORKGROUP) {{
        let f = candidate(slot);
        if (verdict(current, f) != 0u) {{
            for (var corner = 0u; corner < 3u; corner = corner + 1u) {{
                let v = triangles[f * 3u + corner];
                rooms[v] = ldexp(start_room(v), -halvings(v, current, params.faces));
            }}
        }}
    }}
    storageBarrier();
    workgroupBarrier();
    for (var slot = lane; slot < count; slot = slot + WORKGROUP) {{
        let f = candidate(slot);
        set_verdict(0u, f, 0u);
        set_verdict(1u, f, 0u);
        atomicStore(&worklist[MARKS + f], 0u);
    }}
    storageBarrier();
    workgroupBarrier();
    if (lane == 0u) {{
        atomicStore(&worklist[LENGTH], 0u);
    }}
}}
"#,
        worklist = WORKLIST,
        common = common(),
        iterations = SWEEP_ITERATIONS,
        workgroup = SETTLE_WORKGROUP,
    )
}
