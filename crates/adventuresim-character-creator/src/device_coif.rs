//! The mail coif fitted on the device.
//!
//! The coif's drape is measured on the torso and neck skin in the head frame,
//! in dependent steps: rig landmarks set the neck boundary's heights,
//! triangle sections find the neck's width and the chest and back depths there, the
//! boundary then sets the flaps' section heights, and a second reduction
//! finds the body's depth at every flap section. Each step between the two
//! reductions is one invocation.

use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_rig::{RigJointLookupError, RigJointName, RigJointOrdinal};
use std::f32::consts::{PI, TAU};
use std::sync::Arc;

use anyhow::{Result, ensure};
use fabelgeist_armor::gpu::COIF_DRAPE_SECTIONS;
use fabelgeist_armor::gpu::{COIF_DRAPE_WORDS, FIT_PROFILE_WORD, device_error, record_coif, wgsl};
use fabelgeist_armor::{ArmorGpu, CoifDesign, DevicePart, PartFrame};
use fabelgeist_compute::{Kernel, KernelBatch, host_float};
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use crate::armor_frames::{FitRegion, Wearer};
use crate::device_frames::{DeviceFrame, DeviceWearer};
use crate::device_piece::DeviceRecording;

const DRAPE_EASE_M: f32 = 0.004;
/// Neck columns either side of the front and back centre lines the flaps
/// hang from, of the coif's 48 around.
const FLAP_EDGE_COLUMNS: f32 = 4.0;
const AROUND: f32 = 48.0;
/// Work words: landmarks, the neck's reductions, the boundary, the section
/// heights and flap width, then one reduction per flap section and edge.
const QUERIES: usize = 2 * COIF_DRAPE_SECTIONS * 2;
const QUERY_START: usize = 28;
const ORDERED_ZERO: u32 = 0x8000_0000;
const ORDERED_NEGATIVE_INFINITY: u32 = 0x007f_ffff;

/// A worn coif as one open surface, for cloth rather than a rigid shell.
#[derive(Clone, Debug, PartialEq)]
pub struct CoifCarrier {
    /// Head-frame positions: x across, y up, z forward. Vertex 0 is the crown.
    pub positions: Vec<[f32; 3]>,
    /// Outward-wound triangles.
    pub indices: Vec<u32>,
}

impl CoifCarrier {
    /// The surface a recorded coif's shell thickens, in the coif's own
    /// frame, once its batch has run.
    pub fn read(gpu: &ArmorGpu, part: &DevicePart) -> Result<Self> {
        ensure!(
            gpu.read::<u32>(part.status())?[0] == 0,
            "the coif's carrier is not a surface"
        );
        let carriers = part.shell_carriers(0);
        let flat: Vec<[f32; 3]> = gpu.read(part.carriers())?;
        let positions = flat[carriers.start as usize..carriers.end as usize].to_vec();
        Ok(Self {
            positions,
            indices: part.shell_carrier_triangles(0).to_vec(),
        })
    }
}

/// The coif fitted to `wearer` as one open cloth surface, with the head
/// frame that places it.
pub fn fitted_coif_carrier(
    gpu: &ArmorGpu,
    wearer: &Wearer<'_>,
    design: &CoifDesign,
) -> Result<(PartFrame, CoifCarrier)> {
    let body = wearer.upload(gpu)?;
    let device = DeviceWearer {
        gpu,
        body: &body,
        host: wearer,
    };
    let mut batch = gpu.batch("coif carrier");
    let recording = device.record_fitted_coif(&mut batch, design)?;
    batch.submit();
    let (frame, region) = &recording.frames[0];
    frame.check(gpu, *region)?;
    Ok((frame.read(gpu)?, CoifCarrier::read(gpu, &recording.part)?))
}

