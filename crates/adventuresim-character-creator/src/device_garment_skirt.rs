//! Skirts, faulds and tassets fitted on the device. A skirt or fauld wraps a
//! drape cage of convex sections around the pelvis -- and, for a flexible
//! skirt, around the chausses beneath it, sewn to the hem of the shirt above.
//! Tassets sit on a smooth depth datum sampled from the front of the hips.

use anyhow::{Result, bail};
use fabelgeist_armor::gpu::{record_fauld, record_garment_tube, record_tassets, wgsl};
use fabelgeist_armor::{
    DevicePart, GARMENT_RING_SEGMENTS, GarmentArmorDesign, GarmentArmorKind as Kind,
    GarmentPlateShape,
};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

use crate::armor_frames::FitRegion;
use crate::device_frames::{DeviceFrame, DeviceWearer};
use crate::device_garment::{BAND, UprightSpan};
use crate::device_garment_kernel::{Grid, Word, atomic, dispatch, read, read_u32, write};
use crate::device_garment_tube::{RING, RING_AT, RING_FIT_WORDS, hem};
use crate::device_piece::DeviceRecording;

/// Drape cage stations, evenly spaced over the frame's height.
const STATIONS: u32 = 17;
/// Tasset depth samples along each side of the datum grid.
pub(crate) const DEPTH_SIDE: u32 = 17;
/// Where a skirt's fit keeps its drape cage, the cage before smoothing, and
/// its tasset depth datum.
const CAGE: u32 = RING_FIT_WORDS;
const REQUIRED: u32 = CAGE + STATIONS * GARMENT_RING_SEGMENTS as u32;
const DEPTH: u32 = REQUIRED + STATIONS * GARMENT_RING_SEGMENTS as u32;
const SKIRT_FIT_WORDS: u32 = DEPTH + DEPTH_SIDE * DEPTH_SIDE;
/// The fauld hangs from above the waist to below the hip joint.
const FAULD_DROP_M: f32 = 0.085;
/// Tassets hang from just above the hip joint.
const TASSET_RISE_M: f32 = 0.015;
/// A skirt reaches this fraction of the way from the hip to the knee.
const SKIRT_REACH: f32 = 0.48;

pub(crate) fn layout() -> String {
    format!(
        "const RING: u32 = {RING}u;\nconst CAGE: u32 = {CAGE}u;\nconst REQUIRED: u32 = {REQUIRED}u;\n\
         const DEPTH: u32 = {DEPTH}u;\nconst STATIONS: u32 = {STATIONS}u;\n\
         const SEGMENTS: u32 = {GARMENT_RING_SEGMENTS}u;\nconst DEPTH_SIDE: u32 = {DEPTH_SIDE}u;\n"
    )
}

