//! Limb plates on the device: rerebraces, spaulders, cuisses, greaves and joint cups as charts.
//!
//! Each shape is a pair of WGSL chart functions over its design floats. The
//! design floats are listed once, in [`LimbShape::design`], and the WGSL
//! constants name the same slots.

use std::f32::consts::PI;

use super::chart::{AUTHORED_ORIGIN, ChartBoundary, ChartKernel, PlateChart};
use super::part::Extrusion;
use super::recipe::PartRecipe;
use super::{ArmorGpu, BuiltPart};
use crate::{
    ArmorComponentRole, CuisseDesign, GenerateError, GreaveDesign, JointCupConstruction,
    JointCupDesign, JointFluteOrientation, LimbArmorDesign, PartFrame, RerebraceDesign,
    SpaulderDesign,
};

/// Rows along a long plate.
const ALONG: usize = 16;

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

const SPAULDER_DESIGN: &str = r#"
const WRAP: u32 = 0u;
const REAR_EXTENSION: u32 = 1u;
const CROWN: u32 = 2u;
const CROWN_REACH: u32 = 3u;
const LOWER_FLARE: u32 = 4u;
const LENGTH: u32 = 5u;
const GAUGE: u32 = 6u;
const CLEARANCE: u32 = 7u;
"#;

/// One overlapping lame; `value0` and `value1` are its axial span.
const SPAULDER_LAME: &str = r#"
fn chart_offset(u: f32, axial: f32) -> f32 {
    return 0.0;
}

fn chart_point(u: f32, v: f32) -> vec3<f32> {
    let width = fit.half_extents.x;
    let length = fit.half_extents.y;
    let depth = fit.half_extents.z;
    let clearance = design[CLEARANCE];
    let wrap = design[WRAP];
    let axial = lerp(params.value0, params.value1, v);
    let theta = lerp(-PI * wrap * design[REAR_EXTENSION], PI * wrap, u);
    let crown = lerp(0.88, design[CROWN], smoothstep_unclamped(axial));
    let section_depth = lerp(0.78, design[CROWN], smoothstep_unclamped(axial));
    let step = design[GAUGE] * 2.0 * (1.0 - v);
    let radius = 1.0 / sqrt(
        pow2(sin(theta) / (width * crown + clearance))
            + pow2(cos(theta) / (depth * section_depth + clearance)),
    ) + step + design[LOWER_FLARE] * (1.0 - axial) * pow2(sin(theta));
    return vec3<f32>(
        radius * sin(theta),
        lerp(-1.65, -0.10, axial) * length * design[LENGTH],
        radius * cos(theta),
    );
}
"#;

/// The formed crown closing toward a shared apex above the shoulder.
const SPAULDER_CROWN: &str = r#"
fn chart_origin() -> vec3<f32> {
    return vec3<f32>(0.0, -fit.half_extents.y * 0.16, 0.0);
}

// The upper return needs space above the clavicle and shoulder roll.
fn chart_offset(u: f32, axial: f32) -> f32 {
    return 0.004 * pow3((axial - 0.5) * 2.0);
}

fn chart_point(u: f32, v: f32) -> vec3<f32> {
    let width = fit.half_extents.x;
    let length = fit.half_extents.y;
    let depth = fit.half_extents.z;
    let clearance = design[CLEARANCE];
    let wrap = min(design[WRAP], 0.60);
    let theta = lerp(-PI * wrap * design[REAR_EXTENSION], PI * wrap, u);
    let latitude = v * PI * 0.5;
    let step = design[GAUGE] * 2.0;
    let radius = 1.0 / sqrt(
        pow2(sin(theta) / (width * design[CROWN] + clearance))
            + pow2(cos(theta) / (depth * design[CROWN] + clearance)),
    ) + step;
    return vec3<f32>(
        radius * sin(theta) * cos(latitude),
        -length * 0.16 + length * 1.40 * sin(latitude),
        radius * cos(theta) * cos(latitude)
            + depth * (1.10 - design[CROWN_REACH]) * (1.0 - cos(latitude)),
    );
}
"#;

const GREAVE: &str = r#"
const SHIN_RIDGE: u32 = 0u;
const CALF_HEIGHT: u32 = 1u;
const ANKLE_TAPER: u32 = 2u;
const KNEE_TAPER: u32 = 3u;
const LENGTH: u32 = 4u;
const ANKLE_EXTENSION: u32 = 5u;
const CLEARANCE: u32 = 6u;

fn chart_offset(u: f32, axial: f32) -> f32 {
    return pow8(max(cos(TAU * (u - 0.5)), 0.0)) * design[SHIN_RIDGE];
}

