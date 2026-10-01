//! Run with --no-default-features. Uses the actual canonical GLB as input only;
//! generated equipment is never saved or loaded from disk.
use adventuresim_character_creator::{
    bracer::{ForearmSide, ForearmSurfaceInput},
    design_input::{load_bracer_design, load_breastplate_design},
    device_bracer::generate_bracer_on_device,
    device_torso::{TorsoSurfaceInput, generate_breastplate_on_device},
    runtime_equipment,
};
use anyhow::{Context, Result};
use std::time::Instant;

#[cfg(test)]
#[path = "armor_fixture/anime_tests.rs"]
mod anime_tests;
mod armor_fixture;
#[cfg(test)]
#[path = "armor_fixture/pauldron_tests.rs"]
mod pauldron_tests;

fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .context("usage: runtime_equipment_bench BODY.glb [ROUNDS]")?;
    let rounds: usize = std::env::args()
        .nth(2)
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(3);
    let (body, morphs) = armor_fixture::load(&std::fs::read(path)?)?;
    let bracer = load_bracer_design(None)?;
    let breastplate = load_breastplate_design(None)?;
    let started = Instant::now();
    let gpu = adventuresim_character_creator::armor_gpu()?;
    println!(
        "{}",
        serde_json::json!({"stage":"device_open", "ms":started.elapsed().as_secs_f64()*1000.0, "body_vertices":body.positions.len(), "body_targets":morphs.len()})
    );
    for round in 0..rounds {
        for targets in [&[][..], &morphs[..]] {
            for (item, generator) in [
                (
                    "vambrace",
                    adventuresim_character_creator::armor_recipes::DedicatedGenerator::Vambrace,
                ),
                (
                    "breastplate",
                    adventuresim_character_creator::armor_recipes::DedicatedGenerator::Breastplate,
                ),
            ] {
                let started = Instant::now();
                let armor = if matches!(
                    generator,
                    adventuresim_character_creator::armor_recipes::DedicatedGenerator::Vambrace
                ) {
                    generate_bracer_on_device(
                        gpu,
                        &bracer,
                        ForearmSurfaceInput {
                            domain: &body.domain,
                            side: ForearmSide::Left,
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
                } else {
                    generate_breastplate_on_device(
                        gpu,
                        &breastplate,
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
                };
                report(item, round, targets.len(), started, armor)?;
            }
        }
        for (item, placement) in [
            ("gorget", "worn"),
            ("sallet", "worn"),
            ("mail_standard", "worn"),
            ("linen_tunic", "worn"),
            ("leather_boot", "left"),
        ] {
            let started = Instant::now();
            let result = pollster::block_on(runtime_equipment::generate(
                &body,
                item,
                placement,
                &bracer,
                &breastplate,
                &[],
            ));
            report(item, round, 0, started, result)?;
        }
    }
    Ok(())
}

fn report(
    item: &str,
    round: usize,
    targets: usize,
    started: Instant,
    result: Result<fabelgeist_armor::GeneratedArmor>,
) -> Result<()> {
    let ms = started.elapsed().as_secs_f64() * 1000.0;
    match result {
        Ok(armor) => {
            anyhow::ensure!(
                armor.positions.iter().flatten().all(|v| v.is_finite()),
                "non-finite mesh"
            );
            anyhow::ensure!(
                armor
                    .indices
                    .iter()
                    .all(|&i| (i as usize) < armor.positions.len()),
                "invalid triangle"
            );
            anyhow::ensure!(
                armor.morphs.len() == targets,
                "unexpected equipment targets"
            );
            println!(
                "{}",
                serde_json::json!({"item":item,"round":round,"targets":targets,"ms":ms,"vertices":armor.positions.len(),"triangles":armor.indices.len()/3})
            );
        }
        Err(error) => {
            println!(
                "{}",
                serde_json::json!({"item":item,"round":round,"targets":targets,"ms":ms,"error":format!("{error:#}")})
            );
            return Err(error);
        }
    }
    Ok(())
}

#[test]
fn gorget_fits_sparse_body_sections() -> Result<()> {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/animations/biped/unarmed/base.glb"
    ))?;
    let bracer = load_bracer_design(None)?;
    let breastplate = load_breastplate_design(None)?;
    for scale in [[1.0; 3], [0.85, 0.9, 1.1], [1.2, 1.1, 0.85]] {
        let (mut body, _) = armor_fixture::load(&bytes)?;
        for position in &mut body.positions {
            for axis in 0..3 {
                position[axis] *= scale[axis];
            }
        }
        for joint in &mut body.global_joint_states {
            for axis in 0..3 {
                joint[axis] *= scale[axis];
            }
        }
        for normal in &mut body.normals {
            *normal = (bevy::math::Vec3::from(*normal) / bevy::math::Vec3::from(scale))
                .normalize()
                .to_array();
        }
        let armor = pollster::block_on(runtime_equipment::generate(
            &body,
            "gorget",
            "worn",
            &bracer,
            &breastplate,
            &[],
        ))?;
        assert!(armor.morphs.is_empty());
        assert!(!armor.indices.is_empty());
        assert!(armor.positions.iter().flatten().all(|v| v.is_finite()));
        for face in armor.indices.as_chunks::<3>().0 {
            let [a, b, c] = face.map(|i| bevy::math::Vec3::from(armor.positions[i as usize]));
            assert!((b - a).cross(c - a).length_squared() > 1e-18);
        }
    }
    Ok(())
}

