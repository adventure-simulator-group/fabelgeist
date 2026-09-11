//! Shared plate construction for anatomical and standalone gorget carriers.
use crate::{
    BoundaryNormals, GarmentArmorDesign, GenerateError, PartMesh, PlateFluting, ShellExtrusion,
};
use std::f32::consts::TAU;

/// Sweep the posterior trim while keeping every carrier meridian fixed from neck to hem.
pub fn gorget_surface_angle(control: f32, sweep: crate::Permille) -> f32 {
    rear_angle(control, 1.0 - 0.9 * sweep.unit())
}

/// Inverse chart used to sample anatomical sections and maintain uniform angular resolution.
pub fn gorget_control_angle(physical: f32, sweep: crate::Permille) -> f32 {
    rear_angle(physical, 1.0 / (1.0 - 0.9 * sweep.unit()))
}

fn rear_angle(angle: f32, scale: f32) -> f32 {
    if angle.cos() >= 0.0 {
        return angle;
    }
    let rear = angle.rem_euclid(TAU) - std::f32::consts::PI;
    std::f32::consts::PI + (scale * rear.sin()).atan2(rear.cos().max(0.0))
}

/// Construct a shoulder bib and separately thickened overlapping collar lames.
/// Both carrier charts run from the neck toward the chest. Their angular control
/// coordinates share the rear sweep chart; the center is in local X/Z metres.
pub fn generate_gorget_plates(
    design: &GarmentArmorDesign,
    center: [f32; 2],
    collar: impl Fn(f32, f32) -> [f32; 3],
    bib: impl Fn(f32, f32) -> [f32; 3],
) -> Result<PartMesh, GenerateError> {
    design.validate()?;
    let count = usize::from(design.lame_count);
    let gauge = design.wall_thickness.metres();
    let origin = [center[0], 0.0, center[1]];
    const COLLAR_ROWS_FRACTION: f32 = 1.0 / 3.0;
    let collar_start = (count - 1) as f32 / count as f32;
    // The lowest neck band is formed into the bib as one sheet. Its seam is
    // a shared carrier ring, so there is no touching-shell/non-manifold joint.
    let mut mesh = plate(
        design,
        24,
        design.fluting.as_ref(),
        COLLAR_ROWS_FRACTION,
        ShellExtrusion::AngleWeightedNormal,
        |_| 0.0,
        |t, angle| {
            if t <= COLLAR_ROWS_FRACTION {
                collar(
                    collar_start + (1.0 - collar_start) * t / COLLAR_ROWS_FRACTION,
                    angle,
                )
            } else {
                bib(
                    (t - COLLAR_ROWS_FRACTION) / (1.0 - COLLAR_ROWS_FRACTION),
                    angle,
                )
            }
        },
    )?;
    const LAP_FRACTION: f32 = 0.12;
    for lame in 0..count - 1 {
        let start = lame as f32 / count as f32;
        let end = (lame as f32 + 1.0 + LAP_FRACTION) / count as f32;
        mesh.append(plate(
            design,
            8,
            None,
            0.0,
            ShellExtrusion::Radial {
                origin,
                axis: [0.0, 1.0, 0.0],
            },
            |_| gauge * 1.5 * (count - lame - 1) as f32,
            |v, angle| collar(start + (end - start) * v, angle),
        )?);
    }
    Ok(mesh)
}

fn plate(
    design: &GarmentArmorDesign,
    rows: usize,
    fluting: Option<&PlateFluting>,
    fluting_start: f32,
    extrusion: ShellExtrusion,
    offset: impl Fn(f32) -> f32,
    point: impl Fn(f32, f32) -> [f32; 3],
) -> Result<PartMesh, GenerateError> {
    const AROUND: usize = 64;
    // The formed front bib owns its flute chart. The posterior shoulder return
    // stays plain, and changing spread retains every requested flute on the front.
    let mut columns = (0..AROUND)
        .map(|i| i as f32 / AROUND as f32)
        .collect::<Vec<_>>();
    if let crate::GarmentPlateShape::Gorget { rear_sweep, .. } = design.plate_shape {
        columns.extend((0..AROUND).map(|i| {
            let angle = (i as f32 / AROUND as f32 - 0.5) * TAU;
            (gorget_control_angle(angle, rear_sweep) / TAU + 0.5).rem_euclid(1.0)
        }));
        columns.sort_by(f32::total_cmp);
        columns.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    }
    if let Some(pattern) = fluting {
        columns.extend(pattern.columns(AROUND).into_iter().map(|u| 0.25 + 0.5 * u));
        columns.sort_by(f32::total_cmp);
        columns.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    }
    let stride = columns.len();
    let mut positions = Vec::new();
    let mut heights = Vec::new();
    let mut indices = Vec::new();
    for row in 0..=rows {
        let t = row as f32 / rows as f32;
        let pattern_v = 1.0 - ((t - fluting_start) / (1.0 - fluting_start)).max(0.0);
        for u in &columns {
            let front_u = 2.0 * (*u - 0.25);
            let (mapped, relief) =
                fluting
                    .filter(|_| (0.0..=1.0).contains(&front_u))
                    .map_or((*u, 0.0), |p| {
                        (
                            0.25 + 0.5 * p.fan_coordinate(front_u, pattern_v),
                            p.relief(front_u, pattern_v),
                        )
                    });
            positions.push(point(t, (mapped - 0.5) * TAU));
            heights.push(offset(t) + relief);
        }
        if row == 0 {
            continue;
        }
        for col in 0..stride {
            let a = ((row - 1) * stride + col) as u32;
            let b = (row * stride + col) as u32;
            let c = (row * stride + (col + 1) % stride) as u32;
            let d = ((row - 1) * stride + (col + 1) % stride) as u32;
            indices.extend([a, b, c, a, c, d]);
        }
    }
    PartMesh::from_relief_surface(
        positions,
        indices,
        design.wall_thickness.metres(),
        BoundaryNormals::Smooth,
        extrusion,
        Some(heights),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rear_chart_and_inverse_preserve_order_and_side_boundaries() {
        use std::f32::consts::PI;
        for sweep in [0, 500, 850, 1000].map(crate::Permille) {
            let mut previous = 0.0;
            for index in 0..=256 {
                let control = PI * 0.5 + PI * index as f32 / 256.0;
                let physical = gorget_surface_angle(control, sweep);
                let restored = gorget_control_angle(physical, sweep);
                assert!((restored - control).abs() < 1e-5);
                assert!(physical >= previous - 1e-6);
                previous = physical;
                if index == 0 || index == 256 {
                    assert!((physical - control).abs() < 1e-5);
                }
            }
            for front in [-1.0, 0.0, 1.0] {
                assert_eq!(gorget_surface_angle(front, sweep), front);
            }
        }
    }
}
