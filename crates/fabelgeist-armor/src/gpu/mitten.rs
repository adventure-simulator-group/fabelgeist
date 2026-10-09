//! The mitten gauntlet on the device: its flared cuff and finger lames in
//! the hand frame, and its thumb plate in the thumb's own frame.

use super::ArmorGpu;
use super::chart::{AUTHORED_ORIGIN, ChartBoundary, PlateChart};
use super::extremities::{ExtremityShape, chart, chart_kernel};
use super::part::Extrusion;
use super::recipe::PartRecipe;
use crate::{GauntletDesign, GenerateError};

const GAUNTLET_DESIGN: &str = r#"
const GAUGE: u32 = 0u;
const CLEARANCE: u32 = 1u;
const CUFF_CLEARANCE: u32 = 2u;
const CUFF_FLARE: u32 = 3u;
const CUFF_LENGTH: u32 = 4u;
const KNUCKLE_WIDTH: u32 = 5u;
const KNUCKLE_RIDGE: u32 = 6u;
"#;

/// The flared cuff, closed around the wrist.
const GAUNTLET_CUFF: &str = r#"
fn chart_offset(u: f32, axial: f32) -> f32 {
    return design[GAUGE] * 2.0;
}

fn chart_point(u: f32, v: f32) -> vec3<f32> {
    let width = fit.half_extents.x;
    let length = fit.half_extents.y;
    let depth = fit.half_extents.z;
    let clearance = design[CLEARANCE];
    let cuff = design[CUFF_CLEARANCE];
    let theta = TAU * (u - 0.5);
    let radius = lerp(0.73, 0.73 * design[CUFF_FLARE], smoothstep_unclamped(v));
    return vec3<f32>(
        (width * radius + clearance + cuff) * sin(theta),
        length * (0.94 + v * design[CUFF_LENGTH] * 2.0),
        (depth * lerp(1.0, 1.35, v) + clearance + cuff) * cos(theta),
    );
}
"#;

/// One finger lame; `value0` and `value1` are its axial span, and the chart
/// runs from its upper edge down, so its lap relief grows toward the knuckle.
const GAUNTLET_LAME: &str = r#"
fn chart_offset(u: f32, axial: f32) -> f32 {
    let low = params.value0;
    let high = params.value1;
    return design[GAUGE] * 2.0 * (axial - (1.0 - high)) / (high - low);
}

fn chart_point(u: f32, v: f32) -> vec3<f32> {
    let width = fit.half_extents.x;
    let length = fit.half_extents.y;
    let depth = fit.half_extents.z;
    let clearance = design[CLEARANCE];
    let knuckle = design[KNUCKLE_WIDTH];
    let axial = lerp(params.value0, params.value1, v);
    let theta = (2.0 * u - 1.0) * PI * 0.5;
    var taper: f32;
    if (axial < 0.55) {
        taper = lerp(0.84, knuckle, smoothstep_unclamped(axial / 0.55));
    } else {
        taper = lerp(knuckle, 0.73, smoothstep_unclamped((axial - 0.55) / 0.45));
    }
    return vec3<f32>(
        (width * taper + clearance) * sin(theta),
        length * lerp(-0.81, 1.04, axial),
        (depth + clearance) * cos(theta)
            + design[KNUCKLE_RIDGE] * pow2(max(1.0 - abs((axial - 0.55) / 0.18), 0.0))
                * max(cos(theta), 0.0),
    );
}
"#;

/// The distal thumb plate, in the thumb's own frame.
const GAUNTLET_THUMB: &str = r#"
fn chart_offset(u: f32, axial: f32) -> f32 {
    return 0.0;
}

// The palm-facing boundary stays beneath the mitten's side plates, and the
// plate ends before the mitten's metacarpal coverage.
fn chart_point(u: f32, v: f32) -> vec3<f32> {
    let width = fit.half_extents.x;
    let length = fit.half_extents.y;
    let depth = fit.half_extents.z;
    let clearance = design[CLEARANCE];
    let theta = (2.0 * u - 1.0) * PI * 0.30;
    return vec3<f32>(
        (width + clearance) * sin(theta),
        length * lerp(-0.68, 0.30, v),
        (depth + clearance) * cos(theta),
    );
}
"#;

/// The mitten in the hand frame (frame 0) and its thumb in the thumb frame
/// (frame 1). The cuff is the first shell and the thumb the last.
pub(super) fn gauntlet(
    gpu: &ArmorGpu,
    d: &GauntletDesign,
) -> Result<ExtremityShape, GenerateError> {
    let gauge = d.gauge.thickness.metres();
    let authored = |shape: &str| format!("{AUTHORED_ORIGIN}{shape}");
    let mut part = PartRecipe::new();
    part.push_chart(
        chart(
            12,
            ChartBoundary::Cyclic,
            gauge,
            Extrusion::Radial,
            d.fluting,
            [0.0, 1.0],
        ),
        chart_kernel(gpu, GAUNTLET_DESIGN, &authored(GAUNTLET_CUFF))?,
    )?;
    let lame_kernel = chart_kernel(gpu, GAUNTLET_DESIGN, &authored(GAUNTLET_LAME))?;
    let count = usize::from(d.finger_lames);
    for lame in 0..count {
        let low = lame as f32 / count as f32;
        let high = ((lame + 1) as f32 / count as f32 + 0.045).min(1.0);
        let span = [1.0 - low, 1.0 - high];
        // The fingertip lame closes over the fingers in a rounded tip.
        let mut lame_chart = if lame == 0 {
            chart(
                8,
                ChartBoundary::CappedByFrame {
                    per_half_height: 0.29,
                    fixed: 0.0,
                },
                gauge,
                Extrusion::CappedAxis,
                d.fluting,
                span,
            )
        } else {
            chart(
                8,
                ChartBoundary::Open,
                gauge,
                Extrusion::Radial,
                d.fluting,
                span,
            )
        };
        lame_chart.values = [low, high, 0.0, 0.0];
        part.push_chart(lame_chart, lame_kernel.clone())?;
    }
    let clearance = d.gauge.clearance.metres() + gauge;
    part.push_chart(
        PlateChart {
            frame: 1,
            ..chart(
                10,
                ChartBoundary::CappedByFrame {
                    per_half_height: 0.48,
                    fixed: clearance,
                },
                gauge,
                Extrusion::CappedAxis,
                None,
                [0.0, 1.0],
            )
        },
        chart_kernel(gpu, GAUNTLET_DESIGN, &authored(GAUNTLET_THUMB))?,
    )?;
    Ok(ExtremityShape {
        part,
        design: vec![
            gauge,
            clearance,
            d.cuff_clearance.metres(),
            d.cuff_flare.unit(),
            d.cuff_length.unit(),
            d.knuckle_width.unit(),
            d.knuckle_ridge.metres(),
        ],
    })
}