impl DeviceWearer<'_> {
    /// Record a skirt, fauld or tassets, fitted, short of thickening.
    pub(crate) fn record_fitted_skirt(
        &self,
        batch: &mut KernelBatch,
        design: &GarmentArmorDesign,
    ) -> Result<DeviceRecording> {
        let gpu = self.gpu;
        let hips = self.record_frame(batch, FitRegion::Hips)?;
        let head = self.record_frame(batch, FitRegion::Head)?;
        let fit = self.record_upright(batch, SKIRT_FIT_WORDS, &hips, &head, skirt_span(design)?)?;
        let mut frames = vec![(hips, FitRegion::Hips), (head, FitRegion::Head)];
        if design.kind == Kind::Tassets {
            let support = self.record_region_support(batch, &[FitRegion::Hips])?;
            let part = record_tassets(gpu, batch, design, &fit)?;
            self.record_tasset_fit(batch, &fit, &support, &part)?;
            self.record_fit_status(batch, &fit, &part)?;
            return Ok(DeviceRecording {
                part,
                frames,
                checks: Vec::new(),
            });
        }
        let support = self.record_region_support(batch, &[FitRegion::Hips, FitRegion::Torso])?;
        if design.kind == Kind::Fauld {
            // The fauld's breadth is its top section's.
            dispatch(
                self,
                batch,
                &format!("{BAND}{FAULD_SECTION}"),
                &[
                    read("positions", &self.body.positions),
                    read_u32("support", &support),
                    write("fit", &fit),
                ],
                &[],
                Grid::Singles(1),
            )?;
        }
        let flexible = matches!(design.kind, Kind::MailSkirt | Kind::PaddedSkirt);
        let empty = gpu.scratch(4, "no legs")?;
        let legs = if flexible {
            self.record_skirt_layers(batch, design, &fit, &mut frames)?
        } else {
            Vec::new()
        };
        let leg = |i: usize| {
            legs.get(i).map_or((&empty, 0), |p: &DevicePart| {
                (p.positions(), p.vertex_count())
            })
        };
        let (left, left_count) = leg(0);
        let (right, right_count) = leg(1);
        dispatch(
            self,
            batch,
            &format!("{}{DRAPE_STATIONS}", layout()),
            &[
                read("positions", &self.body.positions),
                read_u32("support", &support),
                read("points", left),
                read("extra", right),
                write("fit", &fit),
            ],
            &[Word::U("left", left_count), Word::U("right", right_count)],
            Grid::Singles(STATIONS),
        )?;
        dispatch(
            self,
            batch,
            &format!("{}{DRAPE_FINISH}", layout()),
            &[write("fit", &fit)],
            &[],
            Grid::Singles(1),
        )?;
        let part = if design.kind == Kind::Fauld {
            record_fauld(gpu, batch, design, &fit)?
        } else {
            record_garment_tube(gpu, batch, design, &fit)?
        };
        self.record_wrap(batch, design, &fit, &part, flexible)?;
        self.record_fit_status(batch, &fit, &part)?;
        Ok(DeviceRecording {
            part,
            frames,
            checks: Vec::new(),
        })
    }

    /// Record the garments a flexible skirt drapes over -- both legs'
    /// chausses, thickened, and the torso garment whose hem it hangs from --
    /// adding their frames to `frames`; returns the legs.
    fn record_skirt_layers(
        &self,
        batch: &mut KernelBatch,
        design: &GarmentArmorDesign,
        fit: &Buffer,
        frames: &mut Vec<(DeviceFrame, FitRegion)>,
    ) -> Result<Vec<DevicePart>> {
        let padded = design.kind == Kind::PaddedSkirt;
        let layer = |kind| {
            let mut layer = GarmentArmorDesign::new(kind);
            layer.clearance = design.clearance;
            layer.wall_thickness = design.wall_thickness;
            layer
        };
        let leg_design = layer(if padded {
            Kind::PaddedChausses
        } else {
            Kind::MailChausses
        });
        let mut legs = Vec::new();
        for placement in ["left", "right"] {
            let mut leg = self.record_fitted_tube(batch, &leg_design, placement)?;
            leg.part.record_shells(self.gpu, batch)?;
            frames.extend(leg.frames);
            legs.push(leg.part);
        }
        let top_design = layer(if padded {
            Kind::ArmingDoublet
        } else {
            Kind::MailShirt
        });
        let torso = self.record_fitted_torso(batch, &top_design)?;
        self.record_ring(batch, fit, torso.part.carriers(), &hem())?;
        frames.extend(torso.frames);
        Ok(legs)
    }

    /// Record every shell's carriers wrapped around the drape cage.
    fn record_wrap(
        &self,
        batch: &mut KernelBatch,
        design: &GarmentArmorDesign,
        fit: &Buffer,
        part: &DevicePart,
        flexible: bool,
    ) -> Result<()> {
        let gpu = self.gpu;
        let shells = part.shell_count();
        let mut shell_of = vec![0u32; part.carrier_count() as usize];
        for shell in 0..shells {
            for carrier in part.shell_carriers(shell) {
                shell_of[carrier as usize] = shell as u32;
            }
        }
        let mut bounds = Vec::with_capacity(shells * 2);
        for _ in 0..shells {
            bounds.extend([ORDERED_NEGATIVE_INFINITY, ORDERED_POSITIVE_INFINITY]);
        }
        let shell_of = gpu.upload(&shell_of)?;
        let bounds = gpu.upload(&bounds)?;
        let words = [
            Word::U("count", part.carrier_count()),
            Word::U("flexible", u32::from(flexible)),
            Word::F(
                "gap",
                design.clearance.metres() + design.wall_thickness.metres(),
            ),
            Word::F("wall", design.wall_thickness.metres()),
            Word::F("flare", design.flare.unit()),
            Word::F(
                "inset",
                if design.kind == Kind::PaddedSkirt {
                    design.wall_thickness.metres() * 1.5
                } else {
                    0.0
                },
            ),
        ];
        for entry in [WRAP_BOUNDS, WRAP] {
            dispatch(
                self,
                batch,
                &format!(
                    "{}{}{RING_AT}{CAGE_AT}{entry}",
                    layout(),
                    wgsl::ORDERED_FLOAT
                ),
                &[
                    read("fit", fit),
                    read_u32("shell_of", &shell_of),
                    atomic("bounds", &bounds),
                    write("carriers", part.carriers()),
                ],
                &words,
                Grid::Items(part.carrier_count()),
            )?;
        }
        Ok(())
    }
}