impl DeviceWearer<'_> {
    /// Record a mail coif fitted to this wearer, short of thickening.
    pub fn record_fitted_coif(
        &self,
        batch: &mut KernelBatch,
        design: &CoifDesign,
    ) -> Result<DeviceRecording> {
        let frame = self.record_head_frame(batch)?;
        let fit = self.record_coif_drape(batch, design, &frame)?;
        let part = record_coif(self.gpu, batch, design, &fit, &frame.frame)?;
        Ok(DeviceRecording {
            part,
            frames: vec![(frame, FitRegion::Head)],
            checks: Vec::new(),
        })
    }

    /// Record the coif's drape into a new fit buffer: the coif's own frame,
    /// then its drape measured on the torso and neck skin. Unsupported sections fail the head frame.
    fn record_coif_drape(
        &self,
        batch: &mut KernelBatch,
        design: &CoifDesign,
        frame: &DeviceFrame,
    ) -> Result<Buffer> {
        let gpu = self.gpu;
        let host = self.host;
        let joint =
            |name: &RigJointName| -> std::result::Result<RigJointOrdinal, RigJointLookupError> {
                name.require_in(host.joint_names)
            };
        let mut support = host.support_indices(FitRegion::Torso)?;
        support.extend(host.support_indices(FitRegion::Neck)?);
        support.sort_unstable();
        support.dedup();
        let mut owned = vec![false; host.positions.len()];
        for vertex in support {
            owned[vertex] = true;
        }
        let support = host
            .faces
            .iter()
            .filter(|face| face.iter().any(|&vertex| owned[vertex as usize]))
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        let fit = gpu.scratch(
            ((FIT_PROFILE_WORD + COIF_DRAPE_WORDS) * 4) as u64,
            "coif fit",
        )?;
        let mut work = vec![0u32; QUERY_START + 2 * QUERIES];
        work[4] = ORDERED_ZERO;
        for reduction in [6, 8] {
            work[reduction] = ORDERED_NEGATIVE_INFINITY;
        }
        for query in 0..QUERIES {
            work[QUERY_START + 2 * query] = ORDERED_NEGATIVE_INFINITY;
        }
        // The flaps' attachment edges: the neck columns either side of the
        // centre lines they hang from.
        let flap_half_angle = FLAP_EDGE_COLUMNS / AROUND * TAU;
        let design_words = [
            design.neck_coverage.unit(),
            design.fit.clearance.metres() + design.fit.wall_thickness.metres() + DRAPE_EASE_M,
            design.front_flap_length.metres(),
            design.back_flap_length.metres(),
            design.flap_width.unit(),
            flap_half_angle.cos(),
            (PI - flap_half_angle).cos(),
            flap_half_angle.sin(),
            0.0,
        ];
        let mut parameters = PassParameters::new();
        parameters.insert("count", (support.len() / 3) as u32);
        parameters.insert("neck", usize::from(joint(&RigJointName::C_NECK)?) as u32);
        parameters.insert("jaw", usize::from(joint(&RigJointName::C_JAW_NULL)?) as u32);
        parameters.insert("head", usize::from(joint(&RigJointName::C_HEAD)?) as u32);
        parameters.insert("positions", self.body.positions.clone());
        parameters.insert(
            "support",
            gpu.upload(BufferUpload::from_elements(&support))?,
        );
        parameters.insert("joints", self.body.joints.clone());
        parameters.insert("frame", frame.frame.clone());
        parameters.insert(
            "design",
            gpu.upload(BufferUpload::from_elements(&design_words))?,
        );
        parameters.insert("work", gpu.upload(BufferUpload::from_elements(&work))?);
        parameters.insert("fit", fit.clone());
        parameters.insert("status", frame.status.clone());
        let [landmarks, neck, boundary, sections, drape] = kernels(gpu)?;
        let samples = (support.len() / 3) as u32;
        batch
            .dispatch(&landmarks, &parameters, [1, 1, 1])
            .map_err(device_error)?;
        batch
            .dispatch_items(&neck, &parameters, samples)
            .map_err(device_error)?;
        batch
            .dispatch(&boundary, &parameters, [1, 1, 1])
            .map_err(device_error)?;
        batch
            .dispatch_items(&sections, &parameters, samples)
            .map_err(device_error)?;
        batch
            .dispatch(&drape, &parameters, [1, 1, 1])
            .map_err(device_error)?;
        Ok(fit)
    }
}

