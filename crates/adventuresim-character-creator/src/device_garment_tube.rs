//! Sleeves and chausses fitted on the device: a tube authored in the limb's
//! frame, each carrier moved onto the section of the limb enclosing its
//! station along a curve through the limb's joints, and a sleeve's top sewn to
//! the torso's armhole.

use anyhow::Result;
use fabelgeist_armor::{
    GARMENT_ARMPIT_ROW as ARMPIT, GARMENT_PANEL_ACROSS as ACROSS, GARMENT_PANEL_ALONG as ALONG,
    GARMENT_SHOULDER_DEPTH_SEGMENTS as SHOULDER, GarmentArmorDesign, GarmentArmorKind as Kind,
    gpu::record_garment_tube,
};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;
use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_rig::{RigJointLookupError, RigJointOrdinal, RigJointPart};

use crate::armor_frames::{FitRegion, Side};
use crate::device_frames::DeviceWearer;
use crate::device_garment::{BAND, FRAME_WORDS, JOINTS};
use crate::device_garment_kernel::{Grid, Word, dispatch, read, read_u32, write};
use crate::device_piece::DeviceRecording;

/// Clearance a garment keeps from the skin beyond its gauge.
pub(crate) const GARMENT_FIT_MARGIN_M: f32 = 0.006;
/// Where a tube's fit keeps its joint anchors and its sewing ring.
const ANCHORS: u32 = FRAME_WORDS;
pub(crate) const RING: u32 = ANCHORS + 9;
/// Floats per ring point: its angle, then the point.
const RING_POINT_WORDS: u32 = 4;
/// Sewing rings never have more points than this.
const RING_CAPACITY: u32 = 96;
pub(crate) const RING_FIT_WORDS: u32 = RING + 1 + RING_CAPACITY * RING_POINT_WORDS;

/// Torso carriers along one side's armhole: up the front panel's edge from the
/// armpit, across the shoulder band, down the back panel's edge, and back
/// across the top of the flank.
pub(crate) fn armhole(side: Side) -> Vec<u32> {
    let stride = ACROSS + 1;
    let panel = (ALONG + 1) * stride;
    let left = matches!(side, Side::Left);
    let col = if left { ACROSS } else { 0 };
    let mut points = (ARMPIT..=ALONG)
        .map(|row| row * stride + col)
        .collect::<Vec<_>>();
    let band_width = ACROSS / 4 + 1;
    let band = usize::from(left);
    let band_col = if left { band_width - 1 } else { 0 };
    for depth in 0..SHOULDER - 1 {
        points.push(2 * panel + band * (SHOULDER - 1) * band_width + depth * band_width + band_col);
    }
    points.extend((ARMPIT..=ALONG).rev().map(|row| panel + row * stride + col));
    let flanks_start = 2 * panel + 2 * (SHOULDER - 1) * band_width;
    let flank_start = flanks_start + usize::from(left) * (ARMPIT + 1) * (SHOULDER - 1);
    points.extend(
        (0..SHOULDER - 1)
            .rev()
            .map(|depth| flank_start + ARMPIT * (SHOULDER - 1) + depth),
    );
    points.into_iter().map(|i| i as u32).collect()
}

/// Torso carriers along the hem: across the front panel's bottom row, back
/// across the back panel's, then along each flank's bottom.
pub(crate) fn hem() -> Vec<u32> {
    let panel = (ALONG + 1) * (ACROSS + 1);
    let mut points = (0..=ACROSS).collect::<Vec<_>>();
    points.extend((0..=ACROSS).rev().map(|col| panel + col));
    let flanks_start = 2 * panel + 2 * (SHOULDER - 1) * (ACROSS / 4 + 1);
    for flank in 0..2 {
        let start = flanks_start + flank * (ARMPIT + 1) * (SHOULDER - 1);
        points.extend((0..SHOULDER - 1).map(|depth| start + depth));
    }
    points.into_iter().map(|i| i as u32).collect()
}

/// The anatomical span of a sleeve or chausse, independent of its device flags.
#[derive(Clone, Copy)]
enum TubeRig {
    Arm,
    Leg,
}
impl From<&GarmentArmorDesign> for TubeRig {
    fn from(design: &GarmentArmorDesign) -> Self {
        if matches!(design.kind, Kind::MailSleeve | Kind::QuiltedSleeve) {
            Self::Arm
        } else {
            Self::Leg
        }
    }
}
impl TubeRig {
    fn region(self, side: Side) -> FitRegion {
        match self {
            Self::Arm => FitRegion::WholeArm(side),
            Self::Leg => FitRegion::WholeLeg(side),
        }
    }
    fn anchors(
        self,
        body: &crate::armor_frames::Wearer<'_>,
        side: Side,
    ) -> std::result::Result<[RigJointOrdinal; 3], RigJointLookupError> {
        let parts = match self {
            Self::Arm => [
                RigJointPart::Uparm,
                RigJointPart::Lowarm,
                RigJointPart::Wrist,
            ],
            Self::Leg => [
                RigJointPart::Upleg,
                RigJointPart::Lowleg,
                RigJointPart::Foot,
            ],
        };
        let [first, second, third] = parts;
        Ok([
            side.joint(first).require_in(body.joint_names)?,
            side.joint(second).require_in(body.joint_names)?,
            side.joint(third).require_in(body.joint_names)?,
        ])
    }
}

