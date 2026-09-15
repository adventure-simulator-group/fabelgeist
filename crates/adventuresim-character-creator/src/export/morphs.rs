//! glTF morph channels shared by every primitive of the exported mesh.

use super::*;

fn channels<'a>(
    mesh: &'a RiggedMesh<'a>,
    shells: &'a [RiggedShell<'a>],
) -> Vec<(&'a [RiggedMorphTarget<'a>], usize)> {
    mesh.export_body
        .then_some((mesh.morph_targets, mesh.positions.len()))
        .into_iter()
        .chain(
            shells
                .iter()
                .map(|shell| (shell.morph_targets, shell.positions.len())),
        )
        .collect()
}

pub(super) fn validate(mesh: &RiggedMesh<'_>, shells: &[RiggedShell<'_>]) -> Result<()> {
    let channels = channels(mesh, shells);
    let Some((first, _)) = channels.first() else {
        return Ok(());
    };
    for (targets, vertices) in &channels {
        if !targets
            .iter()
            .map(|target| target.name)
            .eq(first.iter().map(|target| target.name))
        {
            bail!("all body and equipment primitives must have identical ordered morph names");
        }
        let mut names = std::collections::BTreeSet::new();
        for target in *targets {
            if target.name.trim().is_empty()
                || !names.insert(target.name)
                || target.position_deltas.len() != *vertices
                || target.normal_deltas.len() != *vertices
                || target
                    .position_deltas
                    .iter()
                    .flatten()
                    .chain(target.normal_deltas.iter().flatten())
                    .any(|value| !value.is_finite())
            {
                bail!("invalid morph target '{}'", target.name);
            }
        }
    }
    Ok(())
}

pub(super) fn write(
    mesh: &RiggedMesh<'_>,
    shells: &[RiggedShell<'_>],
    exported: &mut Value,
    buffer: &mut BufferBuilder,
    accessors: &mut Vec<Value>,
) {
    let channels = channels(mesh, shells);
    let Some((first, _)) = channels.first() else {
        return;
    };
    if first.is_empty() {
        return;
    }
    exported["weights"] = json!(vec![0.0_f32; first.len()]);
    exported["extras"] = json!({
        "targetNames": first.iter().map(|target| target.name).collect::<Vec<_>>(),
        "adventuresim_anatomical_uv_domain": MHR_ANATOMICAL_UV_DOMAIN,
    });
    for (primitive, (targets, _)) in exported["primitives"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .zip(channels)
    {
        primitive["targets"] = json!(
            targets
                .iter()
                .map(|target| {
                    let position = append_accessor(target.position_deltas, true, buffer, accessors);
                    let normal = append_accessor(target.normal_deltas, false, buffer, accessors);
                    json!({"POSITION": position, "NORMAL": normal})
                })
                .collect::<Vec<_>>()
        );
    }
}

fn append_accessor(
    values: &[[f32; 3]],
    bounds: bool,
    buffer: &mut BufferBuilder,
    accessors: &mut Vec<Value>,
) -> usize {
    let view = buffer.push(&f32_bytes(values.iter().flatten().copied()), Some(34_962));
    let mut accessor =
        json!({"bufferView": view, "componentType": 5_126, "count": values.len(), "type": "VEC3"});
    if bounds {
        let (minimum, maximum) = position_bounds(values);
        accessor["min"] = json!(minimum);
        accessor["max"] = json!(maximum);
    }
    let index = accessors.len();
    accessors.push(accessor);
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_incompatible_channels_and_non_finite_deltas() {
        let deltas = [[0.1, 0.0, 0.0]];
        let body_targets = [RiggedMorphTarget {
            name: "mhr_identity_00",
            position_deltas: &deltas,
            normal_deltas: &deltas,
        }];
        let mesh = RiggedMesh {
            joint_proportions: &[],
            positions: &deltas,
            normals: &deltas,
            faces: &[],
            export_body: true,
            morph_targets: &body_targets,
            joint_indices: &[],
            joint_weights: &[],
            joint_names: &[],
            joint_parents: &[],
            global_joint_states: &[],
        };
        let mut shell = RiggedShell {
            plate_edges: &[],
            textures: None,
            texcoords: None,
            hinge: None,
            name: "test",
            positions: &deltas,
            normals: &deltas,
            faces: &[],
            joint_indices: None,
            joint_weights: None,
            morph_targets: &[],
            base_color: [1.0; 4],
            metallic: 0.0,
            roughness: 1.0,
        };
        assert!(validate(&mesh, std::slice::from_ref(&shell)).is_err());
        shell.morph_targets = &body_targets;
        assert!(validate(&mesh, std::slice::from_ref(&shell)).is_ok());
        let invalid = [RiggedMorphTarget {
            name: "mhr_identity_00",
            position_deltas: &[[f32::NAN; 3]],
            normal_deltas: &deltas,
        }];
        shell.morph_targets = &invalid;
        assert!(validate(&mesh, &[shell]).is_err());
    }
}
