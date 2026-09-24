//! Footwear on the device: sabaton lames and toe cap, and the leather boot.

use std::f32::consts::{PI, TAU};

use super::ArmorGpu;
use super::chart::ChartBoundary;
use super::coord::{self, CoordExtrusion, CoordKernel, CoordShell};
use super::extremities::{ExtremityShape, chart, chart_kernel};
use super::part::Extrusion;
use super::recipe::PartRecipe;
use crate::{BootDesign, BoundaryNormals, FootArmorDesign, GenerateError};

/// Columns around the boot and the toe cap.
const AROUND: usize = 40;
/// Rows from the boot's sole perimeter to its shaft opening.
const BOOT_ROWS: usize = 40;
/// Latitude rings of the sabaton's toe cap, its tip excluded.
const TOE_RINGS: usize = 8;

const FOOT_DESIGN: &str = r#"
const GAUGE: u32 = 0u;
const CLEARANCE: u32 = 1u;
const INSTEP_HEIGHT: u32 = 2u;
const TOE_WIDTH: u32 = 3u;
const ANKLE_CUTAWAY: u32 = 4u;
const TOE_EXTENSION: u32 = 5u;
const TOE_ROUNDNESS: u32 = 6u;
const SHAFT_HEIGHT: u32 = 7u;
const SHAFT_FLARE: u32 = 8u;

fn sole() -> f32 {
    return -fit.half_extents.y + design[GAUGE];
}
"#;

/// One transverse instep lame; `value0` and `value1` are its span from the
/// ankle toward the toe. It thickens radially about the sole's long axis.
const SABATON_LAME: &str = r#"
fn chart_origin() -> vec3<f32> {
    return vec3<f32>(0.0, sole(), 0.0);
}

fn chart_offset(u: f32, axial: f32) -> f32 {
    let low = params.value0;
    let high = params.value1;
    return design[GAUGE] * 2.0 * (axial - low) / (high - low);
}

fn foot_width(t: f32) -> f32 {
    if (t < 0.55) {
        return lerp(0.63, 1.0, smoothstep_unclamped(t / 0.55));
    }
    return lerp(1.0, design[TOE_WIDTH], smoothstep_unclamped((t - 0.55) / 0.45));
}

fn chart_point(u: f32, v: f32) -> vec3<f32> {
    let width = fit.half_extents.x;
    let height = fit.half_extents.y;
    let length = fit.half_extents.z;
    let t = lerp(params.value0, params.value1, v);
    let theta = (0.5 - u) * PI;
    let arch = height * design[INSTEP_HEIGHT] * lerp(1.75, 1.1, smoothstep_unclamped(t));
    return vec3<f32>(
        (width * foot_width(t) + design[CLEARANCE]) * sin(theta),
        -height + design[GAUGE] + arch * cos(theta),
        lerp(-length * 0.05 + design[ANKLE_CUTAWAY], length * 0.65, t),
    );
}
"#;

/// The toe cap: a half ellipsoid with a fan tip. Coordinates are the
/// column's `sin` and `cos` and the ring's latitude `cos` and `sin`; the
/// last vertex is the tip.
const SABATON_TOE: &str = r#"
fn shell_vertex(index: u32, coord: vec4<f32>) -> ShellVertex {
    let width = fit.half_extents.x;
    let height = fit.half_extents.y;
    let length = fit.half_extents.z;
    let end = length + design[TOE_EXTENSION];
    if (index + 1u == params.total) {
        return ShellVertex(vec3<f32>(0.0, sole(), end), 0.0);
    }
    let cap_width = width * design[TOE_WIDTH] + design[CLEARANCE];
    let cap_height = height * 1.1 * design[INSTEP_HEIGHT];
    return ShellVertex(
        vec3<f32>(
            cap_width * coord.x * pow(coord.z, design[TOE_ROUNDNESS]),
            sole() + cap_height * coord.y * coord.z,
            lerp(length * 0.60, end, coord.w),
        ),
        0.0,
    );
}
"#;