// A narrow ankle, high calf belly, then a reduced knee throat.
fn chart_point(u: f32, v: f32) -> vec3<f32> {
    let width = fit.half_extents.x;
    let length = fit.half_extents.y;
    let depth = fit.half_extents.z;
    let clearance = design[CLEARANCE];
    let calf = design[CALF_HEIGHT];
    let theta = TAU * (u - 0.5);
    var shape: f32;
    if (v < calf) {
        shape = lerp(design[ANKLE_TAPER], 1.0, smoothstep_unclamped(v / calf));
    } else {
        shape = lerp(1.0, design[KNEE_TAPER], smoothstep_unclamped((v - calf) / (1.0 - calf)));
    }
    return vec3<f32>(
        (width * shape + clearance) * sin(theta),
        (2.0 * v - 1.0) * length * design[LENGTH]
            - design[ANKLE_EXTENSION] * (1.0 - v) * (1.0 - pow4(max(cos(theta), 0.0))),
        (depth * shape + clearance) * cos(theta),
    );
}
"#;

const CUISSE: &str = r#"
const CENTER_RIDGE: u32 = 0u;
const WRAP: u32 = 1u;
const KNEE_TAPER: u32 = 2u;
const LENGTH: u32 = 3u;
const UPPER_EDGE_SLOPE: u32 = 4u;
const CLEARANCE: u32 = 5u;

fn chart_offset(u: f32, v: f32) -> f32 {
    return design[CENTER_RIDGE] * pow8(max(cos((2.0 * u - 1.0) * PI * design[WRAP]), 0.0))
        * pow2(sin(PI * v));
}

fn chart_point(u: f32, v: f32) -> vec3<f32> {
    let width = fit.half_extents.x;
    let length = fit.half_extents.y;
    let depth = fit.half_extents.z;
    let clearance = design[CLEARANCE];
    let theta = (2.0 * u - 1.0) * PI * design[WRAP];
    let taper = lerp(design[KNEE_TAPER], 1.0, smoothstep_unclamped(v));
    // The lower front edge rises slightly at the sides to clear a flexing knee.
    let y = (2.0 * v - 1.0) * length * design[LENGTH]
        + length * 0.09 * abs(sin(theta)) * pow3(1.0 - v)
        + design[UPPER_EDGE_SLOPE] * sin(theta) * pow3(v);
    return vec3<f32>(
        (width * taper + clearance) * sin(theta),
        y,
        (depth * taper + clearance) * cos(theta),
    );
}
"#;

const REREBRACE: &str = r#"
const CENTER_RIDGE: u32 = 0u;
const WRAP: u32 = 1u;
const DISTAL_TAPER: u32 = 2u;
const LENGTH: u32 = 3u;
const SECTION_DEPTH: u32 = 4u;
const CLEARANCE: u32 = 5u;

fn chart_offset(u: f32, v: f32) -> f32 {
    return design[CENTER_RIDGE] * pow8(max(cos((2.0 * u - 1.0) * PI * design[WRAP]), 0.0))
        * pow2(sin(PI * v));
}

fn chart_point(u: f32, v: f32) -> vec3<f32> {
    let width = fit.half_extents.x;
    let length = fit.half_extents.y;
    let depth = fit.half_extents.z;
    let clearance = design[CLEARANCE];
    let theta = (2.0 * u - 1.0) * PI * design[WRAP];
    let taper = lerp(design[DISTAL_TAPER], 1.0, smoothstep_unclamped(v));
    return vec3<f32>(
        (width * taper + clearance) * sin(theta),
        (2.0 * v - 1.0) * length * design[LENGTH] - length * 0.15,
        (depth * design[SECTION_DEPTH] * taper + clearance) * cos(theta),
    );
}
"#;

/// A limb plate's charts and design floats.
pub(crate) struct LimbShape {
    pub part: PartRecipe,
    pub design: Vec<f32>,
}

fn kernel(gpu: &ArmorGpu, shape: &str) -> Result<ChartKernel, GenerateError> {
    ChartKernel::new(gpu, &format!("{AUTHORED_ORIGIN}{shape}"))
}

/// A chart thickened along its normals or radially, with no shell values.
fn plain_chart(
    rows: usize,
    boundary: ChartBoundary,
    thickness: f32,
    extrusion: Extrusion,
    fluting: Option<crate::PlateFluting>,
    span: [f32; 2],
) -> PlateChart {
    PlateChart {
        rows,
        boundary,
        thickness,
        extrusion,
        origin: [0.0; 3],
        axis: [0.0, 1.0, 0.0],
        fluting,
        span,
        values: [0.0; 4],
        mirrored: false,
        frame: 0,
    }
}

fn greave(gpu: &ArmorGpu, d: &GreaveDesign) -> Result<LimbShape, GenerateError> {
    let mut part = PartRecipe::new();
    part.push_chart(
        plain_chart(
            ALONG,
            ChartBoundary::Cyclic,
            d.gauge.thickness.metres(),
            Extrusion::Normal,
            d.fluting,
            [0.0, 1.0],
        ),
        kernel(gpu, GREAVE)?,
    )?;
    Ok(LimbShape {
        part,
        design: vec![
            d.shin_ridge.metres(),
            d.calf_height.unit(),
            d.ankle_taper.unit(),
            d.knee_taper.unit(),
            d.length.unit(),
            d.ankle_extension.metres(),
            d.gauge.clearance.metres() + d.gauge.thickness.metres(),
        ],
    })
}

