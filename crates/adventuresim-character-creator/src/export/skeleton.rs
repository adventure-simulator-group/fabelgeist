use super::*;
use fabelgeist_rig::{RigJointName, RigJointTable};

pub(super) fn nodes(
    joint_names: &fabelgeist_rig::RigJointTable<RigJointName>,
    joint_parents: &RigJointTable<Option<RigJointOrdinal>>,
    globals: &RigJointTable<Transform>,
    mesh: &RiggedMesh<'_>,
) -> Result<(Vec<Value>, Vec<usize>)> {
    validate_proportions(mesh)?;
    let mut children = vec![Vec::<usize>::new(); joint_names.len()];
    let mut roots = Vec::new();
    for (joint, parent) in joint_parents.indexed() {
        match parent {
            None => roots.push(usize::from(joint)),
            Some(parent) => children[usize::from(*parent)].push(usize::from(joint)),
        }
    }
    let mut nodes = Vec::with_capacity(joint_names.len() + 2);
    for (ordinal, global) in globals.indexed() {
        // glTF node/parent arrays are native collection indexes at this serializer.
        let joint = usize::from(ordinal);
        let local = match joint_parents[ordinal] {
            None => *global,
            Some(parent) => globals[parent].inverse().compose(global),
        };
        let state = local.to_skel_state();
        let mut node = json!({
            "name": joint_names[ordinal],
            "translation": [state[0], state[1], state[2]],
            "rotation": [state[3], state[4], state[5], state[6]],
            "scale": [state[7], state[7], state[7]],
        });
        if let Some(basis) = mesh.joint_proportions.get(ordinal) {
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

/// Extend the ordered skeleton while retaining the attachment parent identity.
pub(super) fn append_attachment(
    names: &mut RigJointTable<RigJointName>,
    parents: &mut RigJointTable<Option<RigJointOrdinal>>,
    globals: &mut RigJointTable<Transform>,
    name: &RigJointName,
    parent: RigJointOrdinal,
    transform: Transform,
) {
    names.push(name.clone());
    parents.push(Some(parent));
    globals.push(transform);
}
