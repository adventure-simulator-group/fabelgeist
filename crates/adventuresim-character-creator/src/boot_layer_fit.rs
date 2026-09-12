//! Boot shaft clearance over the supported leg garments.
use super::boot_shaft_profile::{self, RING_PLANE_TOLERANCE_M};
use super::{PROFILE_SAMPLES, PROFILE_WINDOW_M, foot_exponent, foot_sections, local};
use crate::armor_frames::{Side, Wearer};
use adventuresim_armor_model::{PartFrame, PartMesh, PlateGauge};
use anyhow::Result;

const BOOT_HEM_TRANSITION_M: f32 = 0.04;
const ANKLE_FAIRING_BELOW_HEM_M: f32 = 0.05;
const ANKLE_FAIRING_ABOVE_HEM_M: f32 = 0.08;
const ANKLE_FAIRING_PASSES: usize = 64;

/// The boot shaft encloses either supported leg garment. Its foot and sole
/// retain the anatomical fit outside the continued lower-shaft envelope.
pub(super) fn fit(
    mesh: PartMesh,
    gauge: PlateGauge,
    wearer: &Wearer<'_>,
    side: Side,
    frame: &PartFrame,
) -> Result<PartMesh> {
    let low = mesh
        .positions
        .iter()
        .map(|p| local(frame, *p)[1])
        .fold(f32::INFINITY, f32::min);
    let high = mesh
        .positions
        .iter()
        .map(|p| local(frame, *p)[1])
        .fold(f32::NEG_INFINITY, f32::max);
    use adventuresim_armor_model::{GarmentArmorDesign, GarmentArmorKind};
    let placement = if matches!(side, Side::Left) {
        "left"
    } else {
        "right"
    };
    let mut support = Vec::new();
    let mut hem = f32::INFINITY;
    for kind in [
        GarmentArmorKind::MailChausses,
        GarmentArmorKind::PaddedChausses,
    ] {
        let garment = crate::garment_fit::fitted_garment(
            &GarmentArmorDesign::new(kind),
            placement,
            wearer,
            &[],
        )?;
        let points = garment
            .positions
            .iter()
            .map(|p| local(frame, *p))
            .collect::<Vec<_>>();
        let garment_hem = points.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
        hem = hem.min(garment_hem);
        support.extend(boot_layer_sections(
            &points,
            &garment.indices,
            low,
            high,
            garment_hem,
        ));
    }
    let dressed = foot_sections(&support, low, high, gauge, frame.half_extents[1]);
    refit_supported_shaft(mesh, frame, low..high, hem, &dressed)
}

fn refit_supported_shaft(
    mesh: PartMesh,
    frame: &PartFrame,
    height: std::ops::Range<f32>,
    hem: f32,
    dressed: &[super::FootSection],
) -> Result<PartMesh> {
    let mut shaft_error = None;
    let result = mesh.refit_surfaces(|carrier, _| {
        seat_on_layers(carrier.iter_mut(), frame, &height, hem, dressed);
        fair_ankle(carrier, frame, hem, dressed[PROFILE_SAMPLES - 1].center);
        if let Err(error) =
            boot_shaft_profile::regularize(carrier, frame, hem, ANKLE_FAIRING_ABOVE_HEM_M)
        {
            shaft_error = Some(error);
            return;
        }
        // Coordinate redistribution is followed by the authoritative support
        // projection, including nonelliptical intermediate ankle sections.
        seat_on_layers(
            carrier.iter_mut().filter(|p| local(frame, **p)[1] > hem),
            frame,
            &height,
            hem,
            dressed,
        );
    })?;
    if let Some(error) = shaft_error {
        return Err(error.into());
    }
    Ok(result)
}

fn seat_on_layers<'a>(
    carrier: impl Iterator<Item = &'a mut [f32; 3]>,
    frame: &PartFrame,
    height: &std::ops::Range<f32>,
    hem: f32,
    dressed: &[super::FootSection],
) {
    for point in carrier {
        let mut p = local(frame, *point);
        let t = ((p[1] - hem + BOOT_HEM_TRANSITION_M) / BOOT_HEM_TRANSITION_M).clamp(0.0, 1.0);
        if t == 0.0 {
            continue;
        }
        let axial = ((p[1] - height.start) / (height.end - height.start)
            * (PROFILE_SAMPLES - 1) as f32)
            .clamp(0.0, (PROFILE_SAMPLES - 1) as f32);
        let index = (axial.floor() as usize).min(PROFILE_SAMPLES - 2);
        let fraction = axial - index as f32;
        let center: [f32; 2] = std::array::from_fn(|axis| {
            dressed[index].center[axis]
                + (dressed[index + 1].center[axis] - dressed[index].center[axis]) * fraction
        });
        let radius: [f32; 2] = std::array::from_fn(|axis| {
            dressed[index].radius[axis]
                + (dressed[index + 1].radius[axis] - dressed[index].radius[axis]) * fraction
        });
        let delta = [p[0] - center[0], p[2] - center[1]];
        let exponent = foot_exponent(p[1], frame.half_extents[1]);
        let normalized = ((delta[0] / radius[0]).abs().powf(exponent)
            + (delta[1] / radius[1]).abs().powf(exponent))
        .powf(exponent.recip());
        if normalized > f32::EPSILON && normalized < 1.0 {
            let expansion = (normalized.recip() - 1.0) * t * t * (3.0 - 2.0 * t);
            p[0] += delta[0] * expansion;
            p[2] += delta[1] * expansion;
            *point = frame.point(p);
        }
    }
}

