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

mod armor_fixture;

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
