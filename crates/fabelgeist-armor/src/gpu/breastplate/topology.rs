//! The breastplate's connectivity, decided from the design alone.
//!
//! Each plate is a chart of rows and columns: the main grid rising from the
//! waist seam, then a skirt hanging below it that shares the seam row. The
//! solid shell adds an outer wall along the extrusion, gives the skirt its
//! own seam vertices, closes every boundary edge with a cut wall and, for a
//! medial ridge, splits the crease. None of that depends on where the
//! vertices end up, so the host lays it out and the device only places each
//! solid vertex at its mid vertex, or a gauge outward along its extrusion.

use std::collections::BTreeMap;

use crate::{BreastplateDesign, GenerateError, PlateFace, SurfaceGrid};

/// Columns of the regular carrier chart.
pub(crate) const U_SAMPLES: usize = 49;
/// Rows of the main grid.
pub(crate) const V_SAMPLES: usize = 33;
/// Rows of the skirt, its seam row included.
pub(crate) const SKIRT_SAMPLES: usize = 9;
/// Profile samples across one flute.
const FLUTE_PROFILE_SEGMENTS: usize = 8;
/// Columns closer than this in the chart are merged.
const CHART_MERGE_TOLERANCE: f32 = 5e-4;

/// Set on a solid vertex that lies on the outer wall.
pub(crate) const OUTER_BIT: u32 = 1 << 31;

/// Chart columns across a plate: the regular carrier chart, with every
/// flute's profile samples on a fluted front.
pub(crate) fn chart_columns(rear: bool, design: &BreastplateDesign) -> Vec<f32> {
    let mut columns: Vec<_> = (0..U_SAMPLES)
        .map(|i| -1.0 + 2.0 * i as f32 / (U_SAMPLES - 1) as f32)
        .collect();
    if let Some(pattern) = design.fluting.as_ref().filter(|_| !rear) {
        let pitch = 2.0 * pattern.spread.unit() / f32::from(pattern.count.0);
        let half_width = pitch * pattern.width.unit() * 0.5;
        for flute in 0..pattern.count.0 {
            let center = -pattern.spread.unit() + (f32::from(flute) + 0.5) * pitch;
            for sample in 0..=FLUTE_PROFILE_SEGMENTS {
                columns.push(
                    center
                        + half_width * (2.0 * sample as f32 / FLUTE_PROFILE_SEGMENTS as f32 - 1.0),
                );
            }
        }
        columns.sort_by(f32::total_cmp);
        columns.dedup_by(|a, b| (*a - *b).abs() < CHART_MERGE_TOLERANCE);
    }
    columns
}

/// The mid surface of one plate: main rows, then the skirt's rows below the
/// seam, each `columns` wide.
#[derive(Clone, Debug)]
pub(crate) struct MidTopology {
    pub columns: Vec<f32>,
    pub faces: Vec<[u32; 3]>,
    pub skirt_face_start: usize,
    /// Vertices on the medial crease, and right of it, when the design has
    /// a medial ridge.
    pub medial_crease: Vec<bool>,
    pub crease_right: Vec<bool>,
}

impl MidTopology {
    pub(crate) fn new(rear: bool, columns: Vec<f32>, design: &BreastplateDesign) -> Self {
        let width = columns.len();
        let main_ids = (0..V_SAMPLES)
            .map(|row| (0..width).map(|c| (row * width + c) as u32).collect())
            .collect::<Vec<Vec<u32>>>();
        let mut faces = Vec::new();
        grid_faces(&mut faces, &main_ids, rear);
        let mut skirt_ids = vec![main_ids[0].clone()];
        let mut next = (V_SAMPLES * width) as u32;
        for _ in 1..SKIRT_SAMPLES {
            skirt_ids.push((0..width as u32).map(|c| next + c).collect());
            next += width as u32;
        }
        let skirt_face_start = faces.len();
        // The skirt runs downward from the seam, against the main grid.
        grid_faces(&mut faces, &skirt_ids, !rear);
        let (medial_crease, crease_right) = if !rear && design.profile.medial_ridge.0 > 0 {
            let rows = V_SAMPLES + SKIRT_SAMPLES - 1;
            let coordinates = (0..rows).flat_map(|_| columns.iter().copied());
            (
                coordinates.clone().map(|u| u.abs() < 1e-6).collect(),
                coordinates.map(|u| u > 1e-6).collect(),
            )
        } else {
            (Vec::new(), Vec::new())
        };
        Self {
            columns,
            faces,
            skirt_face_start,
            medial_crease,
            crease_right,
        }
    }

