//! Cut the evaluated carrier facets, retaining their surface and wall offsets.
use std::collections::BTreeMap;

use super::cut_resolution::CUT_RESOLUTION_METERS;
use super::topology::{MidTopology, V_SAMPLES};
use crate::{GenerateError, PlateShellFailure};

pub(super) struct CutSample {
    vertex: usize,
    edge: [usize; 2],
    blend: f64,
}

pub(super) struct ClippedCarrier {
    pub positions: Vec<[f32; 3]>,
    pub directions: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    skirt_face_start: usize,
    samples: Vec<CutSample>,
    cuts: BTreeMap<[u32; 2], u32>,
    pub rim_vertices: Vec<u32>,
    width: usize,
}

impl ClippedCarrier {
    pub fn new(
        mid: &MidTopology,
        positions: &[[f32; 3]],
        directions: &[[f32; 3]],
    ) -> Result<Self, GenerateError> {
        let width = mid.width();
        let mut clipped = Self {
            positions: positions.to_vec(),
            directions: directions.to_vec(),
            faces: Vec::new(),
            skirt_face_start: 0,
            samples: Vec::new(),
            cuts: BTreeMap::new(),
            rim_vertices: Vec::new(),
            width,
        };
        // The final stored row is the trim, not another carrier interval.
        let carrier_faces = (V_SAMPLES - 2) * (width - 1) * 2;
        for &face in &mid.faces[..carrier_faces] {
            let column = face.iter().map(|i| *i as usize % width).min().unwrap();
            let trim = |column: usize| f64::from(positions[(V_SAMPLES - 1) * width + column][1]);
            let distances = face.map(|i| {
                let c = i as usize % width;
                let height = trim(column) + (trim(column + 1) - trim(column)) * (c - column) as f64;
                let distance = f64::from(positions[i as usize][1]) - height;
                if distance.abs() <= CUT_RESOLUTION_METERS {
                    0.0
                } else {
                    distance
                }
            });
            clipped.cut_face(face, distances, true);
        }
        clipped.skirt_face_start = clipped.faces.len();
        clipped
            .faces
            .extend_from_slice(&mid.faces[mid.skirt_face_start..]);
        for face in &clipped.faces {
            face_normal(face.map(|i| clipped.positions[i as usize]))?;
        }
        clipped.rim_vertices.sort_unstable();
        clipped.rim_vertices.dedup();
        Ok(clipped)
    }

    /// The second trim is evaluated over the same facets as the neckline.
    /// Its saturated upper edge is a vertical boundary, with no rail-height
    /// discontinuity when the authored opening width changes.
    pub fn trim_arms(&mut self, carrier_distances: &[f32]) -> Result<(), GenerateError> {
        let mut distances = carrier_distances
            .iter()
            .copied()
            .map(f64::from)
            .collect::<Vec<_>>();
        distances.resize(self.positions.len(), 0.0);
        for sample in &self.samples {
            let [a, b] = sample.edge;
            distances[sample.vertex] = distances[a] + sample.blend * (distances[b] - distances[a]);
        }
        let original = std::mem::take(&mut self.faces);
        let skirt_start = self.skirt_face_start;
        self.cuts.clear();
        for &face in &original[..skirt_start] {
            self.cut_face(face, face.map(|i| distances[i as usize]), false);
        }
        self.skirt_face_start = self.faces.len();
        self.faces.extend_from_slice(&original[skirt_start..]);
        self.rim_vertices.sort_unstable();
        self.rim_vertices.dedup();
        for face in &self.faces {
            face_normal(face.map(|i| self.positions[i as usize]))?;
        }
        Ok(())
    }

    fn cut_face(&mut self, face: [u32; 3], distances: [f64; 3], neckline: bool) {
        let mut polygon = Vec::with_capacity(4);
        for i in 0..3 {
            let next = (i + 1) % 3;
            if distances[i] <= 0.0 {
                polygon.push(face[i]);
                if distances[i] == 0.0 {
                    self.rim_vertices.push(face[i]);
                }
            }
            if (distances[i] < 0.0 && distances[next] > 0.0)
                || (distances[next] < 0.0 && distances[i] > 0.0)
            {
                let vertex = self.intersection(
                    [face[i], face[next]],
                    [distances[i], distances[next]],
                    neckline,
                );
                polygon.push(vertex);
                self.rim_vertices.push(vertex);
            }
        }
        polygon.dedup();
        if polygon.len() > 1 && polygon.first() == polygon.last() {
            polygon.pop();
        }
        for i in 1..polygon.len().saturating_sub(1) {
            self.faces.push([polygon[0], polygon[i], polygon[i + 1]]);
        }
    }

