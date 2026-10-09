//! Pixel conversion for cubemap face uploads.
use super::{Image, TextureFormat};
use anyhow::Result;

pub(super) fn convert_face_pixels(image: &Image, format: TextureFormat) -> Result<Vec<u8>> {
    let pixel_size = format.pixel_size();
    let raw_data = &image.data;
    let converted_data: Vec<u8> = match format {
        TextureFormat::Rgba8Unorm | TextureFormat::Bgra8Unorm => raw_data
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|rgba| {
                let mut out = [0u8; 4];
                for (c, channel) in rgba[..3].iter().enumerate() {
                    let f = *channel as f32 / 255.0;
                    let linear = if f <= 0.04045 {
                        f / 12.92
                    } else {
                        ((f + 0.055) / 1.055).powf(2.4)
                    };
                    out[c] = (linear.clamp(0.0, 1.0) * 255.0) as u8;
                }
                out[3] = rgba[3];
                if matches!(format, TextureFormat::Bgra8Unorm) {
                    out.swap(0, 2);
                }
                out
            })
            .collect(),
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Bgra8UnormSrgb => {
            if matches!(format, TextureFormat::Bgra8UnormSrgb) {
                raw_data
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .flat_map(|rgba| [rgba[2], rgba[1], rgba[0], rgba[3]])
                    .collect()
            } else {
                raw_data.to_vec()
            }
        }
        TextureFormat::Rgba32Float => {
            let mut floats = Vec::with_capacity((image.width * image.height * 4) as usize);
            for rgba in raw_data.as_chunks::<4>().0 {
                for channel in &rgba[..3] {
                    let f = *channel as f32 / 255.0;
                    let linear = if f <= 0.04045 {
                        f / 12.92
                    } else {
                        ((f + 0.055) / 1.055).powf(2.4)
                    };
                    floats.push(linear);
                }
                floats.push(rgba[3] as f32 / 255.0);
            }
            bytemuck::cast_slice(&floats).to_vec()
        }
        _ => {
            if pixel_size == 4 {
                raw_data.to_vec()
            } else {
                return Err(anyhow::anyhow!(
                    "Unsupported image conversion to format: {:?}",
                    format
                ));
            }
        }
    };

    Ok(converted_data)
}
