//! Seat the emitted bib facets, including support between carrier samples.
//! The collar seam stays fixed; anatomical displacement follows each recorded
//! bib chart direction. The inset metric transitions over the collar-to-middle
//! chart domain; every pass recomputes the actual inner gauge and footprints.
use adventuresim_armor_model::{BoundaryNormals, PartMesh, ShellExtrusion};
use anyhow::{Result, ensure};
use bevy::math::{Vec2, Vec3};

const CLEARANCE_TOLERANCE_M: f32 = 0.000001;
const MAXIMUM_SEATING_PASSES: usize = 24;

#[derive(Clone, Copy)]
pub(super) struct BibVertex {
    pub direction: Vec3,
    pub ceiling: f32,
    pub collar_inset: Vec3,
    pub transition: f32,
}

#[derive(Clone, Copy)]
pub(super) enum GorgetVertex {
    /// Difference between the padded and anatomical collar charts.
    Collar(Vec3),
    Bib(BibVertex),
}

impl GorgetVertex {
    fn bib(self) -> Option<BibVertex> {
        match self {
            Self::Collar(_) => None,
            Self::Bib(vertex) => Some(vertex),
        }
    }

    fn inset(self, outer: Vec3, inner: Vec3, bib_clearance: f32) -> Vec3 {
        match self {
            Self::Collar(offset) => outer - offset,
            Self::Bib(vertex) => (outer - vertex.collar_inset)
                .lerp(inner - vertex.direction * bib_clearance, vertex.transition),
        }
    }
}

pub(super) fn seat(
    positions: &mut [[f32; 3]],
    indices: &[u32],
    chart: &[GorgetVertex],
    body: &[[f32; 3]],
    faces: &[[u32; 3]],
    padding: f32,
    wall: f32,
) -> Result<()> {
    ensure!(
        positions.len() == chart.len(),
        "gorget bib chart lost vertex correspondence"
    );
    if chart.iter().all(|v| v.bib().is_none()) {
        return Ok(());
    }
    for _ in 0..MAXIMUM_SEATING_PASSES {
        // Gauge follows the actual carrier normals. A scalar wall allowance
        // along the support ray does not enclose this inner surface on slopes.
        let shell = PartMesh::from_surface(
            positions.to_vec(),
            indices.to_vec(),
            wall,
            BoundaryNormals::Smooth,
            ShellExtrusion::AngleWeightedNormal,
        )?;
        let inset = &shell.positions[positions.len()..2 * positions.len()];
        let mut lifts = vec![0.0_f32; positions.len()];
        for face in indices.as_chunks::<3>().0 {
            let vertices = face.map(|i| chart[i as usize]);
            let Some(section) = FacetSection::new(
                face.map(|i| {
                    chart[i as usize].inset(
                        Vec3::from_array(positions[i as usize]),
                        Vec3::from_array(inset[i as usize]),
                        padding,
                    )
                }),
                vertices,
            )?
            else {
                continue;
            };
            let lift = section.required_lift(body, faces)?;
            for index in face {
                if chart[*index as usize].bib().is_some() {
                    lifts[*index as usize] = lifts[*index as usize].max(lift);
                }
            }
        }
        if lifts.iter().all(|lift| *lift <= CLEARANCE_TOLERANCE_M) {
            return Ok(());
        }
        for ((point, vertex), lift) in positions.iter_mut().zip(chart).zip(lifts) {
            if let Some(vertex) = vertex.bib() {
                *point = (Vec3::from_array(*point) + vertex.direction * lift).to_array();
            }
        }
    }
    anyhow::bail!("gorget bib cannot enclose body support with its fixed collar seam")
}

struct FacetSection {
    direction: Vec3,
    triangle: [Vec3; 3],
    lower: Vec2,
    upper: Vec2,
    area: f32,
    response: [f32; 3],
    ceiling: f32,
}

impl FacetSection {
    fn new(inset: [Vec3; 3], chart: [GorgetVertex; 3]) -> Result<Option<Self>> {
        let sum: Vec3 = chart
            .iter()
            .filter_map(|v| v.bib())
            .map(|v| v.direction)
            .sum();
        if sum.length_squared() <= f32::EPSILON {
            return Ok(None);
        }
        let direction = sum.normalize();
        // Query the actual inset facet, including its changed footprint. A
        // dot product of clearance with the ray direction would be wrong on
        // an oblique face: both its plane and its projected edges move.
        let triangle = inset.map(|point| project(point, direction));
        let area = cross(
            triangle[1].truncate() - triangle[0].truncate(),
            triangle[2].truncate() - triangle[0].truncate(),
        );
        ensure!(
            area.is_finite() && area.abs() > 1e-12,
            "gorget bib facet has no directional fitting projection"
        );
        let lower = triangle
            .iter()
            .fold(Vec2::splat(f32::INFINITY), |a, p| a.min(p.truncate()));
        let upper = triangle
            .iter()
            .fold(Vec2::splat(f32::NEG_INFINITY), |a, p| a.max(p.truncate()));
        Ok(Some(Self {
            direction,
            triangle,
            lower,
            upper,
            area,
            response: chart.map(|v| v.bib().map_or(0.0, |v| v.direction.dot(direction))),
            ceiling: chart
                .iter()
                .filter_map(|v| v.bib())
                .map(|v| v.ceiling)
                .fold(f32::NEG_INFINITY, f32::max),
        }))
    }

