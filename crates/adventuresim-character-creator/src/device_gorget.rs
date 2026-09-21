//! Gorgets fitted on the device: the neck is cut by the collar's oblique
//! planes, broad sections measure the shoulders and the bib's depth, a
//! directional cage seats the bib on the shoulders, and the plates are built
//! from the collar cage those measurements make.

use adventuresim_armor_model::gpu::{record_gorget_plates, wgsl};
use adventuresim_armor_model::{GarmentArmorDesign, GarmentArmorKind, GarmentPlateShape};
use anyhow::{Result, bail, ensure};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

use crate::armor_frames::FitRegion;
use crate::device_frames::DeviceWearer;
use crate::device_garment::JOINTS;
use crate::device_garment_kernel::{Grid, Word, atomic, dispatch, read, read_u32, write};
use crate::device_gorget_bib::{MEASURE, SMOOTH};
use crate::device_gorget_cage::{BIB_COLUMNS, BIB_ROWS, CAGE, GORGET_FIT_WORDS, layout};
use crate::device_piece::DeviceRecording;

/// Collar and bib proportions, as fractions of the neck's height.
const COLLAR_HEIGHT_NECK_RATIO: f32 = 0.20;
const COLLAR_BASE_NECK_RATIO: f32 = 0.58;
const COLLAR_MAX_PITCH: f32 = 0.45;
const FRONT_BIB_DROP_NECK_RATIO: f32 = 0.48;
const REAR_BIB_DROP_NECK_RATIO: f32 = 0.30;
const SIDE_BIB_RISE_NECK_RATIO: f32 = 0.45;
const SHOULDER_WIDTH_SECTION_NECK_RATIO: f32 = 0.30;
const SAGITTAL_SECTION_HALF_WIDTH_NECK_RATIO: f32 = 0.18;
const POSTERIOR_SHOULDER_CROWN_NECK_RATIO: f32 = 0.20;
const SIDE_COLLAR_RISE_NECK_RATIO: f32 = 0.15;
const BIB_FLARE_WIDTH_GAIN: f32 = 0.25;
/// The shoulder section, then three down the front and three down the back.
const BAND_SECTIONS: u32 = 7;

