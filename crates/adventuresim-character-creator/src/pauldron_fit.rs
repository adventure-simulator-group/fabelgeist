//! Chest/back clearance for the shared shoulder carrier, before plate assembly.
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
    let mut carrier = PauldronCarrier::new(d, &frame)?;
    carrier.fit(|mut point| {
        let bounds = depth_bounds(&triangles, point);
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
            if let Some([low, high]) = depth_bounds(layer, point) {
                point[2] = if anterior {
                    point[2].max(high + plate_clearance + relief)
                } else {
                    point[2].min(low - plate_clearance - relief)
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

fn depth_bounds(triangles: &[DepthTriangle], point: [f32; 3]) -> Option<[f32; 2]> {
    let mut low = f32::INFINITY;
    let mut high = f32::NEG_INFINITY;
    for triangle in triangles {
        if let Some(depth) = triangle.depth(point) {
            low = low.min(depth);
            high = high.max(depth);
        }
    }
    low.is_finite().then_some([low, high])
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

    fn depth(&self, point: [f32; 3]) -> Option<f32> {
        if (0..2).any(|i| point[i] < self.low[i] || point[i] > self.high[i]) {
            return None;
        }
        let offset = std::array::from_fn(|i| point[i] - self.points[0][i]);
        let u = cross(offset, self.b) / self.denominator;
        let v = cross(self.a, offset) / self.denominator;
        (u >= 0.0 && v >= 0.0 && u + v <= 1.0).then(|| {
            self.points[0][2]
                + u * (self.points[1][2] - self.points[0][2])
                + v * (self.points[2][2] - self.points[0][2])
        })
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
        assert!((triangle.depth([0.25, 0.25, 99.0]).unwrap() - 0.75).abs() < 1e-6);
        assert!(triangle.depth([0.8, 0.8, 0.0]).is_none());
        assert!(DepthTriangle::new([[0.0; 3]; 3]).is_none());
    }
}
