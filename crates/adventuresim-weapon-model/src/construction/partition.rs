//! Shared-edge planar partitions retain explicit ridges in a lifted surface.
use super::{PlanarPoint, Region, construction_budget};
use std::collections::BTreeMap;

/// Classification only absorbs arithmetic rounding at an existing cut plane.
const PARTITION_ROUNDOFF_ULPS: f64 = 64.0;

#[derive(Clone, Copy)]
pub(crate) enum PlanarCut {
    Axial(f64),
    Transverse {
        start: PlanarPoint,
        end: PlanarPoint,
    },
}

impl PlanarCut {
    pub(super) fn ordinate(self, point: PlanarPoint) -> (usize, f64) {
        match self {
            Self::Axial(y) => (1, y),
            Self::Transverse { start, end } => (
                0,
                start[0] + (end[0] - start[0]) * (point[1] - start[1]) / (end[1] - start[1]),
            ),
        }
    }
    pub(super) fn distance(self, point: PlanarPoint) -> f64 {
        let (axis, value) = self.ordinate(point);
        point[axis] - value
    }
    pub(super) fn active(self, points: &[PlanarPoint]) -> bool {
        match self {
            Self::Axial(_) => true,
            Self::Transverse { start, end } => {
                points.iter().all(|p| p[1] >= start[1] && p[1] <= end[1])
            }
        }
    }
}

impl Region {
    pub(crate) fn partition(&mut self, cut: PlanarCut) -> Result<(), String> {
        for point in &mut self.points {
            if !cut.active(&[*point]) {
                continue;
            }
            let (axis, value) = cut.ordinate(*point);
            let scale = point.iter().map(|v| v.abs()).fold(value.abs(), f64::max);
            if (point[axis] - value).abs() <= PARTITION_ROUNDOFF_ULPS * f64::EPSILON * scale {
                point[axis] = value;
            }
        }
        let mut crossings = BTreeMap::new();
        let mut next = Vec::new();
        for face in self.triangles.clone() {
            if !cut.active(&face.map(|i| self.points[i])) {
                next.push(face);
                continue;
            }
            let mut polygons = [Vec::new(), Vec::new()];
            for i in 0..3 {
                let a = face[i];
                let b = face[(i + 1) % 3];
                let da = cut.distance(self.points[a]);
                let db = cut.distance(self.points[b]);
                if da <= 0.0 {
                    polygons[0].push(a);
                }
                if da >= 0.0 {
                    polygons[1].push(a);
                }
                if (da < 0.0 && db > 0.0) || (da > 0.0 && db < 0.0) {
                    if da.abs().min(db.abs()) < crate::recipe::MIN_MANUFACTURED_METRES {
                        return Err("plate cut leaves a sub-resolution feature".into());
                    }
                    let key = (a.min(b), a.max(b));
                    let index = *crossings.entry(key).or_insert_with(|| {
                        let [a, b] = [self.points[key.0], self.points[key.1]];
                        let t = cut.distance(a) / (cut.distance(a) - cut.distance(b));
                        let mut point: PlanarPoint =
                            std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t);
                        let (axis, value) = cut.ordinate(point);
                        point[axis] = value;
                        self.points.push(point);
                        self.points.len() - 1
                    });
                    polygons[0].push(index);
                    polygons[1].push(index);
                }
            }
            for polygon in polygons {
                for i in 1..polygon.len().saturating_sub(1) {
                    next.push([polygon[0], polygon[i], polygon[i + 1]]);
                }
            }
        }
        construction_budget((next.len() * 4) as f64)?;
        self.triangles = next;
        // A cut ending on a station edge also subdivides its neighboring cell.
        // Both cells must reference the same intersection before later refinement.
        self.split_edges(&crossings);
        Ok(())
    }
}
