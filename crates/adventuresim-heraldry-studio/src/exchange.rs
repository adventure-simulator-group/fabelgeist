//! Explicit local files and browser downloads; import validates before replacement.
use crate::app::Studio;
use adventuresim_heraldry::document::Document;
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
};
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;
#[cfg(target_family = "wasm")]
#[wasm_bindgen(module = "/web/bridge.js")]
extern "C" {
    pub(crate) fn load_saved() -> Option<String>;
    fn save_document(source: &str) -> Option<String>;
    pub(crate) fn choose_import();
    fn take_import() -> Option<String>;
    fn download_bytes(name: &str, bytes: &[u8], mime: &str) -> Option<String>;
}
#[cfg(not(target_family = "wasm"))]
const STORAGE_DIRECTORY: &str = "target/heraldry";
#[cfg(not(target_family = "wasm"))]
pub(crate) fn load_saved() -> Option<String> {
    std::fs::read_to_string(format!("{STORAGE_DIRECTORY}/autosave.json")).ok()
}
pub(crate) fn autosave(source: &str) -> Result<(), String> {
    #[cfg(target_family = "wasm")]
    {
        save_document(source).map_or(Ok(()), Err)
    }
    #[cfg(not(target_family = "wasm"))]
    {
        std::fs::create_dir_all(STORAGE_DIRECTORY)
            .and_then(|()| std::fs::write(format!("{STORAGE_DIRECTORY}/autosave.json"), source))
            .map_err(|e| e.to_string())
    }
}
pub(crate) fn save_bytes(name: &str, bytes: &[u8], mime: &str) -> Result<(), String> {
    #[cfg(target_family = "wasm")]
    {
        download_bytes(name, bytes, mime).map_or(Ok(()), Err)
    }
    #[cfg(not(target_family = "wasm"))]
    {
        let _ = mime;
        let directory = format!("{STORAGE_DIRECTORY}/exports");
        std::fs::create_dir_all(&directory)
            .and_then(|()| std::fs::write(std::path::Path::new(&directory).join(name), bytes))
            .map_err(|e| e.to_string())
    }
}
pub(crate) fn update(mut commands: Commands, mut studio: ResMut<Studio>, time: Res<Time>) {
    const AUTOSAVE_DELAY: f64 = 0.6;
    if studio.save_due && time.elapsed_secs_f64() - studio.last_edit > AUTOSAVE_DELAY {
        studio.save_due = false;
        if let Err(error) = autosave(&studio.document.to_json().unwrap()) {
            studio.status = error;
        }
    }
    #[cfg(not(target_family = "wasm"))]
    let source: Option<String> = None;
    #[cfg(target_family = "wasm")]
    let source = take_import();
    if let Some(source) = source {
        match Document::from_json(&source) {
            Ok(d) => {
                let before = studio.document.clone();
                studio.remember(before);
                studio.replace(d, time.elapsed_secs_f64());
            }
            Err(e) => studio.status = format!("Import rejected: {e}"),
        }
    }
    if studio.screenshot {
        studio.screenshot = false;
        commands.spawn(Screenshot::primary_window()).observe(
            |capture: On<ScreenshotCaptured>, mut studio: ResMut<Studio>| {
                let result = capture
                    .image
                    .clone()
                    .try_into_dynamic()
                    .map_err(|e| e.to_string())
                    .and_then(|image| {
                        let mut bytes = std::io::Cursor::new(vec![]);
                        image
                            .write_to(&mut bytes, image::ImageFormat::Png)
                            .map_err(|e| e.to_string())?;
                        save_bytes("heraldry-studio.png", bytes.get_ref(), "image/png")
                    });
                studio.status = match result {
                    Ok(()) => "Saved heraldry-studio.png".into(),
                    Err(e) => e,
                };
            },
        );
    }
}