    /// Each vertex's faces in face order, as offsets into one list: the
    /// order the normal kernel sums face normals into a vertex.
    pub(crate) fn incident_faces(&self) -> (Vec<u32>, Vec<u32>) {
        let mut counts = vec![0u32; self.vertex_count() + 1];
        for face in &self.faces {
            for vertex in face {
                counts[*vertex as usize + 1] += 1;
            }
        }
        for i in 1..counts.len() {
            counts[i] += counts[i - 1];
        }
        let mut next = counts.clone();
        let mut faces = vec![0u32; counts[counts.len() - 1] as usize];
        for (index, face) in self.faces.iter().enumerate() {
            for vertex in face {
                let slot = &mut next[*vertex as usize];
                faces[*slot as usize] = index as u32;
                *slot += 1;
            }
        }
        (counts, faces)
    }

    pub(crate) fn width(&self) -> usize {
        self.columns.len()
    }

    pub(crate) fn vertex_count(&self) -> usize {
        (V_SAMPLES + SKIRT_SAMPLES - 1) * self.width()
    }

    /// The mid vertices as one grid from the top of the plate down: the
    /// main rows from the neck to the waist seam, then the skirt's.
    fn rows_downward(&self) -> impl Iterator<Item = u32> + '_ {
        let width = self.width() as u32;
        let main = (0..V_SAMPLES as u32).rev();
        let skirt = (1..SKIRT_SAMPLES as u32).map(|row| V_SAMPLES as u32 + row - 1);
        main.chain(skirt)
            .flat_map(move |row| (0..width).map(move |column| row * width + column))
    }
}

fn grid_faces(faces: &mut Vec<[u32; 3]>, ids: &[Vec<u32>], rear: bool) {
    for rows in ids.windows(2) {
        for column in 0..rows[0].len() - 1 {
            let (a, b, d, c) = (
                rows[0][column],
                rows[0][column + 1],
                rows[1][column],
                rows[1][column + 1],
            );
            if rear {
                faces.extend([[a, c, b], [a, d, c]]);
            } else {
                faces.extend([[a, b, c], [a, c, d]]);
            }
        }
    }
}

/// A thickened plate: each vertex is a mid vertex, on the mid surface or
/// with [`OUTER_BIT`] a gauge along its extrusion.
#[derive(Clone, Debug, Default)]
pub(crate) struct SolidTopology {
    pub sources: Vec<u32>,
    pub indices: Vec<u32>,
    /// The plate face of each triangle.
    pub faces: Vec<PlateFace>,
    /// The outer face of each plate.
    pub grids: Vec<SurfaceGrid>,
}

impl SolidTopology {
    /// Thicken `mid` into a solid's connectivity, short of placing anything.
    pub(crate) fn new(mid: &MidTopology) -> Result<Self, GenerateError> {
        let count = mid.vertex_count() as u32;
        let main_columns = mid.width();
        let mut sources = (0..count)
            .chain((0..count).map(|i| i | OUTER_BIT))
            .collect::<Vec<_>>();
        let mut welded = (0..count * 2).collect::<Vec<_>>();
        let mut skirt_inner = vec![0u32; main_columns];
        let mut skirt_outer = vec![0u32; main_columns];
        for index in 0..main_columns {
            skirt_inner[index] = sources.len() as u32;
            sources.push(index as u32);
            welded.push(index as u32);
            skirt_outer[index] = sources.len() as u32;
            sources.push(index as u32 | OUTER_BIT);
            welded.push(index as u32 + count);
        }
        let mut indices = Vec::with_capacity(mid.faces.len() * 6);
        let mut faces = Vec::with_capacity(mid.faces.len() * 2);
        for (face_index, [a, b, c]) in mid.faces.iter().enumerate() {
            let skirt = face_index >= mid.skirt_face_start;
            let inner = |index: u32| {
                if skirt && (index as usize) < main_columns {
                    skirt_inner[index as usize]
                } else {
                    index
                }
            };
            let outer = |index: u32| {
                if skirt && (index as usize) < main_columns {
                    skirt_outer[index as usize]
                } else {
                    index + count
                }
            };
            indices.extend([outer(*a), outer(*b), outer(*c)]);
            indices.extend([inner(*c), inner(*b), inner(*a)]);
            faces.extend([PlateFace::Outer, PlateFace::Inner]);
        }
        for [a, b] in boundary_edges(&mid.faces)? {
            // The cut edge has its own shading normals: sharing surface
            // vertices with the narrow wall creases the armhole corners.
            let first = sources.len() as u32;
            for original in [a, b, b + count, a + count] {
                sources.push(sources[original as usize]);
                welded.push(original);
            }
            indices.extend([first, first + 1, first + 2, first, first + 2, first + 3]);
            faces.extend([PlateFace::Edge; 2]);
        }
        validate_closed_shell(indices.as_chunks::<3>().0, &welded)?;
        let grid = SurfaceGrid {
            rows: (V_SAMPLES + SKIRT_SAMPLES - 1) as u32,
            columns: main_columns as u32,
            cyclic: false,
            vertices: mid.rows_downward().map(|mid| mid + count).collect(),
        };
        let mut solid = Self {
            sources,
            indices,
            faces,
            grids: vec![grid],
        };
        solid.split_medial_crease(mid);
        Ok(solid)
    }

