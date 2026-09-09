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

pub(super) fn solidify(mid: MidMesh, thickness: f32) -> Result<SolidMesh, GenerateError> {
    let mid_normals = vertex_normals(&mid.positions, &mid.faces)?;
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
    let mut skirt_inner = [0_u32; U_SAMPLES];
    let mut skirt_outer = [0_u32; U_SAMPLES];
    if mid.skirt_face_start.is_some() {
        for index in 0..U_SAMPLES {
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
            if skirt && (index as usize) < U_SAMPLES {
                skirt_inner[index as usize]
            } else {
                index
            }
        };
        let outer = |index: u32| {
            if skirt && (index as usize) < U_SAMPLES {
                skirt_outer[index as usize]
            } else {
                index + count
            }
        };
        indices.extend([outer(*a), outer(*b), outer(*c)]);
        indices.extend([inner(*c), inner(*b), inner(*a)]);
    }
    let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    for [a, b, c] in &mid.faces {
        for (start, end) in [(*a, *b), (*b, *c), (*c, *a)] {
            let key = if start < end {
                (start, end)
            } else {
                (end, start)
            };
            edges.entry(key).or_default().push((start, end));
        }
    }
    for uses in edges.values() {
        match uses.as_slice() {
            &[(a, b)] => indices.extend([a, b, b + count, a, b + count, a + count]),
            &[(first_start, first_end), (second_start, second_end)]
                if first_start == second_end && first_end == second_start => {}
            [_, _] => return Err(GenerateError::Degenerate),
            _ => return Err(GenerateError::Degenerate),
        }
    }
    let faces = indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|v| [v[0], v[1], v[2]])
        .collect::<Vec<_>>();
    validate_closed_shell(&faces, &welded_indices)?;
    let normals = vertex_normals(&positions, &faces)?;
    Ok(SolidMesh {
        positions,
        normals,
        indices,
        source_mid_indices,
    })
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

pub(super) fn translate(mesh: &mut MidMesh, offset: [f32; 3]) {
    for position in &mut mesh.positions {
        *position = add(*position, offset);
    }
}
