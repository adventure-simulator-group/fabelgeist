//! Encode one completed original image for bounded diagnostic export.
use anyhow::{Result, ensure};
use bevy::prelude::Image;

const MAX_PNG_BYTES: usize = 64 * 1024 * 1024;

pub(crate) struct CapturedPng {
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
    pub blake3: String,
}

impl CapturedPng {
    pub(crate) fn encode(image: Image) -> Result<Self> {
        let image = image.try_into_dynamic()?;
        let width = image.width();
        let height = image.height();
        ensure!(width > 0 && height > 0, "empty diagnostic readback");
        let mut png = std::io::Cursor::new(Vec::new());
        image
            .to_rgb8()
            .write_to(&mut png, image::ImageFormat::Png)?;
        let bytes = png.into_inner();
        ensure!(
            bytes.len() <= MAX_PNG_BYTES,
            "diagnostic image exceeds export bound"
        );
        Ok(Self {
            width,
            height,
            blake3: blake3::hash(&bytes).to_hex().to_string(),
            bytes,
        })
    }
}
