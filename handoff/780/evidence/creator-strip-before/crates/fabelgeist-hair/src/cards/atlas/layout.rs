//! Admission of atlas recipes and their host/native RGBA8 image extents.

use super::RgbaTexel;
use fabelgeist_color::PixelFormat;
use std::{collections::TryReserveError, num::TryFromIntError};

pub type AtlasResult<T> = Result<T, AtlasError>;

/// Number of painted bundles laid out side by side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtlasVariantCount(u32);

/// A positive tile edge in texels, within the rasterizer's signed addresses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtlasTileSpan(u32);

/// Texel dimensions of one painted bundle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtlasTileSize {
    pub width: AtlasTileSpan,
    pub height: AtlasTileSpan,
}

/// One atlas recipe; randomness is supplied separately as a canonical seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtlasSettings {
    pub variants: AtlasVariantCount,
    pub tile: AtlasTileSize,
}

/// Checked image addresses for one tightly packed RGBA8 mip level.
#[derive(Clone, Copy)]
pub(super) struct AtlasMipLayout {
    width: u32,
    height: u32,
    row_bytes: u32,
    pixels: usize,
}

/// A card atlas cannot be represented or allocated at its image boundary.
#[derive(Debug, thiserror::Error)]
pub enum AtlasError {
    #[error("a card atlas needs at least one variant")]
    EmptyVariants,
    #[error("a card atlas tile edge must be positive")]
    EmptyTileSpan,
    #[error("a card atlas tile edge exceeds signed raster addresses")]
    TileAddress(#[source] TryFromIntError),
    #[error("atlas width overflows for {variants:?} variants of {tile:?}")]
    AtlasWidth {
        variants: AtlasVariantCount,
        tile: AtlasTileSize,
    },
    #[error("card atlas extent {width} by {height} is empty")]
    EmptyExtent { width: u32, height: u32 },
    #[error("card atlas row for {width} texels exceeds native byte addresses")]
    RowBytes { width: u32 },
    #[error("card atlas extent {width} by {height} exceeds host byte addresses")]
    HostExtent { width: u32, height: u32 },
    #[error("card atlas image exceeds host addresses")]
    HostAddress(#[source] TryFromIntError),
    #[error("card atlas allocation failed")]
    Allocation(#[source] TryReserveError),
    #[error("card atlas extent {width} by {height} exceeds native dimension limit {maximum}")]
    NativeExtent {
        width: u32,
        height: u32,
        maximum: u32,
    },
    #[error("card atlas pixel format {format:?} has no native texture representation")]
    NativeFormat { format: PixelFormat },
}

impl TryFrom<u32> for AtlasVariantCount {
    type Error = AtlasError;

    fn try_from(count: u32) -> AtlasResult<Self> {
        if count == 0 {
            return Err(AtlasError::EmptyVariants);
        }
        Ok(Self(count))
    }
}

impl AtlasVariantCount {
    /// Native variant count for a raster/shader or collection boundary.
    pub const fn count(self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for AtlasTileSpan {
    type Error = AtlasError;

    fn try_from(texels: u32) -> AtlasResult<Self> {
        if texels == 0 {
            return Err(AtlasError::EmptyTileSpan);
        }
        i32::try_from(texels).map_err(AtlasError::TileAddress)?;
        Ok(Self(texels))
    }
}

impl AtlasTileSpan {
    /// Native edge extent for the rasterizer or image layout.
    pub const fn texels(self) -> u32 {
        self.0
    }
}

impl Default for AtlasSettings {
    fn default() -> Self {
        // Authored LOD atlas recipe; the private leaves satisfy admission.
        Self {
            variants: AtlasVariantCount(8),
            tile: AtlasTileSize {
                width: AtlasTileSpan(64),
                height: AtlasTileSpan(256),
            },
        }
    }
}

impl AtlasSettings {
    pub(super) const fn one_pixel() -> Self {
        Self {
            variants: AtlasVariantCount(1),
            tile: AtlasTileSize {
                width: AtlasTileSpan(1),
                height: AtlasTileSpan(1),
            },
        }
    }
}

impl AtlasMipLayout {
    pub(super) fn for_atlas(settings: AtlasSettings) -> AtlasResult<Self> {
        let width = settings
            .tile
            .width
            .texels()
            .checked_mul(settings.variants.count())
            .ok_or(AtlasError::AtlasWidth {
                variants: settings.variants,
                tile: settings.tile,
            })?;
        Self::new(width, settings.tile.height.texels())
    }

    pub(super) fn for_tile(tile: AtlasTileSize) -> AtlasResult<Self> {
        Self::new(tile.width.texels(), tile.height.texels())
    }

    fn new(width: u32, height: u32) -> AtlasResult<Self> {
        if width == 0 || height == 0 {
            return Err(AtlasError::EmptyExtent { width, height });
        }
        let row_bytes = width
            .checked_mul(size_of::<RgbaTexel>() as u32)
            .ok_or(AtlasError::RowBytes { width })?;
        // Two u32 extents have a product representable in u64.
        let pixels = usize::try_from(u64::from(width) * u64::from(height))
            .map_err(AtlasError::HostAddress)?;
        let bytes = pixels
            .checked_mul(size_of::<RgbaTexel>())
            .ok_or(AtlasError::HostExtent { width, height })?;
        isize::try_from(bytes).map_err(AtlasError::HostAddress)?;
        Ok(Self {
            width,
            height,
            row_bytes,
            pixels,
        })
    }

    pub(super) fn smaller(self) -> Option<Self> {
        if self.width <= 1 || self.height <= 1 {
            return None;
        }
        let (width, height) = (self.width / 2, self.height / 2);
        // Both dimensions shrink from an admitted image, so row and host
        // products remain within the parent's native/address bounds.
        Some(Self {
            width,
            height,
            row_bytes: width * size_of::<RgbaTexel>() as u32,
            pixels: width as usize * height as usize,
        })
    }

    pub(super) const fn width(self) -> u32 {
        self.width
    }
    pub(super) const fn height(self) -> u32 {
        self.height
    }
    pub(super) const fn row_bytes(self) -> u32 {
        self.row_bytes
    }
    pub(super) const fn pixels(self) -> usize {
        self.pixels
    }
}

/// Allocate actual host elements; image admission supplies checked extents.
pub(super) fn zeroed<T: Default>(count: usize) -> AtlasResult<Vec<T>> {
    let mut data = Vec::new();
    data.try_reserve_exact(count)
        .map_err(AtlasError::Allocation)?;
    data.resize_with(count, T::default);
    Ok(data)
}
