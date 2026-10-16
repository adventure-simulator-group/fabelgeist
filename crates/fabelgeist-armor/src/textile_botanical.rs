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
    cutouts: Vec<Outline>,
}

impl BotanicalTile {
    /// Native ornament boundary: a unit square centered at zero, with authored
    /// curve ratios independent of wearer dimensions or museum attribution.
    pub(crate) fn new(branches: u8, curl: f32) -> Self {
        let mut outlines = vec![vine(curl)];
        for branch in 0..branches {
            let level = (f32::from(branch) + 0.5) / f32::from(branches) - 0.5;
            let height = 1.0 / f32::from(branches);
            for side in [-1.0, 1.0] {
                let mut leaf = leaf(curl);
                let angle = side * (0.4 + level * 0.5);
                let (sin, cos) = angle.sin_cos();
                let root = level + side * height * 0.2;
                for point in &mut leaf.points {
                    let x = point[0] * side;
                    let y = point[1];
                    point[0] = (x * cos - y * sin) * 0.85 + vine_x(root, curl);
                    point[1] = (x * sin + y * cos) * height + root;
                }
                outlines.push(leaf);
            }
            outlines.push(blossom([vine_x(level, curl), level], height));
        }
        Self::from_outlines(outlines)
    }

    /// A nested central flower and paired lobed foliage grow from a curved
    /// stem. Ratios are authored tile geometry, independent of garment
    /// dimensions; count and spread vary the construction before repetition.
    pub(crate) fn damask(pairs: u8, spread: f32) -> Self {
        let mut outlines = vec![vine(0.85)];
        let mut flower = Outline::new([0., -0.43]);
        flower.curve([-0.04, -0.33], [-0.20, -0.29], [-0.11, -0.13]);
        flower.curve([-0.10, -0.04], [0., 0.02], [0., 0.14]);
        flower.curve([0., 0.02], [0.10, -0.04], [0.11, -0.13]);
        flower.curve([0.20, -0.29], [0.04, -0.33], [0., -0.43]);
        let mut heart = Outline::new([0., -0.35]);
        heart.curve([-0.03, -0.26], [-0.11, -0.22], [-0.06, -0.13]);
        heart.curve([-0.04, -0.07], [0., -0.04], [0., 0.02]);
        heart.curve([0., -0.04], [0.04, -0.07], [0.06, -0.13]);
        heart.curve([0.11, -0.22], [0.03, -0.26], [0., -0.35]);
        flower.cutouts.push(heart);
        outlines.push(flower);
        for pair in 0..pairs {
            let level = f32::from(pair) / f32::from(pairs);
            for side in [-1.0, 1.0] {
                let mut blade = leaf(0.85);
                let mut vein = leaf(0.85);
                for point in &mut vein.points {
                    point[0] = 0.25 + (point[0] - 0.25) * 0.6;
                    point[1] *= 0.6;
                }
                blade.cutouts.push(vein);
                let root = 0.3 - level * 0.6;
                let reach = (0.68 + 0.1 * level) * spread;
                for contour in std::iter::once(&mut blade.points)
                    .chain(blade.cutouts.iter_mut().map(|cut| &mut cut.points))
                {
                    for point in contour {
                        point[0] = point[0] * side * reach + vine_x(root, 0.85);
                        point[1] = root + point[1] * (0.95 / f32::from(pairs));
                    }
                }
                outlines.push(blade);
            }
        }
        Self::from_outlines(outlines)
    }

    fn from_outlines(outlines: Vec<Outline>) -> Self {
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
            cutouts: Vec::new(),
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
        inside && !self.cutouts.iter().any(|cut| cut.contains(point))
    }
}

fn vine_x(y: f32, curl: f32) -> f32 {
    (0.1 + 0.12 * curl) * (y * TAU).sin()
}

fn vine(curl: f32) -> Outline {
    let mut outline = Outline::new([-0.012, -0.5]);
    // Opposite sides of the same continuous undulating stem meet the next tile.
    for side in [1.0, -1.0] {
        for step in 0..=CURVE_SEGMENTS * 2 {
            let t = step as f32 / (CURVE_SEGMENTS * 2) as f32;
            let y = if side > 0.0 { t - 0.5 } else { 0.5 - t };
            outline.points.push([vine_x(y, curl) + side * 0.016, y]);
        }
    }
    outline
}

fn blossom(centre: [f32; 2], height: f32) -> Outline {
    let mut outline = Outline::new([centre[0] + 0.075, centre[1]]);
    for step in 0..=CURVE_SEGMENTS * 2 {
        let angle = step as f32 / (CURVE_SEGMENTS * 2) as f32 * TAU;
        let radius = 0.075 + 0.018 * (angle * 5.0).cos();
        outline.points.push([
            centre[0] + angle.cos() * radius,
            centre[1] + angle.sin() * radius * height,
        ]);
    }
    outline
}

/// A broad acanthus-like blade with a coiled tip and rounded hooked lobes.
/// These normalized control points are authored ornament, not a historical
/// attribution; recipes choose their visual scale and arrangement.
fn leaf(curl: f32) -> Outline {
    let mut outline = Outline::new([0.02, -0.13]);
    outline.curve([0.12, -0.16], [0.09, -0.33], [0.28, -0.37]);
    outline.curve([0.49, -0.42], [0.59, -0.15], [0.45, -0.04]);
    outline.curve([0.28, 0.10], [0.16, -0.10], [0.28, -0.22]);
    outline.curve([0.39, -0.30 * curl], [0.48, -0.17], [0.37, -0.12]);
    outline.curve([0.32, -0.10], [0.30, -0.16], [0.33, -0.17]);
    outline.curve([0.35, -0.21], [0.29, -0.22], [0.26, -0.17]);
    outline.curve([0.16, -0.01], [0.34, 0.10], [0.42, 0.05]);
    outline.curve([0.44, 0.16], [0.24, 0.21], [0.16, 0.08]);
    outline.curve([0.16, 0.19], [0.34, 0.19], [0.34, 0.30]);
    outline.curve([0.22, 0.36], [0.08, 0.24], [0.10, 0.12]);
    outline.curve([0.02, 0.19], [0.12, 0.33], [0.05, 0.36]);
    outline.curve([-0.09, 0.24], [0.07, 0.03], [0.02, -0.13]);
    outline
}
