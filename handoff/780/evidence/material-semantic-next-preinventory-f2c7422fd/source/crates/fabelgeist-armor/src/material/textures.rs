//! Coherent baked maps and their explicit native ownership-transfer packet.

use super::MetalTextureSize;
use fabelgeist_gpu::prelude::BufferByteLength;

/// Native RGBA8 bytes entering readback or leaving for an image framework.
pub struct MetalTextureMaps {
    pub normal: Vec<u8>,
    pub metal_roughness: Vec<u8>,
    pub depth: Option<DepthMap>,
}

/// Baked maps whose byte lengths agree with one checked square tile.
pub struct MetalTextures {
    size: MetalTextureSize,
    maps: MetalTextureMaps,
}

/// A parallax depth map: 0 on the surface, 255 at the floor of the cut.
pub struct DepthMap {
    pub pixels: Vec<u8>,
    /// The depth of a full cut in texture coordinate units.
    pub uv_scale: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum MetalTextureMap {
    Normal,
    MetalRoughness,
    Depth,
}

#[derive(Clone, Copy, Debug, thiserror::Error)]
pub enum MetalTextureDataError {
    #[error("metal texture {map:?} has {actual} bytes, expected {expected}")]
    PixelExtent {
        map: MetalTextureMap,
        expected: BufferByteLength,
        actual: BufferByteLength,
    },
    #[error("metal parallax depth scale must be finite and nonnegative: {uv_scale}")]
    DepthScale { uv_scale: f32 },
}

impl MetalTextures {
    pub(crate) fn from_native(
        size: MetalTextureSize,
        maps: MetalTextureMaps,
    ) -> Result<Self, MetalTextureDataError> {
        // One packed native u32 word encodes each RGBA8 texel. The admitted
        // tile population also fits host usize multiplication on GPU targets.
        let expected = BufferByteLength::from(usize::from(size.texels()) * size_of::<u32>());
        MetalTextureMap::Normal.check(&maps.normal, expected)?;
        MetalTextureMap::MetalRoughness.check(&maps.metal_roughness, expected)?;
        if let Some(depth) = &maps.depth {
            MetalTextureMap::Depth.check(&depth.pixels, expected)?;
            if !depth.uv_scale.is_finite() || depth.uv_scale < 0.0 {
                return Err(MetalTextureDataError::DepthScale {
                    uv_scale: depth.uv_scale,
                });
            }
        }
        Ok(Self { size, maps })
    }

    pub fn size(&self) -> MetalTextureSize {
        self.size
    }

    pub fn normal(&self) -> &[u8] {
        &self.maps.normal
    }

    pub fn metal_roughness(&self) -> &[u8] {
        &self.maps.metal_roughness
    }

    pub fn depth(&self) -> Option<&DepthMap> {
        self.maps.depth.as_ref()
    }

    /// Transfer native payload ownership to an image framework.
    pub fn into_maps(self) -> MetalTextureMaps {
        self.maps
    }
}

impl MetalTextureMap {
    fn check(self, pixels: &[u8], expected: BufferByteLength) -> Result<(), MetalTextureDataError> {
        let actual = BufferByteLength::from(pixels.len());
        if actual != expected {
            return Err(MetalTextureDataError::PixelExtent {
                map: self,
                expected,
                actual,
            });
        }
        Ok(())
    }
}
