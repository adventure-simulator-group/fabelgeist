//! Fit the crown to its shoulder without shortening the shoulder with the arm.
//!
//! Requires the canonical unposed MHR body. The rig-derived upper-arm axis is
//! the crown's proximal direction; its transverse projection is fixed. Only
//! the crown's span from its authored rim can increase. Every iteration rebuilds
//! relief and wall gauge, then checks the actual inner facets against complete
//! owned shoulder triangles, including support between mesh samples.
use adventuresim_armor_model::{PartFrame, PartMesh, SpaulderDesign, SpaulderPlates};
use anyhow::{Result, ensure};
use bevy::math::{Vec2, Vec3};

use crate::armor_frames::{FitRegion, Side, Wearer};

const SUPPORT_TOLERANCE_M: f32 = 0.000001;
const PROJECTED_AREA_EPSILON_M2: f32 = 1e-12;
const MAXIMUM_SEATING_PASSES: usize = 16;
const LAYER_SUPPORT_ENVELOPE_SCALE: f32 = 2.0;

pub(crate) fn fit(
    design: &SpaulderDesign,
    wearer: &Wearer<'_>,
    side: Side,
    layers: &[crate::armor_layer::ArmorLayerSurface<'_>],
) -> Result<PartMesh> {
    let region = FitRegion::Shoulder(side);
    let frame = wearer.frame(region)?;
    let selected = wearer.support_indices(region)?;
    let mut owned = vec![false; wearer.positions.len()];
    for i in selected {
        owned[i] = true;
    }
    let mut parts = SpaulderPlates::new(design, &frame)?;
    let crown_span = axial_span(&parts.crown, &frame);
    let mut support = wearer
        .faces
        .iter()
        .filter(|face| face.iter().all(|i| owned[*i as usize]))
        .map(|face| face.map(|i| project(&frame, wearer.positions[i as usize])))
        .collect::<Vec<_>>();
    for layer in layers {
        support.extend(layer_support(layer, &frame, crown_span));
    }
    ensure!(
        !support.is_empty(),
        "spaulder crown has no complete shoulder support triangles"
    );
    let mut lame_owned = owned;
    for index in wearer.support_indices(FitRegion::UpperArm(side))? {
        lame_owned[index] = true;
    }
    let mut lame_support = wearer
        .faces
        .iter()
        .filter(|face| face.iter().all(|i| lame_owned[*i as usize]))
        .map(|face| face.map(|i| project(&frame, wearer.positions[i as usize])))
        .collect::<Vec<_>>();
    let lame_span = axial_span(&parts.lames, &frame);
    for layer in layers {
        lame_support.extend(layer_support(layer, &frame, lame_span));
    }
    parts.lames = seat_lames(
        &parts.lames,
        &frame,
        &lame_support,
        design.gauge.clearance.metres() + design.gauge.thickness.metres(),
    )?;
    parts.crown = seat(&parts.crown, &frame, &support)?;
    Ok(parts.mesh())
}

fn layer_support(
    layer: &crate::armor_layer::ArmorLayerSurface<'_>,
    frame: &PartFrame,
    axial_span: (f32, f32),
) -> Vec<[Vec3; 3]> {
    layer
        .faces
        .iter()
        .flat_map(|face| {
            let mut polygon = face
                .map(|i| project(frame, layer.positions[i as usize]))
                .to_vec();
            let x = frame.half_extents[0] * LAYER_SUPPORT_ENVELOPE_SCALE;
            let y = frame.half_extents[2] * LAYER_SUPPORT_ENVELOPE_SCALE;
            polygon = clip(polygon, |point| point.x + x);
            polygon = clip(polygon, |point| x - point.x);
            polygon = clip(polygon, |point| point.y + y);
            polygon = clip(polygon, |point| y - point.y);
            polygon = clip(polygon, |point| point.z - axial_span.0);
            polygon = clip(polygon, |point| axial_span.1 - point.z);
            (1..polygon.len().saturating_sub(1))
                .map(|i| [polygon[0], polygon[i], polygon[i + 1]])
                .collect::<Vec<_>>()
        })
        .collect()
}

fn axial_span(mesh: &PartMesh, frame: &PartFrame) -> (f32, f32) {
    mesh.positions
        .iter()
        .map(|point| project(frame, *point).z)
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(low, high), axial| {
            (low.min(axial), high.max(axial))
        })
}

