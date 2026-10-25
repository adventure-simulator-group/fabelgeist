use anyhow::Result;
use std::sync::Arc;

use crate::data::gpu::texture::TextureFormat;

use crate::globals::WgpuContext;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Image {
    pub data: Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}
impl Image {
    // Decoding a URI is the only thing here that needs an image decoder and a
    // filesystem, so it is the only thing behind `image-io`. Without the
    // feature an `Image` is still a perfectly good pixel buffer -- it just has
    // to be handed its pixels rather than fetching them.
    #[cfg(feature = "image-io")]
    pub async fn new(_context: WgpuContext, uri: String) -> Result<Image> {
        let bytes = fabelgeist_fs::read_bytes(&uri).await?;

        let img = image::load_from_memory(&bytes)?;
        let img = img.to_rgba8();
        let (width, height) = img.dimensions();

        Ok(Image {
            data: Arc::new(img.into_raw()),
            width,
            height,
        })
    }

    pub fn from_pixels(pixels: Vec<u8>, width: u32, height: u32) -> Result<Image> {
        Ok(Image {
            data: Arc::new(pixels),
            width,
            height,
        })
    }
}

// Its own block: the one above is a graph node definition, and packing pixels
// for a format is plumbing that no graph should have a node for.
impl Image {
    /// The pixels, laid out for a texture of this format.
    ///
    /// No transfer function is applied, on purpose. An `Image` is eight-bit
    /// pixels and they reach the GPU as they are; all this does is reorder the
    /// channels a BGRA texture wants and widen them for a float one. Which
    /// space the pixels are read in is what the format is for: the hardware
    /// hands a shader sampling an sRGB texture linear light, at full
    /// precision, and one sampling a plain texture the stored value.
    ///
    /// Converting here instead would mean rounding linear light back into the
    /// eight bits it came from, and eight bits have no room for it -- the
    /// darkest thirteen levels of a picture all land on zero, and its shadows
    /// go flat black. It would also be wrong for half the pictures that arrive
    /// here: a normal map or a roughness map is measurements, not colour, and
    /// was never sRGB-encoded to begin with.
    ///
    /// A caller that does want linear light out of an image -- to fill a float
    /// texture with it, say -- converts it itself, where the precision is
    /// there to hold it: see [`fabelgeist_color::srgb_to_linear`].
    pub fn packed_for(&self, format: TextureFormat) -> Result<Vec<u8>> {
        let pixels = self.data.as_ref();
        Ok(match format {
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => pixels.clone(),
            TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => pixels
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|rgba| [rgba[2], rgba[1], rgba[0], rgba[3]])
                .collect(),
            TextureFormat::Rgba32Float => {
                let widened: Vec<f32> = pixels.iter().map(|&v| v as f32 / 255.0).collect();
                bytemuck::cast_slice(&widened).to_vec()
            }
            _ => {
                let expected = (self.width * self.height * format.pixel_size()) as usize;
                if pixels.len() == expected || format.pixel_size() == 4 {
                    pixels.clone()
                } else {
                    anyhow::bail!(
                        "an image of {} bytes cannot fill a {format:?} texture of {}x{}, which wants {expected}",
                        pixels.len(),
                        self.width,
                        self.height
                    );
                }
            }
        })
    }
}

impl super::Texture2d {
    pub async fn read_image(&self, context: &WgpuContext) -> Result<Image> {
        let (width, height) = self.size;
        if width == 0 || height == 0 {
            return Ok(Image::default());
        }

        let raw_data = self.read::<u8>(context).await?;
        let narrow = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;

        let rgba_data = match self.format {
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => raw_data,
            TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => raw_data
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|bgra| [bgra[2], bgra[1], bgra[0], bgra[3]])
                .collect(),
            TextureFormat::Rgba32Float => bytemuck::cast_slice::<u8, f32>(&raw_data)
                .iter()
                .map(|&channel| narrow(channel))
                .collect(),
            TextureFormat::R32Float => bytemuck::cast_slice::<u8, f32>(&raw_data)
                .iter()
                .flat_map(|&value| {
                    let grey = narrow(value);
                    [grey, grey, grey, 255]
                })
                .collect(),
            TextureFormat::R8Unorm => raw_data
                .iter()
                .flat_map(|&value| [value, value, value, 255])
                .collect(),
            _ => {
                if self.format.pixel_size() == 4 {
                    raw_data
                } else {
                    anyhow::bail!(
                        "a {:?} texture cannot be read back as an image",
                        self.format
                    );
                }
            }
        };

        Image::from_pixels(rgba_data, width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ramp of every eight-bit level, which is what the loss showed up in.
    fn ramp() -> Image {
        let pixels = (0..256u32)
            .flat_map(|level| [level as u8, level as u8, level as u8, 255])
            .collect();
        Image::from_pixels(pixels, 256, 1).expect("a ramp")
    }

    /// Every level of a picture reaches the GPU.
    ///
    /// This is the whole point of packing without a transfer function. Sending
    /// an sRGB picture to a plain eight-bit format used to convert it to linear
    /// light and round the result back into eight bits, which left the darkest
    /// thirteen levels all on zero and 133 of the 256 moved. Nothing may move.
    #[test]
    fn every_level_survives_being_packed_for_an_eight_bit_format() {
        let image = ramp();
        for format in [
            TextureFormat::Rgba8Unorm,
            TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Bgra8Unorm,
            TextureFormat::Bgra8UnormSrgb,
        ] {
            let packed = image.packed_for(format).expect("packing the ramp");
            let red = match format {
                TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => 2,
                _ => 0,
            };
            let moved: Vec<usize> = (0..256)
                .filter(|&level| packed[level * 4 + red] != level as u8)
                .collect();
            assert!(
                moved.is_empty(),
                "{format:?} moved {} of 256 levels: {:?}",
                moved.len(),
                &packed[..32]
            );
        }
    }

    /// The two eight-bit spellings of a colour hold the same bits.
    ///
    /// A plain format and its sRGB counterpart differ in how the hardware reads
    /// a texture, not in what is stored. If packing ever starts to convert for
    /// one of them, the two drift apart and this says so.
    #[test]
    fn a_plain_format_and_its_srgb_counterpart_store_the_same_bits() {
        let image = ramp();
        assert_eq!(
            image.packed_for(TextureFormat::Rgba8Unorm).unwrap(),
            image.packed_for(TextureFormat::Rgba8UnormSrgb).unwrap()
        );
        assert_eq!(
            image.packed_for(TextureFormat::Bgra8Unorm).unwrap(),
            image.packed_for(TextureFormat::Bgra8UnormSrgb).unwrap()
        );
    }

    /// Widening to floats keeps the value, and does not reinterpret it.
    #[test]
    fn a_float_format_is_widened_and_not_converted() {
        let packed = ramp()
            .packed_for(TextureFormat::Rgba32Float)
            .expect("packing the ramp");
        let floats: &[f32] = bytemuck::cast_slice(&packed);
        for level in 0..256 {
            assert!(
                (floats[level * 4] - level as f32 / 255.0).abs() < 1e-6,
                "level {level} came out {}",
                floats[level * 4]
            );
        }
    }

    /// BGRA is a channel order, and nothing more.
    #[test]
    fn a_bgra_format_only_reorders_the_channels() {
        let image = Image::from_pixels(vec![1, 2, 3, 4], 1, 1).unwrap();
        assert_eq!(
            image.packed_for(TextureFormat::Bgra8Unorm).unwrap(),
            vec![3, 2, 1, 4]
        );
    }
}
