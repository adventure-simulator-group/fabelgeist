//! Native timing probe for client-generated equipment construction ports.
use adventuresim_character_creator::{
    armor_recipes::ParametricDesign,
    design_input::{load_bracer_design, load_breastplate_design},
    device_fit::{self, FitBody, Fitted, Realization},
    runtime_equipment::{self, RuntimeBody},
};
use anyhow::{Context, Result};
use fabelgeist_armor::{
    ArmorGpu, BreastplateConstruction, GarmentArmorDesign, GarmentArmorKind, GarmentPlateShape,
    GeneratedArmor, Millimeters, Permille, WrappedTassetDesign,
};
use std::time::Instant;
mod armor_fixture;

fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .context("usage: runtime_equipment_ports BODY.glb [ROUNDS]")?;
    let rounds = std::env::args()
        .nth(2)
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(6);
    let (body, _) = armor_fixture::load(&std::fs::read(path)?)?;
    let gpu = adventuresim_character_creator::armor_gpu()?;
    let bracer = load_bracer_design(None)?;
    let breastplate = load_breastplate_design(None)?;
    let mut anime = breastplate.clone();
    anime.construction = BreastplateConstruction::Anime(Default::default());
    let cuirass = pollster::block_on(runtime_equipment::generate(
        &body,
        "cuirass",
        "worn",
        &bracer,
        &breastplate,
        &[],
    ))?;
    for round in 0..rounds {
        for (name, item, placement, design, layers) in [
            (
                "puffed_sleeve",
                "puffed_sleeve",
                "left",
                &breastplate,
                &[][..],
            ),
            ("puffed_hose", "puffed_hose", "left", &breastplate, &[][..]),
            ("pauldron", "pauldron", "left", &breastplate, &[][..]),
            (
                "pauldron_over_cuirass",
                "pauldron",
                "left",
                &breastplate,
                &[&cuirass][..],
            ),
            ("anime", "cuirass", "worn", &anime, &[][..]),
        ] {
            let start = Instant::now();
            let armor = pollster::block_on(runtime_equipment::generate(
                &body, item, placement, &bracer, design, layers,
            ))?;
            report(name, round, start, &armor)?;
        }
        for slope in [0, 400] {
            let start = Instant::now();
            let armor = wrapped(gpu, &body, slope)?;
            report(
                &format!("wrapped_tassets_slope_{slope}"),
                round,
                start,
                &armor,
            )?;
        }
    }
    Ok(())
}

fn wrapped(gpu: &ArmorGpu, body: &RuntimeBody, slope: u16) -> Result<GeneratedArmor> {
    let mut garment = GarmentArmorDesign::new(GarmentArmorKind::Tassets);
    garment.lame_count = 8;
    garment.wall_thickness = Millimeters(1);
    garment.plate_shape = GarmentPlateShape::WrappedTassets(WrappedTassetDesign {
        upper_edge_slope: Permille(slope),
        ..Default::default()
    });
    let design = ParametricDesign::Garment(garment);
    let topology = FitBody {
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
    let piece = device_fit::fit_recipe(gpu, &topology, &wearer, &[], &design, "worn", &[])?;
    device_fit::assemble_recipe(
        &design,
        piece,
        &Fitted {
            placement: "worn",
            morphs: &[],
            domain: &body.domain,
            joint_names: &body.joint_names,
            joints: &body.global_joint_states,
        },
    )
}

fn report(name: &str, round: usize, start: Instant, armor: &GeneratedArmor) -> Result<()> {
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    anyhow::ensure!(
        armor.morphs.is_empty() && armor.positions.iter().flatten().all(|v| v.is_finite()),
        "invalid runtime mesh"
    );
    println!(
        "{}",
        serde_json::json!({"item":name,"round":round,"ms":ms,"vertices":armor.positions.len(),"targets":armor.morphs.len()})
    );
    Ok(())
}
