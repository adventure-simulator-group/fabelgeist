//! Monotone cubic interpolation of dimensioned axial profiles.
use super::*;

/// Each cubic stays between its two station values. Zero secants and changes
/// of direction have zero derivative; other interior derivatives use the
/// weighted harmonic mean, so a supplied waist cannot acquire overshoot.
pub(crate) struct SmoothProfile {
    points: Vec<PlanarPoint>,
    slopes: Vec<f64>,
}
impl SmoothProfile {
    pub(crate) fn new(points: Vec<PlanarPoint>) -> Result<Self, String> {
        if points.len() < 2
            || points.iter().flatten().any(|x| !x.is_finite())
            || points.windows(2).any(|p| p[0][0] >= p[1][0])
        {
            return Err("smooth profile needs finite increasing stations".into());
        }
        let intervals: Vec<_> = points.windows(2).map(|p| p[1][0] - p[0][0]).collect();
        let secants: Vec<_> = points
            .windows(2)
            .map(|p| (p[1][1] - p[0][1]) / (p[1][0] - p[0][0]))
            .collect();
        let mut slopes = vec![0.0; points.len()];
        slopes[0] = secants[0];
        slopes[points.len() - 1] = *secants.last().unwrap();
        for i in 1..points.len() - 1 {
            let (a, b) = (secants[i - 1], secants[i]);
            if a * b > 0.0 {
                let wa = 2.0 * intervals[i] + intervals[i - 1];
                let wb = intervals[i] + 2.0 * intervals[i - 1];
                slopes[i] = (wa + wb) / (wa / a + wb / b);
            }
        }
        Ok(Self { points, slopes })
    }
    pub(crate) fn value(&self, x: f64) -> f64 {
        let i = self
            .points
            .partition_point(|p| p[0] <= x)
            .saturating_sub(1)
            .min(self.points.len() - 2);
        let [x0, y0] = self.points[i];
        let [x1, y1] = self.points[i + 1];
        let h = x1 - x0;
        let t = ((x - x0) / h).clamp(0.0, 1.0);
        let t2 = t * t;
        let t3 = t2 * t;
        (2.0 * t3 - 3.0 * t2 + 1.0) * y0
            + (t3 - 2.0 * t2 + t) * h * self.slopes[i]
            + (-2.0 * t3 + 3.0 * t2) * y1
            + (t3 - t2) * h * self.slopes[i + 1]
    }
}