impl DeviceWearer<'_> {
    /// Record a gorget's collar cage and plates, short of thickening.
    pub(crate) fn record_fitted_gorget(
        &self,
        batch: &mut KernelBatch,
        design: &GarmentArmorDesign,
    ) -> Result<DeviceRecording> {
        design.validate()?;
        ensure!(
            design.kind == GarmentArmorKind::Gorget,
            "gorget fitter requires a gorget design"
        );
        let GarmentPlateShape::Gorget {
            neck_clearance,
            collar_slope,
            collar_height,
            hem_flatness,
            rear_hem_flatness,
            rear_sweep,
            front_depth,
            back_depth,
            front_width,
            back_width,
        } = design.plate_shape
        else {
            bail!("gorget requires collar shape controls");
        };
        let gpu = self.gpu;
        let head = self.record_frame(batch, FitRegion::Head)?;
        let fit = gpu.scratch(u64::from(GORGET_FIT_WORDS) * 4, "gorget fit")?;
        let clearance = design.clearance.metres() + design.wall_thickness.metres();
        let collar_padding = neck_clearance.metres() + design.wall_thickness.metres();
        dispatch(
            self,
            batch,
            &format!("{}{JOINTS}{SETUP}", layout()),
            &[
                read("joints", &self.body.joints),
                read("head", &head.frame),
                write("fit", &fit),
            ],
            &[
                Word::U("neck", self.joint_slot("c_neck")?),
                Word::U("crown", self.joint_slot("c_head")?),
                Word::F(
                    "top_ratio",
                    COLLAR_BASE_NECK_RATIO + COLLAR_HEIGHT_NECK_RATIO * collar_height.unit(),
                ),
                Word::F("base_ratio", COLLAR_BASE_NECK_RATIO),
                Word::F("pitch", COLLAR_MAX_PITCH * collar_slope.unit()),
                Word::F(
                    "front_drop",
                    -FRONT_BIB_DROP_NECK_RATIO * design.length.unit() * front_depth.unit(),
                ),
                Word::F("side_rise", SIDE_BIB_RISE_NECK_RATIO),
                Word::F("rear_drop", -REAR_BIB_DROP_NECK_RATIO * back_depth.unit()),
                Word::F("crown_ratio", POSTERIOR_SHOULDER_CROWN_NECK_RATIO),
                Word::F("collar_rise", SIDE_COLLAR_RISE_NECK_RATIO),
                Word::F("hem_flatness", hem_flatness.unit()),
                Word::F("rear_hem_flatness", rear_hem_flatness.unit()),
                Word::F("rear_sweep", rear_sweep.unit()),
            ],
            Grid::Singles(1),
        )?;
        let words = [
            Word::F("clearance", clearance),
            Word::F("collar_padding", collar_padding),
            Word::F("flare_gain", design.flare.unit() * BIB_FLARE_WIDTH_GAIN),
            Word::F("front_width", front_width.unit()),
            Word::F("back_width", back_width.unit()),
            Word::F("shoulder_ratio", SHOULDER_WIDTH_SECTION_NECK_RATIO),
            Word::F("sagittal_ratio", SAGITTAL_SECTION_HALF_WIDTH_NECK_RATIO),
        ];
        let samples = self.record_gorget_sections(batch, &fit, &words)?;
        self.record_bib_fit(batch, &fit, &samples, clearance)?;
        let cage = format!(
            "{}fn cage_word(i: u32) -> f32 {{\n    return frames[i];\n}}\n{CAGE}",
            layout()
        );
        let part = record_gorget_plates(gpu, batch, design, &fit, &cage)?;
        self.record_fit_status(batch, &fit, &part)?;
        Ok(DeviceRecording {
            part,
            frames: vec![(head, FitRegion::Head)],
            checks: Vec::new(),
        })
    }

    /// Record every body vertex in the gorget's frame, and the neck, band and
    /// cage sections the gorget is fitted to; returns the vertices.
    fn record_gorget_sections(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        words: &[Word],
    ) -> Result<Buffer> {
        let gpu = self.gpu;
        // Every body vertex in the gorget's frame.
        let samples = gpu.scratch(u64::from(self.body.vertex_count) * 12, "gorget samples")?;
        dispatch(
            self,
            batch,
            SAMPLES,
            &[
                read("positions", &self.body.positions),
                read("fit", fit),
                write("points", &samples),
            ],
            &[Word::U("count", self.body.vertex_count)],
            Grid::Items(self.body.vertex_count),
        )?;
        let mut planes = Vec::new();
        for _ in 0..2 {
            planes.extend([ORDERED_POSITIVE_INFINITY; 3]);
            planes.extend([ORDERED_NEGATIVE_INFINITY; 3]);
            planes.push(0);
        }
        let planes = gpu.upload(&planes)?;
        dispatch(
            self,
            batch,
            &format!("{}{}{PLANE_SECTIONS}", layout(), wgsl::ORDERED_FLOAT),
            &[
                read_u32("faces", &self.body.faces),
                read("points", &samples),
                read("fit", fit),
                atomic("planes", &planes),
            ],
            &[Word::U("count", self.body.face_count)],
            Grid::Items(self.body.face_count),
        )?;
        for (entry, grid) in [
            (BANDS, Grid::Singles(BAND_SECTIONS)),
            (CAGE_SETUP, Grid::Singles(1)),
        ] {
            dispatch(
                self,
                batch,
                &format!("{}{}{SECTIONS}{entry}", layout(), wgsl::ORDERED_FLOAT),
                &[
                    read("points", &samples),
                    read_u32("planes", &planes),
                    write("fit", fit),
                ],
                words,
                grid,
            )?;
        }
        Ok(samples)
    }

    /// Record the bib's seating: rays from the unseated bib to the body in
    /// each directional section, then the smoothing of their offsets.
    fn record_bib_fit(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        samples: &Buffer,
        padding: f32,
    ) -> Result<()> {
        let cage = format!("fn cage_word(i: u32) -> f32 {{\n    return fit[i];\n}}\n{CAGE}");
        dispatch(
            self,
            batch,
            &format!("{}{cage}{MEASURE}", layout()),
            &[
                read_u32("faces", &self.body.faces),
                read("points", samples),
                write("fit", fit),
            ],
            &[
                Word::U("faces_count", self.body.face_count),
                Word::F("padding", padding),
            ],
            Grid::Items((BIB_ROWS - 1) * BIB_COLUMNS),
        )?;
        dispatch(
            self,
            batch,
            &format!("{}{SMOOTH}", layout()),
            &[write("fit", fit)],
            &[],
            Grid::Items(BIB_ROWS * BIB_COLUMNS),
        )
    }
}

const ORDERED_POSITIVE_INFINITY: u32 = 0xff80_0000;
const ORDERED_NEGATIVE_INFINITY: u32 = 0x007f_ffff;