fn kernels(gpu: &ArmorGpu) -> Result<[Arc<Kernel>; 5]> {
    let compile = |entry: &str| {
        gpu.cache()
            .get(gpu.context(), &source(entry))
            .map_err(device_error)
    };
    Ok([
        compile(LANDMARKS)?,
        compile(NECK)?,
        compile(BOUNDARY)?,
        compile(SECTIONS)?,
        compile(DRAPE)?,
    ])
}

fn source(entry: &str) -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> support: array<u32>;
@group(0) @binding(2) var<storage, read> joints: array<f32>;
@group(0) @binding(3) var<storage, read> frame: array<f32>;
@group(0) @binding(4) var<storage, read> design: array<f32>;
@group(0) @binding(5) var<storage, read_write> work: array<atomic<u32>>;
@group(0) @binding(6) var<storage, read_write> fit: array<f32>;
@group(0) @binding(7) var<storage, read_write> status: array<atomic<u32>>;
struct Params {{
    count: u32,
    neck: u32,
    jaw: u32,
    head: u32,
}};
@group(0) @binding(8) var<uniform> params: Params;
{ordered}
{zero_hook}{host_float}
{positions}

const NECK_COVERAGE: u32 = 0u;
const GAP: u32 = 1u;
const FRONT_FLAP_LENGTH: u32 = 2u;
const BACK_FLAP_LENGTH: u32 = 3u;
const FLAP_WIDTH: u32 = 4u;
const FRONT_EDGE_COSINE: u32 = 5u;
const BACK_EDGE_COSINE: u32 = 6u;
const EDGE_SINE: u32 = 7u;
const ZERO: u32 = 8u;

const NO_ENVELOPE: u32 = 2u;
const INVALID: u32 = 4u;

const TRANSVERSE_BAND_HALF_WIDTH_M: f32 = 0.030;
const SIDE_NECK_BASE_RISE: f32 = 0.38;
const BACK_NECK_BASE_RISE: f32 = 0.24;
const SECTIONS: u32 = {sections}u;

// Work words.
const FRONT_HEIGHT: u32 = 0u;
const SIDE_HEIGHT: u32 = 1u;
const BACK_HEIGHT: u32 = 2u;
const CENTER_DEPTH: u32 = 3u;
const NECK_WIDTH: u32 = 4u;
const FRONT_DEPTH: u32 = 6u;
const BACK_DEPTH: u32 = 8u;
const BOUNDARY: u32 = 10u;
const SECTION_HEIGHTS: u32 = 17u;
const FLAP_HALF_WIDTH: u32 = 27u;
const QUERY_START: u32 = {queries}u;

// The drape in the fit buffer: the neck boundary, then each flap's sections.
const DRAPE: u32 = {drape}u;

fn fail(bit: u32) {{
    atomicOr(&status[0], bit);
}}

fn load(at: u32) -> f32 {{
    return bitcast<f32>(atomicLoad(&work[at]));
}}

fn store(at: u32, value: f32) {{
    atomicStore(&work[at], bitcast<u32>(value));
}}

fn reduced(at: u32) -> f32 {{
    return float_from_ordered(atomicLoad(&work[at]));
}}

// A point in the head frame.
fn local(p: vec3<f32>) -> vec3<f32> {{
    let d = host_sub3(p, vec3<f32>(frame[0], frame[1], frame[2]));
    return vec3<f32>(
        host_dot(vec3<f32>(frame[3], frame[4], frame[5]), d),
        host_dot(vec3<f32>(frame[6], frame[7], frame[8]), d),
        host_dot(vec3<f32>(frame[9], frame[10], frame[11]), d),
    );
}}

