//! Local persistence and explicit file import/export. No server or account is involved.
mod archive;
use crate::{app::Studio, document::Document};
use adventuresim_procedural_textures::{BakedMap, PixelEncoding};
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
};
use image::ImageEncoder;

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;
#[cfg(target_family = "wasm")]
#[wasm_bindgen(module = "/web/bridge.js")]
extern "C" {
    pub(crate) fn load_saved() -> Option<String>;
    fn save_document(source: &str);
    pub(crate) fn choose_import();
    fn take_import() -> Option<String>;
    fn download_bytes(name: &str, bytes: &[u8], mime: &str);
    fn library_source() -> String;
    fn write_library(source: &str) -> Option<String>;
}

#[cfg(not(target_family = "wasm"))]
fn library_source() -> String {
    std::fs::read_to_string("target/texture-studio/presets.json").unwrap_or_else(|_| "{}".into())
}
#[cfg(not(target_family = "wasm"))]
fn write_library(source: &str) -> Option<String> {
    std::fs::create_dir_all("target/texture-studio")
        .and_then(|()| std::fs::write("target/texture-studio/presets.json", source))
        .err()
        .map(|e| e.to_string())
}

pub(crate) fn load_library() -> std::collections::BTreeMap<String, String> {
    serde_json::from_str(&library_source()).unwrap_or_default()
}
pub(crate) fn save_library(
    library: &std::collections::BTreeMap<String, String>,
) -> Result<(), String> {
    match write_library(&serde_json::to_string(library).unwrap()) {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

#[cfg(not(target_family = "wasm"))]
pub(crate) fn load_saved() -> Option<String> {
    std::fs::read_to_string("target/texture-studio/autosave.json").ok()
}
#[cfg(not(target_family = "wasm"))]
fn save_document(source: &str) {
    let _ = std::fs::create_dir_all("target/texture-studio");
    let _ = std::fs::write("target/texture-studio/autosave.json", source);
}
#[cfg(not(target_family = "wasm"))]
pub(crate) fn choose_import() {
    bevy::log::info!("Place a preset at target/texture-studio/import.json to import it");
}
#[cfg(not(target_family = "wasm"))]
fn take_import() -> Option<String> {
    let path = "target/texture-studio/import.json";
    let source = std::fs::read_to_string(path).ok()?;
    let _ = std::fs::remove_file(path);
    Some(source)
}
#[cfg(not(target_family = "wasm"))]
fn download_bytes(name: &str, bytes: &[u8], _mime: &str) {
    let directory = std::path::Path::new("target/texture-studio/exports");
    let _ = std::fs::create_dir_all(directory);
    if let Err(error) = std::fs::write(directory.join(name), bytes) {
        bevy::log::error!("Export failed: {error}");
    }
}

pub(crate) fn save_preset(document: &Document) {
    download_bytes(
        &format!("{}.texture.json", document.recipe.slug()),
        document.to_json().as_bytes(),
        "application/json",
    );
}

pub(crate) fn export_maps(studio: &Studio) -> Result<String, String> {
    let bake = studio.current.as_ref().ok_or("No finished bake yet")?;
    if !matches!(studio.applied, Some((revision, quality)) if revision == studio.revision && quality == studio.document.texture.resolution)
    {
        return Err("Wait for the final-quality bake before exporting maps".into());
    }
    let mut files = vec![
        (
            "preset.texture.json".into(),
            studio.document.to_json().into_bytes(),
        ),
        ("material.bake".into(), bake.to_bytes()),
    ];
    for map in &bake.maps {
        files.push((
            format!("{}-{}.png", bake.recipe.slug(), map.channel.slug()),
            png(map)?,
        ));
        files.push((
            format!("{}-{}.mips", bake.recipe.slug(), map.channel.slug()),
            map.bytes.clone(),
        ));
    }
    let manifest = serde_json::json!({"recipe":bake.recipe,"tile_metres":bake.tile_metres,"height_range_metres":bake.height_range_metres,
        "maps":bake.maps.iter().map(|map|serde_json::json!({"channel":map.channel,"size":map.size,"mip_levels":map.mip_levels,"encoding":map.encoding})).collect::<Vec<_>>()});
    files.push((
        "manifest.json".into(),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    ));
    download_bytes(
        &format!("{}.zip", bake.recipe.slug()),
        &archive::stored_zip(files),
        "application/zip",
    );
    Ok("Exported final maps, mip payloads and preset".into())
}

pub(crate) fn png(map: &BakedMap) -> Result<Vec<u8>, String> {
    let count = map.size.pow(2) as usize * map.encoding.channels();
    let (pixels, color) = match map.encoding {
        PixelEncoding::Rg8 => (
            map.bytes[..count]
                .as_chunks::<2>()
                .0
                .iter()
                .flat_map(|p| [p[0], p[1], 0, 255])
                .collect(),
            image::ExtendedColorType::Rgba8,
        ),
        PixelEncoding::R8 => (map.bytes[..count].to_vec(), image::ExtendedColorType::L8),
        _ => (map.bytes[..count].to_vec(), image::ExtendedColorType::Rgba8),
    };
    let mut output = vec![];
    image::codecs::png::PngEncoder::new(&mut output)
        .write_image(&pixels, map.size, map.size, color)
        .map_err(|e| e.to_string())?;
    Ok(output)
}

pub(crate) fn update(mut commands: Commands, mut studio: ResMut<Studio>, time: Res<Time>) {
    if let Some(source) = take_import() {
        match Document::from_json(&source) {
            Ok(document) => {
                let previous = studio.document.to_json();
                studio.remember(previous);
                studio.replace(document, time.elapsed_secs_f64());
            }
            Err(error) => studio.status = format!("Import rejected: {error}"),
        }
    }
    if studio.save_due && time.elapsed_secs_f64() - studio.last_edit > 0.8 {
        save_document(&studio.document.to_json());
        studio.save_due = false;
    }
    if studio.screenshot {
        studio.screenshot = false;
        commands
            .spawn(Screenshot::primary_window())
            .observe(|capture: On<ScreenshotCaptured>| {
                if let Ok(image) = capture.image.clone().try_into_dynamic() {
                    let mut bytes = std::io::Cursor::new(vec![]);
                    if image.write_to(&mut bytes, image::ImageFormat::Png).is_ok() {
                        download_bytes("texture-studio.png", bytes.get_ref(), "image/png");
                    }
                }
            });
    }
}
