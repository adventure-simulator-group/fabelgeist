//! Torso garments fitted on the device: seventeen sections of the torso's
//! skin make a cage, and every panel vertex is placed from it; the shoulder
//! and flank seams then follow the panels they join.

use adventuresim_armor_model::gpu::record_garment_torso;
use adventuresim_armor_model::{GarmentArmorDesign, GarmentArmorKind as Kind};
use anyhow::Result;
use fabelgeist_compute::KernelBatch;

use crate::armor_frames::FitRegion;
use crate::device_frames::DeviceWearer;
use crate::device_garment::{BAND, FRAME_WORDS, JOINTS, UprightSpan};
use crate::device_garment_kernel::{Grid, Word, dispatch, read, read_u32, write};
use crate::device_garment_tube::GARMENT_FIT_MARGIN_M;
use crate::device_piece::DeviceRecording;

/// The torso frame rises above the neck to hold the shoulder seams.
const SHOULDER_LIFT_M: f32 = 0.045;
/// Stations of the torso cage, evenly spaced over the frame's height.
const CAGE_SECTIONS: u32 = 17;
/// Floats per cage section: its height, then its low and high bounds.
const SECTION_WORDS: u32 = 7;
/// Where the torso fit keeps its landmarks and its cage.
const SCALARS: u32 = FRAME_WORDS;
const SECTIONS: u32 = SCALARS + 8;
const TORSO_FIT_WORDS: u32 = SECTIONS + CAGE_SECTIONS * SECTION_WORDS;

fn kind_code(kind: Kind) -> Result<f32> {
    Ok(match kind {
        Kind::ArmingDoublet => 0.0,
        Kind::Brigandine => 1.0,
        Kind::JackOfPlates => 2.0,
        Kind::MailShirt => 3.0,
        _ => anyhow::bail!("{kind:?} is not a torso garment"),
    })
}

fn layout() -> String {
    format!(
        "const SCALARS: u32 = {SCALARS}u;\nconst SECTIONS: u32 = {SECTIONS}u;\n\
         const SECTION_WORDS: u32 = {SECTION_WORDS}u;\nconst CAGE_SECTIONS: u32 = {CAGE_SECTIONS}u;\n"
    )
}

impl DeviceWearer<'_> {
    /// Record a torso garment's panels and seams, fitted, short of
    /// thickening.
    pub(crate) fn record_fitted_torso(
        &self,
        batch: &mut KernelBatch,
        design: &GarmentArmorDesign,
    ) -> Result<DeviceRecording> {
        let kind = kind_code(design.kind)?;
        let head = self.record_frame(batch, FitRegion::Head)?;
        let raw = self.record_frame(batch, FitRegion::Torso)?;
        let bottom = if matches!(design.kind, Kind::ArmingDoublet | Kind::MailShirt) {
            "c_spine1"
        } else {
            "c_spine0"
        };
        let fit = self.record_upright(
            batch,
            TORSO_FIT_WORDS,
            &raw,
            &head,
            UprightSpan {
                top: ("c_neck", SHOULDER_LIFT_M),
                bottom: (bottom, 0.0),
                reach: (bottom, 0.0),
            },
        )?;
        let gap = design.clearance.metres() + design.wall_thickness.metres() + GARMENT_FIT_MARGIN_M;
        dispatch(
            self,
            batch,
            &format!("{}{JOINTS}{TORSO_SETUP}", layout()),
            &[read("joints", &self.body.joints), write("fit", &fit)],
            &[
                Word::U("neck", self.joint_slot("c_neck")?),
                Word::U("waist", self.joint_slot("c_spine1")?),
                Word::U("armpit", self.joint_slot("l_uparm")?),
                Word::F("length", design.length.unit()),
                Word::F("gap", gap),
                Word::F("waist_scale", design.waist.unit()),
                Word::F("flare", design.flare.unit()),
                Word::F("kind", kind),
            ],
            Grid::Singles(1),
        )?;
        let support = self.record_region_support(batch, &[FitRegion::Torso, FitRegion::Hips])?;
        dispatch(
            self,
            batch,
            &format!("{}{BAND}{TORSO_SECTIONS}", layout()),
            &[
                read("positions", &self.body.positions),
                read_u32("support", &support),
                write("fit", &fit),
            ],
            &[],
            Grid::Singles(CAGE_SECTIONS),
        )?;
        let part = record_garment_torso(
            self.gpu,
            batch,
            design,
            &fit,
            &format!("{}{HOST_CAGE}{TORSO_CAGE}", layout()),
        )?;
        self.record_fit_status(batch, &fit, &part)?;
        Ok(DeviceRecording {
            part,
            frames: vec![(head, FitRegion::Head), (raw, FitRegion::Torso)],
            checks: Vec::new(),
        })
    }
}