fn joint(index: u32) -> vec3<f32> {{
    return vec3<f32>(joints[index * 8u], joints[index * 8u + 1u], joints[index * 8u + 2u]);
}}

fn count_at(at: u32) {{
    atomicAdd(&work[at + 1u], 1u);
}}

{section_source}
{entry}
"#,
        ordered = wgsl::ORDERED_FLOAT,
        zero_hook = host_float::zero_hook("bitcast<u32>(design[ZERO])"),
        host_float = host_float::wgsl(),
        positions = wgsl::read_points("positions"),
        section_source = include_str!("device_coif_sections.wgsl"),
        sections = COIF_DRAPE_SECTIONS,
        queries = QUERY_START,
        drape = FIT_PROFILE_WORD,
    )
}

/// One invocation: the neck boundary's heights from the rig.
const LANDMARKS: &str = r#"
@compute @workgroup_size(1)
fn main() {
    let base = local(joint(params.neck));
    let chin = local(joint(params.jaw));
    let span = host_sub(local(joint(params.head)).y, base.y);
    if (!(span > 0.0)) {
        fail(INVALID);
        return;
    }
    let front = host_add(chin.y, host_mul(host_sub(base.y, chin.y), design[NECK_COVERAGE]));
    store(FRONT_HEIGHT, front);
    store(SIDE_HEIGHT, host_add(front, host_mul(span, SIDE_NECK_BASE_RISE)));
    store(BACK_HEIGHT, host_add(front, host_mul(span, BACK_NECK_BASE_RISE)));
    store(CENTER_DEPTH, base.z);
}
"#;

/// The neck's width at the junction, and the chest and back depths below
/// the chin.
const NECK: &str = r#"
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let center = load(CENTER_DEPTH);
    let neck = section(i, load(SIDE_HEIGHT));
    if (neck.valid) {
        atomicMax(&work[NECK_WIDTH], ordered_from_float(max(abs(neck.a.x), abs(neck.b.x))));
        count_at(NECK_WIDTH);
    }
    reduce_depth(section(i, load(FRONT_HEIGHT)), 0.0, center, 1.0, FRONT_DEPTH);
    reduce_depth(section(i, load(BACK_HEIGHT)), 0.0, center, -1.0, BACK_DEPTH);
}
"#;

/// One invocation: the neck boundary, and the flaps' section heights and
/// half width it sets.
const BOUNDARY: &str = r#"
@compute @workgroup_size(1)
fn main() {
    for (var k = 0u; k < 3u; k = k + 1u) {
        if (atomicLoad(&work[NECK_WIDTH + 2u * k + 1u]) == 0u) {
            fail(NO_ENVELOPE);
            return;
        }
    }
    let gap = design[GAP];
    let front_height = load(FRONT_HEIGHT);
    let side_height = load(SIDE_HEIGHT);
    let back_height = load(BACK_HEIGHT);
    let half_width = host_add(reduced(NECK_WIDTH), gap);
    store(BOUNDARY, front_height);
    store(BOUNDARY + 1u, side_height);
    store(BOUNDARY + 2u, back_height);
    store(BOUNDARY + 3u, half_width);
    store(BOUNDARY + 4u, load(CENTER_DEPTH));
    store(BOUNDARY + 5u, host_add(reduced(FRONT_DEPTH), gap));
    store(BOUNDARY + 6u, host_sub(-reduced(BACK_DEPTH), gap));
    // Each flap runs from its hem up to the boundary at its outer column.
    for (var flap = 0u; flap < 2u; flap = flap + 1u) {
        var bottom = front_height;
        var length = design[FRONT_FLAP_LENGTH];
        var cosine = design[FRONT_EDGE_COSINE];
        if (flap == 1u) {
            bottom = back_height;
            length = design[BACK_FLAP_LENGTH];
            cosine = design[BACK_EDGE_COSINE];
        }
        let top = host_add(side_height, host_mul(host_sub(bottom, side_height), host_mul(cosine, cosine)));
        let hem = host_sub(bottom, length);
        let rise = host_add(host_sub(top, bottom), length);
        for (var j = 0u; j < SECTIONS; j = j + 1u) {
            let t = host_div(f32(j), f32(SECTIONS - 1u));
            store(SECTION_HEIGHTS + flap * SECTIONS + j, host_add(hem, host_mul(rise, t)));
        }
    }
    store(FLAP_HALF_WIDTH, host_mul(host_mul(half_width, design[EDGE_SINE]), design[FLAP_WIDTH]));
}
"#;