const ORDERED_POSITIVE_INFINITY: u32 = 0xff80_0000;
const ORDERED_NEGATIVE_INFINITY: u32 = 0x007f_ffff;

/// The fauld's breadth and depth from the section at its top.
const FAULD_SECTION: &str = r#"
const SUPPORT_MASK: u32 = 3u;

@compute @workgroup_size(1)
fn main() {
    let f = frame_at(0u);
    let center = host_point(f, vec3<f32>(0.0, f.half_extents.y, 0.0));
    let band = band_bounds(
        SUPPORT_MASK,
        center,
        f.x,
        f.y,
        f.z,
        SECTION_HALF_WIDTH_M,
        MINIMUM_SECTION_SAMPLES,
    );
    fit[12] = (band.hi.x - band.lo.x) * 0.5;
    fit[14] = (band.hi.z - band.lo.z) * 0.5;
    if (!(fit[12] > 0.0 && fit[14] > 0.0 && abs(fit[12]) <= INFINITY && abs(fit[14]) <= INFINITY)) {
        fit[FIT_FAILED] = 1.0;
    }
}
"#;

/// One invocation per drape cage station: the support of the section's points
/// in each direction, then the radius each direction must reach to enclose
/// every supporting line.
const DRAPE_STATIONS: &str = r#"
const SUPPORT_MASK: u32 = 3u;
const SECTION_HALF_WIDTH_M: f32 = 0.018;
const MINIMUM_SECTION_POINTS: u32 = 12u;

var<private> supports: array<f32, SEGMENTS>;

// The envelope: supported skin in vertex order, then each leg's garment.
fn envelope_count() -> u32 {
    return arrayLength(&support) + params.left + params.right;
}

// An envelope point in the frame; `w` is zero for unsupported skin.
fn envelope(slot: u32, f: Frame) -> vec4<f32> {
    let skin = arrayLength(&support);
    var p: vec3<f32>;
    if (slot < skin) {
        if ((support[slot] & SUPPORT_MASK) == 0u) {
            return vec4<f32>(0.0);
        }
        p = positions_at(slot);
    } else if (slot < skin + params.left) {
        p = points_at(slot - skin);
    } else {
        p = extra_at(slot - skin - params.left);
    }
    return vec4<f32>(host_local(f, p), 1.0);
}

fn include(p: vec3<f32>) {
    for (var col = 0u; col < SEGMENTS; col = col + 1u) {
        let angle = TAU * f32(col) / f32(SEGMENTS);
        supports[col] = max(supports[col], p.x * sin(angle) + p.z * cos(angle));
    }
}

