//! Smooth enclosing lengthwise sections for formed foot plates.
const STATIONS: usize = 24;
const SLICE_WINDOW_M: f32 = 0.008;

pub(super) struct FootProfile {
    start: f32,
    end: f32,
    /// Lateral center, half width, and dorsal height in the foot frame.
    sections: Vec<[f32; 3]>,
}

impl FootProfile {
    pub fn new(points: &[[f32; 3]], start: f32, end: f32) -> Self {
        let mut sections = Vec::new();
        for i in 0..STATIONS {
            let z = start + (end - start) * i as f32 / (STATIONS - 1) as f32;
            let nearest = points
                .iter()
                .map(|p| (p[2] - z).abs())
                .fold(f32::INFINITY, f32::min);
            let slice = points
                .iter()
                .filter(|p| (p[2] - z).abs() <= nearest + SLICE_WINDOW_M)
                .collect::<Vec<_>>();
            let lo = slice.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
            let hi = slice.iter().map(|p| p[0]).fold(f32::NEG_INFINITY, f32::max);
            let top = slice.iter().map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max);
            sections.push([(lo + hi) * 0.5, (hi - lo) * 0.5, top]);
        }
        Self {
            start,
            end,
            sections,
        }
    }

    pub fn at(&self, z: f32) -> [f32; 3] {
        let station = ((z - self.start) / (self.end - self.start) * (STATIONS - 1) as f32)
            .clamp(0.0, (STATIONS - 1) as f32);
        let i = station.floor() as isize;
        let t = station - i as f32;
        // Cubic B-spline: continuous slope/curvature instead of raw skin slices.
        let weights = [
            (1.0 - t).powi(3),
            3.0 * t.powi(3) - 6.0 * t * t + 4.0,
            -3.0 * t.powi(3) + 3.0 * t * t + 3.0 * t + 1.0,
            t.powi(3),
        ]
        .map(|w| w / 6.0);
        std::array::from_fn(|axis| {
            weights
                .iter()
                .enumerate()
                .map(|(offset, w)| {
                    w * self.sections
                        [(i + offset as isize - 1).clamp(0, (STATIONS - 1) as isize) as usize][axis]
                })
                .sum()
        })
    }
}
