use super::*;
use bevy::math::Vec3;

#[test]
fn pauldron_shared_saddle_fits_body_and_completed_cuirass() -> Result<()> {
    let (original, _) = armor_fixture::load(&std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/animations/biped/unarmed/base.glb"
    ))?)?;
    let bracer = load_bracer_design(None)?;
    let breastplate = load_breastplate_design(None)?;
    for scale in [[1.0; 3], [0.85, 0.9, 1.1], [1.2, 1.1, 0.85]] {
        let mut body = original.clone();
        for p in &mut body.positions {
            for axis in 0..3 {
                p[axis] *= scale[axis];
            }
        }
        for p in &mut body.global_joint_states {
            for axis in 0..3 {
                p[axis] *= scale[axis];
            }
        }
        for n in &mut body.normals {
            *n = (Vec3::from(*n) / Vec3::from(scale)).normalize().to_array();
        }
        let fit = |item, side, layers| {
            pollster::block_on(runtime_equipment::generate(
                &body,
                item,
                side,
                &bracer,
                &breastplate,
                layers,
            ))
        };
        let cuirass = fit("cuirass", "worn", &[])?;
        let lower = [&cuirass];
        for side in ["left", "right"] {
            let bare = fit("pauldron", side, &[])?;
            let dressed = fit("pauldron", side, &lower)?;
            assert!(bare.morphs.is_empty());
            assert_eq!(bare.indices, dressed.indices);
            let adventuresim_character_creator::armor_recipes::ParametricDesign::Limb(
                fabelgeist_armor::LimbArmorDesign::Pauldron(recipe),
            ) = adventuresim_character_creator::armor_recipes::recipe("pauldron").unwrap()
            else {
                panic!("pauldron catalog recipe");
            };
            assert_eq!(
                dressed.components.len(),
                1 + usize::from(recipe.upper_lames) + usize::from(recipe.lower_lames)
            );
            let shift = bare
                .positions
                .iter()
                .zip(&dressed.positions)
                .map(|(a, b)| Vec3::from(*a).distance(Vec3::from(*b)))
                .fold(0.0_f32, f32::max);
            assert!(
                shift > 0.001,
                "completed cuirass must influence shoulder fit: {shift} m"
            );
            let owner = body
                .joint_names
                .iter()
                .position(|name| name == if side == "left" { "l_uparm" } else { "r_uparm" })
                .unwrap() as u32;
            for armor in [&bare, &dressed] {
                for component in &armor.components {
                    assert!(
                        armor.joint_indices[component.vertices.clone()]
                            .iter()
                            .all(|j| *j == [owner; 8])
                    );
                }
                assert!(
                    armor
                        .joint_weights
                        .iter()
                        .all(|w| *w == [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
                );
                assert!(
                    armor
                        .positions
                        .iter()
                        .chain(&armor.normals)
                        .flatten()
                        .all(|v| v.is_finite())
                );
                let mut edges = std::collections::BTreeMap::<_, i32>::new();
                let key = |i: u32| armor.positions[i as usize].map(f32::to_bits);
                for face in armor.indices.as_chunks::<3>().0 {
                    let [a, b, c] = face.map(|i| Vec3::from(armor.positions[i as usize]));
                    assert!((b - a).cross(c - a).length_squared() > 1e-18);
                    for i in 0..3 {
                        let (a, b) = (key(face[i]), key(face[(i + 1) % 3]));
                        *edges.entry((a.min(b), a.max(b))).or_default() +=
                            if a < b { 1 } else { -1 };
                    }
                }
                assert!(
                    edges.values().all(|&balance| balance == 0),
                    "shoulder plates must close"
                );
            }
        }
    }
    Ok(())
}
