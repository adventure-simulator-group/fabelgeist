//! Rasterise one fabric to a flat colour swatch, without fitting a garment.
//!
//! The fabric is a `TextileAppearance` in JSON. Its needlework chart gives
//! the swatch its physical size, so ornament can be judged at true
//! proportions before the fabric is put on an article.
use fabelgeist_armor::TextileAppearance;
use std::{env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args().skip(1).map(PathBuf::from);
    let (Some(fabric), Some(output)) = (arguments.next(), arguments.next()) else {
        return Err("usage: textile_swatch FABRIC.json OUTPUT.png".into());
    };
    let fabric: TextileAppearance = serde_json::from_slice(&fs::read(fabric)?)?;
    let pixels = fabric.pixels()?;
    image::save_buffer(
        output,
        &pixels.color,
        pixels.width,
        pixels.height,
        image::ExtendedColorType::Rgba8,
    )?;
    Ok(())
}
