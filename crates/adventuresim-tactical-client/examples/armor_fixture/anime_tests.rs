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
    for (scale, count, fluted) in [
        ([1.0; 3], 6, true),
        ([0.85, 0.9, 1.1], 3, false),
        ([1.2, 1.1, 0.85], 8, true),
    ] {
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
        design.construction = BreastplateConstruction::Anime(AnimeDesign {
            lame_count: count,
            articulated_height: Permille(800),
            ..Default::default()
        });
        let targets = if scale == [1.0; 3] { &morphs[..] } else { &[] };
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
        assert_eq!(armor.components.len(), 2 * (usize::from(count) + 1));
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
                assert!((b - a).cross(c - a).length_squared() > 1e-18);
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
        assert!(
            owners.len() > 1,
            "articulated courses must follow distinct spine anchors"
        );
        body = original;
    }
    Ok(())
}
