//! Chest/back clearance for the shared shoulder carrier, before plate assembly.
//!
//! Input is the canonical unposed MHR frame: world Y is anatomical height and
//! world Z separates anterior from posterior. Rig landmarks place the shoulder
//! carrier; the X/Y projection here is not an arbitrary-pose refitting policy.
use adventuresim_armor_model::{PartMesh, PauldronCarrier, PauldronDesign};
use anyhow::Result;

use crate::armor_frames::{FitRegion, Wearer};

const PROJECTED_AREA_EPSILON_M2: f32 = 1e-12;
// Inward extrusion moves the wall laterally across sloped body triangles.
const OBLIQUE_WALL_RESERVE_GAUGES: f32 = 2.5;
pub(super) fn fit(
    d: &PauldronDesign,
    wearer: &Wearer<'_>,
    region: FitRegion,
    layers: &[crate::armor_layer::ArmorLayerSurface<'_>],
) -> Result<PartMesh> {
    let frame = wearer.frame(region)?;
    let triangles = projected(wearer.positions, wearer.faces);
    let layers = layers
        .iter()
        .map(|layer| {
            (
                projected(layer.positions, layer.faces),
                layer.relief.metres(),
            )
        })
        .collect::<Vec<_>>();
    let body_clearance = (d.gauge.clearance.metres() + d.gauge.thickness.metres())
        .max(d.gauge.thickness.metres() * OBLIQUE_WALL_RESERVE_GAUGES);
    let plate_clearance = d.plate_clearance.metres() + d.gauge.thickness.metres();
    let edge_reserve = d.gauge.thickness.metres() * OBLIQUE_WALL_RESERVE_GAUGES;
    let mut carrier = PauldronCarrier::new(d, &frame)?;
    carrier.fit(|mut point| {
        let bounds = depth_bounds(&triangles, point, 0.0, 0.0);
        let midpoint = bounds.map_or(frame.origin[2], |[low, high]| (low + high) * 0.5);
        let anterior = point[2] > midpoint;
        if let Some([low, high]) = bounds {
            point[2] = if anterior {
                point[2].max(high + body_clearance)
            } else {
                point[2].min(low - body_clearance)
            };
        }
        for (layer, relief) in &layers {
            if let Some([low, high]) =
                depth_bounds(layer, point, edge_reserve, plate_clearance + relief)
            {
                point[2] = if anterior {
                    point[2].max(high)
                } else {
                    point[2].min(low)
                };
            }
        }
        point
    })?;
    Ok(carrier.mesh()?)
}

fn projected(positions: &[[f32; 3]], faces: &[[u32; 3]]) -> Vec<DepthTriangle> {
    faces
        .iter()
        .filter_map(|face| DepthTriangle::new(face.map(|i| positions[i as usize])))
        .collect()
}

fn depth_bounds(
    triangles: &[DepthTriangle],
    point: [f32; 3],
    edge_reserve: f32,
    clearance: f32,
) -> Option<[f32; 2]> {
    let mut low = f32::INFINITY;
    let mut high = f32::NEG_INFINITY;
    for triangle in triangles {
        if let Some(depth) = triangle.depth(point, edge_reserve) {
            let bounds = depth.bounds_from(point[2], clearance);
            low = low.min(bounds[0]);
            high = high.max(bounds[1]);
        }
    }
    low.is_finite().then_some([low, high])
}

#[derive(Debug, PartialEq)]
struct DepthSample {
    depth: f32,
    influence: f32,
}

impl DepthSample {
    fn bounds_from(&self, origin: f32, clearance: f32) -> [f32; 2] {
        [-clearance, clearance]
            .map(|offset| origin + (self.depth + offset - origin) * self.influence)
    }
}

struct DepthTriangle {
    points: [[f32; 3]; 3],
    low: [f32; 2],
    high: [f32; 2],
    a: [f32; 2],
    b: [f32; 2],
    denominator: f32,
}

impl DepthTriangle {
    fn new(points: [[f32; 3]; 3]) -> Option<Self> {
        let a = std::array::from_fn(|i| points[1][i] - points[0][i]);
        let b = std::array::from_fn(|i| points[2][i] - points[0][i]);
        let denominator = cross(a, b);
        (denominator.abs() > PROJECTED_AREA_EPSILON_M2).then(|| Self {
            low: std::array::from_fn(|i| points.iter().map(|p| p[i]).fold(f32::INFINITY, f32::min)),
            high: std::array::from_fn(|i| {
                points
                    .iter()
                    .map(|p| p[i])
                    .fold(f32::NEG_INFINITY, f32::max)
            }),
            points,
            a,
            b,
            denominator,
        })
    }

    fn depth(&self, point: [f32; 3], edge_reserve: f32) -> Option<DepthSample> {
        if (0..2).any(|i| {
            point[i] < self.low[i] - edge_reserve || point[i] > self.high[i] + edge_reserve
        }) {
            return None;
        }
        let offset = std::array::from_fn(|i| point[i] - self.points[0][i]);
        let u = cross(offset, self.b) / self.denominator;
        let v = cross(self.a, offset) / self.denominator;
        if u >= 0.0 && v >= 0.0 && u + v <= 1.0 {
            return Some(DepthSample {
                depth: self.points[0][2]
                    + u * (self.points[1][2] - self.points[0][2])
                    + v * (self.points[2][2] - self.points[0][2]),
                influence: 1.0,
            });
        }
        // A plate return can bridge an armhole corner even though its endpoint
        // ray just misses the front plate. Extend the physical edge's depth,
        // bounded by sheet gauge, rather than extrapolating its sloped plane.
        // Fade the added displacement to zero at the search boundary: a hard
        // inclusion threshold makes identity morph samples switch support abruptly.
        self.nearest_edge_depth(point)
            .filter(|(distance, _)| edge_reserve > 0.0 && *distance < edge_reserve * edge_reserve)
            .map(|(distance, depth)| {
                let t = distance.sqrt() / edge_reserve;
                DepthSample {
                    depth,
                    influence: 1.0 - t * t * (3.0 - 2.0 * t),
                }
            })
    }

    fn nearest_edge_depth(&self, point: [f32; 3]) -> Option<(f32, f32)> {
        (0..3)
            .filter_map(|i| {
                let a = self.points[i];
                let b = self.points[(i + 1) % 3];
                let edge = [b[0] - a[0], b[1] - a[1]];
                let squared_length = edge[0] * edge[0] + edge[1] * edge[1];
                if squared_length == 0.0 {
                    return None;
                }
                let t = (((point[0] - a[0]) * edge[0] + (point[1] - a[1]) * edge[1])
                    / squared_length)
                    .clamp(0.0, 1.0);
                let distance = (point[0] - a[0] - t * edge[0]).powi(2)
                    + (point[1] - a[1] - t * edge[1]).powi(2);
                Some((distance, a[2] + t * (b[2] - a[2])))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
    }
}

fn cross(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_query_interpolates_inside_but_rejects_bbox_only_hits() {
        let triangle =
            DepthTriangle::new([[0.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 1.0, 2.0]]).unwrap();
        assert!((triangle.depth([0.25, 0.25, 99.0], 0.0).unwrap().depth - 0.75).abs() < 1e-6);
        assert!(triangle.depth([0.8, 0.8, 0.0], 0.0).is_none());
        assert!(DepthTriangle::new([[0.0; 3]; 3]).is_none());
    }

    #[test]
    fn nearby_trim_support_uses_the_physical_edge_without_unbounded_extrapolation() {
        let triangle =
            DepthTriangle::new([[0.0, 0.0, 0.0], [0.01, 0.0, 0.02], [0.0, 0.01, 0.02]]).unwrap();
        let near = [0.005, -0.002, 0.0];
        assert!(triangle.depth(near, 0.0).is_none());
        assert!((triangle.depth(near, 0.003).unwrap().depth - 0.01).abs() < 1e-6);
        assert!(triangle.depth([0.005, -0.004, 0.0], 0.003).is_none());
        let boundary = triangle.depth([0.005, -0.002999, 0.0], 0.003).unwrap();
        assert!(boundary.bounds_from(0.0, 0.002)[1] < 1e-7);
        let nearer = triangle.depth([0.005, -0.001, 0.0], 0.003).unwrap();
        assert!(nearer.bounds_from(0.0, 0.002)[1] > boundary.bounds_from(0.0, 0.002)[1]);
        let inside = [0.002, 0.002, 0.0];
        assert_eq!(triangle.depth(inside, 0.0), triangle.depth(inside, 0.003));
    }
}
