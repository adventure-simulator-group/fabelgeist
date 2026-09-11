//! Bowl and crest share one carrier; flute meridians radiate from the temples.
use super::geometry::{AROUND, Surface};
use crate::HelmetCrown;
use std::f32::consts::{FRAC_PI_2, PI};

struct CrownChart<'a> {
    radii: [f32; 3],
    brow: f32,
    style: &'a HelmetCrown,
    crest: f32,
}
impl CrownChart<'_> {
    fn latitudes(&self) -> Vec<(f32, usize)> {
        let mut values = (1..AROUND / 2)
            .map(|row| (row as f32 / (AROUND / 2) as f32 * PI, row))
            .collect::<Vec<_>>();
        if self.crest > 0.0 {
            // Extra columns fit inside the two central regular intervals for every head.
            for width in [self.half_width(), self.base_width()] {
                let angle = (width / self.radii[0]).asin();
                values.extend([
                    (FRAC_PI_2 - angle, AROUND / 4),
                    (FRAC_PI_2 + angle, AROUND / 4),
                ]);
            }
            values.sort_by(|a, b| a.0.total_cmp(&b.0));
        }
        values
    }
    fn half_width(&self) -> f32 {
        (self.radii[0] * 0.06).min(0.005)
    }
    fn base_width(&self) -> f32 {
        (self.radii[0] * 0.12).min(0.010)
    }
    fn point(&self, latitude: f32, angle: f32) -> [f32; 3] {
        let y = latitude.sin() * angle.sin();
        let radial = (1.0 - y * y).max(0.0).sqrt();
        let fullness = if radial > 1e-6 {
            radial.powf(self.style.fullness.unit() - 1.0)
        } else {
            1.0
        };
        let x = self.radii[0] * latitude.cos();
        let across = if self.crest > 0.0 {
            ((self.base_width() - x.abs()) / (self.base_width() - self.half_width()))
                .clamp(0.0, 1.0)
        } else {
            0.0
        };
        let crest_phase = ((angle / PI - 0.08) / 0.84).clamp(0.0, 1.0);
        let rise = (PI * crest_phase).sin().max(0.0).powf(0.8);
        let crest = self.crest * across * rise;
        [
            x * (fullness + (1.0 - fullness) * across * rise),
            self.brow + (self.radii[1] - self.brow) * y + crest,
            self.radii[2] * latitude.sin() * angle.cos() * fullness,
        ]
    }
}

impl Surface {
    pub(super) fn styled_dome(
        &mut self,
        radii: [f32; 3],
        brow: f32,
        style: &HelmetCrown,
        crest: f32,
    ) -> Vec<u32> {
        let start = self.positions.len();
        let first_index = self.indices.len();
        let fan = style.fluting.is_some() || crest > 0.0;
        let rim = if fan {
            self.fan_dome(&CrownChart {
                radii,
                brow,
                style,
                crest,
            })
        } else {
            self.full_dome(radii, brow, style.fullness.unit())
        };
        if fan {
            for face in self.indices[first_index..].as_chunks_mut::<3>().0 {
                face.swap(1, 2);
            }
        }
        for index in start..self.positions.len() {
            let p = self.positions[index];
            let rise = ((p[1] - brow) / (radii[1] - brow)).clamp(0.0, 1.0);
            let medial = (1.0 - (p[0] / (radii[0] * 0.30)).abs()).max(0.0);
            self.relief[index] += style.ridge_height.metres() * medial.powi(2) * rise;
        }
        rim
    }
    fn fan_dome(&mut self, chart: &CrownChart<'_>) -> Vec<u32> {
        let pattern = chart.style.fluting.as_ref();
        let columns = pattern.map_or_else(
            || (0..=40).map(|i| i as f32 / 40.0).collect(),
            |p| p.columns(40),
        );
        let latitudes = chart.latitudes();
        let rim = (0..AROUND)
            .map(|i| {
                let angle = i as f32 / AROUND as f32 * 2.0 * PI;
                self.vertex([
                    chart.radii[0] * angle.sin(),
                    chart.brow,
                    chart.radii[2] * angle.cos(),
                ])
            })
            .collect::<Vec<_>>();
        let first = rim[AROUND / 4];
        let mut rings: Vec<Vec<u32>> = Vec::new();
        for (latitude, rim_row) in &latitudes {
            let v = latitude.sin();
            let mut ring = Vec::new();
            for (column, u) in columns.iter().enumerate() {
                let angle = pattern.map_or(*u, |p| p.fan_coordinate(*u, v)) * PI;
                let front = (AROUND + AROUND / 4 - rim_row) % AROUND;
                let id = if column == 0 {
                    rim[front]
                } else if column + 1 == columns.len() {
                    rim[(AROUND + AROUND / 2 - front) % AROUND]
                } else {
                    self.vertex(chart.point(*latitude, angle))
                };
                self.relief[id as usize] = pattern.map_or(0.0, |p| p.relief(*u, v));
                ring.push(id);
            }
            if let Some(previous) = rings.last() {
                for i in 0..ring.len() - 1 {
                    for triangle in [
                        [previous[i], ring[i], ring[i + 1]],
                        [previous[i], ring[i + 1], previous[i + 1]],
                    ] {
                        if triangle[0] != triangle[1]
                            && triangle[1] != triangle[2]
                            && triangle[2] != triangle[0]
                        {
                            self.indices.extend(triangle);
                        }
                    }
                }
            } else {
                for pair in ring.windows(2) {
                    self.indices.extend([first, pair[0], pair[1]]);
                }
            }
            rings.push(ring);
        }
        let last = rim[AROUND * 3 / 4];
        for pair in rings.last().expect("crown rings").windows(2) {
            self.indices.extend([pair[0], last, pair[1]]);
        }
        rim
    }
}