    fn mid_of(&self, vertex: u32) -> usize {
        (self.sources[vertex as usize] & !OUTER_BIT) as usize
    }

    fn split_medial_crease(&mut self, mid: &MidTopology) {
        if mid.medial_crease.is_empty() {
            return;
        }
        let right_face = |solid: &Self, face: &[u32]| {
            face.iter()
                .any(|index| mid.crease_right[solid.mid_of(*index)])
        };
        let mut used_left = vec![false; self.sources.len()];
        for face in self.indices.as_chunks::<3>().0 {
            if !right_face(self, face) {
                for index in face {
                    used_left[*index as usize] = true;
                }
            }
        }
        let mut aliases = BTreeMap::new();
        for face in 0..self.indices.len() / 3 {
            let corners = [
                self.indices[face * 3],
                self.indices[face * 3 + 1],
                self.indices[face * 3 + 2],
            ];
            if !right_face(self, &corners) {
                continue;
            }
            for (corner, original) in corners.into_iter().enumerate() {
                if mid.medial_crease[self.mid_of(original)] && used_left[original as usize] {
                    let alias = *aliases.entry(original).or_insert_with(|| {
                        self.sources.push(self.sources[original as usize]);
                        (self.sources.len() - 1) as u32
                    });
                    self.indices[face * 3 + corner] = alias;
                }
            }
        }
    }

    /// Append `other`, whose mid vertices follow this plate's `mid_count`.
    pub(crate) fn extend(&mut self, other: Self, mid_count: u32) {
        let offset = self.sources.len() as u32;
        self.sources.extend(
            other
                .sources
                .into_iter()
                .map(|source| (source & OUTER_BIT) | ((source & !OUTER_BIT) + mid_count)),
        );
        self.indices
            .extend(other.indices.into_iter().map(|index| index + offset));
        self.faces.extend(other.faces);
        self.grids.extend(other.grids.into_iter().map(|mut grid| {
            for vertex in &mut grid.vertices {
                *vertex += offset;
            }
            grid
        }));
    }
}

fn boundary_edges(faces: &[[u32; 3]]) -> Result<Vec<[u32; 2]>, GenerateError> {
    let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    for [a, b, c] in faces {
        for (start, end) in [(*a, *b), (*b, *c), (*c, *a)] {
            edges
                .entry((start.min(end), start.max(end)))
                .or_default()
                .push((start, end));
        }
    }
    let mut boundary = Vec::new();
    for uses in edges.values() {
        match *uses.as_slice() {
            [(a, b)] => boundary.push([a, b]),
            [(first_start, first_end), (second_start, second_end)]
                if first_start == second_end && first_end == second_start => {}
            _ => return Err(GenerateError::Degenerate),
        }
    }
    Ok(boundary)
}

fn validate_closed_shell(faces: &[[u32; 3]], welded: &[u32]) -> Result<(), GenerateError> {
    let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    for face in faces {
        let [a, b, c] = face.map(|index| welded[index as usize]);
        for (start, end) in [(a, b), (b, c), (c, a)] {
            edges
                .entry((start.min(end), start.max(end)))
                .or_default()
                .push((start, end));
        }
    }
    if edges
        .values()
        .any(|uses| !matches!(uses.as_slice(), &[(a, b), (c, d)] if a == d && b == c))
    {
        return Err(GenerateError::Degenerate);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_plates_close_and_keep_their_skirt_seam_apart() {
        let design = BreastplateDesign::default();
        let mid = MidTopology::new(true, chart_columns(true, &design), &design);
        let solid = SolidTopology::new(&mid).unwrap();
        let count = mid.vertex_count();
        // Inner and outer walls, the skirt's own seam, and four vertices per
        // boundary edge.
        let boundary = 2 * (mid.width() - 1) + 2 * (V_SAMPLES + SKIRT_SAMPLES - 2);
        assert_eq!(
            solid.sources.len(),
            2 * count + 2 * mid.width() + 4 * boundary
        );
        assert_eq!(solid.indices.len(), mid.faces.len() * 6 + boundary * 6);
        assert_eq!(solid.faces.len() * 3, solid.indices.len());
        let count = |face| solid.faces.iter().filter(|f| **f == face).count();
        assert_eq!(count(PlateFace::Outer), mid.faces.len());
        assert_eq!(count(PlateFace::Edge), boundary * 2);
    }
}
