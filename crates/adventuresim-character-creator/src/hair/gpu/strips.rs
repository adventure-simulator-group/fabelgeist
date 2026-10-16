//! Hair's levels of detail as triangles, built on the GPU.
//!
//! Cards and strand ribbons are both strips: rows of two points, one either
//! side of the strip's middle, joined by straight segments. Cards come from
//! the hair crate's cuts, already strips; strands become ribbons, each point
//! widened across the strand, square to the direction out from the head's
//! centre, as the hair crate's cards lie, and tapering to the tip. A level
//! drawn with a share of its strands draws them wider by the inverse of
//! that share, so the head's coverage holds; no ribbon is drawn narrower
//! than a renderer without the crate's sub-pixel coverage can show.
//!
//! The hair crate keeps a strand's points and a card's spine outside the
//! body, but over a rounded shoulder or skull a straight segment between two
//! rows just outside the skin passes under it in the middle -- the longer
//! the segment, the deeper: the coarse levels most. So every segment is cut
//! into pieces no longer than [`LONGEST_SEGMENT`], and every row near the
//! skin is moved off it by [`CLEARANCE`] and by as much as the strip sags
//! across its width over the tightest curve hair lies on. A row's lift
//! spreads to its neighbours, tapering over [`LIFT_SPREAD`] rows, so a strip
//! bends off the skin rather than in a sawtooth. The hair crate turns a card
//! to face away from the head's centre, right for a card hanging beside the
//! head; a card lying on a shoulder would stand on its edge there, a fin, so
//! near the skin each row turns to lie flat on it instead, fully within
//! [`LIE_FLAT_REACH`] of touching.
//!
//! One invocation builds one strip, into one row of the output buffers per
//! row of the strip, both edges of a row side by side.

use std::sync::Arc;

use anyhow::Result;
use fabelgeist_compute::{Kernel, KernelBatch};
use fabelgeist_gpu::prelude::Buffer;
use fabelgeist_hair::gpu::wgsl::common::{HEAD, MATH};
use fabelgeist_hair::gpu::{GpuCards, GpuLevel};
use fabelgeist_hair::material::Taper;
use fabelgeist_math::Vec3;

use super::{HairGpu, KernelSource, params, storage, upload};
use crate::hair::HairGeometry;

/// How far off the skin hair is kept, metres: as close as the hair crate
/// keeps a root, over the scalp cap, so roots leave the scalp rather than
/// hover over it.
const CLEARANCE: f32 = 0.0006;
/// The longest segment drawn straight, metres: a straight segment this long
/// over the tightest curve dips under its ends by a quarter of a
/// millimetre, inside the clearance.
const LONGEST_SEGMENT: f32 = 0.01;
/// The tightest curve hair lies over, metres of radius: the neck's.
const TIGHTEST_CURVE: f32 = 0.05;
/// Rows either side a row's lift spreads to, tapering.
const LIFT_SPREAD: u32 = 4;
/// How near the skin a row turns to lie flat on it, metres, fully at the
/// skin and not at all this far off.
const LIE_FLAT_REACH: f32 = 0.03;
/// The narrowest a strand ribbon is drawn, metres.
const MIN_RIBBON_WIDTH: f32 = 0.0004;
/// The most pieces a card's segment is cut into: past this, a segment is
/// longer than any cut of cards leaves one.
const MOST_CARD_PIECES: u32 = 16;
/// The most pieces a ribbon's segment is cut into: a coarse level is drawn
/// for a head far off, where a few rows more keep it off the skin and more
/// would only cost.
const MOST_RIBBON_PIECES: u32 = 8;
/// Samples of the strands' taper from root to tip, blended between.
const TAPER_SAMPLES: usize = 33;
/// Bytes of one vertex attribute: four floats.
const ATTRIBUTE_BYTES: u64 = 16;

/// The strip kernels.
pub(super) struct StripKernels {
    cards: Arc<Kernel>,
    ribbons: Arc<Kernel>,
}