/// The body's depth at every flap section, on the centre line and at the
/// flap's edge.
const SECTIONS: &str = r#"
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let center = load(BOUNDARY + 4u);
    let half_width = load(FLAP_HALF_WIDTH);
    for (var flap = 0u; flap < 2u; flap = flap + 1u) {
        let facing = select(1.0, -1.0, flap == 1u);
        for (var j = 0u; j < SECTIONS; j = j + 1u) {
            let height = load(SECTION_HEIGHTS + flap * SECTIONS + j);
            let surface = section(i, height);
            for (var edge = 0u; edge < 2u; edge = edge + 1u) {
                let across = select(0.0, half_width, edge == 1u);
                let at = QUERY_START + 2u * ((flap * SECTIONS + j) * 2u + edge);
                reduce_depth(surface, across, center, facing, at);
            }
        }
    }
}
"#;

/// One invocation: the drape, validated, into the fit buffer after the
/// coif's own frame.
const DRAPE: &str = r#"
@compute @workgroup_size(1)
fn main() {
    let gap = design[GAP];
    var words: array<f32, 37>;
    for (var k = 0u; k < 7u; k = k + 1u) {
        words[k] = load(BOUNDARY + k);
    }
    for (var flap = 0u; flap < 2u; flap = flap + 1u) {
        let facing = select(1.0, -1.0, flap == 1u);
        for (var j = 0u; j < SECTIONS; j = j + 1u) {
            let at = 7u + (flap * SECTIONS + j) * 3u;
            words[at] = load(SECTION_HEIGHTS + flap * SECTIONS + j);
            for (var edge = 0u; edge < 2u; edge = edge + 1u) {
                let query = QUERY_START + 2u * ((flap * SECTIONS + j) * 2u + edge);
                if (atomicLoad(&work[query + 1u]) == 0u) {
                    fail(NO_ENVELOPE);
                    return;
                }
                let depth = host_mul(float_from_ordered(atomicLoad(&work[query])), facing);
                words[at + 1u + edge] = host_add(depth, host_mul(facing, gap));
            }
        }
    }
    // A drape is valid when finite, with a positive neck width and
    // the chest in front of, and the back behind, the neck's centre.
    var valid = words[3] > 0.0 && words[5] > words[4] && words[6] < words[4];
    for (var k = 0u; k < 37u; k = k + 1u) {
        valid = valid && abs(words[k]) <= 3.4e38;
    }
    for (var flap = 0u; flap < 2u; flap = flap + 1u) {
        for (var j = 0u; j + 1u < SECTIONS; j = j + 1u) {
            let at = 7u + (flap * SECTIONS + j) * 3u;
            valid = valid && words[at] < words[at + 3u];
        }
    }
    if (!valid) {
        fail(INVALID);
        return;
    }
    // The coif's own frame: identity axes at the origin, sized as the head.
    for (var w = 0u; w < DRAPE; w = w + 1u) {
        fit[w] = 0.0;
    }
    fit[3] = 1.0;
    fit[7] = 1.0;
    fit[11] = 1.0;
    fit[12] = frame[12];
    fit[13] = frame[13];
    fit[14] = frame[14];
    for (var k = 0u; k < 37u; k = k + 1u) {
        fit[DRAPE + k] = words[k];
    }
}
"#;
