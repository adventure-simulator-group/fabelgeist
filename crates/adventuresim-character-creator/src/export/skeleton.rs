use super::*;

pub(super) fn nodes(
    joint_names: &[String],
    joint_parents: &[i32],
    globals: &[Transform],
    mesh: &RiggedMesh<'_>,
) -> Result<(Vec<Value>, Vec<usize>)> {
    validate_proportions(mesh)?;
    let mut children = vec![Vec::<usize>::new(); joint_names.len()];
    let mut roots = Vec::new();
    for (joint, parent) in joint_parents.iter().copied().enumerate() {
        if parent < 0 {
            roots.push(joint);
        } else {
            children[parent as usize].push(joint);
        }
    }
    let mut nodes = Vec::with_capacity(joint_names.len() + 2);
    for joint in 0..joint_names.len() {
        let local = if joint_parents[joint] < 0 {
            globals[joint]
        } else {
            globals[joint_parents[joint] as usize]
                .inverse()
                .compose(&globals[joint])
        };
        let state = local.to_skel_state();
        let mut node = json!({
            "name": joint_names[joint],
            "translation": [state[0], state[1], state[2]],
            "rotation": [state[3], state[4], state[5], state[6]],
            "scale": [state[7], state[7], state[7]],
        });
        if let Some(basis) = mesh.joint_proportions.get(joint) {
            node["extras"] = json!({"adventuresim_proportions": basis});
        }
        if !children[joint].is_empty() {
            node["children"] = json!(children[joint]);
        }
        nodes.push(node);
    }
    Ok((nodes, roots))
}

pub(super) fn validate_proportions(mesh: &RiggedMesh<'_>) -> Result<()> {
    anyhow::ensure!(
        mesh.joint_proportions.is_empty() || mesh.joint_proportions.len() == mesh.joint_names.len(),
        "skeletal bases must match the joint count"
    );
    anyhow::ensure!(
        mesh.joint_proportions.iter().all(|basis| basis.is_finite()),
        "skeletal basis contains a non-finite translation"
    );
    Ok(())
}
