//! Explicit authoring step. Normal builds and game startup never execute this binary.

use std::{error::Error, path::Path, time::Instant};

use adventuresim_procedural_textures::{
    BakedRecipe, PROCEDURAL_TEXTURE_CATALOGUE, TextureParameters,
};

fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let [selection] = arguments.as_slice() else {
        return Err("usage: bake-procedural-textures <all|recipe-slug>".into());
    };
    let recipes: Vec<_> = PROCEDURAL_TEXTURE_CATALOGUE
        .iter()
        .filter(|entry| selection == "all" || selection == entry.id.slug())
        .collect();
    if recipes.is_empty() {
        return Err(format!("unknown texture recipe: {selection}").into());
    }
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let params = TextureParameters::default();
    for descriptor in recipes {
        let started = Instant::now();
        println!("Baking {} at full resolution...", descriptor.id.slug());
        let bake = BakedRecipe::generate(descriptor.id, &params);
        let bytes = bake.to_compressed_bytes()?;
        // Verify the exact complete mip payload and metadata before publishing.
        if BakedRecipe::from_compressed_bytes(&bytes)?.to_bytes() != bake.to_bytes() {
            return Err(format!("{} did not round-trip", descriptor.id.slug()).into());
        }
        let output = assets.join(descriptor.id.runtime_asset_path());
        std::fs::create_dir_all(output.parent().expect("asset has a parent directory"))?;
        std::fs::write(&output, &bytes)?;
        println!(
            "Saved {} ({} bytes, {:.2}s)",
            output.display(),
            bytes.len(),
            started.elapsed().as_secs_f64()
        );
    }
    Ok(())
}