/// The leather boot: one carrier from the sole perimeter through the vamp to
/// the shaft opening, its sole closed by a fan about the perimeter's mean.
/// Coordinates are the column's `sin` and `cos` and their rounded powers;
/// the row comes from the index.
fn boot_shape() -> String {
    format!(
        r#"
const COLUMNS: u32 = {AROUND}u;
const ROWS: u32 = {BOOT_ROWS}u;

fn boot_point(coord: vec4<f32>, v: f32) -> vec3<f32> {{
    let width = fit.half_extents.x;
    let height = fit.half_extents.y;
    let length = fit.half_extents.z;
    let gauge = design[GAUGE];
    let clearance = design[CLEARANCE];
    let ankle_z = -length * 0.45;
    let ankle_width = width * 0.69 + clearance;
    let ankle_depth = length * 0.35 + clearance;
    let upper_height = height * 1.65 + clearance + gauge;
    let shaft_height = design[SHAFT_HEIGHT];
    let total_height = upper_height + shaft_height;
    let sine = coord.x;
    let cosine = coord.y;
    let toe_factor = lerp(0.72, design[TOE_WIDTH], (cosine + 1.0) * 0.5);
    let outer_x = (width * toe_factor + clearance) * coord.z;
    let outer_z = (length + clearance) * coord.w;
    let inner_x = ankle_width * sine;
    let inner_z = ankle_z + ankle_depth * cosine;
    let rise = total_height * v;
    let transition = smoothstep_unclamped(min(rise / upper_height, 1.0));
    let shaft = max((rise - upper_height) / shaft_height, 0.0);
    let flare = lerp(1.0, design[SHAFT_FLARE], smoothstep_unclamped(shaft));
    return vec3<f32>(
        lerp(outer_x, inner_x * flare, transition),
        -height - gauge + rise,
        lerp(outer_z, ankle_z + (inner_z - ankle_z) * flare, transition),
    );
}}

fn shell_vertex(index: u32, coord: vec4<f32>) -> ShellVertex {{
    if (index == COLUMNS * (ROWS + 1u)) {{
        // The sole centre: the mean of the perimeter row, summed in order.
        var sum = vec3<f32>(0.0);
        for (var column = 0u; column < COLUMNS; column = column + 1u) {{
            sum = sum + boot_point(coords[column], 0.0);
        }}
        return ShellVertex(sum / f32(COLUMNS), 0.0);
    }}
    let row = index / COLUMNS;
    return ShellVertex(boot_point(coord, f32(row) / f32(ROWS)), 0.0);
}}
"#
    )
}

/// Design floats shared by the foot shapes, in `FOOT_DESIGN`'s slots.
fn foot_design(gauge: crate::PlateGauge, shape: [f32; 7]) -> Vec<f32> {
    let thickness = gauge.thickness.metres();
    let mut design = vec![thickness, gauge.clearance.metres() + thickness];
    design.extend(shape);
    design
}

/// The instep lames, then the toe cap as the last shell.
pub(super) fn sabaton(
    gpu: &ArmorGpu,
    d: &FootArmorDesign,
) -> Result<ExtremityShape, GenerateError> {
    let gauge = d.gauge.thickness.metres();
    let lame_kernel = chart_kernel(gpu, FOOT_DESIGN, SABATON_LAME)?;
    let mut part = PartRecipe::new();
    let count = usize::from(d.lame_count);
    for lame in 0..count {
        let low = lame as f32 / count as f32;
        let high = ((lame + 1) as f32 / count as f32 + 0.080).min(1.0);
        let mut lame_chart = chart(
            8,
            ChartBoundary::Open,
            gauge,
            Extrusion::Radial,
            d.fluting,
            [low, high],
        );
        lame_chart.axis = [0.0, 0.0, 1.0];
        lame_chart.values = [low, high, 0.0, 0.0];
        part.push_chart(lame_chart, lame_kernel.clone())?;
    }
    part.push_coord(
        toe_cap(gauge),
        CoordKernel::new(
            gpu,
            &format!("{}{FOOT_DESIGN}{SABATON_TOE}", coord::AUTHORED),
        )?,
    )?;
    Ok(ExtremityShape {
        part,
        design: foot_design(
            d.gauge,
            [
                d.instep_height.unit(),
                d.toe_width.unit(),
                d.ankle_cutaway.metres(),
                d.toe_extension.metres(),
                d.toe_roundness.unit(),
                0.0,
                0.0,
            ],
        ),
    })
}

