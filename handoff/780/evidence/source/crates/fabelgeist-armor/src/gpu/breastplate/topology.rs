//! Logical carrier correspondence and closed metal shells.
//!
//! Each plate is a chart of rows and columns: the main grid rising from the
//! waist seam, then a skirt hanging below it that shares the seam row. The
//! solid shell adds an outer wall along the extrusion, gives the skirt its
//! own seam vertices, closes every boundary edge with a cut wall and, for a
//! medial ridge, splits the crease. Fitted inner and outer walls retain these
//! logical vertices and boundary edges, including cuts through evaluated
//! carrier facets. The device places each solid vertex at its mid
//! vertex, or a gauge outward along its extrusion.

use super::construction_columns::ConstructionColumns;
use super::course_coordinates::CourseCoordinates;
use std::collections::BTreeMap;

use crate::{BreastplateDesign, GenerateError, PlateFace, SurfaceColumn, SurfaceGrid};

/// Columns of the regular carrier chart.
pub(crate) const U_SAMPLES: usize = 49;
/// Rows of the main grid.
pub(crate) const V_SAMPLES: usize = 34;
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
    pub rows: usize,
    pub skirt: bool,
    pub faces: Vec<[u32; 3]>,
    pub skirt_face_start: usize,
    /// Cut-edge samples appended after the evaluated regular carrier grid.
    pub cut_columns: Vec<SurfaceColumn>,
    /// Vertices on the medial crease, and right of it, when the design has
    /// a medial ridge.
    pub medial_crease: Vec<bool>,
    pub crease_right: Vec<bool>,
    /// Outer construction samples, each a retained rendered wall vertex.
    pub grid_vertices: Option<Vec<u32>>,
    pub construction_columns: ConstructionColumns,
    pub course_coordinates: Option<CourseCoordinates>,
}