fn cuisse(gpu: &ArmorGpu, d: &CuisseDesign) -> Result<LimbShape, GenerateError> {
    let mut part = PartRecipe::new();
    part.push_chart(
        plain_chart(
            ALONG,
            ChartBoundary::Open,
            d.gauge.thickness.metres(),
            Extrusion::Normal,
            d.fluting,
            [0.0, 1.0],
        ),
        kernel(gpu, CUISSE)?,
    )?;
    Ok(LimbShape {
        part,
        design: vec![
            d.center_ridge.metres(),
            d.wrap.unit(),
            d.knee_taper.unit(),
            d.length.unit(),
            d.upper_edge_slope.metres(),
            d.gauge.clearance.metres() + d.gauge.thickness.metres(),
        ],
    })
}

fn rerebrace(gpu: &ArmorGpu, d: &RerebraceDesign) -> Result<LimbShape, GenerateError> {
    let mut part = PartRecipe::new();
    part.push_chart(
        plain_chart(
            ALONG,
            ChartBoundary::Open,
            d.gauge.thickness.metres(),
            Extrusion::Radial,
            d.fluting,
            [0.0, 1.0],
        ),
        kernel(gpu, REREBRACE)?,
    )?;
    Ok(LimbShape {
        part,
        design: vec![
            d.center_ridge.metres(),
            d.wrap.unit(),
            d.distal_taper.unit(),
            d.length.unit(),
            d.section_depth.unit(),
            d.gauge.clearance.metres() + d.gauge.thickness.metres(),
        ],
    })
}

fn joint_cup(gpu: &ArmorGpu, d: &JointCupDesign) -> Result<LimbShape, GenerateError> {
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

fn spaulder(gpu: &ArmorGpu, d: &SpaulderDesign) -> Result<LimbShape, GenerateError> {
    let gauge = d.gauge.thickness.metres();
    let lame_kernel = kernel(gpu, &format!("{SPAULDER_DESIGN}{SPAULDER_LAME}"))?;
    let mut part = PartRecipe::new();
    let lames = usize::from(d.lame_count);
    for lame in 0..lames {
        let bottom = lame as f32 / lames as f32;
        let top = ((lame + 1) as f32 / lames as f32 + 0.055).min(1.0);
        let mut chart = plain_chart(
            8,
            ChartBoundary::Open,
            gauge,
            Extrusion::Radial,
            d.fluting,
            [bottom, top],
        );
        chart.values = [bottom, top, 0.0, 0.0];
        part.push_chart(chart, lame_kernel.clone())?;
    }
    part.push_chart(
        PlateChart {
            axis: [0.0, -1.0, 0.0],
            ..plain_chart(
                12,
                ChartBoundary::Apex,
                gauge,
                Extrusion::CappedAxis,
                d.fluting,
                [0.5, 1.0],
            )
        },
        ChartKernel::new(gpu, &format!("{SPAULDER_DESIGN}{SPAULDER_CROWN}"))?,
    )?;
    Ok(LimbShape {
        part,
        design: vec![
            d.wrap.unit(),
            d.rear_extension.unit(),
            d.crown.unit(),
            d.crown_reach.unit(),
            d.lower_flare.metres(),
            d.length.unit(),
            gauge,
            d.gauge.clearance.metres() + gauge,
        ],
    })
}

/// The charts of a limb design, before any fit.
pub(crate) fn shape(gpu: &ArmorGpu, design: &LimbArmorDesign) -> Result<LimbShape, GenerateError> {
    design.validate()?;
    crate::device_support::on_device(design.device_unsupported())?;
    let _ = PI;
    match design {
        LimbArmorDesign::Greave(d) => greave(gpu, d),
        LimbArmorDesign::Cuisse(d) => cuisse(gpu, d),
        LimbArmorDesign::Rerebrace(d) => rerebrace(gpu, d),
        LimbArmorDesign::Poleyn(d) | LimbArmorDesign::Couter(d) => joint_cup(gpu, d),
        LimbArmorDesign::Spaulder(d) => spaulder(gpu, d),
        _ => Err(GenerateError::InvalidSurface),
    }
}

/// Record a limb design's charts, placed by the part frame at the start of
/// `frame`, into a new device part.
pub fn record_limb_armor(
    gpu: &ArmorGpu,
    batch: &mut fabelgeist_compute::KernelBatch,
    design: &LimbArmorDesign,
    frame: &fabelgeist_gpu::prelude::Buffer,
) -> Result<super::DevicePart, GenerateError> {
    let LimbShape { part, design } = shape(gpu, design)?;
    part.record(gpu, batch, &design, &[frame])
}

/// Generate a limb plate on the device, placed by `fit`.
pub fn generate_limb_armor_on(
    gpu: &ArmorGpu,
    design: &LimbArmorDesign,
    fit: &PartFrame,
) -> Result<BuiltPart, GenerateError> {
    let LimbShape { part, design } = shape(gpu, design)?;
    part.build(gpu, &design, fit)
}
