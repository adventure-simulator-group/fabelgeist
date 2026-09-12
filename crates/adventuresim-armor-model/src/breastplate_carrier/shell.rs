//! Solidify carrier meshes and validate welded shell closure.

use super::*;

const UPPER_RIM_ROWS: usize = 3;

pub(super) fn face_normal(face: [u32; 3], positions: &[[f32; 3]]) -> [f32; 3] {
    let [a, b, c] = face.map(|index| positions[index as usize]);
    cross(sub(b, a), sub(c, a))
}

pub(super) fn vertex_normals(
    positions: &[[f32; 3]],
    faces: &[[u32; 3]],
) -> Result<Vec<[f32; 3]>, GenerateError> {
    let mut normals = vec![[0.0; 3]; positions.len()];
    for (face_index, face) in faces.iter().enumerate() {
        let normal = face_normal(*face, positions);
        if dot(normal, normal) <= 1e-16 || normal.iter().any(|value| !value.is_finite()) {
            eprintln!("breastplate degenerate face {face_index} {face:?}");
            return Err(GenerateError::Degenerate);
        }
        for index in face {
            normals[*index as usize] = add(normals[*index as usize], normal);
        }
    }
    normals.into_iter().map(normalized).collect()
}

/// Continue the carrier's extrusion field from the interior. The curved trim
/// can have a tighter radius than the gauge or articulated lap lift; offsetting
/// along its local boundary normals would turn the rim back into the plate.
pub(super) fn rim_extrusion(mesh: &mut MidMesh) -> Result<(), GenerateError> {
    let mut normals = vertex_normals(&mesh.positions, &mesh.faces)?;
    let start = V_SAMPLES - 1 - UPPER_RIM_ROWS;
    for row in start + 1..V_SAMPLES {
        let blend = (row - start) as f32 / UPPER_RIM_ROWS as f32;
        for column in 0..mesh.main_columns {
            let index = row * mesh.main_columns + column;
            normals[index] = normalized(add(
                scale(normals[index], 1.0 - blend),
                scale(normals[start * mesh.main_columns + column], blend),
            ))?;
        }
    }
    for row in normals.chunks_exact_mut(mesh.main_columns) {
        continue_lateral_extrusion(row)?;
    }
    mesh.extrusion_normals = Some(normals);
    Ok(())
}

impl MidMesh {
    /// The front neckline rises across the shoulder while the chest recedes.
    /// Continue its gauge toward the front at the cut: lateral or upward
    /// extrusion from the chest can enter another face of that twisted strip.
    pub(super) fn front_neckline_extrusion(&mut self, frame: Frame) -> Result<(), GenerateError> {
        let normals = self
            .extrusion_normals
            .as_mut()
            .ok_or(GenerateError::InvalidSurface)?;
        let start = V_SAMPLES - 1 - UPPER_RIM_ROWS;
        for row in start + 1..V_SAMPLES {
            let blend = (row - start) as f32 / UPPER_RIM_ROWS as f32;
            for column in 0..self.main_columns {
                let normal = &mut normals[row * self.main_columns + column];
                *normal = normalized(add(
                    scale(*normal, 1.0 - blend),
                    scale(frame.front, dot(*normal, frame.front) * blend),
                ))?;
            }
        }
        Ok(())
    }
}

fn continue_lateral_extrusion(row: &mut [[f32; 3]]) -> Result<(), GenerateError> {
    const LATERAL_RIM_COLUMNS: usize = 2;
    let columns = row.len();
    for from_end in [false, true] {
        let index = |distance: usize| {
            if from_end {
                columns - 1 - distance
            } else {
                distance
            }
        };
        let interior = row[index(LATERAL_RIM_COLUMNS)];
        for distance in 0..LATERAL_RIM_COLUMNS {
            let column = index(distance);
            let blend = (LATERAL_RIM_COLUMNS - distance) as f32 / LATERAL_RIM_COLUMNS as f32;
            row[column] = normalized(add(scale(row[column], 1.0 - blend), scale(interior, blend)))?;
        }
    }
    Ok(())
}

fn append_cut_wall(
    [a, b]: [u32; 2],
    count: u32,
    positions: &mut Vec<[f32; 3]>,
    source_mid_indices: &mut Vec<usize>,
    welded_indices: &mut Vec<u32>,
    indices: &mut Vec<u32>,
) {
    // The cut edge has its own shading normals. Sharing surface
    // vertices with this narrow wall creates false creases at the
    // armhole corners, despite a smooth physical carrier.
    let mut rim = [0; 4];
    for (slot, original) in [a, b, b + count, a + count].into_iter().enumerate() {
        rim[slot] = positions.len() as u32;
        positions.push(positions[original as usize]);
        source_mid_indices.push(source_mid_indices[original as usize]);
        welded_indices.push(original);
    }
    indices.extend([rim[0], rim[1], rim[2], rim[0], rim[2], rim[3]]);
}