/// The toe cap as a half dome: latitude rings over a half turn of columns,
/// closed by a fan to its tip.
fn toe_cap(gauge: f32) -> CoordShell {
    let stride = AROUND + 1;
    let mut coords = Vec::with_capacity(TOE_RINGS * stride + 1);
    for row in 0..TOE_RINGS {
        let latitude = row as f32 / TOE_RINGS as f32 * PI * 0.5;
        for column in 0..=AROUND {
            let theta = (0.5 - column as f32 / AROUND as f32) * PI;
            coords.push([theta.sin(), theta.cos(), latitude.cos(), latitude.sin()]);
        }
    }
    let mut indices = Vec::new();
    for row in 0..TOE_RINGS - 1 {
        for column in 0..AROUND {
            let a = (row * stride + column) as u32;
            let b = a + 1;
            let c = a + stride as u32;
            let e = c + 1;
            indices.extend_from_slice(&[a, b, e, a, e, c]);
        }
    }
    let tip = coords.len() as u32;
    coords.push([0.0; 4]);
    for column in 0..AROUND {
        let a = ((TOE_RINGS - 1) * stride + column) as u32;
        indices.extend_from_slice(&[a, a + 1, tip]);
    }
    CoordShell {
        coords,
        indices,
        boundary_normals: BoundaryNormals::Smooth,
        thickness: gauge,
        extrusion: CoordExtrusion::Normal,
        values: [0.0; 4],
        mirrored: false,
        frame: 0,
        passes: 1,
        hinge: None,
    }
}

pub(super) fn boot(gpu: &ArmorGpu, d: &BootDesign) -> Result<ExtremityShape, GenerateError> {
    let rounded = |value: f32| value.signum() * value.abs().powf(0.55);
    let mut coords = Vec::with_capacity(AROUND * (BOOT_ROWS + 1) + 1);
    for _ in 0..=BOOT_ROWS {
        for column in 0..AROUND {
            let theta = TAU * (column as f32 / AROUND as f32);
            let (sine, cosine) = (theta.sin(), theta.cos());
            coords.push([sine, cosine, rounded(sine), rounded(cosine)]);
        }
    }
    let mut indices = Vec::with_capacity(AROUND * BOOT_ROWS * 6 + AROUND * 3);
    for row in 0..BOOT_ROWS {
        for column in 0..AROUND {
            let next = (column + 1) % AROUND;
            let a = (row * AROUND + column) as u32;
            let b = (row * AROUND + next) as u32;
            let c = ((row + 1) * AROUND + column) as u32;
            let e = ((row + 1) * AROUND + next) as u32;
            indices.extend_from_slice(&[a, b, e, a, e, c]);
        }
    }
    let center = coords.len() as u32;
    coords.push([0.0; 4]);
    for index in 0..AROUND {
        indices.extend([center, ((index + 1) % AROUND) as u32, index as u32]);
    }
    let mut part = PartRecipe::new();
    part.push_coord(
        CoordShell {
            coords,
            indices,
            boundary_normals: BoundaryNormals::Smooth,
            thickness: d.gauge.thickness.metres(),
            extrusion: CoordExtrusion::Normal,
            values: [0.0; 4],
            mirrored: false,
            frame: 0,
            passes: 1,
            hinge: None,
        },
        CoordKernel::new(
            gpu,
            &format!("{}{FOOT_DESIGN}{}", coord::AUTHORED, boot_shape()),
        )?,
    )?;
    Ok(ExtremityShape {
        part,
        design: foot_design(
            d.gauge,
            [
                0.0,
                d.toe_width.unit(),
                0.0,
                0.0,
                0.0,
                d.shaft_height.metres(),
                d.shaft_flare.unit(),
            ],
        ),
    })
}
