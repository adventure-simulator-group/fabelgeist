//! Lossless storage for committed runtime textures. Mips and samplers are baked once.

use std::io::{self, Read, Write};

use flate2::{Compression, bufread::GzDecoder, write::GzEncoder};

use super::BakedRecipe;

// Bounds one recipe, including eight RGBA 2048-square maps with complete mips.
const MAX_DECOMPRESSED_BAKE_BYTES: u64 = 192 * 1024 * 1024;

impl BakedRecipe {
    pub fn to_compressed_bytes(&self) -> io::Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&self.to_bytes())?;
        encoder.finish()
    }

    pub fn from_compressed_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut decoder = GzDecoder::new(bytes);
        let mut decoded = Vec::new();
        decoder
            .by_ref()
            .take(MAX_DECOMPRESSED_BAKE_BYTES + 1)
            .read_to_end(&mut decoded)?;
        if decoded.len() as u64 > MAX_DECOMPRESSED_BAKE_BYTES {
            return Err(io::Error::other(
                "procedural texture bake exceeds size limit",
            ));
        }
        if !decoder.into_inner().is_empty() {
            return Err(io::Error::other("unexpected trailing compressed bake data"));
        }
        Self::from_bytes(&decoded).map_err(io::Error::other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BakeResolution, PROCEDURAL_TEXTURE_CATALOGUE, TextureParameters};

    #[test]
    fn every_recipe_round_trips_pixels_mips_formats_and_samplers() {
        let params = TextureParameters {
            resolution: BakeResolution::Draft,
            ..Default::default()
        };
        for descriptor in PROCEDURAL_TEXTURE_CATALOGUE {
            let source = BakedRecipe::generate(descriptor.id, &params);
            let compressed = source.to_compressed_bytes().unwrap();
            assert_eq!(compressed, source.to_compressed_bytes().unwrap());
            let decoded = BakedRecipe::from_compressed_bytes(&compressed).unwrap();
            assert_eq!(
                source.to_bytes(),
                decoded.to_bytes(),
                "{}",
                descriptor.id.slug()
            );
            assert!(
                BakedRecipe::from_compressed_bytes(&compressed[..compressed.len() - 1]).is_err()
            );
            let mut trailing = compressed;
            trailing.push(0);
            assert!(BakedRecipe::from_compressed_bytes(&trailing).is_err());
        }
        assert!(BakedRecipe::from_compressed_bytes(b"not a texture bake").is_err());
    }
}
