//! Planar receiving cuts through convex solids, retaining the shared boundary.
use super::*;
use std::collections::BTreeMap;

/// Covers accumulated rotation/interpolation roundoff, not geometric clearance.
const PLANE_CLASSIFICATION_ULPS: f64 = 64.0;

impl Solid {
    /// Keep the portion at or below a horizontal receiving plane.
    pub(crate) fn truncate_y(&self, height: f64) -> Result<Self, String> {
        let scale = self
            .positions
            .iter()
            .flatten()
            .fold(height.abs(), |m, v| m.max(v.abs()));
        let roundoff = scale * f64::EPSILON * PLANE_CLASSIFICATION_ULPS;
        for point in &self.positions {
            let separation = (point[1] - height).abs();
            if separation > roundoff && separation < crate::recipe::MIN_MANUFACTURED_METRES {
                return Err(
                    "receiving cut leaves a fragment below the manufacturing minimum".into(),
                );
            }
        }
        let mut result = Self::default();
        let mut boundary = Vec::new();
        for (face, &surface) in self.faces.iter().zip(&self.surfaces) {
            let input = face.map(|i| {
                let mut point = self.positions[i];
                if (point[1] - height).abs() <= roundoff {
                    point[1] = height;
                }
                point
            });
            let mut polygon = Vec::new();
            for edge in 0..3 {
                let a = input[edge];
                let b = input[(edge + 1) % 3];
                if a[1] <= height {
                    polygon.push(a);
                    if a[1] == height {
                        boundary.push(a);
                    }
                }
                if (a[1] < height && b[1] > height) || (a[1] > height && b[1] < height) {
                    // Always interpolate the same ordered edge in both faces.
                    let (a, b) = if a[1] < b[1] { (a, b) } else { (b, a) };
                    let mut point = lerp(a, b, (height - a[1]) / (b[1] - a[1]));
                    point[1] = height;
                    polygon.push(point);
                    boundary.push(point);
                }
            }
            for index in 1..polygon.len().saturating_sub(1) {
                result.triangle(polygon[0], polygon[index], polygon[index + 1], surface);
            }
        }
        boundary.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[2].total_cmp(&b[2])));
        boundary.dedup();
        if boundary.len() < 3 {
            return Err("receiving plane must cut a finite convex section".into());
        }
        let mut center = mul(
            boundary.iter().copied().fold([0.0; 3], add),
            1.0 / boundary.len() as f64,
        );
        center[1] = height;
        boundary.sort_by(|a, b| {
            (a[2] - center[2])
                .atan2(a[0] - center[0])
                .total_cmp(&(b[2] - center[2]).atan2(b[0] - center[0]))
        });
        for index in 0..boundary.len() {
            result.triangle(
                center,
                boundary[(index + 1) % boundary.len()],
                boundary[index],
                0,
            );
        }
        check_cut_precision(&result)?;
        Ok(result.positive())
    }
}

/// A real sub-resolution fragment is an error, not a candidate for snapping.
fn check_cut_precision(solid: &Solid) -> Result<(), String> {
    for float32 in [false, true] {
        let mut edges = BTreeMap::<[[u64; 3]; 2], Vec<bool>>::new();
        let key = |p: Point| p.map(|v| if v == 0.0 { 0 } else { v.to_bits() });
        for face in &solid.faces {
            let p =
                face.map(|i| solid.positions[i].map(|v| if float32 { v as f32 as f64 } else { v }));
            if magnitude(cross(sub(p[1], p[0]), sub(p[2], p[0]))) == 0.0 {
                return Err("receiving cut has an unresolved float32 surface".into());
            }
            for [a, b] in [[p[0], p[1]], [p[1], p[2]], [p[2], p[0]]] {
                let (a, b) = (key(a), key(b));
                edges
                    .entry(if a < b { [a, b] } else { [b, a] })
                    .or_default()
                    .push(a < b);
            }
        }
        if edges
            .values()
            .any(|uses| uses.len() != 2 || uses[0] == uses[1])
        {
            return Err("receiving cut cannot retain a closed float32 boundary".into());
        }
    }
    Ok(())
}
