//! The character's body as a distance field on the GPU.
//!
//! Placing hair on a character tests every point of it against the body,
//! many times over: lifting the roots onto the skin, the drape's solves,
//! the shrinkwrap, keeping every level of detail clear. The body is a mesh
//! of thousands of triangles, so each test is a search; sampled once onto
//! a grid around the hair, the body's distance is eight reads and a blend,
//! and every one of the hair crate's kernels collides with it as with the
//! mannequin ([`FieldHeader`]).
//!
//! The grid is filled in three steps, all on the GPU. Each triangle marks
//! the cells within a cell and a half of it with itself, keeping the
//! nearest triangle where several reach a cell. A jump flood then carries
//! the nearest triangle out to every cell: each cell looks at the cells a
//! step away in every direction and keeps whichever of their triangles is
//! nearer, the step halving from half the grid down to one. Last, each cell
//! takes its distance to its triangle, signed as the CPU's mesh collider
//! signs it -- by the side of the triangle the cell is on, by the vertex
//! normals blended at the nearest point -- and anything further than
//! [`REACH`] is far outside.

use std::sync::Arc;

use anyhow::Result;
use fabelgeist_compute::{Kernel, KernelBatch};
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_gpu::prelude::Buffer;
use fabelgeist_hair::gpu::FieldHeader;
use fabelgeist_hair::surface::vertex_normals;
use fabelgeist_math::Vec3;

use super::{HairGpu, KernelSource, params, storage, upload};
use crate::hair::BodySurface;

/// The cell aimed for, metres: the body is planar between its vertices, a
/// centimetre or more apart, and a planar field blends exactly, so a few
/// millimetres keep a root's clearance of half a millimetre.
const CELL: f32 = 0.0025;
/// The most cells the grid may have; past it, cells grow.
const MAX_CELLS: f32 = 16_000_000.0;
/// How far from the body the field is exact, metres: further, everything
/// is outside, as the CPU's collider answers past its reach.
pub(super) const REACH: f32 = 0.1;
/// How far round a triangle it marks cells, in cells.
const SEED_REACH: f32 = 1.5;
/// Steps of a seed's distance in a cell's mark: the nearest mark wins.
const SEED_STEPS: u32 = 4095;
/// Bits of a mark the triangle takes: at most a million triangles.
const TRIANGLE_BITS: u32 = 20;
/// Everything is this far from no body at all, metres.
const NOWHERE: f32 = 1000.0;

/// A distance field the hair kernels collide with: the header and the
/// distances, as `head_shape` reads them.
pub(crate) struct BodyField {
    pub(crate) buffer: Buffer,
}

/// The field's kernels.
pub(super) struct FieldKernels {
    seed: Arc<Kernel>,
    flood_start: Arc<Kernel>,
    flood_step: Arc<Kernel>,
    finish: Arc<Kernel>,
}

impl FieldKernels {
    pub(super) fn new(source: &KernelSource) -> Result<Self> {
        Ok(Self {
            seed: source.compile(&grid_kernel(SEED))?,
            flood_start: source.compile(&grid_kernel(FLOOD_START))?,
            flood_step: source.compile(&grid_kernel(FLOOD_STEP))?,
            finish: source.compile(&grid_kernel(FINISH))?,
        })
    }
}

impl BodyField {
    /// `body` sampled over the box from `lo` to `hi`, model space.
    pub(crate) fn sample(
        gpu: &HairGpu,
        body: &BodySurface<'_>,
        [lo, hi]: [Vec3; 2],
    ) -> Result<Self> {
        let context = gpu.context();
        let kernels = &gpu.field;
        let extent = hi - lo;
        let wanted = (extent.x / CELL) * (extent.y / CELL) * (extent.z / CELL);
        let cell = CELL * (wanted / MAX_CELLS).max(1.0).cbrt();
        let header = FieldHeader {
            lo,
            cell,
            dims: [extent.x, extent.y, extent.z].map(|e| ((e / cell).ceil() as u32).max(1)),
            reach: REACH,
        };
        let grid = Grid {
            header,
            mesh: NearMesh::upload(context, body, [lo, hi])?,
        };
        let cells = header.cells() as u32;
        let mut entries = header.entries().to_vec();
        entries.resize(header.len(), [0.0; 4]);
        let buffer = upload(context, &entries, "hair body field")?;
        let marks = storage(context, u64::from(cells) * 4, "hair body marks")?;

        let mut batch = KernelBatch::labelled(context, "hair body field");
        batch.encoder().clear_buffer(&marks.buffer, 0, None);
        batch.dispatch_folded(
            &kernels.seed,
            &grid.params(params! {
                "marks" => marks.clone(),
                "count" => grid.mesh.triangles,
                "reach" => SEED_REACH * cell,
                "steps" => SEED_STEPS,
                "triangle_bits" => TRIANGLE_BITS,
            }),
            grid.mesh.triangles,
        )?;
        let nearest = grid.flood(context, &mut batch, kernels, marks)?;
        batch.dispatch_folded(
            &kernels.finish,
            &grid.params(params! {
                "nearest" => nearest,
                "field" => buffer.clone(),
                "reach" => REACH,
                "header" => FieldHeader::ENTRIES as u32,
                "pad0" => 0u32,
                "pad1" => 0u32,
            }),
            cells.div_ceil(4),
        )?;
        batch.submit();
        Ok(Self { buffer })
    }

