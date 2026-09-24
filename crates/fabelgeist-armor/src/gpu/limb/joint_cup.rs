//! Elbow and knee cops: a wrapped cup flowing into its side fan, or a
//! raised dish with a returned fan, and the lower plates that may follow a
//! wrapped cup's distal edge.

use super::super::chart::ChartBoundary;
use super::super::part::Extrusion;
use super::super::recipe::PartRecipe;
use super::{ALONG, ArmorGpu, LimbShape, kernel, plain_chart};
use crate::{
    ArmorComponentRole, GenerateError, JointCupConstruction, JointCupDesign, JointFluteOrientation,
};

/// The joint cup's design floats and its two surfaces, the wrapped cup and
/// the raised cop, which charts read through `cup_point`.
const JOINT_CUP_SURFACE: &str = r#"
const DOME: u32 = 0u;
const WING: u32 = 1u;
const WING_ROUNDNESS: u32 = 2u;
const WING_NOTCH: u32 = 3u;
const WING_HEIGHT: u32 = 4u;
const DISTAL_WING_SCALE: u32 = 5u;
const PROXIMAL_FLARE: u32 = 6u;
const CENTER_RIDGE: u32 = 7u;
const LENGTH: u32 = 8u;
const CLEARANCE: u32 = 9u;
const RAISED: u32 = 10u;
const MEDIAL_WRAP: u32 = 11u;
const LATERAL_WRAP: u32 = 12u;
const TRANSVERSE: u32 = 13u;
const EXTENSION_LENGTH: u32 = 14u;
const EXTENSION_WRAP: u32 = 15u;
const EXTENSION_TAPER: u32 = 16u;
const EXTENSION_HEM_ROUNDING: u32 = 17u;
const EXTENSION_LAP: u32 = 18u;

// One continuous sheet flows from the projecting joint dish into the side
// fan; the fan's notch removes only reach beyond the enclosing cup.
fn wrapped_point(u: f32, v: f32) -> vec3<f32> {
    let width = fit.half_extents.x;
    let length = fit.half_extents.y;
    let depth = fit.half_extents.z;
    let clearance = design[CLEARANCE];
    let wing = design[WING];
    let basic_angle = lerp(-PI * 0.52, PI * (0.52 + wing * 0.35), u);
    let axial = 2.0 * v - 1.0;
    let fan = smoothstep_unclamped(clamp((basic_angle / PI - 0.20) / 0.38, 0.0, 1.0));
    let end_fraction = clamp((u - 0.72) / 0.28, 0.0, 1.0);
    let rounded_tip = lerp(1.0, sqrt(1.0 - 0.96 * pow2(end_fraction)), design[WING_ROUNDNESS]);
    let theta = basic_angle;
    let distal_scale = lerp(1.0, design[DISTAL_WING_SCALE], smoothstep_unclamped(max(-axial, 0.0)));
    let crown = 1.0 + design[DOME] * 0.8 * (1.0 - axial * axial) * (1.0 - fan);
    let section_width = (width + clearance) * (1.0 - 0.12 * axial * axial * (1.0 - fan));
    let section_depth = (depth + clearance) * crown;
    let radius = 1.0 / sqrt(pow2(sin(theta) / section_width) + pow2(cos(theta) / section_depth));
    let rim_flare = design[PROXIMAL_FLARE] * pow2(max(axial, 0.0));
    let wing_reach = width * wing * fan * (1.0 - design[WING_NOTCH] * pow2(1.0 - axial * axial));
    return vec3<f32>(
        (radius + wing_reach + rim_flare) * sin(theta),
        axial * (length + clearance) * design[LENGTH]
            * lerp(0.72, (0.95 + wing) * design[WING_HEIGHT] * distal_scale, fan)
            * rounded_tip,
        (radius + wing_reach + rim_flare) * cos(theta)
            + design[CENTER_RIDGE] * pow8(max(cos(theta), 0.0)) * (1.0 - axial * axial),
    );
}

