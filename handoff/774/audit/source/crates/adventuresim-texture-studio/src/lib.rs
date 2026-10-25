//! The same material editor and Bevy preview run natively and in a standalone WASM application.
mod app;
mod baking;
pub mod document;
mod exchange;
mod inspector;
#[cfg(not(target_family = "wasm"))]
pub mod review;
mod scene;
mod ui;

pub use app::run;
#[cfg(not(target_family = "wasm"))]
pub fn export_recipe(
    bake: &adventuresim_procedural_textures::BakedRecipe,
    output: &std::path::Path,
) -> Result<(), String> {
    std::fs::create_dir_all(output).map_err(|e| e.to_string())?;
    for map in &bake.maps {
        let stem = format!("{}-{}", bake.recipe.slug(), map.channel.slug());
        std::fs::write(output.join(format!("{stem}.png")), exchange::png(map)?)
            .map_err(|e| e.to_string())?;
        std::fs::write(output.join(format!("{stem}.mips")), &map.bytes)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start_studio() {
    console_error_panic_hook::set_once();
    run();
}

/// Worker entry point. It deliberately does not start a Bevy renderer.
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn bake_texture(source: &str) -> Result<Vec<u8>, wasm_bindgen::JsValue> {
    let doc =
        document::Document::from_json(source).map_err(|e| wasm_bindgen::JsValue::from_str(&e))?;
    Ok(
        adventuresim_procedural_textures::BakedRecipe::generate(doc.recipe, &doc.texture)
            .to_bytes(),
    )
}