/// The gorget frame -- the neck, the head's heading, the neck's height --
/// and the cage's design-only proportions.
const SETUP: &str = r#"
@compute @workgroup_size(1)
fn main() {
    let neck = joint(params.neck);
    let height = joint(params.crown).y - neck.y;
    let words = array<f32, 15>(
        neck.x, neck.y, neck.z,
        head[3], head[4], head[5],
        head[6], head[7], head[8],
        head[9], head[10], head[11],
        height, height, height,
    );
    for (var w = 0u; w < 15u; w = w + 1u) {
        fit[w] = words[w];
    }
    fit[FIT_FAILED] = select(1.0, 0.0, abs(height) <= INFINITY && height > 0.0);
    fit[PLANES] = params.top_ratio * height;
    fit[PLANES + 1u] = params.pitch;
    fit[PLANES + 2u] = params.base_ratio * height;
    fit[PLANES + 3u] = params.pitch;
    fit[HEM] = params.front_drop * height;
    fit[HEM + 1u] = params.side_rise * height;
    fit[HEM + 2u] = params.rear_drop * height;
    fit[CROWN] = height * params.crown_ratio;
    fit[SIDE_RISE] = height * params.collar_rise;
    fit[HEM_FLATNESS] = params.hem_flatness;
    fit[REAR_HEM_FLATNESS] = params.rear_hem_flatness;
    fit[REAR_SWEEP] = params.rear_sweep;
    fit[HEIGHT] = height;
}
"#;

/// Every body vertex in the gorget frame.
const SAMPLES: &str = r#"
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    points_set(i, host_local(frame_at(0u), positions_at(i)));
}
"#;

/// One invocation per body face: the section of both collar planes, bounding
/// where the face's edges cross them and counting the crossings.
const PLANE_SECTIONS: &str = r#"
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let f = id.x;
    if (f >= params.count) {
        return;
    }
    let corners = array<vec3<f32>, 3>(
        points_at(faces[f * 3u]),
        points_at(faces[f * 3u + 1u]),
        points_at(faces[f * 3u + 2u]),
    );
    for (var plane = 0u; plane < 2u; plane = plane + 1u) {
        let height = fit[PLANES + plane * 2u];
        let pitch = fit[PLANES + plane * 2u + 1u];
        let base = plane * 7u;
        for (var e = 0u; e < 3u; e = e + 1u) {
            let a = corners[e];
            let b = corners[(e + 1u) % 3u];
            let da = a.y - (height - pitch * a.z);
            let db = b.y - (height - pitch * b.z);
            if ((da > 0.0) == (db > 0.0)) {
                continue;
            }
            let t = da / (da - db);
            let crossing = a + (b - a) * t;
            for (var axis = 0u; axis < 3u; axis = axis + 1u) {
                atomicMin(&planes[base + axis], ordered_from_float(crossing[axis]));
                atomicMax(&planes[base + 3u + axis], ordered_from_float(crossing[axis]));
            }
            atomicAdd(&planes[base + 6u], 1u);
        }
    }
}
"#;

/// Section readers shared by the band and cage kernels.
const SECTIONS: &str = r#"
const MINIMUM_SECTION_SAMPLES: u32 = 4u;
const SECTION_HALF_BAND_M: f32 = 0.006;
const MAXIMUM_SECTION_HALF_BAND_M: f32 = 0.018;

struct Section {
    low: vec3<f32>,
    high: vec3<f32>,
    valid: bool,
};

fn plane_section(plane: u32) -> Section {
    let base = plane * 7u;
    var s = Section(vec3<f32>(0.0), vec3<f32>(0.0), planes[base + 6u] >= MINIMUM_SECTION_SAMPLES);
    for (var axis = 0u; axis < 3u; axis = axis + 1u) {
        s.low[axis] = float_from_ordered(planes[base + axis]);
        s.high[axis] = float_from_ordered(planes[base + 3u + axis]);
    }
    return s;
}

fn plane_at(plane: u32, z: f32) -> f32 {
    return fit[PLANES + plane * 2u] - fit[PLANES + plane * 2u + 1u] * z;
}
"#;

/// One invocation per broad section: the bounds of the samples in a thin band
/// about a height, or of the four nearest in a wider one.
const BANDS: &str = r#"
fn in_reach(p: vec3<f32>, y: f32, half_width: f32) -> bool {
    return abs(p.x) <= half_width && abs(p.y - y) <= MAXIMUM_SECTION_HALF_BAND_M;
}