    /// No body at all: everything is far outside it.
    pub(super) fn nowhere(context: &WgpuContext) -> Result<Self> {
        let header = FieldHeader {
            lo: Vec3::splat(-NOWHERE),
            cell: NOWHERE,
            dims: [1, 1, 1],
            reach: NOWHERE,
        };
        let mut entries = header.entries().to_vec();
        entries.push([NOWHERE; 4]);
        Ok(Self {
            buffer: upload(context, &entries, "hair no body")?,
        })
    }
}

/// The body's triangles near enough a box to be any of its cells' nearest,
/// on the GPU: every point and its vertex normal, and those triangles.
struct NearMesh {
    points: Buffer,
    normals: Buffer,
    faces: Buffer,
    triangles: u32,
}

impl NearMesh {
    fn upload(context: &WgpuContext, body: &BodySurface<'_>, [lo, hi]: [Vec3; 2]) -> Result<Self> {
        let positions: Vec<Vec3> = body
            .positions
            .iter()
            .copied()
            .map(Vec3::from_array)
            .collect();
        let normals = vertex_normals(&positions, body.faces);
        let (near_lo, near_hi) = (lo - Vec3::splat(REACH), hi + Vec3::splat(REACH));
        let faces: Vec<[u32; 4]> = body
            .faces
            .iter()
            .filter(|face| {
                let corners = face.map(|corner| positions[corner as usize]);
                let low = corners[0].min(corners[1]).min(corners[2]);
                let high = corners[0].max(corners[1]).max(corners[2]);
                low.x <= near_hi.x
                    && low.y <= near_hi.y
                    && low.z <= near_hi.z
                    && high.x >= near_lo.x
                    && high.y >= near_lo.y
                    && high.z >= near_lo.z
            })
            .map(|&[a, b, c]| [a, b, c, 0])
            .collect();
        let four = |v: &[Vec3]| v.iter().map(|p| [p.x, p.y, p.z, 0.0]).collect::<Vec<_>>();
        Ok(Self {
            points: upload(context, &four(&positions), "hair body points")?,
            normals: upload(context, &four(&normals), "hair body normals")?,
            faces: upload(context, &faces, "hair body triangles")?,
            triangles: faces.len() as u32,
        })
    }
}

/// The field's grid and the triangles it is sampled from, as every field
/// kernel takes them.
struct Grid {
    header: FieldHeader,
    mesh: NearMesh,
}

impl Grid {
    /// `extra`, with the grid and the triangles.
    fn params(&self, mut p: super::Params) -> super::Params {
        let FieldHeader { lo, cell, dims, .. } = self.header;
        p.insert("lo_x", lo.x);
        p.insert("lo_y", lo.y);
        p.insert("lo_z", lo.z);
        p.insert("cell", cell);
        p.insert("nx", dims[0]);
        p.insert("ny", dims[1]);
        p.insert("nz", dims[2]);
        p.insert("cells", self.header.cells() as u32);
        p.insert("points", self.mesh.points.clone());
        p.insert("normals", self.mesh.normals.clone());
        p.insert("triangles", self.mesh.faces.clone());
        p
    }