/// The torso's landmarks, relative to its frame.
const TORSO_SETUP: &str = r#"
@compute @workgroup_size(1)
fn main() {
    let f = frame_at(0u);
    let neck = joint(params.neck);
    let waist = joint(params.waist).y;
    fit[SCALARS] = neck.y - f.origin.y;
    fit[SCALARS + 1u] = -f.half_extents.y * params.length;
    fit[SCALARS + 2u] = joint(params.armpit).y - (neck.y - waist) * 0.30 - f.origin.y;
    fit[SCALARS + 3u] = params.gap;
    fit[SCALARS + 4u] = waist - f.origin.y;
    fit[SCALARS + 5u] = params.waist_scale;
    fit[SCALARS + 6u] = params.flare;
    fit[SCALARS + 7u] = params.kind;
}
"#;

/// One invocation per cage section: its height and the bounds of the torso
/// and hip skin near it.
const TORSO_SECTIONS: &str = r#"
const SUPPORT_MASK: u32 = 3u;

@compute @workgroup_size(1)
fn main(@builtin(workgroup_id) group: vec3<u32>) {
    let row = group.x;
    let f = frame_at(0u);
    let half_height = f.half_extents.y;
    let y = -half_height + 2.0 * half_height * f32(row) / f32(CAGE_SECTIONS - 1u);
    let center = host_point(f, vec3<f32>(0.0, y, 0.0));
    let band = band_bounds(
        SUPPORT_MASK,
        center,
        f.x,
        f.y,
        f.z,
        SECTION_HALF_WIDTH_M,
        MINIMUM_SECTION_SAMPLES,
    );
    let at = SECTIONS + row * SECTION_WORDS;
    fit[at] = y;
    for (var a = 0u; a < 3u; a = a + 1u) {
        fit[at + 1u + a] = band.lo[a];
        fit[at + 4u + a] = band.hi[a];
    }
}
"#;

/// Fixed-order vector arithmetic for the cage, which the shape kernel lacks.
const HOST_CAGE: &str = r#"
fn host_dot(a: vec3<f32>, b: vec3<f32>) -> f32 {
    return (a.x * b.x + a.y * b.y) + a.z * b.z;
}