    fn required_lift(&self, body: &[[f32; 3]], faces: &[[u32; 3]]) -> Result<f32> {
        let mut lift = 0.0_f32;
        for face in faces {
            let triangle =
                face.map(|i| project(Vec3::from_array(body[i as usize]), self.direction));
            let lower = triangle
                .iter()
                .fold(Vec2::splat(f32::INFINITY), |a, p| a.min(p.truncate()));
            let upper = triangle
                .iter()
                .fold(Vec2::splat(f32::NEG_INFINITY), |a, p| a.max(p.truncate()));
            if lower.x > self.upper.x
                || lower.y > self.upper.y
                || upper.x < self.lower.x
                || upper.y < self.lower.y
            {
                continue;
            }
            let polygon = self.support_fragment(triangle);
            for point in polygon {
                let weights = self.weights(point.truncate());
                let depth: f32 = (0..3).map(|i| weights[i] * self.triangle[i].z).sum();
                let deficit = point.z - depth;
                if deficit <= CLEARANCE_TOLERANCE_M {
                    continue;
                }
                let response: f32 = (0..3).map(|i| weights[i] * self.response[i]).sum();
                ensure!(
                    response > 1e-6,
                    "body support intersects the fixed gorget collar seam: body face {face:?}, projected fragment {point:?}, projected carrier {:?}, direction {:?}, barycentric {weights:?}, deficit {deficit}m",
                    self.triangle,
                    self.direction
                );
                lift = lift.max(deficit / response);
            }
        }
        Ok(lift)
    }

    fn support_fragment(&self, triangle: [Vec3; 3]) -> Vec<Vec3> {
        let mut polygon = clip(triangle.to_vec(), |p| {
            self.ceiling - (p.y * self.direction.z + p.z * self.direction.y)
        });
        for edge in 0..3 {
            let a = self.triangle[edge].truncate();
            let b = self.triangle[(edge + 1) % 3].truncate();
            polygon = clip(polygon, |p| {
                cross(b - a, p.truncate() - a) * self.area.signum()
            });
        }
        polygon
    }

    fn weights(&self, point: Vec2) -> [f32; 3] {
        let [a, b, c] = self.triangle.map(Vec3::truncate);
        let u = cross(b - point, c - point) / self.area;
        let v = cross(c - point, a - point) / self.area;
        [u, v, 1.0 - u - v]
    }
}

fn project(point: Vec3, direction: Vec3) -> Vec3 {
    Vec3::new(
        point.x,
        point.y * direction.z - point.z * direction.y,
        point.dot(direction),
    )
}

fn cross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

