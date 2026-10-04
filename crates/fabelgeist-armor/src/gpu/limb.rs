//! Limb plates on the device: rerebraces, spaulders, cuisses, greaves and joint cups as charts.
//!
//! Each shape is a pair of WGSL chart functions over its design floats. The
//! design floats are listed once, in [`LimbShape::design`], and the WGSL
//! constants name the same slots.

use std::f32::consts::PI;

use super::ArmorGpu;
use super::chart::{AROUND, AUTHORED_ORIGIN, ChartBoundary, ChartKernel, PlateChart};
use super::part::Extrusion;
use super::recipe::PartRecipe;
use crate::{
    ArmorComponentRole, CuisseDesign, GenerateError, GreaveDesign, LimbArmorDesign,
    RerebraceDesign, SpaulderDesign,
};

/// Rows along a long plate.
const ALONG: usize = 16;

mod joint_cup;
use joint_cup::joint_cup;

const SPAULDER_DESIGN: &str = r#"
const WRAP: u32 = 0u;
const REAR_EXTENSION: u32 = 1u;
const CROWN: u32 = 2u;
const CROWN_REACH: u32 = 3u;
const LOWER_FLARE: u32 = 4u;
const LENGTH: u32 = 5u;
const GAUGE: u32 = 6u;
const CLEARANCE: u32 = 7u;
const COVERAGE: u32 = 8u;
const RADIUS: u32 = 9u;
const BOSS_HEIGHT: u32 = 10u;
const SHOULDER_DROP: u32 = 11u;
const MEDIAL_OFFSET: u32 = 12u;
const PLATE_CLEARANCE: u32 = 13u;
const OUTWARD_TILT: u32 = 14u;
const FLUTE_COUNT: u32 = 15u;
const FLUTE_WIDTH: u32 = 16u;
const FLUTE_DEPTH: u32 = 17u;
const FLUTE_START: u32 = 18u;
const FLUTE_END: u32 = 19u;
const FLUTE_FADE: u32 = 20u;
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
    // Coverage trims the formed dome toward the neck rather than compressing
    // it into the shoulder.
    let latitude = v * design[COVERAGE] * PI * 0.5;
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

/// A spaulder's besagew: a disc with a central boss and optional radial
/// flutes, hung in front of the shoulder and tilted outward. Rows run from
/// the rim to the boss's apex.
const BESAGEW: &str = r#"
fn besagew_fade(t: f32) -> f32 {
    return smoothstep_clamped(t);
}

// RadialFluting::relief: spokes across the disc, fading in and out radially.
fn chart_offset(u: f32, axial: f32) -> f32 {
    let count = design[FLUTE_COUNT];
    if (count == 0.0) {
        return 0.0;
    }
    let radial = 1.0 - axial;
    let distance = abs(fract(u * count) - 0.5) * 2.0 / design[FLUTE_WIDTH];
    if (distance >= 1.0) {
        return 0.0;
    }
    let fade = design[FLUTE_FADE];
    return design[FLUTE_DEPTH] * (1.0 + cos(PI * distance)) * 0.5
        * besagew_fade((radial - design[FLUTE_START]) / fade)
        * besagew_fade((design[FLUTE_END] - radial) / fade);
}

fn chart_point(u: f32, v: f32) -> vec3<f32> {
    let radial = 1.0 - v;
    let boss = min(radial / 0.28, 1.0);
    let height = design[BOSS_HEIGHT] * (1.0 - boss * boss * (3.0 - 2.0 * boss));
    let angle = TAU * u;
    let disc = vec3<f32>(
        design[RADIUS] * radial * cos(angle),
        design[RADIUS] * radial * sin(angle),
        height,
    );
    let tilt = design[OUTWARD_TILT];
    let origin = vec3<f32>(
        -design[MEDIAL_OFFSET],
        fit.half_extents.y - design[SHOULDER_DROP],
        fit.half_extents.z + design[CLEARANCE] + design[PLATE_CLEARANCE],
    );
    return origin + vec3<f32>(
        cos(tilt) * disc.x + sin(tilt) * disc.z,
        disc.y,
        -sin(tilt) * disc.x + cos(tilt) * disc.z,
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
        around: AROUND,
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
    // A complete crown closes at an apex; a shorter one ends in a thickened
    // neckward edge along the same formed surface.
    let coverage = d.crown_coverage.unit();
    let (boundary, extrusion) = if d.crown_coverage.0 == 1000 {
        (ChartBoundary::Apex, Extrusion::CappedAxis)
    } else {
        (ChartBoundary::Open, Extrusion::Normal)
    };
    part.push_chart(
        PlateChart {
            axis: [0.0, -1.0, 0.0],
            ..plain_chart(
                12,
                boundary,
                gauge,
                extrusion,
                d.fluting,
                [0.5, 0.5 + 0.5 * coverage],
            )
        },
        ChartKernel::new(gpu, &format!("{SPAULDER_DESIGN}{SPAULDER_CROWN}"))?,
    )?;
    if let Some(disc) = &d.besagew {
        const DISC_ROWS: usize = 24;
        const MINIMUM_COLUMNS: usize = 96;
        part.component(ArmorComponentRole::Plate, None);
        let tilt = f32::from(disc.outward_tilt.0) / 1000.0;
        part.push_chart(
            PlateChart {
                axis: [tilt.sin(), 0.0, tilt.cos()],
                around: disc
                    .fluting
                    .map_or(MINIMUM_COLUMNS, |f| f.columns().max(MINIMUM_COLUMNS)),
                ..plain_chart(
                    DISC_ROWS,
                    ChartBoundary::CyclicApex,
                    gauge,
                    Extrusion::Along,
                    None,
                    [0.0, 1.0],
                )
            },
            kernel(gpu, &format!("{SPAULDER_DESIGN}{BESAGEW}"))?,
        )?;
        part.component(ArmorComponentRole::Besagew, None);
    }
    let disc = d.besagew.as_ref();
    let flutes = disc.and_then(|disc| disc.fluting);
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
            coverage,
            disc.map_or(0.0, |disc| disc.radius.metres()),
            disc.map_or(0.0, |disc| disc.boss_height.metres()),
            disc.map_or(0.0, |disc| disc.shoulder_drop.metres()),
            disc.map_or(0.0, |disc| disc.medial_offset.metres()),
            disc.map_or(0.0, |disc| disc.plate_clearance.metres()),
            disc.map_or(0.0, |disc| f32::from(disc.outward_tilt.0) / 1000.0),
            flutes.map_or(0.0, |f| f32::from(f.count.0)),
            flutes.map_or(1.0, |f| f.width.unit()),
            flutes.map_or(0.0, |f| f.depth.metres()),
            flutes.map_or(0.0, |f| f.start.unit()),
            flutes.map_or(1.0, |f| f.end.unit()),
            flutes.map_or(1.0, |f| f.fade.unit()),
        ],
    })
}

/// The charts of a limb design, before any fit.
pub(crate) fn shape(gpu: &ArmorGpu, design: &LimbArmorDesign) -> Result<LimbShape, GenerateError> {
    design.validate()?;
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