/// Relax ankle valleys outward along the authored meridians. Fixed foot/shaft
/// anchors and the original radii prevent the fitted radial envelope contracting.
fn fair_ankle(carrier: &mut [[f32; 3]], frame: &PartFrame, hem: f32, center: [f32; 2]) {
    let points = carrier.iter().map(|p| local(frame, *p)).collect::<Vec<_>>();
    let ring_size = points
        .iter()
        .take_while(|p| (p[1] - points[0][1]).abs() < RING_PLANE_TOLERANCE_M)
        .count();
    let ring_count = points.len() / ring_size;
    let mut radii = points
        .iter()
        .map(|p| (p[0] - center[0]).hypot(p[2] - center[1]))
        .collect::<Vec<_>>();
    for _ in 0..ANKLE_FAIRING_PASSES {
        let previous = radii.clone();
        for row in 1..ring_count - 1 {
            for col in 0..ring_size {
                let i = row * ring_size + col;
                if points[i][1] > hem - ANKLE_FAIRING_BELOW_HEM_M
                    && points[i][1] < hem + ANKLE_FAIRING_ABOVE_HEM_M
                {
                    radii[i] =
                        radii[i].max((previous[i - ring_size] + previous[i + ring_size]) * 0.5);
                }
            }
        }
    }
    for (i, point) in carrier.iter_mut().enumerate() {
        let mut p = points[i];
        let radius = (p[0] - center[0]).hypot(p[2] - center[1]);
        if radius > f32::EPSILON && radii[i] > radius {
            let scale = radii[i] / radius;
            p[0] = center[0] + (p[0] - center[0]) * scale;
            p[2] = center[1] + (p[2] - center[1]) * scale;
            *point = frame.point(p);
        }
    }
}

/// Intersect complete garment triangles: a narrow vertex band around a tilted
/// ring only sees an arc, whose changing bounds would corrugate the boot shaft.
fn boot_layer_sections(
    points: &[[f32; 3]],
    indices: &[u32],
    low: f32,
    high: f32,
    hem: f32,
) -> Vec<[f32; 3]> {
    let mut support = Vec::new();
    for index in 0..PROFILE_SAMPLES {
        let height = low + (high - low) * index as f32 / (PROFILE_SAMPLES - 1) as f32;
        // Continue a complete low-shaft contour into the foot. Fading its
        // radius to zero below the hem leaves a rigid projecting cuff shelf.
        let plane = height.max(hem + PROFILE_WINDOW_M);
        for triangle in indices.as_chunks::<3>().0 {
            for edge in 0..3 {
                let a = points[triangle[edge] as usize];
                let b = points[triangle[(edge + 1) % 3] as usize];
                if (a[1] <= plane && b[1] > plane) || (b[1] <= plane && a[1] > plane) {
                    let t = (plane - a[1]) / (b[1] - a[1]);
                    support.push([a[0] + (b[0] - a[0]) * t, height, a[2] + (b[2] - a[2]) * t]);
                }
            }
        }
    }
    support
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redistributed_shaft_is_projected_back_to_support_without_reseating_the_foot() {
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.1, 0.05, 0.08],
        };
        let dressed: Vec<_> = (0..PROFILE_SAMPLES)
            .map(|_| super::super::FootSection {
                center: [0.03, -0.02],
                radius: [0.095, 0.075],
            })
            .collect();
        let mut points = [[0.02, -0.01, 0.0], [0.10268, 0.04, -0.05565]];
        let foot = points[0];
        seat_on_layers(
            points.iter_mut().filter(|p| p[1] > 0.0),
            &frame,
            &(-0.01..0.09),
            0.0,
            &dressed,
        );
        assert_eq!(points[0], foot);
        let radius = ((points[1][0] - 0.03) / 0.095).hypot((points[1][2] + 0.02) / 0.075);
        assert!((radius - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ankle_fairing_fills_a_valley_outward_without_moving_its_anchors() {
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [1.0; 3],
        };
        let mut points = Vec::new();
        for (row, radius) in [0.10, 0.04, 0.08, 0.07, 0.07].into_iter().enumerate() {
            for col in 0..8 {
                let angle = std::f32::consts::TAU * col as f32 / 8.0;
                points.push([
                    radius * angle.sin(),
                    row as f32 * 0.04,
                    radius * angle.cos(),
                ]);
            }
        }
        let original = points.clone();
        fair_ankle(&mut points, &frame, 0.07, [0.0; 2]);
        assert_eq!(&points[..8], &original[..8]);
        assert_eq!(&points[32..], &original[32..]);
        assert!(points[8][0].hypot(points[8][2]) > 0.08);
        for (point, before) in points.iter().zip(&original) {
            assert!(point[0].hypot(point[2]) + f32::EPSILON >= before[0].hypot(before[2]));
        }
    }

    #[test]
    fn complete_triangle_sections_cover_a_tilted_ring_between_its_vertices() {
        let points = [
            [-1.0, -2.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, -1.0, 1.0],
            [-1.0, -2.0, 1.0],
            [-1.0, 1.0, -1.0],
            [1.0, 2.0, -1.0],
            [1.0, 2.0, 1.0],
            [-1.0, 1.0, 1.0],
        ];
        let mut indices = Vec::new();
        for corner in 0..4 {
            let next = (corner + 1) % 4;
            indices.extend([corner, next, corner + 4, next, next + 4, corner + 4]);
        }
        let support = boot_layer_sections(&points, &indices, 0.0, 0.0, -2.0);
        for axis in [0, 2] {
            assert_eq!(
                support
                    .iter()
                    .map(|p| p[axis])
                    .fold(f32::INFINITY, f32::min),
                -1.0
            );
            assert_eq!(
                support
                    .iter()
                    .map(|p| p[axis])
                    .fold(f32::NEG_INFINITY, f32::max),
                1.0
            );
        }
        assert!(support.iter().all(|p| p[1] == 0.0));
    }
}