impl DeviceWearer<'_> {
    /// Record the sewing ring through `torso` carriers `indices` into `fit`,
    /// in the angular chart of the fit's frame.
    pub(crate) fn record_ring(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        torso: &Buffer,
        indices: &[u32],
    ) -> Result<()> {
        anyhow::ensure!(
            indices.len() as u32 <= RING_CAPACITY,
            "sewing ring too long"
        );
        let indices_buffer = self.gpu.upload(BufferUpload::from_elements(indices))?;
        dispatch(
            self,
            batch,
            &format!("const RING: u32 = {RING}u;\n{RING_SOURCE}"),
            &[
                read("points", torso),
                read_u32("ring_indices", &indices_buffer),
                write("fit", fit),
            ],
            &[Word::U("count", indices.len() as u32)],
            Grid::Singles(1),
        )
    }

    /// Record a sleeve or chausse: authored, fitted, and sewn.
    pub(crate) fn record_fitted_tube(
        &self,
        batch: &mut KernelBatch,
        design: &GarmentArmorDesign,
        placement: &str,
    ) -> Result<DeviceRecording> {
        let side = Side::from_placement(placement)?;

        let rig = TubeRig::from(design);
        let anchors = rig.anchors(self.host, side)?;
        let region = rig.region(side);
        let frame = self.record_frame(batch, region)?;
        let fit = self
            .gpu
            .scratch(u64::from(RING_FIT_WORDS) * 4, "tube fit")?;
        dispatch(
            self,
            batch,
            &format!(
                "const ANCHORS: u32 = {ANCHORS}u;\nconst RING: u32 = {RING}u;\n{JOINTS}{TUBE_SETUP}"
            ),
            &[
                read("joints", &self.body.joints),
                read("region", &frame.frame),
                write("fit", &fit),
            ],
            &[
                Word::U("anchor0", usize::from(anchors[0]) as u32),
                Word::U("anchor1", usize::from(anchors[1]) as u32),
                Word::U("anchor2", usize::from(anchors[2]) as u32),
            ],
            Grid::Singles(1),
        )?;
        let mut frames = vec![(frame, region)];
        let attachment = if matches!(rig, TubeRig::Arm) {
            let mut torso_design = GarmentArmorDesign::new(if design.kind == Kind::QuiltedSleeve {
                Kind::ArmingDoublet
            } else {
                Kind::MailShirt
            });
            torso_design.clearance = design.clearance;
            torso_design.wall_thickness = design.wall_thickness;
            let torso = self.record_fitted_torso(batch, &torso_design)?;
            self.record_ring(batch, &fit, torso.part.carriers(), &armhole(side))?;
            frames.extend(torso.frames);
            true
        } else {
            false
        };
        let support = self.record_region_support(batch, &[region])?;
        let part = record_garment_tube(self.gpu, batch, design, &fit)?;
        let gap = design.clearance.metres() + design.wall_thickness.metres() + GARMENT_FIT_MARGIN_M;
        dispatch(
            self,
            batch,
            &format!(
                "const ANCHORS: u32 = {ANCHORS}u;\nconst RING: u32 = {RING}u;\n{BAND}{RING_AT}{TUBE_FIT}"
            ),
            &[
                read("positions", &self.body.positions),
                read_u32("support", &support),
                read("fit", &fit),
                write("carriers", part.carriers()),
            ],
            &[
                Word::U("count", part.carrier_count()),
                Word::U("arm", u32::from(matches!(rig, TubeRig::Arm))),
                Word::U("attached", u32::from(attachment)),
                Word::U(
                    "quilted",
                    u32::from(matches!(
                        design.kind,
                        Kind::QuiltedSleeve | Kind::PaddedChausses
                    )),
                ),
                Word::U("reserved", u32::from(design.kind == Kind::QuiltedSleeve)),
                Word::F("gap", gap),
            ],
            Grid::Items(part.carrier_count()),
        )?;
        Ok(DeviceRecording {
            part,
            frames,
            checks: Vec::new(),
        })
    }
}

