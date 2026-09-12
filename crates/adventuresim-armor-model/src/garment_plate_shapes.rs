//! Articulated waist plates and a continuous neck-to-chest gorget shell.
use crate::garment_armor::GARMENT_LAME_OVERLAP;
use crate::{
    FauldLameChart, GARMENT_LAME_SPACING_GAUGES, GarmentArmorDesign, GarmentPlateShape,
    GenerateError, PartFrame, PartMesh,
};
use std::f32::consts::TAU;

const ARCH_SPRING_BLEND_START: f32 = 0.75;

/// Retain the round crown while easing its spring into the straight hem.
/// A square-root edge alone has infinite slope, which can reverse facets
/// when neighboring fanned flute rows cross that edge at different angles.
fn fauld_arch(sine: f32, width: f32) -> f32 {
    let across = (sine.abs() / width).min(1.0);
    let blend =
        ((across - ARCH_SPRING_BLEND_START) / (1.0 - ARCH_SPRING_BLEND_START)).clamp(0.0, 1.0);
    let easing = blend * blend * (3.0 - 2.0 * blend);
    (1.0 - across * across).sqrt() * (1.0 - easing)
}

pub(super) fn fauld(d: &GarmentArmorDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    let GarmentPlateShape::Fauld {
        front_arch,
        front_arch_width,
        chevron_slope,
        ..
    } = d.plate_shape
    else {
        return Err(GenerateError::InvalidSurface);
    };
    let [width, height, depth] = fit.half_extents;
    let gauge = d.wall_thickness.metres();
    let count = usize::from(d.lame_count);
    let mut mesh = PartMesh::new();
    for lame in 0..count {
        let chart = FauldLameChart::new(lame, count)?;
        let [bottom, top] = chart.span;
        mesh.append(crate::plate_patch::fluted_lapped_radial_patch(
            &chart.rows,
            gauge,
            d.fluting.as_ref(),
            [bottom, top],
            |u, v| {
                let axial = bottom + (top - bottom) * v;
                let angle = (u - 0.5) * TAU;
                let arch = if angle.cos() > 0.0 {
                    fauld_arch(angle.sin(), front_arch_width.unit())
                } else {
                    0.0
                };
                let radius = 1.0 + d.flare.unit() * (1.0 - axial);
                let padding = d.clearance.metres()
                    + gauge
                    + if count > 1 {
                        gauge * GARMENT_LAME_SPACING_GAUGES * (1.0 - v)
                    } else {
                        0.0
                    };
                [
                    (width * radius + padding) * angle.sin(),
                    height - 2.0 * height * d.length.unit() * (1.0 - axial)
                        + width * chevron_slope.unit() * angle.sin().abs()
                        + height * front_arch.unit() * arch * (1.0 - axial).powi(2),
                    (depth * radius + padding) * angle.cos(),
                ]
            },
        )?);
    }
    Ok(mesh)
}

pub(super) fn gorget(d: &GarmentArmorDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    let GarmentPlateShape::Gorget {
        neck_clearance,
        front_depth,
        back_depth,
        front_width,
        back_width,
        collar_slope,
        collar_height,
        hem_flatness,
        rear_hem_flatness,
        rear_sweep,
    } = d.plate_shape
    else {
        return Err(GenerateError::InvalidSurface);
    };
    let [width, height, depth] = fit.half_extents;
    let padding = d.clearance.metres() + d.wall_thickness.metres();
    let point = |angle: f32, v: f32| {
        let rear = angle.cos() < 0.0;
        let hem_angle = angle;
        let angle = crate::gorget_surface_angle(angle, rear_sweep);
        let flatness = if rear {
            rear_hem_flatness
        } else {
            hem_flatness
        };
        let neck_padding = neck_clearance.metres() + d.wall_thickness.metres();
        let padding = padding + (neck_padding - padding) * (v / 0.60).min(1.0);
        let spread = ((0.60 - v) / 0.60).max(0.0).powi(2);
        let drop = 0.35
            + 2.8 * front_depth.unit() * hem_angle.cos().max(0.0)
            + 0.85 * back_depth.unit() * (-hem_angle.cos()).max(0.0);
        let bib_width = if rear {
            back_width.unit()
        } else {
            front_width.unit()
        };
        let side_width = front_width.unit().min(back_width.unit());
        let outline = bib_width + (side_width - bib_width) * angle.sin().powi(2);
        [
            (width * (1.0 + (0.7 + d.flare.unit()) * outline * spread) + padding) * angle.sin(),
            height
                * d.length.unit()
                * (v * collar_height.unit()
                    * (1.0 - collar_slope.unit() * angle.cos().max(0.0) * 0.2)
                    - (1.0 - v) * drop)
                - height * flatness.unit() * hem_angle.sin().powi(2) * (1.0 - v),
            (depth * (1.0 + 0.85 * spread) + padding) * angle.cos(),
        ]
    };
    const COLLAR_BASE: f32 = 0.60;
    crate::generate_gorget_plates(
        d,
        |t, angle| point(angle, 1.0 - (1.0 - COLLAR_BASE) * t),
        |t, angle| point(angle, COLLAR_BASE * (1.0 - t)),
    )
}

