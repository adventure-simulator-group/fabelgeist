//! Anatomical section bounds for collar and bib measurements.
use anyhow::{Result, ensure};

const SECTION_HALF_BAND_M: f32 = 0.006;
const MAXIMUM_SECTION_HALF_BAND_M: f32 = 0.018;
pub(super) const MINIMUM_SECTION_SAMPLES: usize = 4;

#[derive(Clone, Copy)]
pub(super) struct Section {
    pub low: [f32; 3],
    pub high: [f32; 3],
}

impl Section {
    pub fn measure(samples: &[[f32; 3]], y: f32) -> Result<Self> {
        let mut nearby = samples
            .iter()
            .filter(|p| (p[1] - y).abs() <= MAXIMUM_SECTION_HALF_BAND_M)
            .collect::<Vec<_>>();
        nearby.sort_unstable_by(|a, b| (a[1] - y).abs().total_cmp(&(b[1] - y).abs()));
        ensure!(
            nearby.len() >= MINIMUM_SECTION_SAMPLES,
            "insufficient gorget section support at {y}m"
        );
        let count = nearby
            .iter()
            .take_while(|p| (p[1] - y).abs() <= SECTION_HALF_BAND_M)
            .count()
            .max(MINIMUM_SECTION_SAMPLES);
        let mut section = Self::empty();
        for point in &nearby[..count] {
            section.include(**point);
        }
        Ok(section)
    }

    /// Slice actual body triangles and clip each segment to the sagittal strip.
    /// Posterior support must not disappear when no mesh vertex falls near the
    /// plane; both anterior and posterior surface bounds define positive radii.
    pub fn sagittal_slice(
        samples: &[[f32; 3]],
        faces: &[[u32; 3]],
        y: f32,
        half_width: f32,
    ) -> Result<Self> {
        let mut section = Self::empty();
        let mut count = 0;
        for face in faces {
            let mut points = Vec::new();
            for (a, b) in [(0, 1), (1, 2), (2, 0)] {
                let a = samples[face[a] as usize];
                let b = samples[face[b] as usize];
                let da = a[1] - y;
                let db = b[1] - y;
                if (da > 0.0) == (db > 0.0) || da == db {
                    continue;
                }
                let t = da / (da - db);
                points.push(std::array::from_fn(|axis| {
                    a[axis] + (b[axis] - a[axis]) * t
                }));
            }
            let [a, b] = points.as_slice() else { continue };
            let Some([a, b]) = clip_sagittal(*a, *b, half_width) else {
                continue;
            };
            section.include(a);
            section.include(b);
            count += 2;
        }
        ensure!(
            count >= MINIMUM_SECTION_SAMPLES,
            "insufficient sagittal gorget surface support at {y}m"
        );
        Ok(section)
    }

    fn empty() -> Self {
        Self {
            low: [f32::INFINITY; 3],
            high: [f32::NEG_INFINITY; 3],
        }
    }

    fn include(&mut self, point: [f32; 3]) {
        for (axis, value) in point.into_iter().enumerate() {
            self.low[axis] = self.low[axis].min(value);
            self.high[axis] = self.high[axis].max(value);
        }
    }
}

fn clip_sagittal(a: [f32; 3], b: [f32; 3], half_width: f32) -> Option<[[f32; 3]; 2]> {
    let dx = b[0] - a[0];
    if dx == 0.0 {
        return (a[0].abs() <= half_width).then_some([a, b]);
    }
    let limits = [(-half_width - a[0]) / dx, (half_width - a[0]) / dx];
    let start = limits[0].min(limits[1]).max(0.0);
    let end = limits[0].max(limits[1]).min(1.0);
    (start <= end).then(|| {
        [start, end].map(|t| std::array::from_fn(|axis| a[axis] + (b[axis] - a[axis]) * t))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sagittal_slice_retains_sparse_posterior_surface_between_vertex_rows() {
        let mut points = Vec::new();
        let mut faces = Vec::new();
        for depth in [-0.1, 0.08] {
            let start = points.len() as u32;
            points.extend([
                [-0.08, -0.03, depth],
                [0.08, -0.03, depth],
                [0.08, 0.03, depth],
                [-0.08, 0.03, depth],
            ]);
            faces.extend([[start, start + 1, start + 2], [start, start + 2, start + 3]]);
        }
        for y in [-0.01, 0.0, 0.01] {
            let section = Section::sagittal_slice(&points, &faces, y, 0.016).unwrap();
            assert!((section.low[2] + 0.1).abs() < 1e-6);
            assert!((section.high[2] - 0.08).abs() < 1e-6);
            assert!((section.low[0] + 0.016).abs() < 1e-6);
            assert!((section.high[0] - 0.016).abs() < 1e-6);
        }
        assert!(Section::sagittal_slice(&points, &[], 0.0, 0.016).is_err());
    }

    #[test]
    fn sagittal_strip_rejects_unrelated_lateral_surfaces() {
        assert!(clip_sagittal([0.03, 0.0, -0.1], [0.08, 0.0, -0.1], 0.016).is_none());
        assert!(clip_sagittal([0.03, 0.0, -0.1], [0.03, 0.0, 0.1], 0.016).is_none());
    }

    #[test]
    fn sparse_section_uses_nearby_support_without_admitting_distant_anatomy() {
        let samples = [
            [-0.05, -0.008, 0.04],
            [0.05, -0.008, 0.04],
            [-0.05, 0.008, -0.04],
            [0.05, 0.008, -0.04],
            [0.20, 0.030, 0.0],
        ];
        let section = Section::measure(&samples, 0.0).unwrap();
        assert!((section.high[0] - 0.05).abs() < 1e-6);
        assert!((section.low[2] + 0.04).abs() < 1e-6);
        assert!(Section::measure(&samples, 0.10).is_err());
        assert!(Section::measure(&[], 0.04).is_err());
    }
}
