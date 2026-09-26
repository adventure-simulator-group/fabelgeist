//! Reconstruct body morph endpoints from the loaded base mesh.

use super::*;

pub(super) fn runtime_body_morphs(
    mesh: &Mesh,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    global_joint_states: &[[f32; 8]],
) -> Vec<RuntimeBodyMorph> {
    let Some(targets) = mesh.get_morph_targets() else {
        return Vec::new();
    };
    let names = mesh.morph_target_names().unwrap_or_default();
    let vertex_count = positions.len();
    names
        .iter()
        .enumerate()
        .filter_map(|(index, name)| {
            let start = index.checked_mul(vertex_count)?;
            let target = targets.get(start..start + vertex_count)?;
            Some(RuntimeBodyMorph {
                name: name.clone(),
                positions: positions
                    .iter()
                    .zip(target)
                    .map(|(base, delta)| {
                        let delta = delta.position.to_array();
                        [base[0] + delta[0], base[1] + delta[1], base[2] + delta[2]]
                    })
                    .collect(),
                normals: normals
                    .iter()
                    .zip(target)
                    .map(|(base, delta)| {
                        let delta = delta.normal.to_array();
                        [base[0] + delta[0], base[1] + delta[1], base[2] + delta[2]]
                    })
                    .collect(),
                global_joint_states: global_joint_states.to_vec(),
            })
        })
        .collect()
}
