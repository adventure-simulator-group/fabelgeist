//! Waist metal sheets have one rigid animation owner per disconnected course.
//! Body proportions are fitted by residual morphs, independently of limb motion.
use super::*;
use adventuresim_armor_model::ArmorComponentRole;
use std::ops::Range;

pub(super) fn attach(
    model: &BodyModel,
    body: &GeneratedCharacter,
    sheets: &[Range<usize>],
    armor: &mut GeneratedArmor,
) -> Result<()> {
    let names = &model.mhr.character.skeleton.names;
    let root = names
        .iter()
        .position(|name| name == "root")
        .context("missing pelvic attachment joint")?;
    attach_courses(names, body.global_joint_states[root][0], sheets, armor)
}

fn attach_courses(
    names: &[String],
    center: f32,
    sheets: &[Range<usize>],
    armor: &mut GeneratedArmor,
) -> Result<()> {
    let joint = |name| {
        names
            .iter()
            .position(|n| n == name)
            .map(|index| index as u32)
            .context("missing waist attachment joint")
    };
    let pelvis = joint("root")?;
    let hips = [joint("l_upleg")?, joint("r_upleg")?];
    for component in &armor.components {
        let mut covered = component.vertices.start;
        for vertices in sheets
            .iter()
            .filter(|sheet| component.vertices.contains(&sheet.start))
        {
            anyhow::ensure!(
                vertices.start == covered && vertices.end <= component.vertices.end,
                "waist sheet provenance does not match its component"
            );
            covered = vertices.end;
            let anchor = match component.role {
                ArmorComponentRole::Fauld => pelvis,
                ArmorComponentRole::Tassets => {
                    let middle = vertices.clone().map(|i| armor.positions[i][0]).sum::<f32>()
                        / vertices.len() as f32;
                    hips[usize::from(middle < center)]
                }
                _ => anyhow::bail!("unexpected waist plate component"),
            };
            for vertex in vertices.clone() {
                armor.joint_indices[vertex] = [anchor; 8];
                armor.joint_weights[vertex] = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
            }
        }
        anyhow::ensure!(
            covered == component.vertices.end,
            "waist component lacks sheet provenance"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_armor_model::*;

    #[test]
    fn touching_sheets_keep_distinct_hip_owners_with_hard_normal_returns() {
        let mut mesh = PartMesh::new();
        for sign in [1.0, -1.0] {
            mesh.append(
                PartMesh::from_surface(
                    vec![
                        [0.0, 0.9, 0.08],
                        [sign * 0.2, 0.9, 0.08],
                        [sign * 0.1, 0.7, 0.1],
                    ],
                    vec![0, 1, 2],
                    0.001,
                    BoundaryNormals::Separate,
                    ShellExtrusion::Normal,
                )
                .unwrap(),
            );
        }
        let mesh = mesh.with_component(ArmorComponentRole::Tassets, None);
        let sheets = mesh.shell_vertex_ranges().collect::<Vec<_>>();
        assert_eq!(sheets.len(), 2);
        assert_eq!(
            mesh.positions[sheets[0].start],
            mesh.positions[sheets[1].start]
        );
        let count = mesh.positions.len();
        let mut armor = GeneratedArmor {
            design_hash: [0; 32],
            surface_domain: "test".into(),
            normals: mesh.normals().unwrap(),
            texcoords: vec![[0.0; 2]; count],
            indices: mesh.indices,
            joint_indices: vec![[0; 8]; count],
            joint_weights: vec![[0.0; 8]; count],
            positions: mesh.positions,
            components: mesh.components,
            morphs: vec![],
            plate_edges: vec![],
        };
        let names = ["root", "l_upleg", "r_upleg"].map(str::to_owned);
        attach_courses(&names, 0.0, &sheets, &mut armor).unwrap();
        for (side, sheet) in sheets.iter().enumerate() {
            for vertex in sheet.clone() {
                assert_eq!(armor.joint_indices[vertex][0], side as u32 + 1);
            }
        }
    }

    #[test]
    fn fluted_course_returns_share_the_physical_sheet_owner_across_medial_plane() {
        let mut design = GarmentArmorDesign::new(GarmentArmorKind::Tassets);
        design.fluting = Some(PlateFluting::default());
        design.plate_shape = GarmentPlateShape::WrappedTassets(WrappedTassetDesign {
            inner_gap: Millimeters(0),
            section_break: 2,
            ..WrappedTassetDesign::default()
        });
        let mesh = generate_wrapped_tasset(
            &design,
            TassetSide::Left,
            TassetSpan::new(0.6, 0.9).unwrap(),
            |angle, _height| [0.01 + 0.08 * angle.sin(), 0.08 * angle.cos()],
        )
        .unwrap()
        .with_component(ArmorComponentRole::Tassets, None);
        let sheets = mesh.shell_vertex_ranges().collect::<Vec<_>>();
        let count = mesh.positions.len();
        assert!(
            mesh.positions.iter().any(|p| p[0] < 0.0),
            "fixture must exercise medial relief/return aliases"
        );
        let mut armor = GeneratedArmor {
            design_hash: [0; 32],
            surface_domain: "test".into(),
            normals: mesh.normals().unwrap(),
            texcoords: vec![[0.0; 2]; count],
            indices: mesh.indices,
            joint_indices: vec![[0; 8]; count],
            joint_weights: vec![[0.0; 8]; count],
            positions: mesh.positions,
            components: mesh.components,
            morphs: vec![],
            plate_edges: vec![],
        };
        let names = ["root", "l_upleg", "r_upleg"].map(str::to_owned);
        attach_courses(&names, 0.0, &sheets, &mut armor).unwrap();
        for weights in &armor.joint_indices {
            assert_eq!(
                weights[0], 1,
                "a hard-normal alias selected the opposite hip"
            );
        }
    }

    #[test]
    fn disconnected_waist_courses_preserve_distances_under_independent_limb_motion() {
        let positions = vec![
            [-0.2, 0.9, 0.0],
            [0.2, 0.9, 0.0],
            [0.0, 0.8, 0.1],
            [-0.02, 0.7, 0.1],
            [0.2, 0.7, 0.1],
            [0.1, 0.5, 0.15],
            [0.02, 0.7, 0.1],
            [-0.2, 0.7, 0.1],
            [-0.1, 0.5, 0.15],
        ];
        let component = |role, vertices: std::ops::Range<usize>| ArmorComponent {
            role,
            indices: vertices.clone(),
            vertices,
            hinge: None,
            material: None,
        };
        let mut armor = GeneratedArmor {
            design_hash: [0; 32],
            surface_domain: "test".into(),
            normals: vec![[0.0, 0.0, 1.0]; 9],
            texcoords: vec![[0.0; 2]; 9],
            indices: (0..9).collect(),
            joint_indices: vec![[0; 8]; 9],
            joint_weights: vec![[0.0; 8]; 9],
            positions,
            components: vec![
                component(ArmorComponentRole::Fauld, 0..3),
                component(ArmorComponentRole::Tassets, 3..9),
            ],
            morphs: vec![],
            plate_edges: vec![],
        };
        let names = ["root", "l_upleg", "r_upleg", "l_lowleg", "r_lowleg"].map(str::to_owned);
        attach_courses(&names, 0.0, &[0..3, 3..6, 6..9], &mut armor).unwrap();
        let rotations = [
            Quat::from_rotation_x(0.3),
            Quat::from_rotation_y(0.9),
            Quat::from_rotation_z(-0.7),
            Quat::from_rotation_x(1.0),
            Quat::from_rotation_x(-1.0),
        ];
        let posed: Vec<_> = armor
            .positions
            .iter()
            .enumerate()
            .map(|(i, p)| {
                armor.joint_indices[i]
                    .iter()
                    .zip(armor.joint_weights[i])
                    .map(|(j, w)| rotations[*j as usize] * Vec3::from_array(*p) * w)
                    .sum::<Vec3>()
            })
            .collect();
        for course in 0..3 {
            for i in course * 3..course * 3 + 3 {
                assert_eq!(armor.joint_indices[i][0], course as u32);
                for j in course * 3..course * 3 + 3 {
                    let before = Vec3::from_array(armor.positions[i])
                        .distance(Vec3::from_array(armor.positions[j]));
                    assert!((posed[i].distance(posed[j]) - before).abs() < 1e-6);
                }
            }
        }
    }
}
