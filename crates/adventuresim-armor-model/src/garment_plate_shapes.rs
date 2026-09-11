//! Articulated waist plates and a continuous neck-to-chest gorget shell.
use crate::{GarmentArmorDesign, GarmentPlateShape, GenerateError, PartFrame, PartMesh};
use std::f32::consts::TAU;

const LAME_OVERLAP: f32 = 0.12;
const LAME_SPACING_GAUGES: f32 = 2.5;

pub(super) fn fauld(d: &GarmentArmorDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    let GarmentPlateShape::Fauld { front_arch, .. } = d.plate_shape else {
        return Err(GenerateError::InvalidSurface);
    };
    let [width, height, depth] = fit.half_extents;
    let gauge = d.wall_thickness.metres();
    let count = usize::from(d.lame_count);
    let mut mesh = PartMesh::new();
    for lame in 0..count {
        let top = 1.0 - lame as f32 / count as f32;
        let bottom = (1.0
            - (lame + 1) as f32 / count as f32
            - if lame + 1 < count {
                LAME_OVERLAP / count as f32
            } else {
                0.0
            })
        .max(0.0);
        mesh.append(crate::plate_patch::fluted_radial_patch(
            16,
            true,
            gauge,
            d.fluting.as_ref(),
            [bottom, top],
            |_, _| 0.0,
            |u, v| {
                let axial = bottom + (top - bottom) * v;
                let angle = (u - 0.5) * TAU;
                let radius = 1.0 + d.flare.unit() * (1.0 - axial);
                let padding = d.clearance.metres()
                    + gauge
                    + if count > 1 {
                        gauge * LAME_SPACING_GAUGES * (1.0 - v)
                    } else {
                        0.0
                    };
                [
                    (width * radius + padding) * angle.sin(),
                    height - 2.0 * height * d.length.unit() * (1.0 - axial)
                        + height
                            * front_arch.unit()
                            * angle.cos().max(0.0).powi(4)
                            * (1.0 - axial).powi(4),
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
        [0.0; 2],
        |t, angle| point(angle, 1.0 - (1.0 - COLLAR_BASE) * t),
        |t, angle| point(angle, COLLAR_BASE * (1.0 - t)),
    )
}

pub(super) fn tassets(d: &GarmentArmorDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
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
                - (lame as f32 + 1.0 + if lame + 1 < count { LAME_OVERLAP } else { 0.0 })
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
