//! Solidify carrier meshes and validate welded shell closure.

use super::*;

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

/// Continue the upper carrier's extrusion field from the interior. The curved
/// trim can have a tighter radius than the plate gauge, so its local boundary
/// normals must not make the outer wall fold over itself.
pub(super) fn upper_extrusion(mesh: &mut MidMesh) -> Result<(), GenerateError> {
    let mut normals = vertex_normals(&mesh.positions, &mesh.faces)?;
    const UPPER_RIM_ROWS: usize = 3;
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
    mesh.extrusion_normals = Some(normals);
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
