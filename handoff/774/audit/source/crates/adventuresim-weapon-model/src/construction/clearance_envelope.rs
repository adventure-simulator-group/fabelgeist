//! Convex section envelopes with clearance to sloped supporting faces.
use super::*;
use std::collections::BTreeMap;
pub(crate) struct ClearanceEnvelope {
    levels: Vec<(f64, Vec<f64>)>,
    normals: Vec<PlanarPoint>,
    slope_factors: Vec<f64>,
}
impl ClearanceEnvelope {
    pub(crate) fn new(points: &[Point], sides: usize) -> Result<Self, String> {
        if sides < 3 {
            return Err("clearance envelope needs three supporting planes".into());
        }
        let mut groups = BTreeMap::<i64, Vec<Point>>::new();
        for &point in points {
            groups
                .entry((point[1] * 1e9).round() as i64)
                .or_default()
                .push(point);
        }
        if groups.len() < 2 {
            return Err("clearance envelope needs two axial sections".into());
        }
        let normals: Vec<_> = (0..sides)
            .map(|i| {
                let a = i as f64 / sides as f64 * std::f64::consts::TAU;
                [a.cos(), a.sin()]
            })
            .collect();
        let levels: Vec<(f64, Vec<f64>)> = groups
            .into_values()
            .map(|points| {
                (
                    points[0][1],
                    normals
                        .iter()
                        .map(|n| {
                            points
                                .iter()
                                .map(|p| n[0] * p[0] + n[1] * p[2])
                                .fold(f64::NEG_INFINITY, f64::max)
                        })
                        .collect(),
                )
            })
            .collect();
        let mut slope_factors = vec![1.0_f64; sides];
        for pair in levels.windows(2) {
            let length = pair[1].0 - pair[0].0;
            for (i, factor) in slope_factors.iter_mut().enumerate() {
                let slope = (pair[1].1[i] - pair[0].1[i]) / length;
                *factor = factor.max((1.0 + slope * slope).sqrt());
            }
        }
        // One isotropic offset preserves convexity between supporting planes.
        // Its slope correction bounds the distance to every longitudinal face.
        let maximum = slope_factors.iter().copied().fold(1.0_f64, f64::max);
        slope_factors.fill(maximum);
        Ok(Self {
            levels,
            normals,
            slope_factors,
        })
    }
    pub(crate) fn extent(&self) -> [f64; 2] {
        [self.levels[0].0, self.levels.last().unwrap().0]
    }
    pub(crate) fn heights(&self, lower: f64, upper: f64) -> Vec<f64> {
        let mut ys = vec![lower, upper];
        ys.extend(
            self.levels
                .iter()
                .map(|l| l.0)
                .filter(|y| *y > lower && *y < upper),
        );
        ys.sort_by(f64::total_cmp);
        ys.dedup();
        ys
    }
    fn supports(&self, y: f64) -> Vec<f64> {
        for pair in self.levels.windows(2) {
            if y >= pair[0].0 && y <= pair[1].0 {
                let t = (y - pair[0].0) / (pair[1].0 - pair[0].0);
                return pair[0]
                    .1
                    .iter()
                    .zip(&pair[1].1)
                    .map(|(a, b)| a * (1.0 - t) + b * t)
                    .collect();
            }
        }
        if y < self.levels[0].0 {
            self.levels[0].1.clone()
        } else {
            self.levels.last().unwrap().1.clone()
        }
    }
    pub(crate) fn ring(&self, y: f64, gap: f64) -> Vec<Point> {
        let supports: Vec<_> = self
            .supports(y)
            .iter()
            .zip(&self.slope_factors)
            .map(|(h, s)| h + gap * s)
            .collect();
        (0..self.normals.len())
            .map(|i| {
                let j = (i + 1) % self.normals.len();
                let [a, b] = self.normals[i];
                let [c, d] = self.normals[j];
                let det = a * d - b * c;
                [
                    (supports[i] * d - b * supports[j]) / det,
                    y,
                    (a * supports[j] - supports[i] * c) / det,
                ]
            })
            .collect()
    }
    pub(crate) fn shell(
        &self,
        lower: f64,
        upper: f64,
        gap: f64,
        wall: f64,
        end: LoftEnd,
    ) -> Result<Solid, String> {
        let heights = self.heights(lower, upper);
        let inner: Vec<_> = heights.iter().map(|&y| self.ring(y, gap)).collect();
        let mut outer: Vec<_> = heights.iter().map(|&y| self.ring(y, gap + wall)).collect();
        if matches!(end, LoftEnd::Closed) {
            for p in outer.last_mut().unwrap() {
                p[1] += wall;
            }
        }
        Solid::section_shell(&inner, &outer, end)
    }
}