pub(super) fn tassets(d: &GarmentArmorDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    if let GarmentPlateShape::WrappedTassets(shape) = d.plate_shape {
        let [w, h, depth] = fit.half_extents;
        let mut mesh = PartMesh::new();
        let padding = d.clearance.metres() + d.wall_thickness.metres();
        let span = crate::TassetSpan::new(
            h * (1.0 - 2.0 * d.length.unit() * shape.knee_reach.unit()),
            h,
        )?;
        for (side, orientation) in [
            (-1.0, crate::TassetSide::Right),
            (1.0, crate::TassetSide::Left),
        ] {
            mesh.append(crate::generate_wrapped_tasset(
                d,
                orientation,
                span,
                |angle, height| {
                    let axial = span.axial(height);
                    let flare = 1.0 + d.flare.unit() * (1.0 - axial);
                    [
                        side * w * 0.55 + (w * 0.43 * flare + padding) * angle.sin(),
                        (depth * flare + padding) * angle.cos(),
                    ]
                },
            )?);
        }
        return Ok(mesh);
    }
    let GarmentPlateShape::Tassets {
        inner_cutaway,
        hem_point,
        width: scale,
        gap,
        hem_roundness,
    } = d.plate_shape
    else {
        return Err(GenerateError::InvalidSurface);
    };
    let [width, height, depth] = fit.half_extents;
    let gauge = d.wall_thickness.metres();
    let count = usize::from(d.lame_count);
    let mut mesh = PartMesh::new();
    let half_width = width * 0.44 * scale.unit();
    for side in [-1.0, 1.0] {
        for lame in 0..count {
            let top = 1.0 - lame as f32 / count as f32;
            let bottom = 1.0
                - (lame as f32
                    + 1.0
                    + if lame + 1 < count {
                        GARMENT_LAME_OVERLAP
                    } else {
                        0.0
                    })
                    / count as f32;
            mesh.append(crate::plate_patch::fluted_front_plate(
                8,
                gauge,
                d.fluting.as_ref(),
                [bottom, top],
                |_, axial| gauge * 3.0 * (top - axial) / (top - bottom),
                |column, v| {
                    let u = 2.0 * column - 1.0;
                    let descent = 1.0 - (bottom + (top - bottom) * v);
                    let inner = ((1.0 - side * u) * 0.5).powi(3);
                    let upper_cutaway = inner_cutaway.unit() * inner;
                    let span =
                        (1.0 - 2.0 * upper_cutaway) * (1.0 - hem_roundness.unit() * u.powi(8));
                    let shaped_descent = upper_cutaway + span * descent;
                    let point_drop = if lame + 1 == count {
                        hem_point.unit()
                            * (1.0 - (u - side * 0.35).abs() / 1.35).max(0.0)
                            * (1.0 - v)
                    } else {
                        0.0
                    };
                    // Taper follows the trimmed height, preserving the planar
                    // chart's orientation even at a deep inner cutaway.
                    let taper = 1.0 - 0.14 * (shaped_descent + point_drop);
                    [
                        side * (half_width + width * gap.unit() * 0.5) + half_width * taper * u,
                        height - 2.0 * height * d.length.unit() * (shaped_descent + point_drop),
                        depth * (1.0 + d.flare.unit() * descent)
                            + d.clearance.metres()
                            + gauge
                            + width * 0.18 * (1.0 - u * u),
                    ]
                },
            )?);
        }
    }
    Ok(mesh)
}

#[cfg(test)]
mod arch_tests {
    use super::fauld_arch;

    #[test]
    fn arch_keeps_its_round_crown_and_joins_the_hem_without_a_corner() {
        for width in [0.25, 0.6, 0.8] {
            assert_eq!(fauld_arch(0.0, width), 1.0);
            assert_eq!(fauld_arch(width, width), 0.0);
            assert_eq!(fauld_arch(width + 0.01, width), 0.0);
            let mut previous = 1.0;
            for step in 1..=100 {
                let across = step as f32 / 100.0;
                let height = fauld_arch(width * across, width);
                assert!(height.is_finite() && height >= 0.0 && height <= previous);
                assert_eq!(height, fauld_arch(-width * across, width));
                if across <= 0.7 {
                    assert!((height - (1.0 - across * across).sqrt()).abs() < 1e-6);
                }
                previous = height;
            }
            // Both the flat hem and the arch approach zero tangent here.
            let endpoint_slope = fauld_arch(width * 0.999, width) / (width * 0.001);
            assert!(endpoint_slope < 0.01);
        }
    }
}