fn seat_lames(
    template: &PartMesh,
    frame: &PartFrame,
    support: &[[Vec3; 3]],
    gap: f32,
) -> Result<PartMesh> {
    let axial = template
        .positions
        .iter()
        .map(|point| project(frame, *point).z);
    let low = axial.clone().fold(f32::INFINITY, f32::min);
    let high = axial.fold(f32::NEG_INFINITY, f32::max);
    Ok(template.refit_surfaces(|points, _| {
        for point in points {
            let p = project(frame, *point);
            let distance = p.x.hypot(p.y);
            if distance <= f32::EPSILON {
                continue;
            }
            let direction = Vec3::new(p.x / distance, p.y / distance, 0.0);
            let origin = Vec3::new(0.0, 0.0, p.z);
            let required = support
                .iter()
                .filter_map(|triangle| {
                    ray_triangle(origin, direction, *triangle).map(|hit| hit + gap)
                })
                .fold(distance, f32::max);
            if required > distance {
                let attachment_blend = ((high - p.z) / (high - low) * 4.0).clamp(0.0, 1.0);
                let fitted =
                    origin + direction * (distance + (required - distance) * attachment_blend);
                *point = frame.point([fitted.x, fitted.z, fitted.y]);
            }
        }
    })?)
}

fn ray_triangle(origin: Vec3, direction: Vec3, [a, b, c]: [Vec3; 3]) -> Option<f32> {
    let edge_ab = b - a;
    let edge_ac = c - a;
    let perpendicular = direction.cross(edge_ac);
    let determinant = edge_ab.dot(perpendicular);
    if determinant.abs() <= PROJECTED_AREA_EPSILON_M2 {
        return None;
    }
    let inverse = determinant.recip();
    let from_a = origin - a;
    let u = from_a.dot(perpendicular) * inverse;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let cross = from_a.cross(edge_ab);
    let v = direction.dot(cross) * inverse;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let distance = edge_ac.dot(cross) * inverse;
    (distance >= 0.0).then_some(distance)
}

fn seat(template: &PartMesh, frame: &PartFrame, support: &[[Vec3; 3]]) -> Result<PartMesh> {
    let mut count = 0;
    let mut faces = Vec::new();
    let mut rim = f32::INFINITY;
    template.refit_surfaces(|points, indices| {
        count = points.len();
        faces = indices.as_chunks::<3>().0.to_vec();
        rim = points
            .iter()
            .map(|p| project(frame, *p).z)
            .fold(f32::INFINITY, f32::min);
    })?;
    ensure!(
        template.shell_vertex_ranges().count() == 1 && count > 0,
        "spaulder crown must be one physical sheet"
    );
    let mut scale = 1.0;
    for _ in 0..MAXIMUM_SEATING_PASSES {
        let fitted = if scale == 1.0 {
            template.clone()
        } else {
            template.refit_surfaces(|points, _| {
                for p in points {
                    let lift = (project(frame, *p).z - rim) * (scale - 1.0);
                    for (axis, value) in p.iter_mut().enumerate() {
                        *value += frame.axes[1][axis] * lift;
                    }
                }
            })?
        };
        let inner = fitted.positions[count..2 * count]
            .iter()
            .map(|p| project(frame, *p))
            .collect::<Vec<_>>();
        let correction = required_scale(&inner, &faces, support, rim)?;
        if correction == 1.0 {
            return Ok(fitted);
        }
        scale *= correction;
        ensure!(
            scale.is_finite(),
            "spaulder crown support requires an unbounded axial span"
        );
    }
    anyhow::bail!("spaulder crown cannot enclose shoulder support with its fixed attachment rim")
}

