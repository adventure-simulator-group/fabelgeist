use super::*;
use bevy::math::{Quat, Vec3};
use fabelgeist_armor::{AnimeDesign, BreastplateConstruction, Permille};

#[test]
fn anime_courses_are_closed_rigid_and_keep_morph_correspondence() -> Result<()> {
    let (mut body, morphs) = armor_fixture::load(&std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/animations/biped/unarmed/base.glb"
    ))?)?;
    let translation = Vec3::new(0.01, -0.02, 0.015);
    let translated = adventuresim_character_creator::bracer::ForearmMorphSample {
        name: "uniform_translation".into(),
        positions: body
            .positions
            .iter()
            .map(|p| (Vec3::from(*p) + translation).to_array())
            .collect(),
        normals: body.normals.clone(),
        global_joint_states: body.global_joint_states.clone(),
        device: Default::default(),
    };
    let morphs = [morphs.into_iter().next().unwrap(), translated];
    for (scale, count, fluted, deep, side_return, opening_width) in [
        ([1.0; 3], 6, true, false, 1000, 1000),
        ([0.85, 0.9, 1.1], 3, false, false, 1000, 1000),
        ([1.2, 1.1, 0.85], 8, true, false, 1000, 1000),
        ([1.0; 3], 6, true, true, 500, 500),
        ([0.85, 0.9, 1.1], 3, false, true, 500, 500),
        ([1.2, 1.1, 0.85], 8, true, true, 500, 500),
        ([1.0; 3], 6, true, true, 1000, 100),
        ([0.85, 0.9, 1.1], 3, false, true, 1000, 100),
        ([1.2, 1.1, 0.85], 8, true, true, 1000, 100),
    ]
    .into_iter()
    .rev()
    {
        let original = body.clone();
        for p in &mut body.positions {
            for k in 0..3 {
                p[k] *= scale[k];
            }
        }
        for p in &mut body.global_joint_states {
            for k in 0..3 {
                p[k] *= scale[k];
            }
        }
        for n in &mut body.normals {
            *n = (Vec3::from(*n) / Vec3::from(scale)).normalize().to_array();
        }
        let mut design = load_breastplate_design(None)?;
        if !fluted {
            design.fluting = None;
        }
        let mut articulation = AnimeDesign {
            lame_count: count,
            articulated_height: Permille(800),
            ..Default::default()
        };
        if deep {
            design.arm_opening_depth = Permille(1300);
            design.profile.waist_projection = fabelgeist_armor::Millimeters(10);
            articulation.articulated_height = Permille(950);
            articulation.overlap = fabelgeist_armor::Millimeters(40);
            articulation.lap_lift = fabelgeist_armor::Millimeters(12);
        }
        design.side_return = Permille(side_return);
        design.arm_opening_width = Permille(opening_width);
        design.construction = BreastplateConstruction::Anime(articulation);
        let targets = if scale == [1.0; 3] { &morphs[..] } else { &[] };
        {
            let mut solid = design.clone();
            solid.construction = BreastplateConstruction::Solid;
            let source = generate_breastplate_on_device(
                adventuresim_character_creator::armor_gpu()?,
                &solid,
                TorsoSurfaceInput {
                    domain: &body.domain,
                    positions: &body.positions,
                    normals: &body.normals,
                    faces: &body.faces,
                    texcoords: &body.texcoords,
                    texcoord_faces: &body.texcoord_faces,
                    joint_indices: &body.joint_indices,
                    joint_weights: &body.joint_weights,
                    joint_names: &body.joint_names,
                    global_joint_states: &body.global_joint_states,
                    morphs: &[],
                },
            )
            .with_context(|| {
                format!(
                    "solid {count} courses {scale:?} side={side_return} opening={opening_width}"
                )
            })?;
            if let Ok(path) = std::env::var("ARMOR_SOURCE_OUTPUT") {
                let path = std::path::PathBuf::from(path);
                std::fs::create_dir_all(&path)?;
                std::fs::write(
                    path.join(format!("source-{count}-{deep}.json")),
                    serde_json::to_vec(
                        &serde_json::json!({"positions":source.positions,"indices":source.indices,"faces":source.faces,"components":source.components.iter().map(|c|serde_json::json!({"vertices":c.vertices,"indices":c.indices})).collect::<Vec<_>>()}),
                    )?,
                )?;
            }
            shell_validation::assert_closed_oriented_manifold(&source);
            shell_validation::assert_walls_disjoint(&source);
            shell_validation::assert_no_shell_crossings(&source);
            eprintln!("solid {count} courses {scale:?} deep={deep} valid");
        }
        let armor = generate_breastplate_on_device(
            adventuresim_character_creator::armor_gpu()?,
            &design,
            TorsoSurfaceInput {
                domain: &body.domain,
                positions: &body.positions,
                normals: &body.normals,
                faces: &body.faces,
                texcoords: &body.texcoords,
                texcoord_faces: &body.texcoord_faces,
                joint_indices: &body.joint_indices,
                joint_weights: &body.joint_weights,
                joint_names: &body.joint_names,
                global_joint_states: &body.global_joint_states,
                morphs: targets,
            },
        )
        .with_context(|| format!("anime {count} courses {scale:?}"))?;
        if let Ok(path) = std::env::var("ARMOR_TEST_OUTPUT") {
            let path = std::path::PathBuf::from(path);
            std::fs::create_dir_all(&path)?;
            std::fs::write(
                path.join(format!("course-{count}-{deep}.json")),
                serde_json::to_vec(
                    &serde_json::json!({"positions":armor.positions,"indices":armor.indices,"faces":armor.faces,
                    "components":armor.components.iter().map(|c| serde_json::json!({"vertices":c.vertices,"indices":c.indices})).collect::<Vec<_>>()}),
                )?,
            )?;
        }
        assert_eq!(armor.components.len(), 2 * (usize::from(count) + 1));
        shell_validation::assert_closed_oriented_manifold(&armor);
        shell_validation::assert_walls_disjoint(&armor);
        shell_validation::assert_no_shell_crossings(&armor);
        eprintln!("anime {count} courses {scale:?} deep={deep} valid");
        assert_eq!(armor.morphs.len(), targets.len());
        if let Some(translated) = armor.morphs.get(1) {
            for (base, target) in armor.positions.iter().zip(&translated.direct_positions) {
                assert!(
                    (Vec3::from(*target) - Vec3::from(*base) - translation).length() < 2e-6,
                    "course morph must preserve the exact carrier translation"
                );
            }
        }
        assert!(
            armor
                .positions
                .iter()
                .chain(&armor.normals)
                .flatten()
                .all(|v| v.is_finite())
        );
        for target in &armor.morphs {
            assert_eq!(target.direct_positions.len(), armor.positions.len());
            assert!(
                target
                    .direct_positions
                    .iter()
                    .flatten()
                    .all(|v| v.is_finite())
            );
        }
        let mut owners = std::collections::BTreeSet::new();
        for component in &armor.components {
            let range = component.vertices.clone();
            let joint = armor.joint_indices[range.start][0];
            owners.insert(joint);
            assert!(
                armor.joint_indices[range.clone()]
                    .iter()
                    .all(|j| j[0] == joint)
            );
            assert!(
                armor.joint_weights[range.clone()]
                    .iter()
                    .all(|w| *w == [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
            );
            // Distinct posed rotations on neighboring spine joints must not
            // stretch any edge within an individual metal course.
            let rotation = Quat::from_rotation_x(0.3 + joint as f32 * 0.01);
            let mut edges = std::collections::BTreeMap::<_, i32>::new();
            let key = |i: u32| armor.positions[i as usize].map(f32::to_bits);
            for face in armor.indices[component.indices.clone()].as_chunks::<3>().0 {
                let [a, b, c] = face.map(|i| Vec3::from(armor.positions[i as usize]));
                shell_validation::assert_triangle_normal_resolved([a, b, c].map(|p| p.to_array()));
                assert!(((rotation * b - rotation * a).length() - (b - a).length()).abs() < 1e-6);
                for i in 0..3 {
                    let (a, b) = (key(face[i]), key(face[(i + 1) % 3]));
                    *edges.entry((a.min(b), a.max(b))).or_default() += if a < b { 1 } else { -1 };
                }
            }
            assert!(
                edges.values().all(|&balance| balance == 0),
                "course boundary must close"
            );
        }
        assert_eq!(
            owners.len(),
            1,
            "body-spine pivots must not pull course laps apart"
        );
        assert_eq!(
            body.joint_names[*owners.first().unwrap() as usize],
            "c_spine3"
        );
        body = original;
    }
    Ok(())
}