fn clip(polygon: Vec<Vec3>, distance: impl Fn(Vec3) -> f32) -> Vec<Vec3> {
    let mut result = Vec::new();
    for index in 0..polygon.len() {
        let a = polygon[index];
        let b = polygon[(index + 1) % polygon.len()];
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
    use adventuresim_armor_model::{BoundaryNormals, PartMesh, ShellExtrusion};

    fn tent(x: f32, y: f32) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let points = vec![
            [-0.2, 0.0, 0.03],
            [0.2, 0.0, 0.03],
            [0.2, 0.11, 0.03],
            [-0.2, 0.11, 0.03],
            [x - 0.007, y - 0.005, 0.03],
            [x + 0.007, y - 0.005, 0.03],
            [x + 0.007, y + 0.005, 0.03],
            [x - 0.007, y + 0.005, 0.03],
            [x, y, 0.08],
        ];
        (
            points,
            vec![
                [0, 1, 2],
                [0, 2, 3],
                [4, 5, 8],
                [5, 6, 8],
                [6, 7, 8],
                [7, 4, 8],
            ],
        )
    }

    fn carrier(angular: bool, collar_clearance: f32, rows: usize) -> (PartMesh, Vec<GorgetVertex>) {
        let mut points = Vec::new();
        let mut chart = Vec::new();
        for row in 0..=rows {
            for column in 0..=4 {
                let x = -0.1 + column as f32 * 0.05;
                points.push([x, 0.1 - row as f32 / rows as f32 * 0.1, 0.05]);
                chart.push(if row == 0 {
                    GorgetVertex::Collar(Vec3::Z * collar_clearance)
                } else {
                    GorgetVertex::Bib(BibVertex {
                        direction: Vec3::new(0.0, if angular { x * x * 20.0 } else { 0.0 }, 1.0)
                            .normalize(),
                        ceiling: 0.1,
                        collar_inset: Vec3::Z * collar_clearance,
                        transition: super::super::meridian::clearance_blend(
                            row as f32 / rows as f32,
                        ),
                    })
                });
            }
        }
        let indices = (0..rows as u32)
            .flat_map(|row| {
                (0..4).flat_map(move |column| {
                    let a = row * 5 + column;
                    [a, a + 5, a + 6, a, a + 6, a + 1]
                })
            })
            .collect();
        (
            PartMesh::from_surface(
                points,
                indices,
                0.001,
                BoundaryNormals::Separate,
                ShellExtrusion::AngleWeightedNormal,
            )
            .unwrap(),
            chart,
        )
    }

    #[test]
    fn final_bib_facets_enclose_peaks_between_both_chart_axes() {
        for (x, y, angular) in [
            (0.0, 0.05, false),
            (0.013, 0.047, false),
            (0.026, 0.061, true),
        ] {
            let (body, faces) = tent(x, y);
            let (mesh, chart) = carrier(angular, 0.003, 4);
            let mut result = Ok(());
            let fitted = mesh
                .refit_surfaces(|points, indices| {
                    result = seat(points, indices, &chart, &body, &faces, 0.002, 0.001);
                })
                .unwrap();
            result.unwrap();
            assert_eq!(fitted.indices, mesh.indices);
            assert_eq!(
                &fitted.positions[..5],
                &mesh.positions[..5],
                "collar seam moved"
            );
            let mut depths = Vec::new();
            for face in fitted.indices.as_chunks::<3>().0 {
                let points = face.map(|i| fitted.positions[i as usize]);
                let section = FacetSection::new(
                    points.map(Vec3::from_array),
                    [GorgetVertex::Bib(BibVertex {
                        direction: Vec3::Z,
                        ceiling: 0.1,
                        collar_inset: Vec3::ZERO,
                        transition: 1.0,
                    }); 3],
                );
                let Ok(Some(section)) = section else { continue };
                let weights = section.weights(Vec2::new(x, y));
                if weights.iter().all(|weight| *weight >= -1e-6) {
                    depths.push((0..3).map(|i| weights[i] * points[i][2]).sum::<f32>());
                }
            }
            assert!(depths.len() >= 2, "fixture did not hit both shell faces");
            assert!(
                depths.iter().all(|depth| *depth >= 0.082 - 1e-6),
                "finished inner facet entered the support peak: {depths:?}"
            );
        }
    }

    #[test]
    fn infeasible_support_at_the_fixed_collar_is_rejected() {
        let (body, faces) = tent(0.0, 0.1);
        let (mesh, chart) = carrier(false, 0.003, 4);
        let mut result = Ok(());
        let _ = mesh.refit_surfaces(|points, indices| {
            result = seat(points, indices, &chart, &body, &faces, 0.002, 0.001);
        });
        assert!(result.is_err());
    }

    #[test]
    fn collar_and_bib_keep_their_independently_authored_clearance() {
        let body = [
            [-0.2, -0.1, 0.039],
            [0.2, -0.1, 0.039],
            [0.2, 0.2, 0.039],
            [-0.2, 0.2, 0.039],
        ];
        let faces = [[0, 1, 2], [0, 2, 3]];
        let (mesh, chart) = carrier(false, 0.010, 4);
        let mut result = Ok(());
        let fitted = mesh
            .refit_surfaces(|points, indices| {
                result = seat(points, indices, &chart, &body, &faces, 0.012, 0.001);
            })
            .unwrap();
        result.unwrap();
        assert_eq!(&fitted.positions[..5], &mesh.positions[..5]);
        assert!(
            fitted.positions[10..25]
                .iter()
                .all(|p| p[2] >= 0.052 - 1e-6)
        );
    }

    #[test]
    fn transverse_collar_offset_preserves_its_sloped_inset_facet() {
        // The anatomical plane is x + z = 0. Moving a collar point along X
        // changes its Z-ray intersection even though that offset dot Z is zero.
        let inset = [
            Vec3::new(-0.1, 0.1, 0.1),
            Vec3::new(0.1, 0.1, -0.1),
            Vec3::new(0.0, 0.0, 0.0),
        ];
        let chart = [
            GorgetVertex::Collar(Vec3::X * 0.014),
            GorgetVertex::Collar(Vec3::X * 0.014),
            GorgetVertex::Bib(BibVertex {
                direction: Vec3::Z,
                ceiling: 0.1,
                collar_inset: Vec3::X * 0.014,
                transition: 1.0,
            }),
        ];
        let points = [
            (inset[0] + Vec3::X * 0.014).to_array(),
            (inset[1] + Vec3::X * 0.014).to_array(),
            (inset[2] + Vec3::Z * 0.013).to_array(),
        ];
        let actual_inset = std::array::from_fn(|i| {
            let point = Vec3::from_array(points[i]);
            chart[i].inset(point, point, 0.013)
        });
        let section = FacetSection::new(actual_inset, chart).unwrap().unwrap();
        for (height, should_fit) in [(-0.001, true), (0.007, false)] {
            let body = inset.map(|p| (p + Vec3::Z * height).to_array());
            let result = section.required_lift(&body, &[[0, 1, 2]]);
            assert_eq!(result.is_ok(), should_fit, "inset plane depth {height}");
        }
    }

    #[test]
    fn distinct_clearances_do_not_waive_an_infeasible_collar() {
        let (mesh, chart) = carrier(false, 0.010, 4);
        let body = [
            [-0.2, -0.1, 0.041],
            [0.2, -0.1, 0.041],
            [0.2, 0.2, 0.041],
            [-0.2, 0.2, 0.041],
        ];
        let mut result = Ok(());
        let _ = mesh.refit_surfaces(|points, indices| {
            result = seat(
                points,
                indices,
                &chart,
                &body,
                &[[0, 1, 2], [0, 2, 3]],
                0.012,
                0.001,
            );
        });
        assert!(
            result.is_err(),
            "9mm collar gap cannot satisfy authored 10mm"
        );
    }

    #[test]
    fn refining_the_first_cell_does_not_compress_the_clearance_transition() {
        let body = [
            [-0.2, -0.1, 0.039],
            [0.2, -0.1, 0.039],
            [0.2, 0.2, 0.039],
            [-0.2, 0.2, 0.039],
        ];
        let faces = [[0, 1, 2], [0, 2, 3]];
        let mut maximum_movements = Vec::new();
        for rows in [4, 16] {
            let (mesh, chart) = carrier(false, 0.010, rows);
            let mut carrier_points = mesh.positions[..(rows + 1) * 5].to_vec();
            let surface_indices = &mesh.indices[..rows * 4 * 6];
            seat(
                &mut carrier_points,
                surface_indices,
                &chart,
                &body,
                &faces,
                0.012,
                0.001,
            )
            .unwrap();
            assert_eq!(&carrier_points[..5], &mesh.positions[..5]);
            let shell = PartMesh::from_surface(
                carrier_points.clone(),
                surface_indices.to_vec(),
                0.001,
                BoundaryNormals::Smooth,
                ShellExtrusion::AngleWeightedNormal,
            )
            .unwrap();
            // A triangle lies in the half-space of its vertices: this checks
            // the finished outer, inner and return faces against the plane.
            assert!(shell.positions.iter().all(|point| point[2] > 0.039));
            let maximum = carrier_points
                .iter()
                .zip(&mesh.positions)
                .map(|(a, b)| Vec3::from_array(*a).distance(Vec3::from_array(*b)))
                .fold(0.0_f32, f32::max);
            // A 2mm missing bib allowance cannot become a centimetre-scale
            // movement merely because the first axial cell is subdivided.
            assert!(maximum <= 0.0021, "{rows} rows moved {maximum}m");
            maximum_movements.push(maximum);
            for column in 0..5 {
                let index = rows / 4 * 5 + column;
                let inset = chart[index].inset(
                    Vec3::new(0.0, 0.075, 0.05),
                    Vec3::new(0.0, 0.075, 0.049),
                    0.012,
                );
                assert!((inset.z - 0.0385).abs() < 1e-7);
            }
        }
        assert!((maximum_movements[0] - maximum_movements[1]).abs() < 0.0001);
    }

    #[test]
    fn vector_inset_is_continuous_at_the_collar_and_complete_by_the_middle() {
        let outer = Vec3::new(0.1, 0.1, 0.1);
        let inner = outer - Vec3::new(0.0006, 0.0, 0.0008);
        let collar = Vec3::X * 0.014;
        let inset = |t| {
            GorgetVertex::Bib(BibVertex {
                direction: Vec3::Z,
                ceiling: 0.1,
                collar_inset: collar,
                transition: super::super::meridian::clearance_blend(t),
            })
            .inset(outer, inner, 0.012)
        };
        assert_eq!(inset(0.0), outer - collar);
        for t in [0.5, 0.75, 1.0] {
            assert_eq!(inset(t), inner - Vec3::Z * 0.012);
        }
        let full_change = inset(0.5).distance(inset(0.0));
        assert!(inset(0.001).distance(inset(0.0)) < full_change * 0.0001);
        assert!(inset(0.25).distance((outer - collar).lerp(inner - Vec3::Z * 0.012, 0.5)) < 1e-8);
    }
}