    /// Every cell's nearest triangle, flooded out from the triangles'
    /// `marks`: a buffer of triangles, plus one, a cell.
    fn flood(
        &self,
        context: &WgpuContext,
        batch: &mut KernelBatch<'_>,
        kernels: &FieldKernels,
        marks: Buffer,
    ) -> Result<Buffer> {
        let cells = self.header.cells() as u32;
        let flood = [
            storage(context, u64::from(cells) * 4, "hair body nearest")?,
            storage(context, u64::from(cells) * 4, "hair body nearest, next")?,
        ];
        batch.dispatch_folded(
            &kernels.flood_start,
            &self.params(params! {
                "marks" => marks,
                "nearest" => flood[0].clone(),
                "triangle_bits" => TRIANGLE_BITS,
                "pad0" => 0u32,
                "pad1" => 0u32,
                "pad2" => 0u32,
            }),
            cells,
        )?;
        let widest = self
            .header
            .dims
            .into_iter()
            .max()
            .unwrap_or(1)
            .next_power_of_two();
        let mut steps: Vec<u32> =
            std::iter::successors(Some(widest / 2), |&step| (step > 1).then_some(step / 2))
                .filter(|&step| step >= 1)
                .collect();
        // One more step of one: the flood's few wrong cells come right.
        steps.push(1);
        let mut current = 0;
        for step in steps {
            batch.dispatch_folded(
                &kernels.flood_step,
                &self.params(params! {
                    "nearest" => flood[current].clone(),
                    "next" => flood[1 - current].clone(),
                    "step" => step,
                    "pad0" => 0u32,
                    "pad1" => 0u32,
                    "pad2" => 0u32,
                }),
                cells,
            )?;
            current = 1 - current;
        }
        let [first, second] = flood;
        Ok(if current == 0 { first } else { second })
    }
}

/// The grid, the body's triangles and the WGSL both need.
const GRID: &str = r#"
struct Params {
    lo_x: f32,
    lo_y: f32,
    lo_z: f32,
    cell: f32,
    nx: u32,
    ny: u32,
    nz: u32,
    cells: u32,
    EXTRA
};
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> points: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> normals: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> triangles: array<vec4<u32>>;

fn dims() -> vec3<i32> {
    return vec3<i32>(i32(params.nx), i32(params.ny), i32(params.nz));
}

fn cell_of(k: u32) -> vec3<i32> {
    let n = dims();
    let i = i32(k);
    return vec3<i32>(i % n.x, (i / n.x) % n.y, i / (n.x * n.y));
}

fn index_of(c: vec3<i32>) -> u32 {
    let n = dims();
    return u32(c.x + n.x * (c.y + n.y * c.z));
}

fn centre(c: vec3<i32>) -> vec3<f32> {
    return vec3<f32>(params.lo_x, params.lo_y, params.lo_z) + (vec3<f32>(c) + vec3<f32>(0.5)) * params.cell;
}

fn corner(t: u32, k: u32) -> vec3<f32> {
    return points[triangles[t][k]].xyz;
}

// The nearest point of triangle `t` to `p` (Ericson's).
fn closest(p: vec3<f32>, t: u32) -> vec3<f32> {
    let a = corner(t, 0u);
    let b = corner(t, 1u);
    let c = corner(t, 2u);
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if (d1 <= 0.0 && d2 <= 0.0) {
        return a;
    }
    let bp = p - b;
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if (d3 >= 0.0 && d4 <= d3) {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if (vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0) {
        return a + ab * (d1 / (d1 - d3));
    }
    let cp = p - c;
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if (d6 >= 0.0 && d5 <= d6) {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if (vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0) {
        return a + ac * (d2 / (d2 - d6));
    }
    let va = d3 * d6 - d5 * d4;
    if (va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0) {
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }
    let scale = 1.0 / (va + vb + vc);
    return a + ab * (vb * scale) + ac * (vc * scale);
}

fn folded(id: vec3<u32>, groups: vec3<u32>) -> u32 {
    return id.y * groups.x * 64u + id.x;
}
"#;

const SEED: &str = r#"
    count: u32,
    reach: f32,
    steps: u32,
    triangle_bits: u32,
@group(0) @binding(4) var<storage, read_write> marks: array<atomic<u32>>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let t = folded(id, groups);
    if (t >= params.count) {
        return;
    }
    let lo = vec3<f32>(params.lo_x, params.lo_y, params.lo_z);
    let a = corner(t, 0u);
    let b = corner(t, 1u);
    let c = corner(t, 2u);
    let low = (min(a, min(b, c)) - lo - vec3<f32>(params.reach)) / params.cell - vec3<f32>(0.5);
    let high = (max(a, max(b, c)) - lo + vec3<f32>(params.reach)) / params.cell - vec3<f32>(0.5);
    let n = dims();
    let first = clamp(vec3<i32>(floor(low)), vec3<i32>(0), n - vec3<i32>(1));
    let last = clamp(vec3<i32>(ceil(high)), vec3<i32>(0), n - vec3<i32>(1));
    if (any(vec3<i32>(ceil(high)) < vec3<i32>(0)) || any(vec3<i32>(floor(low)) > n - vec3<i32>(1))) {
        return;
    }
    for (var z = first.z; z <= last.z; z++) {
        for (var y = first.y; y <= last.y; y++) {
            for (var x = first.x; x <= last.x; x++) {
                let cell = vec3<i32>(x, y, z);
                let p = centre(cell);
                let d = distance(p, closest(p, t));
                if (d < params.reach) {
                    let near = params.steps - u32(d / params.reach * f32(params.steps));
                    atomicMax(&marks[index_of(cell)], (near << params.triangle_bits) | t);
                }
            }
        }
    }
}
"#;

const FLOOD_START: &str = r#"
    triangle_bits: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
@group(0) @binding(4) var<storage, read> marks: array<u32>;
@group(0) @binding(5) var<storage, read_write> nearest: array<u32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let k = folded(id, groups);
    if (k >= params.cells) {
        return;
    }
    let mark = marks[k];
    // A cell's nearest triangle, plus one: none is 0.
    nearest[k] = select(0u, (mark & ((1u << params.triangle_bits) - 1u)) + 1u, mark != 0u);
}
"#;

