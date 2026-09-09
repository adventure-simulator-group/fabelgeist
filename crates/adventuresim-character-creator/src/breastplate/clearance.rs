//! Map selected shoulder triangles and corresponding body poses into the clearance domain.

use super::*;

pub(super) fn build(
    input: &TorsoSurfaceInput<'_>,
    shoulder_faces: &[[u32; 3]],
    section_faces: &[[u32; 3]],
) -> Result<TorsoClearanceMesh, String> {
    let mut clearance_source = BTreeMap::<u32, u32>::new();
    let mut clearance_body_vertices = Vec::<usize>::new();
    let clearance_faces = shoulder_faces
        .iter()
        .map(|body_face| {
            body_face.map(|body_vertex| {
                *clearance_source.entry(body_vertex).or_insert_with(|| {
                    clearance_body_vertices.push(body_vertex as usize);
                    (clearance_body_vertices.len() - 1) as u32
                })
            })
        })
        .collect::<Vec<_>>();
    let clearance_mesh = TorsoClearanceMesh {
        base: TorsoClearancePose::from_full_body(
            input.positions,
            input.normals,
            &clearance_body_vertices,
        )
        .map_err(|e| e.to_string())?,
        faces: clearance_faces,
        enclosure_faces: input.faces.to_vec(),
        enclosure_torso_faces: section_faces.to_vec(),
        enclosure_texcoords: input.texcoords.to_vec(),
        enclosure_texcoord_faces: input.texcoord_faces.to_vec(),
        enclosure_joint_indices: input.joint_indices.to_vec(),
        enclosure_joint_weights: input.joint_weights.to_vec(),
        morphs: input
            .morphs
            .iter()
            .map(|morph| {
                TorsoClearancePose::from_full_body(
                    &morph.positions,
                    &morph.normals,
                    &clearance_body_vertices,
                )
                .map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<_>, String>>()?,
    };
    Ok(clearance_mesh)
}