@compute @workgroup_size(1)
fn main(@builtin(workgroup_id) group: vec3<u32>) {
    let row = group.x;
    let f = frame_at(0u);
    let y = f.half_extents.y * (2.0 * f32(row) / f32(STATIONS - 1u) - 1.0);
    for (var col = 0u; col < SEGMENTS; col = col + 1u) {
        supports[col] = 0.0;
    }
    let count = envelope_count();
    var near = 0u;
    var total = 0u;
    for (var slot = 0u; slot < count; slot = slot + 1u) {
        let p = envelope(slot, f);
        if (p.w == 0.0) {
            continue;
        }
        total = total + 1u;
        if (abs(p.y - y) < SECTION_HALF_WIDTH_M) {
            near = near + 1u;
            include(p.xyz);
        }
    }
    if (near < MINIMUM_SECTION_POINTS && near < total) {
        // The nearest points by height instead, ties to the earliest.
        for (var col = 0u; col < SEGMENTS; col = col + 1u) {
            supports[col] = 0.0;
        }
        var last_distance = -1.0;
        var last = 0u;
        for (var k = 0u; k < min(MINIMUM_SECTION_POINTS, total); k = k + 1u) {
            var best_distance = INFINITY;
            var best = 0xffffffffu;
            var best_point = vec3<f32>(0.0);
            for (var slot = 0u; slot < count; slot = slot + 1u) {
                let p = envelope(slot, f);
                if (p.w == 0.0) {
                    continue;
                }
                let d = abs(p.y - y);
                let after = d > last_distance || (d == last_distance && slot > last);
                if (after && (d < best_distance || (d == best_distance && slot < best))) {
                    best_distance = d;
                    best = slot;
                    best_point = p.xyz;
                }
            }
            if (best == 0xffffffffu) {
                break;
            }
            include(best_point);
            last_distance = best_distance;
            last = best;
        }
    }
    for (var col = 0u; col < SEGMENTS; col = col + 1u) {
        let angle = TAU * f32(col) / f32(SEGMENTS);
        var radius = INFINITY;
        for (var normal = 0u; normal < SEGMENTS; normal = normal + 1u) {
            let normal_angle = TAU * f32(normal) / f32(SEGMENTS);
            let facing = cos(angle - normal_angle);
            if (facing > FLT_EPSILON) {
                radius = min(radius, supports[normal] / facing);
            }
        }
        fit[CAGE + row * SEGMENTS + col] = radius;
    }
}
"#;

/// The drape cage finished: the hem hangs beyond the hip, the profile is
/// smoothed, and each direction regains any clearance smoothing lost.
const DRAPE_FINISH: &str = r#"
const PROFILE_SMOOTHING_PASSES: u32 = 8u;

fn at(row: u32, col: u32) -> u32 {
    return CAGE + row * SEGMENTS + col;
}

@compute @workgroup_size(1)
fn main() {
    for (var row = STATIONS - 1u; row > 0u; row = row - 1u) {
        for (var col = 0u; col < SEGMENTS; col = col + 1u) {
            fit[at(row - 1u, col)] = max(fit[at(row - 1u, col)], fit[at(row, col)]);
        }
    }
    for (var i = 0u; i < STATIONS * SEGMENTS; i = i + 1u) {
        fit[REQUIRED + i] = fit[CAGE + i];
    }
    for (var sweep = 0u; sweep < PROFILE_SMOOTHING_PASSES; sweep = sweep + 1u) {
        for (var col = 0u; col < SEGMENTS; col = col + 1u) {
            var below = fit[at(0u, col)];
            for (var row = 1u; row < STATIONS - 1u; row = row + 1u) {
                let here = fit[at(row, col)];
                fit[at(row, col)] = (below + 2.0 * here + fit[at(row + 1u, col)]) * 0.25;
                below = here;
            }
        }
    }
    for (var col = 0u; col < SEGMENTS; col = col + 1u) {
        var shortfall = 0.0;
        for (var row = 0u; row < STATIONS; row = row + 1u) {
            shortfall = max(shortfall, fit[REQUIRED + row * SEGMENTS + col] - fit[at(row, col)]);
        }
        for (var row = 0u; row < STATIONS; row = row + 1u) {
            fit[at(row, col)] = fit[at(row, col)] + shortfall;
        }
    }
}
"#;

