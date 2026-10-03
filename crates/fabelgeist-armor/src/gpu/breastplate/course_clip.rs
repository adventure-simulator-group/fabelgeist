//! Cut course boundaries through evaluated carrier facets without resampling.
use std::collections::BTreeMap;

use super::construction_columns::ConstructionColumns;
use super::course_coordinates::CourseCoordinates;
use super::cut_frame::CourseFrame;
use super::cut_resolution::CUT_RESOLUTION_METERS;
use super::topology::MidTopology;
use crate::GenerateError;

/// The authored medial rail must separate the two chevron half-planes.
const MEDIAL_RAIL_TOLERANCE_METERS: f64 = 1e-6;

#[derive(Clone, Copy)]
pub(super) struct SourceSample {
    pub edge: [u32; 2],
    pub blend: f32,
}

impl SourceSample {
    fn vertex(vertex: u32) -> Self {
        Self {
            edge: [vertex; 2],
            blend: 0.0,
        }
    }
}

pub(super) struct CourseClip {
    pub mid: MidTopology,
    pub samples: Vec<SourceSample>,
    points: Vec<[f32; 3]>,
    levels: Vec<f64>,
    cuts: BTreeMap<([u32; 2], bool), u32>,
    cut_candidates: Vec<[bool; 2]>,
}

impl CourseClip {
    pub fn new(
        source: &MidTopology,
        points: Vec<[f32; 3]>,
        frame: &CourseFrame,
        slope: f32,
        low: f64,
        high: Option<f64>,
    ) -> Result<Self, GenerateError> {
        let mut mid = source.clone();
        mid.skirt = false;
        mid.faces.clear();
        let samples = (0..points.len() as u32).map(SourceSample::vertex).collect();
        let levels = points
            .iter()
            .map(|&p| frame.course_level(p, slope))
            .collect();
        let cut_candidates = vec![[false; 2]; points.len()];
        let mut clip = Self {
            mid,
            samples,
            levels,
            points,
            cuts: BTreeMap::new(),
            cut_candidates,
        };
        for &face in &source.faces {
            let lateral = face.map(|i| frame.lateral_coordinate(clip.points[i as usize]));
            if lateral.iter().any(|&x| x < -MEDIAL_RAIL_TOLERANCE_METERS)
                && lateral.iter().any(|&x| x > MEDIAL_RAIL_TOLERANCE_METERS)
            {
                return Err(GenerateError::InvalidSurface);
            }
            let polygon = clip.cut(&face, low, false)?;
            let polygon = if let Some(high) = high {
                clip.cut(&polygon, high, true)?
            } else {
                polygon
            };
            for i in 1..polygon.len().saturating_sub(1) {
                clip.mid
                    .faces
                    .push([polygon[0], polygon[i], polygon[i + 1]]);
            }
        }
        if clip.mid.faces.is_empty() {
            return Err(GenerateError::Degenerate);
        }
        clip.mid.skirt_face_start = clip.mid.faces.len();
        let regular = clip.mid.rows * clip.mid.width();
        clip.mid.cut_columns = clip.samples[regular..]
            .iter()
            .map(|sample| {
                crate::SurfaceColumn::between(
                    source.surface_column(sample.edge[0] as usize),
                    source.surface_column(sample.edge[1] as usize),
                    sample.blend,
                )
            })
            .collect();
        clip.install_crease();
        clip.install_grid(low, high)?;
        clip.mid.course_coordinates = Some(CourseCoordinates::new(
            clip.levels.clone(),
            clip.cut_candidates.clone(),
            low,
            high,
            slope,
        ));
        Ok(clip)
    }

    fn cut(
        &mut self,
        polygon: &[u32],
        boundary: f64,
        upper: bool,
    ) -> Result<Vec<u32>, GenerateError> {
        let end = usize::from(!upper);
        for &vertex in polygon {
            if (self.levels[vertex as usize] - boundary).abs() <= CUT_RESOLUTION_METERS {
                self.cut_candidates[vertex as usize][end] = true;
            }
        }
        let mut result = Vec::with_capacity(polygon.len() + 1);
        for i in 0..polygon.len() {
            let a = polygon[i];
            let b = polygon[(i + 1) % polygon.len()];
            let distance = |vertex: u32| {
                let distance = self.levels[vertex as usize] - boundary;
                let distance = if distance.abs() <= CUT_RESOLUTION_METERS {
                    0.0
                } else {
                    distance
                };
                if upper { distance } else { -distance }
            };
            let [da, db] = [distance(a), distance(b)];
            if da <= 0.0 {
                result.push(a);
            }
            if (da < 0.0 && db > 0.0) || (da > 0.0 && db < 0.0) {
                result.push(self.intersection(a, b, boundary, upper)?);
            }
        }
        result.dedup();
        if result.len() > 1 && result.first() == result.last() {
            result.pop();
        }
        Ok(result)
    }

