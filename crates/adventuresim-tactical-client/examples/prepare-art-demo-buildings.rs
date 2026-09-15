//! Compile the shipped city recipes offline, with separate inspection assets.
use adventuresim_building_generator::{generate, prepared::*};
use adventuresim_tactical_core::prelude::DistantBuildingPlacement;
use clap::Parser;
use serde::Deserialize;

#[derive(Parser)]
struct Args {
    /// Write prepared meshes to an isolated directory instead of shipped assets.
    #[arg(long)]
    output: Option<std::path::PathBuf>,
}

#[derive(Deserialize)]
struct CityLayout {
    buildings: Vec<DistantBuildingPlacement>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let layout: CityLayout = serde_json::from_slice(&std::fs::read(
        root.join("assets/art-demo/city-layout.json"),
    )?)?;
    let output = Args::parse()
        .output
        .unwrap_or_else(|| root.join("assets/art-demo/buildings"));
    std::fs::create_dir_all(&output)?;
    let mut recipes = std::collections::BTreeMap::new();
    for placement in layout.buildings {
        let program = placement.program();
        recipes.entry(recipe_key(&program)).or_insert(program);
    }
    let started = std::time::Instant::now();
    for (index, (key, program)) in recipes.iter().enumerate() {
        let recipe_started = std::time::Instant::now();
        let plan = generate(program)?;
        let mut facade = PreparedBuilding::from_plan(program.clone(), &plan);
        let detail = facade.take_detail();
        for (suffix, data) in [("facade", facade), ("detail", detail)] {
            let file = std::fs::File::create(output.join(format!("{key}.{suffix}.building")))?;
            serde_json::to_writer(std::io::BufWriter::new(file), &data)?;
        }
        println!(
            "{}/{} {key}: {:.2}s",
            index + 1,
            recipes.len(),
            recipe_started.elapsed().as_secs_f32()
        );
    }
    println!(
        "Prepared {} recipes in {:.2}s",
        recipes.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(())
}
