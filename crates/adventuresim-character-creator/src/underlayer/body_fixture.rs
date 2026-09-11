use super::*;
#[test]
#[ignore = "requires an exported MHR review body in UNDERLAYER_BODY"]
fn exported_body_has_valid_underlayers() -> Result<()> {
    let path = std::env::var("UNDERLAYER_BODY")?;
    let row: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    macro_rules! field {
        ($name:literal) => {
            serde_json::from_value(row[$name].clone())?
        };
    }
    let positions: Vec<_> = field!("positions");
    let normals: Vec<_> = field!("normals");
    let faces: Vec<_> = field!("faces");
    let joint_indices: Vec<_> = field!("joint_indices");
    let joint_weights: Vec<_> = field!("joint_weights");
    let joint_names: Vec<_> = field!("joint_names");
    let joints: Vec<_> = field!("joints");
    let uv_faces: Vec<[u32; 3]> = field!("texcoord_faces");
    let body = Wearer {
        positions: &positions,
        normals: &normals,
        faces: &faces,
        joint_indices: &joint_indices,
        joint_weights: &joint_weights,
        joint_names: &joint_names,
        joints: &joints,
    };
    for (kind, placement) in [
        (UnderlayerKind::ArmingDoublet, "worn"),
        (UnderlayerKind::PaddedHose, "left"),
        (UnderlayerKind::PaddedHose, "right"),
        (UnderlayerKind::MailVoiders, "worn"),
        (UnderlayerKind::MailBrayette, "worn"),
        (UnderlayerKind::MailKneeVoider, "left"),
        (UnderlayerKind::MailKneeVoider, "right"),
        (UnderlayerKind::MailStandard, "worn"),
    ] {
        let mail = kind.is_mail();
        let design = UnderlayerDesign {
            kind,
            clearance: Millimeters(if mail {
                6
            } else if kind == UnderlayerKind::ArmingDoublet {
                4
            } else {
                1
            }),
            thickness: Millimeters(if mail || kind == UnderlayerKind::ArmingDoublet {
                1
            } else {
                2
            }),
            length: Permille(if kind == UnderlayerKind::PaddedHose {
                1000
            } else {
                std::env::var("UNDERLAYER_LENGTH")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1000)
            }),
            sleeve_length: Permille(1000),
            patch_width: Millimeters(
                std::env::var("UNDERLAYER_PATCH_WIDTH")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(80),
            ),
            cuts: vec![],
        };
        let pattern = UnderlayerPattern::new(&design, placement, &body, &uv_faces)?;
        let mesh = pattern.evaluate(&design, &body);
        let normals = mesh.normals()?;
        let texcoords: Vec<[f32; 2]> = field!("texcoords");
        let uv = pattern
            .cut
            .points
            .iter()
            .map(|s| {
                interpolate(
                    uv_faces[s.triangle].map(|i| texcoords[i as usize]),
                    s.weights,
                )
            })
            .collect::<Vec<_>>();
        let uv = pattern.shell_attributes(&uv);
        check_tangents(&mesh, &normals, &uv)?;
        if let Ok(output) = std::env::var("UNDERLAYER_REVIEW_OUTPUT") {
            let output = std::path::Path::new(&output);
            std::fs::create_dir_all(output)?;
            std::fs::write(output.join("body.json"), serde_json::to_vec(&row)?)?;
            let id = match kind {
                UnderlayerKind::ArmingDoublet => "arming_doublet",
                UnderlayerKind::PaddedHose => "padded_chausses",
                UnderlayerKind::MailVoiders => "mail_voiders",
                UnderlayerKind::MailBrayette => "mail_brayette",
                UnderlayerKind::MailKneeVoider => "mail_knee_voider",
                UnderlayerKind::MailStandard => "mail_standard",
            };
            std::fs::write(
                output.join(format!("{id}--{placement}.json")),
                serde_json::to_vec(
                    &serde_json::json!({"id":id,"placement":placement,"positions":mesh.positions,"normals":normals,"indices":mesh.indices,"texcoords":uv,"outer_vertex_count":pattern.cut.points.len(),"outer_triangle_count":pattern.cut.faces.len(),"design":design}),
                )?,
            )?;
        }
    }
    Ok(())
}

fn check_tangents(mesh: &PartMesh, normals: &[[f32; 3]], uv: &[[f32; 2]]) -> Result<()> {
    use bevy::{
        asset::RenderAssetUsages,
        mesh::{Indices, Mesh, PrimitiveTopology, VertexAttributeValues},
    };
    let mut preview = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh.positions.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals.to_vec())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv.to_vec())
    .with_inserted_indices(Indices::U32(mesh.indices.clone()));
    preview.generate_tangents()?;
    let Some(VertexAttributeValues::Float32x4(values)) = preview.attribute(Mesh::ATTRIBUTE_TANGENT)
    else {
        anyhow::bail!("normal-mapped underlayer has no tangents");
    };
    ensure!(
        values.iter().flatten().all(|v| v.is_finite()),
        "invalid underlayer tangent"
    );
    Ok(())
}