impl StripKernels {
    pub(super) fn new(source: &KernelSource) -> Result<Self> {
        let kernel = |rows: &str| format!("{BINDINGS}{rows}{WRAP}{MATH}{HEAD}");
        Ok(Self {
            cards: source.compile(&kernel(CARD_ROWS))?,
            ribbons: source.compile(&kernel(RIBBON_ROWS))?,
        })
    }
}

/// Triangles on the GPU: each vertex's attributes, four floats each, and
/// how many strips of how many rows, each row's two edges side by side.
pub struct GpuGeometry {
    /// Position, and baked sky visibility.
    positions: Buffer,
    /// Normal, and the strip's own random value.
    normals: Buffer,
    /// Along the strands, root to tip.
    tangents: Buffer,
    /// Texture coordinates.
    texcoords: Buffer,
    strips: u32,
    rows: u32,
}

/// What a strip kernel builds from: the strips' source rows and how long
/// their segments may be.
pub(crate) struct StripSource<'a> {
    /// Kept clear of: `head_shape`'s layout, shapes or a sampled field.
    pub(crate) body: &'a Buffer,
    /// The longest any strand is, metres: how finely segments are cut.
    pub(crate) longest: f32,
}

impl StripSource<'_> {
    /// Pieces each of `segments` segments is cut into, at most `most`.
    fn pieces(&self, segments: u32, most: u32) -> u32 {
        let segment = self.longest / segments.max(1) as f32;
        ((segment / LONGEST_SEGMENT).ceil() as u32).clamp(1, most)
    }

    /// Rows a ribbon of `points` points is drawn with.
    pub(crate) fn ribbon_rows(&self, points: u32) -> u32 {
        let segments = points.max(2) - 1;
        segments * self.pieces(segments, MOST_RIBBON_PIECES) + 1
    }

    /// The cut `cards` as strips, their atlas `variants` wide.
    pub(crate) fn cards(
        &self,
        gpu: &HairGpu,
        cards: &GpuCards,
        variants: u32,
    ) -> Result<GpuGeometry> {
        let source_rows = cards.segments + 1;
        let pieces = self.pieces(cards.segments, MOST_CARD_PIECES);
        let p = params! {
            "card_vertices" => cards.vertices.clone(),
            "source_rows" => source_rows,
            "variants" => variants.max(1) as f32,
            "pad2" => 0u32,
        };
        self.build(gpu, &gpu.strips.cards, p, cards.cards, source_rows, pieces)
    }

    /// The first `drawn` strands of `level` as ribbons, `widen` times as
    /// wide as the strands they draw, `center` the head's.
    pub(crate) fn ribbons(
        &self,
        gpu: &HairGpu,
        level: &GpuLevel,
        drawn: u32,
        widen: f32,
        center: Vec3,
    ) -> Result<GpuGeometry> {
        let taper = Taper::default();
        let samples: Vec<f32> = (0..TAPER_SAMPLES)
            .map(|k| taper.at(k as f32 / (TAPER_SAMPLES - 1) as f32))
            .collect();
        let pieces = self.pieces(level.points.max(2) - 1, MOST_RIBBON_PIECES);
        let p = params! {
            "level_positions" => level.positions.clone(),
            "level_infos" => level.infos.clone(),
            "taper" => upload(gpu.context(), &samples, "hair taper")?,
            "source_rows" => level.points,
            "widen" => widen,
            "min_width" => MIN_RIBBON_WIDTH,
            "taper_samples" => TAPER_SAMPLES as u32,
            "center_x" => center.x,
            "center_y" => center.y,
            "center_z" => center.z,
        };
        self.build(gpu, &gpu.strips.ribbons, p, drawn, level.points, pieces)
    }

    fn build(
        &self,
        gpu: &HairGpu,
        kernel: &Kernel,
        mut p: super::Params,
        strips: u32,
        source_rows: u32,
        pieces: u32,
    ) -> Result<GpuGeometry> {
        let context = gpu.context();
        let rows = (source_rows.max(2) - 1) * pieces + 1;
        let vertices = u64::from(strips) * u64::from(rows) * 2;
        let attribute = |label: &str| storage(context, vertices * ATTRIBUTE_BYTES, label);
        let geometry = GpuGeometry {
            positions: attribute("hair strip positions")?,
            normals: attribute("hair strip normals")?,
            tangents: attribute("hair strip tangents")?,
            texcoords: attribute("hair strip texture coordinates")?,
            strips,
            rows,
        };
        let lifts = storage(
            context,
            u64::from(strips) * u64::from(rows) * ATTRIBUTE_BYTES,
            "hair strip lifts",
        )?;
        p.insert("positions", geometry.positions.clone());
        p.insert("normals", geometry.normals.clone());
        p.insert("tangents", geometry.tangents.clone());
        p.insert("texcoords", geometry.texcoords.clone());
        p.insert("lifts", lifts);
        p.insert("head_shape", self.body.clone());
        p.insert("strips", strips);
        p.insert("rows", rows);
        p.insert("pieces", pieces);
        p.insert("spread", LIFT_SPREAD);
        p.insert("clearance", CLEARANCE);
        p.insert("tightest_curve", TIGHTEST_CURVE);
        p.insert("lie_flat_reach", LIE_FLAT_REACH);
        p.insert("pad0", 0u32);
        p.insert("pad1", 0u32);
        let mut batch = KernelBatch::labelled(context, "hair strips");
        batch.dispatch_folded(kernel, &p, strips)?;
        batch.submit();
        Ok(geometry)
    }
}

