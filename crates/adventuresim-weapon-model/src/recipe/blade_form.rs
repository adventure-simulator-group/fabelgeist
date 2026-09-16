//! Dimensioned recessed sections and independent terminal point geometry.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FullerFaces {
    Front,
    Back,
    Both,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FullerParameters {
    pub faces: FullerFaces,
    pub mouth_width: Metres,
    pub depth: Metres,
    pub floor_width_ratio: Ratio,
    pub bevel_width_ratio: Ratio,
    pub start: Metres,
    pub end: Metres,
    pub entry_length: Metres,
    pub exit_length: Metres,
}
impl FullerParameters {
    pub(crate) fn envelope(&self, y: f64) -> f64 {
        if y <= self.start.get() || y >= self.end.get() {
            return 0.0;
        }
        fn smooth(u: f64) -> f64 {
            let u = u.clamp(0.0, 1.0);
            u * u * u * (10.0 + u * (-15.0 + 6.0 * u))
        }
        smooth((y - self.start.get()) / self.entry_length.get())
            * smooth((self.end.get() - y) / self.exit_length.get())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BladePoint {
    /// Position within the working blade, after its ricasso.
    pub start: Ratio,
    pub roundness: Ratio,
}

/// Geometric construction of a monotone point in (axial, half-width) space.
pub(crate) struct PointCurve {
    points: [[f64; 2]; 4],
}
impl PointCurve {
    /// Each Bernstein term gives w >= slope * (tip_y-y), including a rounded
    /// endpoint whose width approaches zero more slowly than axial distance.
    pub(crate) fn minimum_width_slope(&self) -> f64 {
        (self.points[0][1] / (self.points[3][0] - self.points[0][0]))
            .min(self.points[1][1] / (self.points[3][0] - self.points[1][0]))
    }
    pub(crate) fn new(
        y: f64,
        end: f64,
        width: f64,
        slope: f64,
        roundness: f64,
    ) -> Result<Self, String> {
        if !(y < end
            && width > 0.0
            && slope.is_finite()
            && slope <= 0.0
            && (0.0..=1.0).contains(&roundness))
        {
            return Err("point needs a positive monotonically tapering body section".into());
        }
        let k = roundness * width / 3.0;
        let h = if slope < 0.0 {
            ((end - y) / 3.0).min((width - k) / (-2.0 * slope))
        } else {
            (end - y) / 3.0
        };
        Ok(Self {
            points: [[y, width], [y + h, width + slope * h], [end, k], [end, 0.0]],
        })
    }
    pub(crate) fn at(&self, u: f64) -> [f64; 2] {
        let v = 1.0 - u;
        let weights = [v * v * v, 3.0 * v * v * u, 3.0 * v * u * u, u * u * u];
        std::array::from_fn(|axis| {
            self.points
                .iter()
                .zip(weights)
                .map(|(p, w)| p[axis] * w)
                .sum()
        })
    }
    pub(crate) fn width_at(&self, y: f64) -> f64 {
        if y <= self.points[0][0] {
            return self.points[0][1];
        }
        if y >= self.points[3][0] {
            return 0.0;
        }
        let (mut lo, mut hi) = (0.0, 1.0);
        for _ in 0..52 {
            let mid = (lo + hi) / 2.0;
            if self.at(mid)[0] < y {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        self.at((lo + hi) / 2.0)[1]
    }
}