// A closed-outline joint dish with an integral, laterally returned fan. A
// square-to-disc chart keeps an ordinary closed perimeter and fixed
// connectivity, without a collapsed polar apex or an axial opening.
fn raised_point(u: f32, v: f32) -> vec3<f32> {
    let width = fit.half_extents.x;
    let length = fit.half_extents.y;
    let depth = fit.half_extents.z;
    let clearance = design[CLEARANCE];
    let square_x = 2.0 * u - 1.0;
    let square_y = 2.0 * v - 1.0;
    let rounding = design[WING_ROUNDNESS] * 0.5;
    let x = square_x * sqrt(1.0 - rounding * pow2(square_y));
    let y = square_y * sqrt(1.0 - rounding * pow2(square_x));
    let fan = smoothstep_unclamped(clamp((x - 0.2) / 0.8, 0.0, 1.0));
    let notch = 1.0 - design[WING_NOTCH] * pow2(max(1.0 - y * y, 0.0));
    // The lateral fan ends in a median point, its upper and lower edges
    // returning to the enclosing cup; roundness softens both slopes.
    var fan_height = 1.0;
    if (square_y != 0.0) {
        fan_height = 1.0 - pow(abs(square_y), 1.0 + design[WING_ROUNDNESS]);
    }
    let wing_reach = width * design[WING] * 4.0 * fan * (1.0 - fan) * notch * fan_height;
    let distal = 1.0 + (design[DISTAL_WING_SCALE] - 1.0) * smoothstep_unclamped(max(-y, 0.0));
    let dish = max(1.0 - x * x - y * y, 0.0);
    let angle = -DEFAULT_MEDIAL_WRAP * PI
        + (DEFAULT_LATERAL_WRAP + DEFAULT_MEDIAL_WRAP) * PI * (x + 1.0) * 0.5;
    var theta = angle * design[LATERAL_WRAP] / DEFAULT_LATERAL_WRAP;
    if (angle < 0.0) {
        theta = angle * design[MEDIAL_WRAP] / DEFAULT_MEDIAL_WRAP;
    }
    let radius = 1.0
        / sqrt(pow2(sin(theta) / (width + clearance)) + pow2(cos(theta) / (depth + clearance)));
    let ridge = design[CENTER_RIDGE] * max(1.0 - abs(y), 0.0) * fan;
    let radial = radius
        + depth * 0.8 * design[DOME] * dish * (1.0 - fan)
        + design[PROXIMAL_FLARE] * pow4(max(y, 0.0));
    // The fan reaches its lateral extremity before returning to its frontal
    // edge, which stays on the enclosing radius.
    return vec3<f32>(
        (radial + ridge) * sin(theta) + wing_reach,
        (length + clearance) * design[LENGTH] * y
            * (1.0 + (design[WING_HEIGHT] - 1.0) * fan) * distal,
        (radial + ridge) * cos(theta),
    );
}

// Transverse flutes run across the plate: the chart's columns follow its
// length instead of its width.
fn cup_point(u: f32, v: f32) -> vec3<f32> {
    var s = u;
    var t = v;
    if (design[TRANSVERSE] != 0.0) {
        s = v;
        t = 1.0 - u;
    }
    if (design[RAISED] != 0.0) {
        return raised_point(s, t);
    }
    return wrapped_point(s, t);
}
"#;

const JOINT_CUP: &str = r#"
fn chart_offset(u: f32, axial: f32) -> f32 {
    return 0.0;
}

fn chart_point(u: f32, v: f32) -> vec3<f32> {
    return cup_point(u, v);
}
"#;

/// One lower plate following the cup's distal edge; `value0` and `value1`
/// are its share of the extension, `value2` its index and `value3` whether
/// it is the last.
const JOINT_EXTENSION: &str = r#"
fn chart_offset(u: f32, axial: f32) -> f32 {
    return 0.0;
}

fn chart_point(u: f32, v: f32) -> vec3<f32> {
    let start = params.value0;
    let end = params.value1;
    let down = end - (end - start) * v;
    let angle = design[EXTENSION_WRAP] * (2.0 * u - 1.0);
    // Every plate follows the same cup edge; the lap offset is independent
    // of sampling within a plate, so adjacent lames cannot cross.
    let edge = wrapped_point((angle / PI + 0.52) / (1.04 + design[WING] * 0.35), 0.0);
    let lap = design[EXTENSION_LAP] * (params.value2 + 1.0);
    let taper = 1.0 + (design[EXTENSION_TAPER] - 1.0) * down;
    var overlap = 0.002 * v;
    if (params.value2 == 0.0) {
        overlap = 0.0005 * v;
    }
    var hem = 0.0;
    if (params.value3 != 0.0) {
        hem = design[EXTENSION_HEM_ROUNDING] * pow2(2.0 * u - 1.0) * pow4(1.0 - v);
    }
    return vec3<f32>(
        edge.x * taper + lap * sin(angle),
        edge.y - design[EXTENSION_LENGTH] * down + overlap + hem,
        edge.z * taper + lap * cos(angle),
    );
}
"#;