impl MidTopology {
    #[cfg(test)]
    /// A separate test sheet, with its own closed boundary.
    pub(super) fn course(rear: bool, columns: Vec<f32>, rows: usize, ridge: bool) -> Self {
        let width = columns.len();
        let ids = (0..rows)
            .map(|r| (0..width).map(|c| (r * width + c) as u32).collect())
            .collect::<Vec<Vec<u32>>>();
        let mut faces = Vec::new();
        grid_faces(&mut faces, &ids, rear);
        let coords = (0..rows).flat_map(|_| columns.iter().copied());
        Self {
            rows,
            skirt: false,
            skirt_face_start: faces.len(),
            faces,
            cut_columns: Vec::new(),
            grid_vertices: None,
            construction_columns: ConstructionColumns::full(width),
            course_coordinates: None,
            medial_crease: if ridge {
                coords.clone().map(|u| u.abs() < 1e-6).collect()
            } else {
                Vec::new()
            },
            crease_right: if ridge {
                coords.map(|u| u > 1e-6).collect()
            } else {
                Vec::new()
            },
            columns,
        }
    }

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
            rows: V_SAMPLES + SKIRT_SAMPLES - 1,
            skirt: true,
            faces,
            skirt_face_start,
            cut_columns: Vec::new(),
            grid_vertices: None,
            construction_columns: ConstructionColumns::full(width),
            course_coordinates: None,
            medial_crease,
            crease_right,
        }
    }

    pub(crate) fn width(&self) -> usize {
        self.columns.len()
    }

    pub(crate) fn vertex_count(&self) -> usize {
        self.rows * self.width() + self.cut_columns.len()
    }

    pub(super) fn surface_column(&self, vertex: usize) -> SurfaceColumn {
        let regular = self.rows * self.width();
        if vertex < regular {
            SurfaceColumn::at((vertex % self.width()) as u32)
        } else {
            self.cut_columns[vertex - regular]
        }
    }

    /// The mid vertices as one grid from the top of the plate down: the
    /// main rows from the neck to the waist seam, then the skirt's.
    pub(super) fn rows_downward(&self) -> impl Iterator<Item = u32> + '_ {
        let width = self.width() as u32;
        let main_rows = if self.skirt { V_SAMPLES } else { self.rows };
        let skirt_rows = if self.skirt { SKIRT_SAMPLES } else { 1 };
        let main = (0..main_rows as u32).rev();
        let skirt = (1..skirt_rows as u32).map(move |row| main_rows as u32 + row - 1);
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
            // Mirror the diagonal across the centre column. An armscye's
            // shortened outer column makes its upper cells concave in the
            // angular/height chart; the chord toward the inner upper corner
            // would cross outside that cell on the left wing.
            if column < (rows[0].len() - 1) / 2 {
                if rear {
                    faces.extend([[a, d, b], [b, d, c]]);
                } else {
                    faces.extend([[a, b, d], [b, c, d]]);
                }
            } else if rear {
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
    pub(crate) fn new(mid: &MidTopology, outer_mid: &MidTopology) -> Result<Self, GenerateError> {
        let count = mid.vertex_count() as u32;
        let main_columns = mid.width();
        let mut sources = (0..count)
            .chain((0..count).map(|i| i | OUTER_BIT))
            .collect::<Vec<_>>();
        let mut welded = (0..count * 2).collect::<Vec<_>>();
        let mut skirt_inner = vec![0u32; main_columns];
        let mut skirt_outer = vec![0u32; main_columns];
        for index in 0..if mid.skirt { main_columns } else { 0 } {
            skirt_inner[index] = sources.len() as u32;
            sources.push(index as u32);
            welded.push(index as u32);
            skirt_outer[index] = sources.len() as u32;
            sources.push(index as u32 | OUTER_BIT);
            welded.push(index as u32 + count);
        }
        let mut indices = Vec::with_capacity(mid.faces.len() * 6);
        let mut faces = Vec::with_capacity(mid.faces.len() * 2);
        for (surface, outer) in [(mid, false), (outer_mid, true)] {
            for (face_index, triangle) in surface.faces.iter().enumerate() {
                let skirt = face_index >= surface.skirt_face_start;
                let corners = triangle.map(|index| {
                    if skirt && (index as usize) < main_columns {
                        if outer {
                            skirt_outer[index as usize]
                        } else {
                            skirt_inner[index as usize]
                        }
                    } else if outer {
                        index + count
                    } else {
                        index
                    }
                });
                indices.extend(if outer {
                    corners
                } else {
                    [corners[2], corners[1], corners[0]]
                });
                faces.push(if outer {
                    PlateFace::Outer
                } else {
                    PlateFace::Inner
                });
            }
        }
        let boundaries = boundary_edges(&mid.faces)?;
        for &[a, b] in &boundaries {
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
        let mut solid = Self {
            sources,
            indices,
            faces,
            grids: Vec::new(),
        };
        solid.split_medial_crease(mid);
        solid.install_grid(mid, outer_mid, &boundaries);
        Ok(solid)
    }

    pub(super) fn mid_of(&self, vertex: u32) -> usize {
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

    /// Remove carrier samples outside the rendered trim, preserving source IDs.
    pub(super) fn compact(&mut self) {
        let mut used = vec![false; self.sources.len()];
        for &v in &self.indices {
            used[v as usize] = true;
        }
        let mut remap = vec![u32::MAX; used.len()];
        let mut sources = Vec::new();
        for (i, (&source, active)) in self.sources.iter().zip(used).enumerate() {
            if active {
                remap[i] = sources.len() as u32;
                sources.push(source);
            }
        }
        self.sources = sources;
        for v in &mut self.indices {
            *v = remap[*v as usize];
        }
        for grid in &mut self.grids {
            grid.remap_vertices(|v| {
                let v = remap[v as usize];
                assert_ne!(
                    v,
                    u32::MAX,
                    "construction samples must be rendered wall vertices"
                );
                v
            });
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
            grid.remap_vertices(|vertex| vertex + offset);
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
    fn independently_triangulated_walls_still_close_with_the_same_boundary() {
        let inner = MidTopology::course(false, vec![-1.0, 0.0, 1.0], 2, false);
        let mut outer = inner.clone();
        outer.faces[..2].copy_from_slice(&[[0, 1, 4], [0, 4, 3]]);
        let solid = SolidTopology::new(&inner, &outer).unwrap();
        let inner_faces = solid
            .indices
            .as_chunks::<3>()
            .0
            .iter()
            .zip(&solid.faces)
            .filter(|(_, face)| **face == PlateFace::Inner)
            .map(|(face, _)| face.map(|i| solid.sources[i as usize] & !OUTER_BIT))
            .collect::<Vec<_>>();
        let outer_faces = solid
            .indices
            .as_chunks::<3>()
            .0
            .iter()
            .zip(&solid.faces)
            .filter(|(_, face)| **face == PlateFace::Outer)
            .map(|(face, _)| face.map(|i| solid.sources[i as usize] & !OUTER_BIT))
            .collect::<Vec<_>>();
        assert_eq!(outer_faces, outer.faces);
        assert_eq!(
            inner_faces,
            inner
                .faces
                .iter()
                .map(|[a, b, c]| [*c, *b, *a])
                .collect::<Vec<_>>()
        );
        assert_eq!(solid.grids[0].vertices.len(), inner.vertex_count());
    }

    #[test]
    fn solid_plates_close_and_keep_their_skirt_seam_apart() {
        let design = BreastplateDesign::default();
        let mid = MidTopology::new(true, chart_columns(true, &design), &design);
        let solid = SolidTopology::new(&mid, &mid).unwrap();
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