#[test]
fn puffed_garments_fit_sparse_wearers_without_morph_targets() -> Result<()> {
    use fabelgeist_armor::ArmorComponentRole;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/animations/biped/unarmed/base.glb"
    ))?;
    let bracer = load_bracer_design(None)?;
    let breastplate = load_breastplate_design(None)?;
    for scale in [[1.0; 3], [0.85, 0.9, 1.1], [1.2, 1.1, 0.85]] {
        let (mut body, _) = armor_fixture::load(&bytes)?;
        for position in &mut body.positions {
            for axis in 0..3 {
                position[axis] *= scale[axis];
            }
        }
        for joint in &mut body.global_joint_states {
            for axis in 0..3 {
                joint[axis] *= scale[axis];
            }
        }
        for normal in &mut body.normals {
            *normal = (bevy::math::Vec3::from(*normal) / bevy::math::Vec3::from(scale))
                .normalize()
                .to_array();
        }
        for item in ["puffed_sleeve", "puffed_hose"] {
            for side in ["left", "right"] {
                let armor = pollster::block_on(runtime_equipment::generate(
                    &body,
                    item,
                    side,
                    &bracer,
                    &breastplate,
                    &[],
                ))
                .with_context(|| format!("{item} {side} at {scale:?}"))?;
                assert!(armor.morphs.is_empty());
                assert!(armor.positions.iter().flatten().all(|v| v.is_finite()));
                let outer = armor
                    .components
                    .iter()
                    .find(|c| c.role == ArmorComponentRole::OuterFabric)
                    .unwrap();
                let inner = armor
                    .components
                    .iter()
                    .find(|c| c.role == ArmorComponentRole::Undercloth)
                    .unwrap();
                assert_ne!(outer.material, inner.material);
                assert!(outer.material.is_some() && inner.material.is_some());
                for face in armor.indices.as_chunks::<3>().0 {
                    let [a, b, c] =
                        face.map(|i| bevy::math::Vec3::from(armor.positions[i as usize]));
                    assert!(
                        (b - a).cross(c - a).length_squared() > 1e-18,
                        "degenerate {item} triangle"
                    );
                }
                for weights in &armor.joint_weights {
                    assert!((weights.iter().sum::<f32>() - 1.0).abs() < 1e-4);
                }
            }
        }
    }
    Ok(())
}