/// Project along the arm axis: XY is the unchanged crown footprint, Z its span.
fn project(frame: &PartFrame, p: [f32; 3]) -> Vec3 {
    let delta = Vec3::from_array(p) - Vec3::from_array(frame.origin);
    Vec3::new(
        delta.dot(Vec3::from_array(frame.axes[0])),
        delta.dot(Vec3::from_array(frame.axes[2])),
        delta.dot(Vec3::from_array(frame.axes[1])),
    )
}

fn required_scale(
    inner: &[Vec3],
    faces: &[[u32; 3]],
    support: &[[Vec3; 3]],
    rim: f32,
) -> Result<f32> {
    let mut scale = 1.0_f32;
    for face in faces {
        let triangle = face.map(|i| inner[i as usize]);
        let [a, b, c] = triangle.map(Vec3::truncate);
        let area = cross(b - a, c - a);
        if area.abs() <= PROJECTED_AREA_EPSILON_M2 {
            continue;
        }
        let low = a.min(b).min(c);
        let high = a.max(b).max(c);
        for body in support {
            let body_low = body
                .iter()
                .map(|p| p.truncate())
                .fold(Vec2::splat(f32::INFINITY), Vec2::min);
            let body_high = body
                .iter()
                .map(|p| p.truncate())
                .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
            if body_low.x > high.x
                || body_low.y > high.y
                || body_high.x < low.x
                || body_high.y < low.y
            {
                continue;
            }
            let mut polygon = body.to_vec();
            for edge in 0..3 {
                let start = triangle[edge].truncate();
                let end = triangle[(edge + 1) % 3].truncate();
                polygon = clip(polygon, |p| {
                    cross(end - start, p.truncate() - start) * area.signum()
                });
            }
            for p in polygon {
                let u = cross(b - p.truncate(), c - p.truncate()) / area;
                let v = cross(c - p.truncate(), a - p.truncate()) / area;
                let height = u * triangle[0].z + v * triangle[1].z + (1.0 - u - v) * triangle[2].z;
                let deficit = p.z - height;
                if deficit <= SUPPORT_TOLERANCE_M {
                    continue;
                }
                let response = height - rim;
                ensure!(
                    response > 0.0,
                    "shoulder support intersects the fixed spaulder crown rim"
                );
                scale = scale.max(1.0 + (deficit + SUPPORT_TOLERANCE_M) / response);
            }
        }
    }
    Ok(scale)
}

fn cross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

