//! Input invariants checked before serializing rigged meshes.

use super::{RiggedMesh, RiggedShell, RiggedSocket, morphs};
use anyhow::{Result, bail};

pub(super) fn validate(
    mesh: &RiggedMesh<'_>,
    shells: &[RiggedShell<'_>],
    sockets: &[RiggedSocket<'_>],
) -> Result<()> {
    let vertices = mesh.positions.len();
    if vertices == 0 || mesh.normals.len() != vertices {
        bail!("positions and normals must contain the same non-zero vertex count");
    }
    if mesh.joint_indices.len() != vertices || mesh.joint_weights.len() != vertices {
        bail!("skinning arrays must match the vertex count");
    }
    let joints = mesh.joint_names.len();
    if joints == 0 || mesh.joint_parents.len() != joints || mesh.global_joint_states.len() != joints
    {
        bail!("joint names, parents, and transforms must have the same non-zero length");
    }
    if joints > u16::MAX as usize {
        bail!("glTF export supports at most 65535 joints");
    }
    for (index, parent) in mesh.joint_parents.iter().copied().enumerate() {
        if parent >= index as i32 || parent < -1 {
            bail!("joint {index} has invalid parent {parent}");
        }
    }
    if mesh
        .faces
        .iter()
        .flatten()
        .any(|index| *index as usize >= vertices)
    {
        bail!("a face references a missing vertex");
    }
    for (vertex, (indices, weights)) in mesh
        .joint_indices
        .iter()
        .zip(mesh.joint_weights)
        .enumerate()
    {
        if indices
            .iter()
            .zip(weights)
            .any(|(joint, weight)| *weight > 0.0 && *joint as usize >= joints)
        {
            bail!("vertex {vertex} references a missing joint");
        }
        let sum: f32 = weights.iter().sum();
        if !sum.is_finite() || (sum - 1.0).abs() > 1e-4 {
            bail!("vertex {vertex} skin weights sum to {sum}, not 1");
        }
    }
    for shell in shells {
        validate_shell(mesh, shell)?;
    }
    morphs::validate(mesh, shells)?;
    let mut socket_ids = std::collections::BTreeSet::new();
    for socket in sockets {
        let state = socket.transform.to_skel_state();
        if socket.attachment_point_id.trim().is_empty()
            || !socket_ids.insert(socket.attachment_point_id)
        {
            bail!("equipment socket attachment-point IDs must be non-empty and unique");
        }
        if state.iter().any(|value| !value.is_finite()) {
            bail!(
                "equipment socket '{}' contains a non-finite transform",
                socket.attachment_point_id
            );
        }
        if socket.surface_uv_domain.trim().is_empty()
            || socket.surface_uv.iter().any(|value| !value.is_finite())
        {
            bail!(
                "equipment socket '{}' has an invalid anatomical surface UV",
                socket.attachment_point_id
            );
        }
    }
    Ok(())
}

fn validate_shell(mesh: &RiggedMesh<'_>, shell: &RiggedShell<'_>) -> Result<()> {
    let vertices = mesh.positions.len();
    let joints = mesh.joint_names.len();
    let shell_vertices = shell.positions.len();
    if shell.name.trim().is_empty() {
        bail!("clothing shell name cannot be empty");
    }
    if shell_vertices == 0 || shell.normals.len() != shell_vertices || shell.faces.is_empty() {
        bail!(
            "clothing shell '{}' must have matching non-empty positions, normals, and faces",
            shell.name
        );
    }
    let (shell_joint_indices, shell_joint_weights) =
        match (shell.joint_indices, shell.joint_weights) {
            (Some(indices), Some(weights)) => (indices, weights),
            (None, None) if shell_vertices == vertices => (mesh.joint_indices, mesh.joint_weights),
            _ => bail!(
                "clothing shell '{}' must supply both skin arrays for independent topology",
                shell.name
            ),
        };
    if shell_joint_indices.len() != shell_vertices || shell_joint_weights.len() != shell_vertices {
        bail!("clothing shell '{}' has mismatched skin arrays", shell.name);
    }
    for (vertex, (indices, weights)) in shell_joint_indices
        .iter()
        .zip(shell_joint_weights)
        .enumerate()
    {
        if indices
            .iter()
            .zip(weights)
            .any(|(joint, weight)| *weight > 0.0 && *joint as usize >= joints)
        {
            bail!("shell vertex {vertex} references a missing joint");
        }
        let sum = weights.iter().sum::<f32>();
        if !sum.is_finite() || (sum - 1.0).abs() > 1e-4 {
            bail!("shell vertex {vertex} skin weights sum to {sum}, not 1");
        }
    }
    if shell
        .faces
        .iter()
        .flatten()
        .any(|index| *index as usize >= shell_vertices)
    {
        bail!(
            "clothing shell '{}' references a missing vertex",
            shell.name
        );
    }
    if shell
        .positions
        .iter()
        .flatten()
        .any(|value| !value.is_finite())
        || shell
            .normals
            .iter()
            .flatten()
            .any(|value| !value.is_finite())
        || !shell.metallic.is_finite()
        || !shell.roughness.is_finite()
        || shell.base_color.iter().any(|value| !value.is_finite())
    {
        bail!(
            "clothing shell '{}' contains a non-finite value",
            shell.name
        );
    }
    Ok(())
}
