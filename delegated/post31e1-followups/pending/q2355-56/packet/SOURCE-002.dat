//! A small JSON header followed by raw mip payloads; pixel bytes never expand into JSON arrays.
use super::*;

const HEADER_LENGTH_BYTES: usize = 4;
const MAX_HEADER_BYTES: usize = 16 * 1024;
const MAX_MAPS: usize = 8;
const MAX_TEXTURE_SIZE: u32 = 2048;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    recipe: TextureRecipeId,
    tile_metres: f32,
    height_range_metres: f32,
    maps: Vec<MapHeader>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MapHeader {
    channel: MapChannel,
    size: u32,
    mip_levels: u32,
    encoding: PixelEncoding,
    sampler: bevy::image::ImageSampler,
}

impl BakedRecipe {
    pub fn to_bytes(&self) -> Vec<u8> {
        let header = Header {
            recipe: self.recipe,
            tile_metres: self.tile_metres,
            height_range_metres: self.height_range_metres,
            maps: self
                .maps
                .iter()
                .map(|map| MapHeader {
                    channel: map.channel,
                    size: map.size,
                    mip_levels: map.mip_levels,
                    encoding: map.encoding,
                    sampler: map.sampler.clone(),
                })
                .collect(),
        };
        let header = serde_json::to_vec(&header).expect("finite generated metadata serializes");
        let mut bytes = Vec::with_capacity(
            HEADER_LENGTH_BYTES
                + header.len()
                + self.maps.iter().map(|map| map.bytes.len()).sum::<usize>(),
        );
        bytes.extend_from_slice(&(header.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&header);
        for map in &self.maps {
            bytes.extend_from_slice(&map.bytes);
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let length_bytes: [u8; HEADER_LENGTH_BYTES] = bytes
            .get(..HEADER_LENGTH_BYTES)
            .ok_or("missing bake header length")?
            .try_into()
            .unwrap();
        let length = u32::from_le_bytes(length_bytes) as usize;
        if length > MAX_HEADER_BYTES {
            return Err("bake header exceeds limit".into());
        }
        let header: Header = serde_json::from_slice(
            bytes
                .get(HEADER_LENGTH_BYTES..HEADER_LENGTH_BYTES + length)
                .ok_or("truncated bake header")?,
        )
        .map_err(|error| error.to_string())?;
        if header.maps.is_empty()
            || header.maps.len() > MAX_MAPS
            || !header.tile_metres.is_finite()
            || header.tile_metres <= 0.0
            || !header.height_range_metres.is_finite()
            || header.height_range_metres < 0.0
        {
            return Err("invalid bake metadata".into());
        }
        let mut offset = HEADER_LENGTH_BYTES + length;
        let mut maps: Vec<BakedMap> = Vec::with_capacity(header.maps.len());
        for map in header.maps {
            if !map.size.is_power_of_two()
                || map.size > MAX_TEXTURE_SIZE
                || map.mip_levels == 0
                || map.mip_levels > map.size.ilog2() + 1
                || maps.iter().any(|previous| previous.channel == map.channel)
            {
                return Err("invalid map dimensions, mips, or duplicate channel".into());
            }
            let count = (0..map.mip_levels)
                .map(|level| (map.size >> level).pow(2) as usize)
                .sum::<usize>()
                * map.encoding.channels();
            let data = bytes
                .get(offset..offset + count)
                .ok_or("truncated map payload")?
                .to_vec();
            offset += count;
            maps.push(BakedMap {
                channel: map.channel,
                size: map.size,
                mip_levels: map.mip_levels,
                encoding: map.encoding,
                sampler: map.sampler,
                bytes: data,
            });
        }
        if offset != bytes.len() {
            return Err("unexpected trailing bake data".into());
        }
        Ok(Self {
            recipe: header.recipe,
            tile_metres: header.tile_metres,
            height_range_metres: header.height_range_metres,
            maps,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_preserves_all_mips_and_rejects_truncated_or_trailing_data() {
        let source = BakedRecipe {
            recipe: TextureRecipeId::CrenellationMask,
            tile_metres: 1.0,
            height_range_metres: 0.0,
            maps: vec![BakedMap {
                channel: MapChannel::Opacity,
                size: 2,
                mip_levels: 2,
                encoding: PixelEncoding::R8,
                sampler: bevy::image::ImageSampler::linear(),
                bytes: vec![0, 255, 128, 64, 112],
            }],
        };
        let mut bytes = source.to_bytes();
        assert_eq!(
            BakedRecipe::from_bytes(&bytes).unwrap().maps[0].bytes,
            source.maps[0].bytes
        );
        assert!(BakedRecipe::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        bytes.push(0);
        assert!(BakedRecipe::from_bytes(&bytes).is_err());
    }
}