pub(super) fn joint_cup(gpu: &ArmorGpu, d: &JointCupDesign) -> Result<LimbShape, GenerateError> {
    const JOINT_EDGE_MARGIN_M: f32 = 0.004;
    /// Rows across a raised cop's square-to-disc chart.
    const DISH_ROWS: usize = 24;
    /// Rows along each lower plate.
    const LAME_ROWS: usize = 12;
    /// Each lower plate laps outward over the one above by this many gauges.
    const LAP_SPACING_GAUGES: f32 = 2.5;
    let raised = d.construction == JointCupConstruction::RaisedCop;
    let gauge = d.gauge.thickness.metres();
    let surface = |shape: &str| {
        kernel(
            gpu,
            &format!(
                "const DEFAULT_MEDIAL_WRAP: f32 = {:?};\nconst DEFAULT_LATERAL_WRAP: f32 = {:?};\n{JOINT_CUP_SURFACE}{shape}",
                JointCupDesign::DEFAULT_MEDIAL_WRAP.unit(),
                JointCupDesign::DEFAULT_LATERAL_WRAP.unit(),
            ),
        )
    };
    let mut part = PartRecipe::new();
    let (rows, extrusion) = if raised {
        (DISH_ROWS, Extrusion::Normal)
    } else {
        (ALONG, Extrusion::Radial)
    };
    part.push_chart(
        plain_chart(
            rows,
            ChartBoundary::Open,
            gauge,
            extrusion,
            d.fluting,
            [0.0, 1.0],
        ),
        surface(JOINT_CUP)?,
    )?;
    part.component(ArmorComponentRole::Plate, None);
    let extension = d.distal_extension.as_ref();
    if let Some(extension) = extension {
        let lames = extension.lame_count;
        let terminal = extension.terminal_share.unit();
        let lame_kernel = surface(JOINT_EXTENSION)?;
        let mut start = 0.0;
        for lame in 0..lames {
            let last = lame + 1 == lames;
            let share = if lames == 1 {
                1.0
            } else if last {
                terminal
            } else {
                (1.0 - terminal) / f32::from(lames - 1)
            };
            let end = start + share;
            let mut chart = plain_chart(
                LAME_ROWS,
                ChartBoundary::Open,
                gauge,
                Extrusion::Normal,
                d.fluting,
                [start, end],
            );
            chart.values = [start, end, f32::from(lame), f32::from(u8::from(last))];
            part.push_chart(chart, lame_kernel.clone())?;
            start = end;
        }
        part.component(ArmorComponentRole::JointExtension, None);
    }
    let relief = d.fluting.as_ref().map_or(0.0, |f| f.depth.metres());
    Ok(LimbShape {
        part,
        design: vec![
            d.dome.unit(),
            d.wing.unit(),
            d.wing_roundness.unit(),
            d.wing_notch.unit(),
            d.wing_height.unit(),
            d.distal_wing_scale.unit(),
            d.proximal_flare.metres(),
            d.center_ridge.metres(),
            d.length.unit(),
            d.gauge.clearance.metres() + gauge + JOINT_EDGE_MARGIN_M,
            f32::from(u8::from(raised)),
            d.medial_wrap.unit(),
            d.lateral_wrap.unit(),
            f32::from(u8::from(
                d.flute_orientation == JointFluteOrientation::Transverse,
            )),
            extension.map_or(0.0, |e| e.length.metres()),
            extension.map_or(0.0, |e| e.wrap.radians()),
            extension.map_or(1.0, |e| e.distal_taper.unit()),
            extension.map_or(0.0, |e| e.hem_rounding.metres()),
            (gauge + relief) * LAP_SPACING_GAUGES,
        ],
    })
}