    fn intersection(
        &mut self,
        a: u32,
        b: u32,
        boundary: f64,
        upper: bool,
    ) -> Result<u32, GenerateError> {
        let mut sources = self.samples[a as usize]
            .edge
            .into_iter()
            .chain(self.samples[b as usize].edge)
            .collect::<Vec<_>>();
        sources.sort_unstable();
        sources.dedup();
        let [a, b] = *sources.as_slice() else {
            return Err(GenerateError::InvalidSurface);
        };
        let edge = [a, b];
        if let Some(&vertex) = self.cuts.get(&(edge, upper)) {
            return Ok(vertex);
        }
        let blend = ((boundary - self.levels[a as usize])
            / (self.levels[b as usize] - self.levels[a as usize])) as f32;
        let point = std::array::from_fn(|k| {
            self.points[a as usize][k]
                + blend * (self.points[b as usize][k] - self.points[a as usize][k])
        });
        let vertex = self.points.len() as u32;
        self.points.push(point);
        self.levels.push(boundary);
        self.samples.push(SourceSample { edge, blend });
        let mut candidate = [false; 2];
        candidate[usize::from(!upper)] = true;
        self.cut_candidates.push(candidate);
        self.cuts.insert((edge, upper), vertex);
        Ok(vertex)
    }

    fn install_crease(&mut self) {
        if self.mid.medial_crease.is_empty() {
            return;
        }
        let original = self.mid.medial_crease.len();
        self.mid.medial_crease.resize(self.points.len(), false);
        self.mid.crease_right.resize(self.points.len(), false);
        for (vertex, sample) in self.samples.iter().enumerate().skip(original) {
            self.mid.medial_crease[vertex] = sample
                .edge
                .iter()
                .all(|&i| self.mid.medial_crease[i as usize]);
            self.mid.crease_right[vertex] = sample
                .edge
                .iter()
                .any(|&i| self.mid.crease_right[i as usize]);
        }
    }

    /// Construction probes follow actual retained column rails. End rows lie
    /// on the cut edges, rather than on a resampled approximation of the metal.
    fn install_grid(&mut self, low: f64, high: Option<f64>) -> Result<(), GenerateError> {
        let width = self.mid.width();
        let mut used = vec![false; self.points.len()];
        for &v in self.mid.faces.iter().flatten() {
            used[v as usize] = true;
        }
        let mut columns = vec![Vec::new(); width];
        for (vertex, sample) in self.samples.iter().enumerate() {
            let [a, b] = sample.edge.map(|i| i as usize);
            let column = crate::SurfaceColumn::between(
                self.mid.surface_column(a),
                self.mid.surface_column(b),
                sample.blend,
            );
            let (column, _, blend) = column
                .bracket(width as u32)
                .ok_or(GenerateError::InvalidSurface)?;
            if used[vertex] && blend == 0.0 {
                columns[column as usize].push(vertex);
            }
        }
        let retained = ConstructionColumns::retained(&columns)?;
        let mut grid = Vec::with_capacity(self.mid.rows * retained.width());
        for row in (0..self.mid.rows).rev() {
            for vertices in &columns[retained.range()] {
                let top = high.unwrap_or_else(|| {
                    vertices
                        .iter()
                        .map(|&i| self.levels[i])
                        .fold(f64::NEG_INFINITY, f64::max)
                });
                let q = low + (top - low) * row as f64 / (self.mid.rows - 1) as f64;
                let vertex = vertices
                    .iter()
                    .min_by(|&&a, &&b| {
                        (self.levels[a] - q)
                            .abs()
                            .total_cmp(&(self.levels[b] - q).abs())
                    })
                    .ok_or(GenerateError::Degenerate)?;
                grid.push(*vertex as u32);
            }
        }
        self.mid.grid_vertices = Some(grid);
        self.mid.construction_columns = retained;
        Ok(())
    }
}