const FLOOD_STEP: &str = r#"
    step: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
@group(0) @binding(4) var<storage, read> nearest: array<u32>;
@group(0) @binding(5) var<storage, read_write> next: array<u32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let k = folded(id, groups);
    if (k >= params.cells) {
        return;
    }
    let here = cell_of(k);
    let p = centre(here);
    let n = dims();
    let step = i32(params.step);
    var best = nearest[k];
    var best_distance = 3.4e38;
    if (best != 0u) {
        best_distance = distance(p, closest(p, best - 1u));
    }
    for (var dz = -1; dz <= 1; dz++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dx = -1; dx <= 1; dx++) {
                let other = here + vec3<i32>(dx, dy, dz) * step;
                if (any(other < vec3<i32>(0)) || any(other >= n)) {
                    continue;
                }
                let candidate = nearest[index_of(other)];
                if (candidate == 0u || candidate == best) {
                    continue;
                }
                let d = distance(p, closest(p, candidate - 1u));
                if (d < best_distance) {
                    best = candidate;
                    best_distance = d;
                }
            }
        }
    }
    next[k] = best;
}
"#;

const FINISH: &str = r#"
    reach: f32,
    header: u32,
    pad0: u32,
    pad1: u32,
@group(0) @binding(4) var<storage, read> nearest: array<u32>;
@group(0) @binding(5) var<storage, read_write> field: array<vec4<f32>>;

// The vertex normals of triangle `t` blended at `q`, a point of it.
fn normal_at(q: vec3<f32>, t: u32) -> vec3<f32> {
    let a = corner(t, 0u);
    let v0 = corner(t, 1u) - a;
    let v1 = corner(t, 2u) - a;
    let v2 = q - a;
    let d00 = dot(v0, v0);
    let d01 = dot(v0, v1);
    let d11 = dot(v1, v1);
    let d20 = dot(v2, v0);
    let d21 = dot(v2, v1);
    let denominator = d00 * d11 - d01 * d01;
    var w = vec3<f32>(1.0, 0.0, 0.0);
    if (abs(denominator) > 1e-20) {
        let v = (d11 * d20 - d01 * d21) / denominator;
        let u = (d00 * d21 - d01 * d20) / denominator;
        w = vec3<f32>(1.0 - v - u, v, u);
    }
    let tri = triangles[t];
    return normals[tri.x].xyz * w.x + normals[tri.y].xyz * w.y + normals[tri.z].xyz * w.z;
}

fn signed_distance(k: u32) -> f32 {
    if (k >= params.cells) {
        return params.reach;
    }
    let t = nearest[k];
    if (t == 0u) {
        return params.reach;
    }
    let p = centre(cell_of(k));
    let q = closest(p, t - 1u);
    let d = distance(p, q);
    if (d > params.reach) {
        return params.reach;
    }
    if (dot(p - q, normal_at(q, t - 1u)) >= 0.0) {
        return d;
    }
    return -d;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let e = folded(id, groups);
    if (e * 4u >= params.cells) {
        return;
    }
    field[params.header + e] = vec4<f32>(
        signed_distance(e * 4u),
        signed_distance(e * 4u + 1u),
        signed_distance(e * 4u + 2u),
        signed_distance(e * 4u + 3u),
    );
}
"#;

/// A kernel's source: the grid, then its own parameters spliced into the
/// grid's, and its body.
fn grid_kernel(own: &str) -> String {
    let (extra, body) = own
        .split_once("@group")
        .map_or((own, ""), |(extra, body)| (extra, body));
    GRID.replace("EXTRA", extra.trim()) + "@group" + body
}
