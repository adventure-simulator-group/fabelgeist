//! Filled curved ornament in a normalized repeating fabric tile.
//!
//! Cubic outlines describe lobed leaves rather than isolated ellipse stamps.
//! Branch count and curl shape the tile before the shared fabric compositor
//! applies region, repeat count, scale, rotation and stagger.
use std::f32::consts::TAU;

const TILE_SAMPLES: usize = 256;
const CURVE_SEGMENTS: usize = 16;

pub(crate) struct BotanicalTile {
    covered: Vec<bool>,
}

struct Outline {
    points: Vec<[f32; 2]>,
}

impl BotanicalTile {
    /// Native ornament boundary: a unit square centered at zero, with authored
    /// curve ratios independent of wearer dimensions or museum attribution.
    pub(crate) fn new(branches: u8, curl: f32) -> Self {
        let mut outlines = vec![vine()];
        for branch in 0..branches {
            let level = (f32::from(branch) + 0.5) / f32::from(branches) - 0.5;
            let height = 1.0 / f32::from(branches);
            for side in [-1.0, 1.0] {
                let mut leaf = leaf(curl);
                for point in &mut leaf.points {
                    point[0] *= side;
                    point[1] = point[1] * height + level;
                }
                outlines.push(leaf);
            }
        }
        let mut covered = Vec::with_capacity(TILE_SAMPLES * TILE_SAMPLES);
        for row in 0..TILE_SAMPLES {
            for column in 0..TILE_SAMPLES {
                let point = [
                    (column as f32 + 0.5) / TILE_SAMPLES as f32 - 0.5,
                    (row as f32 + 0.5) / TILE_SAMPLES as f32 - 0.5,
                ];
                covered.push(outlines.iter().any(|outline| outline.contains(point)));
            }
        }
        Self { covered }
    }

    pub(crate) fn covers(&self, point: [f32; 2]) -> bool {
        if point.iter().any(|value| !(-0.5..0.5).contains(value)) {
            return false;
        }
        let [column, row] = point.map(|value| {
            // The last admitted f32 below +0.5 can round onto the upper bin.
            (((value + 0.5) * TILE_SAMPLES as f32) as usize).min(TILE_SAMPLES - 1)
        });
        self.covered[row * TILE_SAMPLES + column]
    }
}

impl Outline {
    fn new(start: [f32; 2]) -> Self {
        Self {
            points: vec![start],
        }
    }

    fn curve(&mut self, first: [f32; 2], second: [f32; 2], end: [f32; 2]) {
        let start = self.points[self.points.len() - 1];
        for segment in 1..=CURVE_SEGMENTS {
            let t = segment as f32 / CURVE_SEGMENTS as f32;
            let s = 1.0 - t;
            self.points.push(std::array::from_fn(|axis| {
                s.powi(3) * start[axis]
                    + 3.0 * s * s * t * first[axis]
                    + 3.0 * s * t * t * second[axis]
                    + t.powi(3) * end[axis]
            }));
        }
    }

    fn contains(&self, point: [f32; 2]) -> bool {
        let mut inside = false;
        let mut previous = self.points[self.points.len() - 1];
        for &next in &self.points {
            if (previous[1] > point[1]) != (next[1] > point[1]) {
                let crossing = (next[0] - previous[0]) * (point[1] - previous[1])
                    / (next[1] - previous[1])
                    + previous[0];
                if point[0] < crossing {
                    inside = !inside;
                }
            }
            previous = next;
        }
        inside
    }
}

fn vine() -> Outline {
    let mut outline = Outline::new([-0.012, -0.5]);
    // Opposite sides of the same continuous undulating stem meet the next tile.
    for side in [1.0, -1.0] {
        for step in 0..=CURVE_SEGMENTS * 2 {
            let t = step as f32 / (CURVE_SEGMENTS * 2) as f32;
            let y = if side > 0.0 { t - 0.5 } else { 0.5 - t };
            outline
                .points
                .push([0.025 * (y * TAU).sin() + side * 0.012, y]);
        }
    }
    outline
}

/// A broad acanthus-like blade with a curled tip and three deep lobes.
/// These normalized control points are authored ornament, not a historical
/// attribution; recipes choose their visual scale and arrangement.
fn leaf(curl: f32) -> Outline {
    let mut outline = Outline::new([0.0, -0.12]);
    outline.curve([0.12, -0.12], [0.22, -0.38], [0.39, -0.32]);
    outline.curve([0.56, -0.25], [0.48, 0.02], [0.32, -0.06]);
    outline.curve([0.22, -0.14], [0.32, -0.27 * curl], [0.39, -0.18]);
    outline.curve([0.42, -0.10], [0.33, -0.13], [0.31, -0.16]);
    outline.curve([0.24, -0.06], [0.29, 0.16], [0.40, 0.20]);
    outline.curve([0.26, 0.24], [0.20, 0.12], [0.22, 0.07]);
    outline.curve([0.16, 0.14], [0.22, 0.32], [0.29, 0.36]);
    outline.curve([0.14, 0.37], [0.10, 0.18], [0.12, 0.11]);
    outline.curve([0.07, 0.15], [0.12, 0.30], [0.15, 0.34]);
    outline.curve([-0.02, 0.24], [0.07, 0.04], [0.0, -0.12]);
    outline
}
