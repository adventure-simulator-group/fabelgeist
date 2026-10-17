//! Boolean openings on a periodic cloth pattern, with fixed fit correspondence.
use crate::surface_cut::{
    CORRESPONDENCE_PRECISION, ConvexRegion, Plane, SourcePoint, SurfaceCut, interpolate,
};
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::SewnSleeves;
use std::collections::BTreeMap;

/// Samples per broad free-edge fold, independent of the body armhole density.
const FRILL_SAMPLES_PER_FOLD: usize = 8;

/// Native source-grid records and material coordinates for sewn sleeve points.
pub(crate) struct SleevePattern {
    pub table: Vec<u32>,
    pub grid: Vec<u32>,
    pub chart: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub opening: Option<crate::cloth_opening::ClothOpening>,
}
struct PatternGrid {
    points: Vec<[f32; 3]>,
    words: Vec<u32>,
    faces: Vec<[u32; 3]>,
    circumference: f32,
    reach: f32,
}
struct PatternRing {
    first: u32,
    arcs: Vec<f32>,
}
impl PatternGrid {
    /// The unwrapped grid is in metres: hem arc length and axial distance.
    fn new(angles: &[f32], progress: &[f32], reach: f32, design: &SewnSleeves) -> Self {
        let turn = std::f32::consts::TAU;
        let winding = (angles[1] - angles[0]).signum();
        let circumference = design.cuff_circumference.metres()
            + turn * design.frill.as_ref().map_or(0., |f| f.flare.metres());
        let original = angles
            .iter()
            .map(|&a| (a - angles[0]) * winding / turn)
            .chain([1.])
            .collect::<Vec<_>>();
        let refined = design
            .frill
            .as_ref()
            .and_then(|f| f.folds.as_ref())
            .map(|f| {
                let columns = usize::from(f.count.0) * FRILL_SAMPLES_PER_FOLD;
                (0..=columns)
                    .map(|i| i as f32 / columns as f32)
                    .collect::<Vec<_>>()
            });
        let mut points = Vec::new();
        let mut words = Vec::new();
        let mut rings = Vec::new();
        for (row, down) in std::iter::once(&0.).chain(progress).enumerate() {
            let arcs = if row >= progress.len() - 1 {
                refined.as_ref().unwrap_or(&original)
            } else {
                &original
            };
            rings.push(PatternRing {
                first: points.len() as u32,
                arcs: arcs.clone(),
            });
            for (column, &across) in arcs.iter().enumerate() {
                points.push([across * circumference, down * reach, 0.]);
                // Native grid records: source station, angle radians, original
                // root column. Refined distal points are fully released.
                words.extend([
                    row as u32,
                    (angles[0] + winding * turn * across).to_bits(),
                    (column % angles.len()) as u32,
                ]);
            }
        }
        let faces = rings.windows(2).flat_map(|r| r[0].stitch(&r[1])).collect();
        Self {
            points,
            words,
            faces,
            circumference,
            reach,
        }
    }
    fn cut(&self, angles: &[f32], design: &SewnSleeves) -> SurfaceCut {
        let include = vec![vec![
            Plane {
                normal: [-1., 0., 0.],
                offset: 0.,
            },
            Plane {
                normal: [1., 0., 0.],
                offset: self.circumference,
            },
            Plane {
                normal: [0., -1., 0.],
                offset: 0.,
            },
            Plane {
                normal: [0., 1., 0.],
                offset: self.reach,
            },
        ]];
        let mut subtract = Vec::<ConvexRegion>::new();
        if let Some(slit) = &design.cuff_slit {
            let turn = std::f32::consts::TAU;
            let winding = (angles[1] - angles[0]).signum();
            let center = ((slit.rotation.unit().rem_euclid(1.) * turn - angles[0]) * winding
                / turn)
                .rem_euclid(1.)
                * self.circumference;
            let tip = self.reach - slit.length.metres();
            let slope = slit.gap.metres() / (2. * slit.length.metres());
            for wrap in [-1., 0., 1.] {
                let center = center + wrap * self.circumference;
                subtract.push(vec![
                    Plane {
                        normal: [0., -1., 0.],
                        offset: -tip,
                    },
                    Plane {
                        normal: [1., -slope, 0.],
                        offset: center - slope * tip,
                    },
                    Plane {
                        normal: [-1., -slope, 0.],
                        offset: -center - slope * tip,
                    },
                ]);
            }
        }
        for plane in subtract.iter_mut().flatten() {
            let length = plane.normal.iter().map(|v| v * v).sum::<f32>().sqrt();
            plane.normal = plane.normal.map(|v| v / length);
            plane.offset /= length;
        }
        SurfaceCut::new(&self.points, &self.faces, &self.faces, &include, &subtract)
    }
}
impl PatternRing {
    fn stitch(&self, lower: &Self) -> Vec<[u32; 3]> {
        let mut faces = Vec::new();
        let (mut upper_column, mut lower_column) = (0, 0);
        while upper_column + 1 < self.arcs.len() || lower_column + 1 < lower.arcs.len() {
            let a = self.first + upper_column as u32;
            let b = lower.first + lower_column as u32;
            if lower_column + 1 == lower.arcs.len()
                || (upper_column + 1 < self.arcs.len()
                    && self.arcs[upper_column + 1] <= lower.arcs[lower_column + 1])
            {
                faces.push([a + 1, a, b]);
                upper_column += 1;
            } else {
                faces.push([a, b, b + 1]);
                lower_column += 1;
            }
        }
        faces
    }
}
impl SleevePattern {
    pub fn new(
        boundary: &[u32],
        angles: &[f32],
        progress: &[f32],
        reach: f32,
        design: &SewnSleeves,
        first: u32,
    ) -> Result<Self> {
        let grid = PatternGrid::new(angles, progress, reach, design);
        let cut = grid.cut(angles, design);
        let mut remap = Vec::new();
        let mut sources = Vec::<SourcePoint>::new();
        let mut chart = Vec::new();
        let mut welded = BTreeMap::new();
        let turn = std::f32::consts::TAU;
        let winding = (angles[1] - angles[0]).signum();
        for point in cut.points {
            let position = interpolate(
                grid.faces[point.triangle].map(|v| grid.points[v as usize]),
                point.weights,
            );
            let arc = if position[0].abs() <= 1. / CORRESPONDENCE_PRECISION
                || (position[0] - grid.circumference).abs() <= 1. / CORRESPONDENCE_PRECISION
            {
                0.
            } else {
                position[0]
            };
            let across = arc / grid.circumference;
            if position[1].abs() <= 1. / CORRESPONDENCE_PRECISION {
                let column = (0..angles.len())
                    .min_by(|&a, &b| {
                        let distance =
                            |i: usize| ((angles[i] - angles[0]) * winding / turn - across).abs();
                        distance(a).total_cmp(&distance(b))
                    })
                    .context("sleeve has no armhole columns")?;
                remap.push(boundary[column]);
                continue;
            }
            let key = [arc, position[1]].map(|v| (v * CORRESPONDENCE_PRECISION).round() as i64);
            let vertex = *welded.entry(key).or_insert_with(|| {
                let vertex = first + sources.len() as u32;
                sources.push(point);
                chart.push([
                    ((angles[0] + winding * turn * across) / turn).rem_euclid(1.),
                    position[1] / reach,
                ]);
                vertex
            });
            remap.push(vertex);
        }
        let indices = cut
            .faces
            .into_iter()
            .map(|f| f.map(|v| remap[v as usize]))
            .filter(|[a, b, c]| a != b && b != c && c != a)
            .flatten()
            .collect::<Vec<_>>();
        ensure!(!indices.is_empty(), "cuff opening removed the sleeve");
        let table = SurfaceCut {
            points: sources,
            faces: Vec::new(),
        }
        .source_table(&grid.faces, &grid.faces);
        let opening =
            crate::cloth_opening::ClothOpening::cuff(first, &chart, &indices, design, reach)?;
        Ok(Self {
            table,
            grid: grid.words,
            chart,
            indices,
            opening,
        })
    }
}
