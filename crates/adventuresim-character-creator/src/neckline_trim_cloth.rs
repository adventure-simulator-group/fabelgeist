//! Reference geometry of a gown's cut face cloth, for laying a band on it.
//! Canonical unposed reference in metres, Y up, front +Z.
use super::outline::TrimColumn;
use crate::armor_frames::Wearer;
use crate::cloth_opening::ClothSurfaceSample;
use crate::surface_cut::SurfaceCut;
use anyhow::{Context, Result};
use bevy::math::Vec3;

/// Steps along the cloth between neighbouring rows. Short steps keep a deep
/// band on a curved surface, where one straight reach would leave it.
const TRIM_ROW_STEPS: usize = 3;

pub(super) struct ReferenceCloth<'a> {
    cut: &'a SurfaceCut,
    pub points: Vec<Vec3>,
    /// Area-weighted vertex normals, not normalized.
    pub normals: Vec<Vec3>,
}

impl<'a> ReferenceCloth<'a> {
    pub fn new(cut: &'a SurfaceCut, body: &Wearer<'_>) -> Self {
        let points = cut
            .points
            .iter()
            .map(|point| {
                Vec3::from_array(crate::surface_cut::interpolate(
                    body.faces[point.triangle].map(|v| body.positions[v as usize]),
                    point.weights,
                ))
            })
            .collect::<Vec<_>>();
        let mut normals = vec![Vec3::ZERO; points.len()];
        for face in &cut.faces {
            let [a, b, c] = face.map(|v| points[v as usize]);
            let area_normal = (b - a).cross(c - a);
            for &vertex in face {
                normals[vertex as usize] += area_normal;
            }
        }
        Self {
            cut,
            points,
            normals,
        }
    }

    pub fn centre(&self, boundary: &[u32]) -> Vec3 {
        boundary
            .iter()
            .map(|&v| self.points[v as usize])
            .sum::<Vec3>()
            / boundary.len() as f32
    }

    pub fn height(&self, boundary: &[u32]) -> f32 {
        self.centre(boundary).y
    }

    /// Unit directions in the cloth, perpendicular to the boundary and
    /// pointing away from the opening.
    pub fn inward_directions(&self, boundary: &[u32]) -> Result<Vec<Vec3>> {
        let on_boundary = |vertex: u32| boundary.contains(&vertex);
        boundary
            .iter()
            .enumerate()
            .map(|(i, &vertex)| {
                let count = boundary.len();
                let point = self.points[vertex as usize];
                let previous = self.points[boundary[(i + count - 1) % count] as usize];
                let next = self.points[boundary[(i + 1) % count] as usize];
                let normal = self.normals[vertex as usize]
                    .try_normalize()
                    .context("neckline point has no cloth normal")?;
                let across = normal
                    .cross(next - previous)
                    .try_normalize()
                    .context("neckline has a stationary point")?;
                // Cloth lies toward the interior corners of incident faces.
                let interior = self
                    .cut
                    .faces
                    .iter()
                    .filter(|face| face.contains(&vertex))
                    .flat_map(|face| face.iter().copied())
                    .filter(|&corner| !on_boundary(corner))
                    .map(|corner| self.points[corner as usize] - point)
                    .sum::<Vec3>();
                let toward = if interior.length_squared() > 0. {
                    interior
                } else {
                    // A boundary-only fan still lies on the cloth side.
                    self.cut
                        .faces
                        .iter()
                        .filter(|face| face.contains(&vertex))
                        .map(|face| {
                            face.map(|v| self.points[v as usize])
                                .into_iter()
                                .sum::<Vec3>()
                                / 3.
                                - point
                        })
                        .sum::<Vec3>()
                };
                Ok(if across.dot(toward) < 0. {
                    -across
                } else {
                    across
                })
            })
            .collect()
    }

    /// The edge source of a column and then one source per row, walked along
    /// the cloth in the column's direction for `width` metres of inset.
    pub fn rows(
        &self,
        column: &TrimColumn,
        width: f32,
        rows: usize,
    ) -> Result<Vec<ClothSurfaceSample>> {
        let mut sources = vec![column.edge];
        let mut point = column.point;
        let reach = column.reach.length();
        let mut direction = column
            .reach
            .try_normalize()
            .context("neckline trim column has no direction")?;
        let step = width * reach / (rows * TRIM_ROW_STEPS) as f32;
        for _ in 0..rows {
            let mut source = column.edge;
            for _ in 0..TRIM_ROW_STEPS {
                source = self.closest(point + direction * step)?;
                let corner = |values: &[Vec3]| {
                    (0..3)
                        .map(|i| values[source.vertices[i]] * source.weights[i])
                        .sum::<Vec3>()
                };
                point = corner(&self.points);
                let normal = corner(&self.normals)
                    .try_normalize()
                    .context("neckline trim crosses cloth without a normal")?;
                direction = (direction - normal * direction.dot(normal))
                    .try_normalize()
                    .context("neckline trim direction leaves the cloth")?;
            }
            sources.push(source);
        }
        Ok(sources)
    }

    /// The nearest face-cloth point, as a frozen barycentric source.
    fn closest(&self, target: Vec3) -> Result<ClothSurfaceSample> {
        self.cut
            .faces
            .iter()
            .filter_map(|face| {
                let [a, b, c] = face.map(|v| self.points[v as usize]);
                let weights = closest_weights(a, b, c, target)?;
                let point = a * weights[0] + b * weights[1] + c * weights[2];
                Some(((point - target).length_squared(), face, weights))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, face, weights)| ClothSurfaceSample {
                vertices: face.map(|v| v as usize),
                weights,
            })
            .context("neckline trim has no face cloth to lie on")
    }
}

/// Barycentric weights of the point of triangle `abc` nearest `p`; `None` for
/// a triangle without area.
fn closest_weights(a: Vec3, b: Vec3, c: Vec3, p: Vec3) -> Option<[f32; 3]> {
    let (ab, ac, ap) = (b - a, c - a, p - a);
    let normal = ab.cross(ac);
    let area = normal.length_squared();
    if area.partial_cmp(&0.) != Some(std::cmp::Ordering::Greater) {
        return None;
    }
    let v = ap.cross(ac).dot(normal) / area;
    let w = ab.cross(ap).dot(normal) / area;
    if v >= 0. && w >= 0. && v + w <= 1. {
        return Some([1. - v - w, v, w]);
    }
    // Outside the face, the nearest point lies on one of its three edges.
    let on_edge = |from: Vec3, to: Vec3| {
        let edge = to - from;
        ((p - from).dot(edge) / edge.length_squared()).clamp(0., 1.)
    };
    let (t_ab, t_bc, t_ca) = (on_edge(a, b), on_edge(b, c), on_edge(c, a));
    [
        [1. - t_ab, t_ab, 0.],
        [0., 1. - t_bc, t_bc],
        [t_ca, 0., 1. - t_ca],
    ]
    .into_iter()
    .min_by(|x, y| {
        let distance = |weights: &[f32; 3]| {
            (a * weights[0] + b * weights[1] + c * weights[2] - p).length_squared()
        };
        distance(x).total_cmp(&distance(y))
    })
}