fn clip(polygon: Vec<Vec3>, distance: impl Fn(Vec3) -> f32) -> Vec<Vec3> {
    let mut result = Vec::new();
    for i in 0..polygon.len() {
        let a = polygon[i];
        let b = polygon[(i + 1) % polygon.len()];
        let da = distance(a);
        let db = distance(b);
        if da >= 0.0 {
            result.push(a);
        }
        if (da < 0.0) != (db < 0.0) {
            result.push(a.lerp(b, da / (da - db)));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_armor_model::{BoundaryNormals, Millimeters, ShellExtrusion, SurfaceRelief};

    use crate::armor_layer::ArmorLayerSurface;

    fn crown() -> (PartMesh, PartFrame, Vec<[u32; 3]>) {
        let positions = vec![
            [-0.1, 0.0, -0.1],
            [0.1, 0.0, -0.1],
            [0.1, 0.0, 0.1],
            [-0.1, 0.0, 0.1],
            [0.0, 0.04, 0.0],
        ];
        let faces = vec![[0, 4, 1], [1, 4, 2], [2, 4, 3], [3, 4, 0]];
        let mesh = PartMesh::from_relief_surface(
            positions,
            faces.iter().flatten().copied().collect(),
            0.001,
            BoundaryNormals::Separate,
            ShellExtrusion::AngleWeightedNormal,
            Some(SurfaceRelief::ShellHeights(vec![0.0, 0.0, 0.0, 0.0, 0.002])),
        )
        .unwrap();
        let frame = PartFrame {
            detail: adventuresim_armor_model::ArmorDetail::BakeSource,
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [1.0; 3],
        };
        (mesh, frame, faces)
    }

    #[test]
    fn clear_crown_is_exact_and_off_center_support_fits_between_vertices() {
        let (mesh, frame, faces) = crown();
        let clear = [[
            Vec3::new(-0.3, -0.3, -0.02),
            Vec3::new(0.3, -0.3, -0.02),
            Vec3::new(0.0, 0.3, -0.02),
        ]];
        assert_eq!(
            seat(&mesh, &frame, &clear).unwrap().positions,
            mesh.positions
        );
        for peak in [Vec3::new(0.0, 0.0, 0.05), Vec3::new(0.035, -0.025, 0.055)] {
            let boundary = [
                Vec3::new(-0.12, -0.12, -0.10),
                Vec3::new(0.12, -0.12, -0.10),
                Vec3::new(0.12, 0.12, -0.10),
                Vec3::new(-0.12, 0.12, -0.10),
            ];
            let support = (0..4)
                .map(|i| [boundary[i], boundary[(i + 1) % 4], peak])
                .collect::<Vec<_>>();
            let before = mesh.positions[5..10]
                .iter()
                .map(|p| project(&frame, *p))
                .collect::<Vec<_>>();
            assert!(required_scale(&before, &faces, &support, 0.0).unwrap() > 1.0);
            let fitted = seat(&mesh, &frame, &support).unwrap();
            assert_eq!(fitted.indices, mesh.indices);
            let inner = fitted.positions[5..10]
                .iter()
                .map(|p| project(&frame, *p))
                .collect::<Vec<_>>();
            assert_eq!(required_scale(&inner, &faces, &support, 0.0).unwrap(), 1.0);
            fitted
                .refit_surfaces(|carrier, _| {
                    for (i, p) in carrier.iter().enumerate() {
                        assert_eq!(p[0], mesh.positions[i][0]);
                        assert_eq!(p[2], mesh.positions[i][2]);
                        if i < 4 {
                            assert_eq!(p[1], 0.0, "attachment rim moved");
                        }
                    }
                })
                .unwrap();
            for i in 0..5 {
                let gauge = (Vec3::from_array(fitted.positions[i])
                    - Vec3::from_array(fitted.positions[i + 5]))
                .length();
                assert!((gauge - 0.001).abs() < 1e-7);
            }
        }
    }

    #[test]
    fn fixed_rim_support_cannot_be_hidden_by_infinite_span() {
        let inner = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let body = [inner.map(|p| p + Vec3::Z * 0.001)];
        assert!(required_scale(&inner, &[[0, 1, 2]], &body, 0.0).is_err());
    }

    #[test]
    fn layer_support_keeps_local_geometry_without_importing_remote_plate_extent() {
        let (mesh, frame, faces) = crown();
        let positions = [
            [-0.08, 0.03, -0.08],
            [0.08, 0.03, -0.08],
            [0.0, 0.03, 0.08],
            [-0.08, 0.5, -0.08],
            [0.08, 0.5, -0.08],
            [0.0, 0.5, 0.08],
        ];
        let layer_faces = [[0, 1, 2], [3, 4, 5]];
        let layer = ArmorLayerSurface {
            relief: Millimeters(0),
            positions: &positions,
            faces: &layer_faces,
            joint_indices: &[],
            joint_weights: &[],
        };
        let span = axial_span(&mesh, &frame);
        let support = layer_support(&layer, &frame, span);
        assert_eq!(support.len(), 1);
        assert!(
            required_scale(
                &mesh.positions[5..10]
                    .iter()
                    .map(|point| project(&frame, *point))
                    .collect::<Vec<_>>(),
                &faces,
                &support,
                0.0,
            )
            .unwrap()
                > 1.0
        );
    }

    #[test]
    fn radial_ray_uses_the_actual_triangle_surface_at_the_lame_height() {
        let triangle = [
            Vec3::new(0.08, -0.03, -0.02),
            Vec3::new(0.08, 0.03, -0.02),
            Vec3::new(0.08, 0.0, 0.02),
        ];
        let hit = ray_triangle(Vec3::ZERO, Vec3::X, triangle).unwrap();
        assert!((hit - 0.08).abs() < 1e-6);
        assert!(ray_triangle(Vec3::new(0.0, 0.0, 0.03), Vec3::X, triangle).is_none());
    }
}

#[cfg(test)]
#[path = "spaulder_sampling_tests.rs"]
mod sampling_tests;