    fn intersection(&mut self, edge: [u32; 2], distances: [f64; 2], neckline: bool) -> u32 {
        let mut key = edge;
        key.sort_unstable();
        if let Some(&vertex) = self.cuts.get(&key) {
            return vertex;
        }
        let blend = distances[0] / (distances[0] - distances[1]);
        let [a, b] = edge.map(|v| v as usize);
        let point = interpolate(self.positions[a], self.positions[b], blend);
        for source in [a, b] {
            if point == self.positions[source] {
                self.cuts.insert(key, source as u32);
                return source as u32;
            }
        }
        let direction = interpolate(self.directions[a], self.directions[b], blend);
        let vertex = if neckline && a % self.width == b % self.width {
            // Keep the logical neckline rail sample used by course sampling.
            let vertex = (V_SAMPLES - 1) * self.width + a % self.width;
            self.positions[vertex] = point;
            self.directions[vertex] = direction;
            vertex
        } else {
            let vertex = self.positions.len();
            self.positions.push(point);
            self.directions.push(direction);
            vertex
        };
        self.samples.push(CutSample {
            vertex,
            edge: [a, b],
            blend,
        });
        self.cuts.insert(key, vertex as u32);
        vertex as u32
    }

    pub fn carrier_positions(&self, mut coarse: Vec<[f32; 3]>) -> Vec<[f32; 3]> {
        coarse.resize(self.positions.len(), [0.0; 3]);
        for sample in &self.samples {
            let [a, b] = sample.edge;
            coarse[sample.vertex] = interpolate(coarse[a], coarse[b], sample.blend);
        }
        coarse
    }

    pub fn install(&self, mid: &mut MidTopology) {
        let regular = mid.rows * mid.width();
        mid.cut_columns = vec![crate::SurfaceColumn::at(0); self.positions.len() - regular];
        for sample in self.samples.iter().filter(|s| s.vertex >= regular) {
            mid.cut_columns[sample.vertex - regular] = crate::SurfaceColumn::between(
                mid.surface_column(sample.edge[0]),
                mid.surface_column(sample.edge[1]),
                sample.blend as f32,
            );
        }
        if !mid.medial_crease.is_empty() {
            let width = mid.width();
            mid.medial_crease.resize(self.positions.len(), false);
            mid.crease_right.resize(self.positions.len(), false);
            for sample in &self.samples {
                let [a, b] = sample.edge.map(|i| {
                    let (a, b, blend) = mid.surface_column(i).bracket(width as u32).unwrap();
                    f64::from(mid.columns[a as usize])
                        + f64::from(blend)
                            * f64::from(mid.columns[b as usize] - mid.columns[a as usize])
                });
                let u = a + (b - a) * sample.blend;
                mid.medial_crease[sample.vertex] = u.abs() < 1e-6;
                mid.crease_right[sample.vertex] = u > 1e-6;
            }
        }
        mid.faces.clone_from(&self.faces);
        mid.skirt_face_start = self.skirt_face_start;
        mid.construction_grid(&self.positions);
    }

    pub fn validate_outer(&self, gauge: crate::Millimeters) -> Result<(), GenerateError> {
        let points = self
            .positions
            .iter()
            .zip(&self.directions)
            .map(|(p, n)| std::array::from_fn(|k| p[k] + n[k] * gauge.metres()))
            .collect::<Vec<_>>();
        // Winding follows the clipped carrier. An embedded concave shell can
        // have an outer facet whose normal opposes an individual offset;
        // that is not evidence of a boundary intersection or inverted shell.
        for face in &self.faces {
            face_normal(face.map(|i| points[i as usize]))?;
        }
        Ok(())
    }
}

fn face_normal(points: [[f32; 3]; 3]) -> Result<[f64; 3], PlateShellFailure> {
    let points = points.map(|p| p.map(f64::from));
    let a = std::array::from_fn::<_, 3, _>(|k| points[1][k] - points[0][k]);
    let b = std::array::from_fn::<_, 3, _>(|k| points[2][k] - points[0][k]);
    let normal = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    if normal.iter().any(|n| !n.is_finite()) || normal.iter().all(|&n| n == 0.0) {
        return Err(PlateShellFailure::CollapsedFace);
    }
    Ok(normal)
}

fn interpolate(a: [f32; 3], b: [f32; 3], blend: f64) -> [f32; 3] {
    std::array::from_fn(|k| (f64::from(a[k]) + (f64::from(b[k]) - f64::from(a[k])) * blend) as f32)
}