impl GpuGeometry {
    /// Position, and baked sky visibility.
    pub fn positions(&self) -> &Buffer {
        &self.positions
    }

    /// Normal, and the strip's own random value.
    pub fn normals(&self) -> &Buffer {
        &self.normals
    }

    /// Along the strands, root to tip.
    pub fn tangents(&self) -> &Buffer {
        &self.tangents
    }

    /// Texture coordinates.
    pub fn texcoords(&self) -> &Buffer {
        &self.texcoords
    }

    /// The triangles back on the host.
    pub fn read(&self, gpu: &HairGpu) -> Result<HairGeometry> {
        let context = gpu.context();
        let read = |buffer: &Buffer| -> Result<Vec<[f32; 4]>> {
            let mut all: Vec<[f32; 4]> = pollster::block_on(buffer.read(context))?;
            all.truncate(self.vertices());
            Ok(all)
        };
        let positions = read(&self.positions)?;
        let normals = read(&self.normals)?;
        let tangents = read(&self.tangents)?;
        let texcoords = read(&self.texcoords)?;
        let xyz = |v: &[f32; 4]| [v[0], v[1], v[2]];
        Ok(HairGeometry {
            positions: positions.iter().map(xyz).collect(),
            normals: normals.iter().map(xyz).collect(),
            texcoords: texcoords.iter().map(|t| [t[0], t[1]]).collect(),
            occlusion: positions.iter().map(|p| p[3]).collect(),
            randoms: normals.iter().map(|n| n[3]).collect(),
            tangents: tangents.iter().map(xyz).collect(),
            faces: self.faces(),
        })
    }

    /// Vertices: two a row.
    pub fn vertices(&self) -> usize {
        self.strips as usize * self.rows as usize * 2
    }

    /// Two triangles a segment, every strip's rows in order.
    pub fn faces(&self) -> Vec<[u32; 3]> {
        let rows = self.rows;
        (0..self.strips)
            .flat_map(|strip| {
                let base = strip * rows * 2;
                (0..rows.saturating_sub(1)).flat_map(move |segment| {
                    let [a, b] = [base + 2 * segment, base + 2 * segment + 1];
                    let [c, d] = [a + 2, b + 2];
                    [[a, c, b], [b, c, d]]
                })
            })
            .collect()
    }
}

const BINDINGS: &str = r#"
@group(0) @binding(0) var<storage, read_write> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> normals: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> tangents: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> texcoords: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> lifts: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read> head_shape: array<vec4<f32>>;

// One edge point of a strip's row.
struct StripPoint {
    position: vec3<f32>,
    normal: vec3<f32>,
    tangent: vec3<f32>,
    texcoord: vec2<f32>,
    occlusion: f32,
    random: f32,
};