/// The limb frame, with its failure flag cleared, then the joint anchors.
const TUBE_SETUP: &str = r#"
@compute @workgroup_size(1)
fn main() {
    for (var w = 0u; w < 15u; w = w + 1u) {
        fit[w] = region[w];
    }
    fit[FIT_FAILED] = 0.0;
    let anchors = array<u32, 3>(params.anchor0, params.anchor1, params.anchor2);
    for (var a = 0u; a < 3u; a = a + 1u) {
        let p = joint(anchors[a]);
        fit[ANCHORS + a * 3u] = p.x;
        fit[ANCHORS + a * 3u + 1u] = p.y;
        fit[ANCHORS + a * 3u + 2u] = p.z;
    }
    fit[RING] = 0.0;
}
"#;

/// The sewing ring: points about their mean, stably sorted by angle.
const RING_SOURCE: &str = r#"
@compute @workgroup_size(1)
fn main() {
    let f = frame_at(0u);
    let n = params.count;
    var sum = vec3<f32>(0.0);
    for (var k = 0u; k < n; k = k + 1u) {
        sum = sum + points_at(ring_indices[k]);
    }
    let center = sum / f32(n);
    for (var k = 0u; k < n; k = k + 1u) {
        let p = points_at(ring_indices[k]);
        let offset = p - center;
        let angle = rem_tau(atan2(host_dot(offset, f.x), host_dot(offset, f.z)));
        // Insertion keeps equal angles in their original order.
        var j = k;
        loop {
            if (j == 0u || !(angle < fit[RING + 1u + (j - 1u) * 4u])) {
                break;
            }
            for (var w = 0u; w < 4u; w = w + 1u) {
                fit[RING + 1u + j * 4u + w] = fit[RING + 1u + (j - 1u) * 4u + w];
            }
            j = j - 1u;
        }
        fit[RING + 1u + j * 4u] = angle;
        fit[RING + 2u + j * 4u] = p.x;
        fit[RING + 3u + j * 4u] = p.y;
        fit[RING + 4u + j * 4u] = p.z;
    }
    fit[RING] = f32(n);
}
"#;

/// The sewing ring at an angle of the fit's frame, interpolated between the
/// ring points on either side of it.
pub(crate) const RING_AT: &str = r#"
fn ring_angle(k: u32) -> f32 {
    return fit[RING + 1u + k * 4u];
}

fn ring_point(k: u32) -> vec3<f32> {
    let at = RING + 2u + k * 4u;
    return vec3<f32>(fit[at], fit[at + 1u], fit[at + 2u]);
}

fn ring_at(angle: f32) -> vec3<f32> {
    let n = u32(fit[RING]);
    let a = rem_tau(angle);
    var next = 0u;
    loop {
        if (next >= n || !(ring_angle(next) < a)) {
            break;
        }
        next = next + 1u;
    }
    next = next % n;
    let previous = (next + n - 1u) % n;
    let first = ring_angle(previous);
    let span = rem_tau(ring_angle(next) - first);
    var t = 0.0;
    if (span > FLT_EPSILON) {
        t = rem_tau(a - first) / span;
    }
    return mix3(ring_point(previous), ring_point(next), t);
}
"#;

/// Every carrier moved onto the enclosing section of the limb at its station,
/// and a sleeve's top blended into the sewing ring.
const TUBE_FIT: &str = r#"
const SUPPORT_MASK: u32 = 1u;
const QUILTED_UPPER_ARM_BLEND_RESERVE_M: f32 = 0.011;
const UPPER_ARM_RESERVE_CENTER: f32 = 0.18;
const UPPER_ARM_RESERVE_HALF_SPAN: f32 = 0.18;
const MINIMUM_ENCLOSING_RADIUS_M: f32 = 0.005;

fn anchor(i: u32) -> vec3<f32> {
    return vec3<f32>(fit[ANCHORS + i * 3u], fit[ANCHORS + i * 3u + 1u], fit[ANCHORS + i * 3u + 2u]);
}

fn host_length(a: vec3<f32>) -> f32 {
    return sqrt(host_dot(a, a));
}

struct Station {
    center: vec3<f32>,
    tangent: vec3<f32>,
};

