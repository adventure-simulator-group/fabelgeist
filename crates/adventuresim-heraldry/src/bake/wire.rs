//! Checked worker transport. Bytes stay binary across the WASM boundary.
use super::Baked;
use crate::Error;
const STAMP_BYTES: usize = 64;
const HEADER_BYTES: usize = 4 + STAMP_BYTES;
const RGBA_MAP_COUNT: usize = 5;
impl Baked {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_BYTES + self.flat.len() * (RGBA_MAP_COUNT + 1));
        out.extend(self.size.to_le_bytes());
        out.extend(self.stamp.as_bytes());
        for channel in [
            &self.flat,
            &self.albedo,
            &self.normal,
            &self.orm,
            &self.coat,
        ] {
            out.extend(channel);
        }
        for value in &self.height {
            out.extend(value.to_le_bytes());
        }
        out
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let invalid = || Error::Invalid("invalid baked map transport".into());
        if bytes.len() < HEADER_BYTES {
            return Err(invalid());
        }
        let size = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        if ![128, 512, 1024, 2048].contains(&size) {
            return Err(invalid());
        }
        let channel = (size * size * 4) as usize;
        if bytes.len() != HEADER_BYTES + channel * (RGBA_MAP_COUNT + 1) {
            return Err(invalid());
        }
        let stamp = std::str::from_utf8(&bytes[4..HEADER_BYTES])
            .map_err(|_| invalid())?
            .to_owned();
        if !stamp.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(invalid());
        }
        let get = |i| bytes[HEADER_BYTES + i * channel..HEADER_BYTES + (i + 1) * channel].to_vec();
        let height = bytes[HEADER_BYTES + channel * RGBA_MAP_COUNT..]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| f32::from_le_bytes(*b))
            .collect::<Vec<_>>();
        if height.iter().any(|v| !v.is_finite()) {
            return Err(invalid());
        }
        Ok(Self {
            size,
            stamp,
            flat: get(0),
            albedo: get(1),
            normal: get(2),
            orm: get(3),
            coat: get(4),
            height,
        })
    }
}