fn lerp_point(a: StripPoint, b: StripPoint, t: f32) -> StripPoint {
    var p: StripPoint;
    p.position = a.position + (b.position - a.position) * t;
    p.normal = safe_normalize(mix(a.normal, b.normal, t), a.normal);
    p.tangent = safe_normalize(mix(a.tangent, b.tangent, t), a.tangent);
    p.texcoord = a.texcoord + (b.texcoord - a.texcoord) * t;
    p.occlusion = a.occlusion + (b.occlusion - a.occlusion) * t;
    p.random = a.random;
    return p;
}
"#;

/// Cards: the hair crate's card vertices, four entries each, a card's rows
/// in order, both edges of a row side by side.
const CARD_ROWS: &str = r#"
@group(0) @binding(6) var<storage, read> card_vertices: array<vec4<f32>>;
struct Params {
    strips: u32,
    rows: u32,
    pieces: u32,
    spread: u32,
    clearance: f32,
    tightest_curve: f32,
    lie_flat_reach: f32,
    source_rows: u32,
    variants: f32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(7) var<uniform> params: Params;

fn source_point(strip: u32, row: u32, edge: u32) -> StripPoint {
    let v = ((strip * params.source_rows + row) * 2u + edge) * 4u;
    var p: StripPoint;
    p.position = card_vertices[v].xyz;
    p.tangent = card_vertices[v + 1u].xyz;
    p.normal = card_vertices[v + 2u].xyz;
    let variant = card_vertices[v + 2u].w;
    p.texcoord = vec2<f32>((variant + card_vertices[v].w) / params.variants, card_vertices[v + 1u].w);
    p.random = card_vertices[v + 3u].x;
    p.occlusion = card_vertices[v + 3u].y;
    return p;
}
"#;

/// Ribbons: a strand level's points, position and sky visibility, and each
/// strand's radius among its infos.
const RIBBON_ROWS: &str = r#"
@group(0) @binding(6) var<storage, read> level_positions: array<vec4<f32>>;
@group(0) @binding(7) var<storage, read> level_infos: array<vec4<f32>>;
@group(0) @binding(8) var<storage, read> taper: array<f32>;
struct Params {
    strips: u32,
    rows: u32,
    pieces: u32,
    spread: u32,
    clearance: f32,
    tightest_curve: f32,
    lie_flat_reach: f32,
    source_rows: u32,
    widen: f32,
    min_width: f32,
    taper_samples: u32,
    center_x: f32,
    center_y: f32,
    center_z: f32,
    pad0: u32,
    pad1: u32,
};
@group(0) @binding(9) var<uniform> params: Params;

fn taper_at(along: f32) -> f32 {
    let x = clamp(along, 0.0, 1.0) * f32(params.taper_samples - 1u);
    let i = min(u32(floor(x)), params.taper_samples - 2u);
    return mix(taper[i], taper[i + 1u], x - f32(i));
}

fn level_point(strip: u32, i: u32) -> vec3<f32> {
    return level_positions[strip * params.source_rows + i].xyz;
}

fn source_point(strip: u32, row: u32, edge: u32) -> StripPoint {
    let points = params.source_rows;
    let point = level_point(strip, row);
    let tangent = safe_normalize(
        level_point(strip, min(row + 1u, points - 1u)) - level_point(strip, select(row - 1u, 0u, row == 0u)),
        vec3<f32>(0.0, -1.0, 0.0),
    );
    let center = vec3<f32>(params.center_x, params.center_y, params.center_z);
    let side = safe_normalize(cross(tangent, point - center), vec3<f32>(1.0, 0.0, 0.0));
    let normal = safe_normalize(cross(side, tangent), vec3<f32>(0.0, 0.0, 1.0));
    let along = f32(row) / f32(max(points - 1u, 1u));
    let radius = level_infos[strip].x;
    let width = max(2.0 * radius * params.widen, params.min_width);
    let half = 0.5 * width * taper_at(along);
    var p: StripPoint;
    p.position = point + side * select(-half, half, edge == 1u);
    p.normal = normal;
    p.tangent = tangent;
    p.texcoord = vec2<f32>(f32(edge), along);
    p.occlusion = level_positions[strip * points + row].w;
    p.random = 0.0;
    return p;
}
"#;

/// The strip itself: cut, laid flat near the skin, written, then lifted off
/// it with the lift spread along the strip.
const WRAP: &str = r#"
fn smooth01(t: f32) -> f32 {
    let c = clamp(t, 0.0, 1.0);
    return c * c * (3.0 - 2.0 * c);
}

// Row `row` of the cut strip: between two of its source rows.
fn cut_point(strip: u32, row: u32, edge: u32) -> StripPoint {
    let segment = min(row / params.pieces, params.source_rows - 2u);
    let t = f32(row - segment * params.pieces) / f32(params.pieces);
    return lerp_point(source_point(strip, segment, edge), source_point(strip, segment + 1u, edge), t);
}

fn out_index(strip: u32, row: u32, edge: u32) -> u32 {
    return (strip * params.rows + row) * 2u + edge;
}

fn write_point(index: u32, p: StripPoint) {
    positions[index] = vec4<f32>(p.position, p.occlusion);
    normals[index] = vec4<f32>(p.normal, p.random);
    tangents[index] = vec4<f32>(p.tangent, 0.0);
    texcoords[index] = vec4<f32>(p.texcoord, 0.0, 0.0);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let strip = id.y * groups.x * 64u + id.x;
    if (strip >= params.strips) {
        return;
    }
    for (var row = 0u; row < params.rows; row++) {
        var left = cut_point(strip, row, 0u);
        var right = cut_point(strip, row, 1u);
        // Turned about its middle to lie flat on the skin near it: its width
        // square to both the strands and the skin's normal.
        let centre = (left.position + right.position) * 0.5;
        let weight = smooth01(1.0 - head_distance(centre) / params.lie_flat_reach);
        let across = right.position - left.position;
        let width = length(across);
        let flat_raw = cross(left.tangent, head_gradient(centre));
        if (weight > 0.0 && width > 1e-6 && length(flat_raw) > 1e-6) {
            let side = across / width;
            var flat = normalize(flat_raw);
            if (dot(flat, side) < 0.0) {
                flat = -flat;
            }
            let turned = normalize(mix(side, flat, weight));
            var face = normalize(cross(turned, left.tangent));
            if (dot(face, left.normal) < 0.0) {
                face = -face;
            }
            left.position = centre - turned * (0.5 * width);
            right.position = centre + turned * (0.5 * width);
            left.normal = face;
            right.normal = face;
        }
        write_point(out_index(strip, row, 0u), left);
        write_point(out_index(strip, row, 1u), right);
        // How far the row must move off the skin: whole, as far as its
        // deepest point must, by its clearance and its sag across its
        // width.
        let clearance = params.clearance + width * width / (8.0 * params.tightest_curve);
        var deepest = centre;
        var depth = head_distance(centre);
        for (var e = 0u; e < 2u; e++) {
            let point = select(left.position, right.position, e == 1u);
            let d = head_distance(point);
            if (d < depth) {
                depth = d;
                deepest = point;
            }
        }
        var need = vec3<f32>(0.0);
        if (depth < clearance) {
            need = push_out(deepest, clearance) - deepest;
        }
        lifts[strip * params.rows + row] = vec4<f32>(need, 0.0);
    }
    // Every row moves as far as the most any row near it asks, scaled by
    // how near.
    for (var row = 0u; row < params.rows; row++) {
        let first = select(row - params.spread, 0u, row < params.spread);
        let last = min(row + params.spread, params.rows - 1u);
        var lift = vec3<f32>(0.0);
        for (var other = first; other <= last; other++) {
            let apart = select(row - other, other - row, other > row);
            let taper = 1.0 - f32(apart) / f32(params.spread + 1u);
            let asked = lifts[strip * params.rows + other].xyz * taper;
            if (dot(asked, asked) > dot(lift, lift)) {
                lift = asked;
            }
        }
        for (var e = 0u; e < 2u; e++) {
            let index = out_index(strip, row, e);
            positions[index] = vec4<f32>(positions[index].xyz + lift, positions[index].w);
        }
    }
}
"#;