#[test]
fn puffed_garment_extremes_are_closed_finite_shells() -> Result<()> {
    use adventuresim_character_creator::{
        armor_recipes::ParametricDesign,
        device_fit::{self, FitBody, Realization},
    };
    use fabelgeist_armor::{Permille, PuffAndSlashDesign};
    let (body, _) = armor_fixture::load(&std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/animations/biped/unarmed/base.glb"
    ))?)?;
    let fit_body = FitBody {
        faces: &body.faces,
        texcoords: &body.texcoords,
        joint_indices: &body.joint_indices,
        joint_weights: &body.joint_weights,
        joint_names: &body.joint_names,
    };
    let wearer = Realization {
        positions: &body.positions,
        normals: &body.normals,
        joints: &body.global_joint_states,
        device: &body.device,
    };
    for (puff_count, slash_count) in [(1, 0), (8, 16)] {
        let design = ParametricDesign::PuffAndSlash(PuffAndSlashDesign {
            puff_count,
            slash_count,
            length: Permille(1000),
            proximal_position: Permille(1000),
            ..Default::default()
        });
        let piece = device_fit::fit_recipe(
            adventuresim_character_creator::armor_gpu()?,
            &fit_body,
            &wearer,
            &[],
            &design,
            "left",
            &[],
        )?;
        assert!(piece.endpoints.is_empty());
        let part = piece.base;
        assert!(
            part.positions
                .iter()
                .chain(&part.normals)
                .flatten()
                .all(|v| v.is_finite())
        );
        assert_eq!(part.components.len(), 2);
        // Every shell must have positive area, including the fully unslashed
        // middle. Closed shells have no unpaired geometric boundary edges.
        let mut edges = std::collections::BTreeMap::<_, i32>::new();
        let key = |i: u32| part.positions[i as usize].map(f32::to_bits);
        for face in part.indices.as_chunks::<3>().0 {
            let [a, b, c] = face.map(|i| bevy::math::Vec3::from(part.positions[i as usize]));
            assert!((b - a).cross(c - a).length_squared() > 1e-18);
            for i in 0..3 {
                let (a, b) = (key(face[i]), key(face[(i + 1) % 3]));
                *edges.entry((a.min(b), a.max(b))).or_default() += if a < b { 1 } else { -1 };
            }
        }
        assert!(
            edges.values().all(|&balance| balance == 0),
            "open or reversed shell at {puff_count} puffs, {slash_count} slashes"
        );
    }
    Ok(())
}

#[test]
fn puffed_sleeve_seats_over_generated_padding() -> Result<()> {
    let (body, _) = armor_fixture::load(&std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/animations/biped/unarmed/base.glb"
    ))?)?;
    let bracer = load_bracer_design(None)?;
    let breastplate = load_breastplate_design(None)?;
    let fit = |item, layers| {
        pollster::block_on(runtime_equipment::generate(
            &body,
            item,
            "left",
            &bracer,
            &breastplate,
            layers,
        ))
    };
    let padding = fit("quilted_sleeve", &[])?;
    let bare = fit("puffed_sleeve", &[])?;
    let clothed = fit("puffed_sleeve", &[&padding])?;
    assert_eq!(bare.indices, clothed.indices);
    let maximum_shift = bare
        .positions
        .iter()
        .zip(&clothed.positions)
        .map(|(a, b)| bevy::math::Vec3::from(*a).distance(bevy::math::Vec3::from(*b)))
        .fold(0.0_f32, f32::max);
    assert!(
        maximum_shift > 0.001,
        "lower padding must affect the fit: {maximum_shift} m"
    );
    assert!(
        clothed
            .positions
            .iter()
            .chain(&clothed.normals)
            .flatten()
            .all(|v| v.is_finite())
    );
    assert!(clothed.morphs.is_empty());
    Ok(())
}

#[cfg(test)]
#[path = "armor_fixture/tasset_tests.rs"]
mod tasset_tests;