fn mix3(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32> {
    return a * (1.0 - t) + b * t;
}

fn host_local(f: Frame, p: vec3<f32>) -> vec3<f32> {
    let d = p - f.origin;
    return vec3<f32>(host_dot(f.x, d), host_dot(f.y, d), host_dot(f.z, d));
}
"#;

/// Every panel vertex placed from the torso cage, and the shoulder and flank
/// seams bridging the fitted panels.
const TORSO_CAGE: &str = r#"
const ARMPIT_SEAM_DROP_M: f32 = 0.023;
const SHOULDER_SEAM_EASE_M: f32 = 0.005;
const SHOULDER_SEAM_LIFT_M: f32 = 0.045;
const ARMING_DOUBLET: f32 = 0.0;
const BRIGANDINE: f32 = 1.0;
const JACK_OF_PLATES: f32 = 2.0;
const MAIL_SHIRT: f32 = 3.0;

struct Bounds {
    lo: vec3<f32>,
    hi: vec3<f32>,
};

fn section_y(k: u32) -> f32 {
    return frames[SECTIONS + k * SECTION_WORDS];
}

fn section_bound(k: u32, high: u32) -> vec3<f32> {
    let at = SECTIONS + k * SECTION_WORDS + 1u + high * 3u;
    return vec3<f32>(frames[at], frames[at + 1u], frames[at + 2u]);
}

// The cage's bounds at a height, interpolated between the sections around it.
fn cage_at(y: f32) -> Bounds {
    var upper = 0u;
    loop {
        if (upper >= CAGE_SECTIONS || !(section_y(upper) < y)) {
            break;
        }
        upper = upper + 1u;
    }
    upper = min(upper, CAGE_SECTIONS - 1u);
    let lower = max(upper, 1u) - 1u;
    let a = section_y(lower);
    let b = section_y(upper);
    var t = 0.0;
    if (b > a) {
        t = clamp((y - a) / (b - a), 0.0, 1.0);
    }
    return Bounds(
        mix3(section_bound(lower, 0u), section_bound(upper, 0u), t),
        mix3(section_bound(lower, 1u), section_bound(upper, 1u), t),
    );
}

fn torso_panel(front: bool, row: u32, col: u32) -> vec3<f32> {
    let neck_y = frames[SCALARS];
    let bottom_y = frames[SCALARS + 1u];
    let armpit_y = frames[SCALARS + 2u];
    let gap = frames[SCALARS + 3u];
    let waist_y = frames[SCALARS + 4u];
    let waist = frames[SCALARS + 5u];
    let flare = frames[SCALARS + 6u];
    let kind = frames[SCALARS + 7u];
    let u = 2.0 * f32(col) / f32(PANEL_ACROSS) - 1.0;
    let v = f32(row) / f32(PANEL_ALONG);
    let neckline = max(1.0 - pow2(min(abs(u) / 0.5, 1.0)), 0.0);
    let shoulder_top = neck_y + 0.018 + SHOULDER_SEAM_EASE_M - 0.035 * pow2(abs(u));
    let top = shoulder_top - select(0.025, 0.090, front) * neckline;
    var y: f32;
    if (row <= ARMPIT_ROW) {
        y = bottom_y + (armpit_y - bottom_y) * f32(row) / f32(ARMPIT_ROW);
    } else {
        y = armpit_y + (top - armpit_y) * f32(row - ARMPIT_ROW) / f32(PANEL_ALONG - ARMPIT_ROW);
    }
    // Upper width comes from the shoulder girdle, independently of the
    // narrowing neck section and neckline drop.
    let girdle = cage_at(min(y, neck_y - 0.035));
    let width = (girdle.hi.x - girdle.lo.x) * 0.5;
    let chest = cage_at(0.0);
    let chest_width = (chest.hi.x - chest.lo.x) * 0.5;
    let seam_transition = clamp(v / 0.65, 0.0, 1.0);
    let seam_blend = seam_transition * seam_transition * (3.0 - 2.0 * seam_transition);
    var lower_ease = pow2(1.0 - v);
    if (kind == MAIL_SHIRT) {
        lower_ease = 1.0 - seam_blend;
    }
    let waist_ease = max(chest_width * waist - width, 0.0) * lower_ease;
    let center_x = (girdle.hi.x + girdle.lo.x) * 0.5;
    let shoulder_inset = pow2(max((v - 0.6) / 0.4, 0.0));
    var hip_flare = 0.0;
    if (kind == BRIGANDINE || kind == JACK_OF_PLATES) {
        hip_flare = flare * clamp((waist_y - y) / (waist_y - bottom_y), 0.0, 1.0);
    }
    let x = center_x
        + u * (width * (1.0 + hip_flare) + gap + waist_ease) * (1.0 - 0.13 * shoulder_inset);
    let level = cage_at(y);
    var center_z = (level.hi.z + level.lo.z) * 0.5;
    var radius = (level.hi.z - level.lo.z) * 0.5;
    if (kind == MAIL_SHIRT) {
        let chest_radius = (chest.hi.z - chest.lo.z) * 0.5;
        radius = radius + max(chest_radius * waist - radius, 0.0) * lower_ease;
        let hem = cage_at(bottom_y);
        let hem_center = (hem.lo.z + hem.hi.z) * 0.5;
        center_z = hem_center * (1.0 - seam_blend) + center_z * seam_blend;
    }
    radius = radius * (1.0 + hip_flare * 0.5);
    let contour = pow(1.0 - 0.78 * u * u, 0.25);
    var quilting = 0.0;
    if (kind == ARMING_DOUBLET) {
        quilting = 0.0015 * pow2(cos(u * PI * 6.0));
    }
    let z = center_z + select(-1.0, 1.0, front) * (radius * contour + gap + quilting);
    return vec3<f32>(x, y, z);
}

// A fitted panel vertex back in the frame.
fn panel_local(front: bool, row: u32, col: u32) -> vec3<f32> {
    let stride = PANEL_ACROSS + 1u;
    let panel_count = (PANEL_ALONG + 1u) * stride;
    return host_local(fit, carriers_at(params.first + select(panel_count, 0u, front) + row * stride + col));
}

fn torso_vertex(index: u32, coord: vec4<f32>) -> vec3<f32> {
    let kind = u32(coord.x);
    let a = u32(coord.y);
    let b = u32(coord.z);
    let c = u32(coord.w);
    if (kind == 0u) {
        return torso_panel(a == 1u, b, c);
    }
    let t = f32(select(c, b, kind == 1u)) / f32(SHOULDER_DEPTH);
    if (kind == 1u) {
        // A shoulder is a curved saddle, not a planar bridge.
        let col = c + select(3u * PANEL_ACROSS / 4u, 0u, a == 0u);
        var local = mix3(panel_local(true, PANEL_ALONG, col), panel_local(false, PANEL_ALONG, col), t);
        local.y = local.y + SHOULDER_SEAM_LIFT_M * sin(PI * t);
        return local;
    }
    // The flank's upper edge dips beneath the armpit.
    let col = select(PANEL_ACROSS, 0u, a == 0u);
    var local = mix3(panel_local(true, b, col), panel_local(false, b, col), t);
    local.y = local.y - ARMPIT_SEAM_DROP_M * pow4(f32(b) / f32(ARMPIT_ROW)) * sin(PI * t);
    return local;
}
"#;
