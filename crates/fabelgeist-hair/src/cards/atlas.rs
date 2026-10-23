//! Painted bundle images with one admitted recipe and exact mip layouts.

use fabelgeist_color::{ChannelLayout, PixelFormat};
use fabelgeist_determinism::Seed;
pub use layout::{
    AtlasError, AtlasResult, AtlasSettings, AtlasTileSize, AtlasTileSpan, AtlasVariantCount,
};
use layout::{AtlasMipLayout, zeroed};

mod layout;
mod paint;

/// The authored mip budget; the chain also stops at either unit axis.
const MAX_ATLAS_MIPS: usize = 12;

type RgbaTexel = [u8; ChannelLayout::Rgba.channels()];

/// Painted strand bundles side by side, with an admitted RGBA8 mip chain.
pub struct CardAtlas {
    settings: AtlasSettings,
    base: AtlasMip,
    smaller: Vec<AtlasMip>,
}

/// One image level: coverage, strand shade, bundle depth, then coverage again.
pub struct AtlasMip {
    layout: AtlasMipLayout,
    pixels: Vec<RgbaTexel>,
}

impl CardAtlas {
    pub const FORMAT: PixelFormat = PixelFormat::Rgba8Unorm;

    /// Paint a checked recipe with deterministic per-variant randomness.
    /// Mips stop when either axis reaches one texel or twelve levels exist.
    pub fn paint(settings: AtlasSettings, seed: Seed) -> AtlasResult<Self> {
        let base = paint::paint_base(settings, seed)?;
        let mut smaller = Vec::new();
        smaller
            .try_reserve_exact(MAX_ATLAS_MIPS - 1)
            .map_err(AtlasError::Allocation)?;
        while smaller.len() + 1 < MAX_ATLAS_MIPS {
            let previous = match smaller.last() {
                Some(previous) => previous,
                None => &base,
            };
            let Some(next) = previous.downsample()? else {
                break;
            };
            smaller.push(next);
        }
        Ok(Self {
            settings,
            base,
            smaller,
        })
    }

    /// The transparent one-texel atlas used before a groom is installed.
    pub fn transparent() -> AtlasResult<Self> {
        let settings = AtlasSettings::one_pixel();
        let base = AtlasMip::blank(AtlasMipLayout::for_atlas(settings)?)?;
        Ok(Self {
            settings,
            base,
            smaller: Vec::new(),
        })
    }

    pub fn variants(&self) -> AtlasVariantCount {
        self.settings.variants
    }
    pub fn settings(&self) -> AtlasSettings {
        self.settings
    }
    pub fn base(&self) -> &AtlasMip {
        &self.base
    }

    pub fn levels(&self) -> impl Iterator<Item = &AtlasMip> {
        std::iter::once(&self.base).chain(&self.smaller)
    }

    /// Native mip count; construction admits at most twelve levels.
    pub fn mip_count(&self) -> u32 {
        (self.smaller.len() + 1) as u32
    }
}

impl AtlasMip {
    fn blank(layout: AtlasMipLayout) -> AtlasResult<Self> {
        Ok(Self {
            pixels: zeroed(layout.pixels())?,
            layout,
        })
    }

    pub fn width_texels(&self) -> u32 {
        self.layout.width()
    }
    pub fn height_texels(&self) -> u32 {
        self.layout.height()
    }
    pub fn bytes_per_row(&self) -> u32 {
        self.layout.row_bytes()
    }

    /// Tightly packed native RGBA8 bytes; padding is not part of this image.
    pub fn pixel_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.pixels)
    }

    fn downsample(&self) -> AtlasResult<Option<Self>> {
        let Some(layout) = self.layout.smaller() else {
            return Ok(None);
        };
        let mut next = Self::blank(layout)?;
        // Admission proves these extents fit host indexing. Filter 2x2
        // texels with rounded integer means and clamped edge addresses.
        let (width, height) = (self.width_texels() as usize, self.height_texels() as usize);
        let (next_width, next_height) = (layout.width() as usize, layout.height() as usize);
        for y in 0..next_height {
            for x in 0..next_width {
                for (channel, value) in next.pixels[y * next_width + x].iter_mut().enumerate() {
                    let texel =
                        |xx: usize, yy: usize| u32::from(self.pixels[yy * width + xx][channel]);
                    let sum = texel(2 * x, 2 * y)
                        + texel((2 * x + 1).min(width - 1), 2 * y)
                        + texel(2 * x, (2 * y + 1).min(height - 1))
                        + texel((2 * x + 1).min(width - 1), (2 * y + 1).min(height - 1));
                    *value = ((sum + 2) / 4) as u8;
                }
            }
        }
        Ok(Some(next))
    }
}