// A station on a Hermite curve through the three joints.
fn limb_station(segment: u32, t: f32) -> Station {
    let a = anchor(segment);
    let b = anchor(segment + 1u);
    if (t > 1.0) {
        return Station(b + (b - a) * (t - 1.0), host_normalized(a - b));
    }
    let bend_tangent = (anchor(2u) - anchor(0u)) * 0.5;
    var m0 = bend_tangent;
    var m1 = b - a;
    if (segment == 0u) {
        m0 = b - a;
        m1 = bend_tangent;
    }
    let t3 = t * t * t;
    let position = (((2.0 * t3 - 3.0 * t * t + 1.0) * a
        + (t3 - 2.0 * t * t + t) * m0)
        + (-2.0 * t3 + 3.0 * t * t) * b)
        + (t3 - t * t) * m1;
    let derivative = (((6.0 * t * t - 6.0 * t) * a
        + (3.0 * t * t - 4.0 * t + 1.0) * m0)
        + (-6.0 * t * t + 6.0 * t) * b)
        + (3.0 * t * t - 2.0 * t) * m1;
    return Station(position, host_normalized(derivative * -1.0));
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let f = frame_at(0u);
    let h = f.half_extents;
    let local = host_local(f, carriers_at(i));
    let raw_t = max((h.y - local.y) / (2.0 * h.y), 0.0);
    let angle = atan2(local.x / h.x, local.z / h.z);
    let s = sin(angle);
    let c = cos(angle);
    let radial_direction = f.x * s + f.z * c;
    let a0 = anchor(0u);
    let toward = vec3<f32>(0.0, a0.y, a0.z) - a0;
    let toward_trunk = host_normalized(toward - f.y * host_dot(toward, f.y));
    let medial = max(host_dot(radial_direction, toward_trunk), 0.0);
    let arm = params.arm != 0u;
    // The proximal opening climbs over the outer shoulder or hip and drops
    // beneath the armpit or groin.
    var setback = 0.015 + 0.20 * pow2(medial);
    var along = 0.985;
    if (arm) {
        setback = 0.025 + 0.22 * pow2(medial);
        along = 0.96;
    }
    let t = raw_t * along + setback * pow3(max(1.0 - raw_t, 0.0));
    let upper = host_length(anchor(1u) - anchor(0u));
    let lower = host_length(anchor(2u) - anchor(1u));
    let bend = upper / (upper + lower);
    var station: Station;
    if (t < bend) {
        station = limb_station(0u, t / bend);
    } else {
        station = limb_station(1u, (t - bend) / (1.0 - bend));
    }
    let tangent = station.tangent;
    let front = host_normalized(f.z - tangent * host_dot(f.z, tangent));
    let across = host_cross(tangent, front);
    let band = band_bounds(
        SUPPORT_MASK, station.center, across, tangent, front,
        SECTION_HALF_WIDTH_M, MINIMUM_SECTION_SAMPLES,
    );
    // Widen the bounds' ellipse to hold the band.
    var lo = band.lo;
    var hi = band.hi;
    let radial_center = vec2<f32>((lo.x + hi.x) * 0.5, (lo.z + hi.z) * 0.5);
    let radius = vec2<f32>(
        max((hi.x - lo.x) * 0.5, MINIMUM_ENCLOSING_RADIUS_M),
        max((hi.z - lo.z) * 0.5, MINIMUM_ENCLOSING_RADIUS_M),
    );
    var scale = 1.0;
    let count = arrayLength(&support);
    for (var v = 0u; v < count; v = v + 1u) {
        if ((support[v] & SUPPORT_MASK) == 0u) {
            continue;
        }
        let p = project(v, station.center, across, tangent, front);
        if (abs(p.y) < SECTION_HALF_WIDTH_M) {
            scale = max(scale, sqrt(
                pow2((p.x - radial_center.x) / radius.x) + pow2((p.z - radial_center.y) / radius.y),
            ));
        }
    }
    lo.x = radial_center.x - radius.x * scale;
    hi.x = radial_center.x + radius.x * scale;
    lo.z = radial_center.y - radius.y * scale;
    hi.z = radial_center.y + radius.y * scale;
    let section = vec2<f32>((hi.x - lo.x) * 0.5, (hi.z - lo.z) * 0.5);
    var quilting = 0.0;
    if (params.quilted != 0u) {
        quilting = 0.0015 * pow2(cos(angle * 6.0));
    }
    // Signed identity blends can pull the fitted exterior upper sleeve
    // inward; reserve cloth ease there.
    var reserve = 0.0;
    if (params.reserved != 0u) {
        let axial = (raw_t - UPPER_ARM_RESERVE_CENTER) / UPPER_ARM_RESERVE_HALF_SPAN;
        reserve = QUILTED_UPPER_ARM_BLEND_RESERVE_M * pow2(max(1.0 - axial * axial, 0.0))
            * max(max(-c, 0.0), 1.0 - medial);
    }
    let offset = vec3<f32>(
        (hi.x + lo.x) * 0.5 + (section.x + params.gap + quilting + reserve) * s,
        0.0,
        (hi.z + lo.z) * 0.5 + (section.y + params.gap + quilting + reserve) * c,
    );
    var point = station.center + ((across * offset.x + tangent * offset.y) + front * offset.z);
    if (params.attached != 0u) {
        let transition = clamp(raw_t / 0.24, 0.0, 1.0);
        let blend = 1.0 - transition * transition * (3.0 - 2.0 * transition);
        point = mix3(point, ring_at(angle), blend);
    }
    carriers_set(i, point);
}
"#;