fn boundary_edges(faces: &[[u32; 3]]) -> Result<Vec<[u32; 2]>, GenerateError> {
    let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    for [a, b, c] in faces {
        for (start, end) in [(*a, *b), (*b, *c), (*c, *a)] {
            let key = (start.min(end), start.max(end));
            edges.entry(key).or_default().push((start, end));
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

pub(super) fn solidify(mid: MidMesh, thickness: f32) -> Result<SolidMesh, GenerateError> {
    let mid_normals = match &mid.extrusion_normals {
        Some(normals) => normals.clone(),
        None => vertex_normals(&mid.positions, &mid.faces)?,
    };
    let count = mid.positions.len() as u32;
    let mut positions = mid.positions.clone();
    positions.extend(
        mid.positions
            .iter()
            .zip(&mid_normals)
            .map(|(point, normal)| add(*point, scale(*normal, thickness))),
    );
    let mut source_mid_indices = (0..mid.positions.len())
        .chain(0..mid.positions.len())
        .collect::<Vec<_>>();
    let mut welded_indices = (0..count * 2).collect::<Vec<_>>();
    let mut skirt_inner = vec![0_u32; mid.main_columns];
    let mut skirt_outer = vec![0_u32; mid.main_columns];
    if mid.skirt_face_start.is_some() {
        for index in 0..mid.main_columns {
            skirt_inner[index] = positions.len() as u32;
            positions.push(mid.positions[index]);
            source_mid_indices.push(index);
            welded_indices.push(index as u32);
            skirt_outer[index] = positions.len() as u32;
            positions.push(add(
                mid.positions[index],
                scale(mid_normals[index], thickness),
            ));
            source_mid_indices.push(index);
            welded_indices.push(index as u32 + count);
        }
    }
    let mut indices = Vec::with_capacity(mid.faces.len() * 6);
    for (face_index, [a, b, c]) in mid.faces.iter().enumerate() {
        let skirt = mid
            .skirt_face_start
            .is_some_and(|start| face_index >= start);
        let inner = |index: u32| {
            if skirt && (index as usize) < mid.main_columns {
                skirt_inner[index as usize]
            } else {
                index
            }
        };
        let outer = |index: u32| {
            if skirt && (index as usize) < mid.main_columns {
                skirt_outer[index as usize]
            } else {
                index + count
            }
        };
        indices.extend([outer(*a), outer(*b), outer(*c)]);
        indices.extend([inner(*c), inner(*b), inner(*a)]);
    }
    let rim = boundary_edges(&mid.faces)?;
    for edge in &rim {
        append_cut_wall(
            *edge,
            count,
            &mut positions,
            &mut source_mid_indices,
            &mut welded_indices,
            &mut indices,
        );
    }
    let faces = indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|v| [v[0], v[1], v[2]])
        .collect::<Vec<_>>();
    validate_closed_shell(&faces, &welded_indices)?;
    let mut solid = SolidMesh {
        plate_edges: rim
            .into_iter()
            .map(|[a, b]| [a + count, b + count])
            .collect(),
        positions,
        normals: Vec::new(),
        indices,
        source_mid_indices,
    };
    solid.split_medial_crease(&mid);
    solid.normals = vertex_normals(&solid.positions, solid.indices.as_chunks::<3>().0)?;
    Ok(solid)
}

impl SolidMesh {
    fn split_medial_crease(&mut self, mid: &MidMesh) {
        if mid.medial_crease.is_empty() {
            return;
        }
        let mut used_left = vec![false; self.positions.len()];
        for face in self.indices.as_chunks::<3>().0 {
            let right = face
                .iter()
                .any(|index| mid.crease_right[self.source_mid_indices[*index as usize]]);
            if !right {
                for index in face {
                    used_left[*index as usize] = true;
                }
            }
        }
        let mut aliases = BTreeMap::new();
        for face in self.indices.as_chunks_mut::<3>().0 {
            let right = face
                .iter()
                .any(|index| mid.crease_right[self.source_mid_indices[*index as usize]]);
            if !right {
                continue;
            }
            for index in face {
                let original = *index as usize;
                let source = self.source_mid_indices[original];
                if mid.medial_crease[source] && used_left[original] {
                    *index = *aliases.entry(original).or_insert_with(|| {
                        let alias = self.positions.len() as u32;
                        self.positions.push(self.positions[original]);
                        self.source_mid_indices.push(source);
                        alias
                    });
                }
            }
        }
    }
}

pub(super) fn validate_closed_shell(
    faces: &[[u32; 3]],
    welded_indices: &[u32],
) -> Result<(), GenerateError> {
    let mut solid_edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    for [a, b, c] in faces {
        let [a, b, c] = [*a, *b, *c].map(|index| welded_indices[index as usize]);
        for (start, end) in [(a, b), (b, c), (c, a)] {
            let key = if start < end {
                (start, end)
            } else {
                (end, start)
            };
            solid_edges.entry(key).or_default().push((start, end));
        }
    }
    if solid_edges
        .values()
        .any(|uses| !matches!(uses.as_slice(), &[(a, b), (c, d)] if a == d && b == c))
    {
        return Err(GenerateError::Degenerate);
    }
    Ok(())
}

#[cfg(test)]
mod rim_tests {
    use super::*;

    #[test]
    fn front_neckline_gauge_stays_outside_a_twisted_shoulder_strip() {
        let columns = 2;
        let mut mesh = MidMesh {
            main_columns: columns,
            positions: vec![[0.0; 3]; V_SAMPLES * columns],
            extrusion_normals: Some(vec![
                normalized([-0.73, 0.30, 0.68]).unwrap();
                V_SAMPLES * columns
            ]),
            ..MidMesh::default()
        };
        // A receding front neckline from the narrow museum mannequin.
        let start = (V_SAMPLES - 2) * columns;
        mesh.positions[start..].copy_from_slice(&[
            [0.096_027, 1.425_819, 0.087_574],
            [0.099_899, 1.426_434, 0.087_537],
            [0.119_578, 1.437_261, 0.075_968],
            [0.123_352, 1.438_168, 0.072_315],
        ]);
        let face = [start as u32, (start + 3) as u32, (start + 2) as u32];
        let normal = normalized(face_normal(face, &mesh.positions)).unwrap();
        let old = mesh.extrusion_normals.as_ref().unwrap().clone();
        assert!(dot(normal, old[start]) < 0.0);
        let carrier = mesh.positions.clone();
        mesh.front_neckline_extrusion(Frame::from_front([0.0, 0.0, 1.0]).unwrap())
            .unwrap();
        assert_eq!(mesh.positions, carrier);
        let normals = mesh.extrusion_normals.unwrap();
        let unaffected = (V_SAMPLES - UPPER_RIM_ROWS) * columns;
        assert_eq!(normals[..unaffected], old[..unaffected]);
        for index in face {
            let direction = normals[index as usize];
            assert!(dot(normal, direction) > 0.0, "gauge entered the carrier");
            assert!((length(scale(direction, 0.002)) - 0.002).abs() < 1e-7);
        }
    }

    #[test]
    fn lateral_rim_does_not_reverse_under_a_lap_or_wall_offset() {
        let positions = [-0.004, -0.003, -0.002, 0.0, 0.002, 0.003, 0.004];
        let mut directions = [
            [0.0, 0.0, 1.0],
            [-0.8, 0.0, 0.6],
            [-0.8, 0.0, 0.6],
            [0.0, 0.0, 1.0],
            [0.8, 0.0, 0.6],
            [0.8, 0.0, 0.6],
            [0.0, 0.0, 1.0],
        ];
        // Local boundary normals drive the next vertex past the rim.
        assert!(positions[1] + directions[1][0] * 0.005 < positions[0]);
        let interior = directions[2..5].to_vec();
        continue_lateral_extrusion(&mut directions).unwrap();
        assert_eq!(directions[2..5], interior);
        for offset in [0.002, 0.005, 0.007, 0.016] {
            let points = positions
                .into_iter()
                .zip(directions)
                .map(|(x, n)| add([x, 0.0, 0.0], scale(n, offset)))
                .collect::<Vec<_>>();
            for pair in points.windows(2) {
                assert!(pair[1][0] > pair[0][0], "offset reversed the carrier strip");
            }
            for (left, right) in points.iter().zip(points.iter().rev()) {
                assert!((left[0] + right[0]).abs() < 1e-6);
                assert!((left[2] - right[2]).abs() < 1e-6);
            }
        }
    }
}