/// The drape cage's radius at a height and angle, interpolated between
/// stations and between directions.
const CAGE_AT: &str = r#"
fn cage_radius(y: f32, col: u32) -> f32 {
    let f = frame_at(0u);
    let last = f32(STATIONS - 1u);
    let station = clamp((y / f.half_extents.y + 1.0) * 0.5 * last, 0.0, last);
    let lower = min(u32(floor(station)), STATIONS - 2u);
    let t = station - f32(lower);
    return fit[CAGE + lower * SEGMENTS + col] * (1.0 - t)
        + fit[CAGE + (lower + 1u) * SEGMENTS + col] * t;
}

fn radius_at_angle(y: f32, angle: f32) -> f32 {
    let column = rem_tau(angle) / TAU * f32(SEGMENTS);
    let lower = u32(floor(column)) % SEGMENTS;
    let fraction = column - floor(column);
    return cage_radius(y, lower) * (1.0 - fraction)
        + cage_radius(y, (lower + 1u) % SEGMENTS) * fraction;
}
"#;

/// Each shell's height range, which spaces a fauld's lames.
const WRAP_BOUNDS: &str = r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let y = carriers_at(i).y;
    atomicMax(&bounds[shell_of[i] * 2u], ordered_from_float(y));
    atomicMin(&bounds[shell_of[i] * 2u + 1u], ordered_from_float(y));
}
"#;

/// Every carrier wrapped around the drape cage at its angle and height: flared
/// toward the hem and held off by the gap, each fauld lame stepped out toward
/// its bottom, and a flexible skirt blended into the shirt's hem near its top.
const WRAP: &str = r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let f = frame_at(0u);
    let p = carriers_at(i);
    let local = host_local(f, p);
    let angle = atan2(local.x, local.z);
    let descent = clamp((f.half_extents.y - local.y) / (2.0 * f.half_extents.y), 0.0, 1.0);
    let radius = radius_at_angle(local.y, angle);
    var step = 0.0;
    if (params.flexible == 0u) {
        let top = float_from_ordered(atomicLoad(&bounds[shell_of[i] * 2u]));
        let bottom = float_from_ordered(atomicLoad(&bounds[shell_of[i] * 2u + 1u]));
        step = params.wall * 2.5 * (top - p.y) / (top - bottom);
    }
    let flare = params.flare * descent;
    let s = sin(angle);
    let c = cos(angle);
    let x = (radius * (1.0 + flare) + params.gap + step) * s;
    let z = (radius * (1.0 + flare) + params.gap + step) * c;
    var fitted = host_point(f, vec3<f32>(x, local.y, z));
    if (params.flexible != 0u) {
        let transition = clamp(descent / 0.45, 0.0, 1.0);
        let blend = 1.0 - transition * transition * (3.0 - 2.0 * transition);
        var seam = ring_at(angle);
        seam.y = fitted.y;
        seam.x = seam.x - f.x.x * s * params.inset;
        seam.z = seam.z - f.z.z * c * params.inset;
        fitted = mix3(fitted, seam, blend);
    }
    carriers_set(i, fitted);
}
"#;

/// Where a skirt's upright frame runs, from the waist down.
fn skirt_span(design: &GarmentArmorDesign) -> Result<UprightSpan> {
    Ok(match (design.kind, design.plate_shape) {
        (Kind::Fauld, GarmentPlateShape::Fauld { waist_rise, .. }) => UprightSpan {
            top: ("c_spine1", waist_rise.metres()),
            bottom: ("root", FAULD_DROP_M),
            reach: ("root", 0.0),
        },
        (Kind::Fauld, _) => bail!("fauld shape required"),
        (Kind::Tassets, _) => UprightSpan {
            top: ("root", TASSET_RISE_M),
            bottom: ("root", 0.0),
            reach: ("l_lowleg", SKIRT_REACH),
        },
        (kind, _) => UprightSpan {
            top: (
                "c_spine1",
                if kind == Kind::PaddedSkirt {
                    design.wall_thickness.metres() * 3.0
                } else {
                    0.0
                },
            ),
            bottom: ("root", 0.0),
            reach: ("l_lowleg", SKIRT_REACH),
        },
    })
}