@compute @workgroup_size(1)
fn main(@builtin(workgroup_id) group: vec3<u32>) {
    let band = group.x;
    let height = fit[HEIGHT];
    let lower = plane_section(1u);
    var y = height * params.shoulder_ratio;
    var half_width = INFINITY;
    if (band > 0u) {
        let t = f32((band - 1u) % 3u) * 0.5;
        half_width = height * params.sagittal_ratio;
        if (band < 4u) {
            let start = plane_at(1u, lower.high.z);
            y = start + (fit[HEM] - start) * t;
        } else {
            let start = plane_at(1u, lower.low.z);
            y = start + (fit[HEM + 2u] - start) * t;
        }
    }
    let count = arrayLength(&points) / 3u;
    var low = vec3<f32>(INFINITY);
    var high = vec3<f32>(-INFINITY);
    var nearby = 0u;
    var close = 0u;
    for (var v = 0u; v < count; v = v + 1u) {
        let p = points_at(v);
        if (!in_reach(p, y, half_width)) {
            continue;
        }
        nearby = nearby + 1u;
        if (abs(p.y - y) <= SECTION_HALF_BAND_M) {
            close = close + 1u;
            low = min(low, p);
            high = max(high, p);
        }
    }
    if (nearby < MINIMUM_SECTION_SAMPLES) {
        fit[FIT_FAILED] = 1.0;
    } else if (close < MINIMUM_SECTION_SAMPLES) {
        // The nearest by height, ties to the lowest vertex.
        low = vec3<f32>(INFINITY);
        high = vec3<f32>(-INFINITY);
        var last_distance = -1.0;
        var last = 0u;
        for (var k = 0u; k < MINIMUM_SECTION_SAMPLES; k = k + 1u) {
            var best_distance = INFINITY;
            var best = 0u;
            for (var v = 0u; v < count; v = v + 1u) {
                let p = points_at(v);
                if (!in_reach(p, y, half_width)) {
                    continue;
                }
                let d = abs(p.y - y);
                let after = d > last_distance || (d == last_distance && v > last);
                if (after && d < best_distance) {
                    best_distance = d;
                    best = v;
                }
            }
            low = min(low, points_at(best));
            high = max(high, points_at(best));
            last_distance = best_distance;
            last = best;
        }
    }
    let at = BANDS + band * 6u;
    for (var axis = 0u; axis < 3u; axis = axis + 1u) {
        fit[at + axis] = low[axis];
        fit[at + 3u + axis] = high[axis];
    }
}
"#;

/// The collar cage from the plane and band sections: the collar's centre and
/// radii, the bib's outer widths, and its front and back depths.
const CAGE_SETUP: &str = r#"
fn band_low(band: u32) -> vec3<f32> {
    let at = BANDS + band * 6u;
    return vec3<f32>(fit[at], fit[at + 1u], fit[at + 2u]);
}

fn band_high(band: u32) -> vec3<f32> {
    let at = BANDS + band * 6u + 3u;
    return vec3<f32>(fit[at], fit[at + 1u], fit[at + 2u]);
}

@compute @workgroup_size(1)
fn main() {
    let neck = plane_section(0u);
    let lower = plane_section(1u);
    if (!neck.valid || !lower.valid) {
        fit[FIT_FAILED] = 1.0;
    }
    let center = vec2<f32>((neck.low.x + neck.high.x) * 0.5, (neck.low.z + neck.high.z) * 0.5);
    fit[CENTER] = center.x;
    fit[CENTER + 1u] = center.y;
    fit[COLLAR_RADIUS] = (neck.high.x - neck.low.x) * 0.5 + params.collar_padding;
    fit[COLLAR_RADIUS + 1u] = (neck.high.z - neck.low.z) * 0.5 + params.collar_padding;
    fit[BASE_RADIUS] = (lower.high.x - lower.low.x) * 0.5 + params.collar_padding;
    let outer = ((band_high(0u).x - band_low(0u).x) * 0.5 + params.clearance)
        * (1.0 + params.flare_gain);
    fit[OUTER_WIDTH] = params.front_width * outer;
    fit[OUTER_WIDTH + 1u] = params.back_width * outer;
    for (var i = 1u; i < 3u; i = i + 1u) {
        fit[FRONT + i] = band_high(1u + i).z - center.y + params.clearance;
        fit[BACK + i] = center.y - band_low(4u + i).z + params.clearance;
    }
    fit[FRONT] = lower.high.z - center.y + params.collar_padding;
    fit[BACK] = center.y - lower.low.z + params.collar_padding;
}
"#;
