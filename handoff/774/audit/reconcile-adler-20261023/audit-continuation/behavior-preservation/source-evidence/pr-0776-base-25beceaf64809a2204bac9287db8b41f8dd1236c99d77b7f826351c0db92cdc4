//! Serialize anatomical equipment sockets into the glTF node hierarchy.

use super::{EQUIPMENT_SOCKET_NODE_PREFIX, RiggedMesh, RiggedSocket};
use anyhow::{Context, Result};
use serde_json::{Value, json};

pub(super) fn append(
    nodes: &mut Vec<Value>,
    mesh: &RiggedMesh<'_>,
    sockets: &[RiggedSocket<'_>],
) -> Result<()> {
    let socket_parent = if sockets.is_empty() {
        None
    } else {
        Some(
            mesh.joint_names
                .iter()
                .position(|name| name == "root")
                .context("MHR skeleton has no anatomical pelvis joint")?,
        )
    };
    let socket_nodes = sockets
        .iter()
        .map(|socket| {
            let node = nodes.len();
            let state = socket.transform.to_skel_state();
            nodes.push(json!({
                "name": format!("{EQUIPMENT_SOCKET_NODE_PREFIX}{}", socket.attachment_point_id),
                "translation": [state[0], state[1], state[2]],
                "rotation": [state[3], state[4], state[5], state[6]],
                "scale": [state[7], state[7], state[7]],
                "extras": {
                    "adventuresim_equipment_socket": {
                        "attachment_point_id": socket.attachment_point_id,
                        "space": "pelvis_local",
                        "surface_uv": {
                            "domain": socket.surface_uv_domain,
                            "uv": socket.surface_uv
                        },
                        "tangent_axis": "+Y",
                        "normal_axis": "+Z"
                    }
                }
            }));
            node
        })
        .collect::<Vec<_>>();
    if let Some(socket_parent) = socket_parent {
        let root_children = nodes[socket_parent]
            .as_object_mut()
            .context("MHR anatomical pelvis node is not an object")?
            .entry("children")
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .context("MHR anatomical pelvis children are not an array")?;
        root_children.extend(socket_nodes.iter().copied().map(Value::from));
    }
    Ok(())
}
