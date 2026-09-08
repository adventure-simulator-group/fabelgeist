//! Export every preset and gameplay recipe, mesh, and physical measurement for review.

use std::{env, fs, path::PathBuf};

use adventuresim_weapon_model::{
    MELEE_CATALOG_IDS, PRESET_IDS, WeaponDesign, default_design, generate, preset_design,
};
use serde_json::{Value, json};

fn specimen(
    group: &str,
    id: &str,
    design: WeaponDesign,
) -> Result<Value, Box<dyn std::error::Error>> {
    let weapon = generate(&design)?;
    let physical = weapon.derived;
    let parts: Vec<_> = weapon
        .parts
        .iter()
        .map(|part| {
            json!({
                "id": part.component_id,
                "material": part.material,
                "density": part.material.density_kg_m3(),
                "positions": part.positions,
                "indices": part.indices,
            })
        })
        .collect();
    Ok(json!({
        "group": group, "id": id, "design": design, "parts": parts,
        "physical": {
            "massKg": physical.mass_kg,
            "lengthM": physical.length_m,
            "gripToTipM": physical.grip_to_tip_m,
            "headLengthM": physical.striking_head_length_m,
            "centerOfMassFromGripM": physical.center_of_mass_from_grip_m,
            "momentOfInertiaKgM2": physical.moment_of_inertia_kg_m2,
            "balance": physical.balance,
        },
    }))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        env::args()
            .nth(1)
            .ok_or("usage: audit_catalog <output.json>")?,
    );
    let mut specimens = Vec::new();
    for id in PRESET_IDS {
        specimens.push(specimen(
            "preset",
            id,
            preset_design(id).ok_or("missing preset")?,
        )?);
    }
    for id in MELEE_CATALOG_IDS {
        specimens.push(specimen(
            "catalog",
            id,
            default_design(id).ok_or("missing catalog recipe")?,
        )?);
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, serde_json::to_vec(&specimens)?)?;
    println!(
        "Exported {} recipes with meshes and measurements",
        specimens.len()
    );
    Ok(())
}
