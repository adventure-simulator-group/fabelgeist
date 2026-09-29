//! Compact serialization of a drape's per-vertex arrays: their words in
//! little-endian order as one base64 string, rather than one JSON number per
//! value, so a saved drape stays small in pretty-printed recipes.

use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Deserializer, Serializer, de::Error};

/// A value stored as a fixed number of 32-bit words.
pub(super) trait Packed: Sized {
    const WORDS: usize;
    fn pack(&self, words: &mut Vec<u32>);
    /// Rebuild the value from exactly [`Self::WORDS`] words.
    fn unpack(words: &[u32]) -> Self;
}

impl Packed for [f32; 2] {
    const WORDS: usize = 2;
    fn pack(&self, words: &mut Vec<u32>) {
        words.extend(self.map(f32::to_bits));
    }
    fn unpack(words: &[u32]) -> Self {
        std::array::from_fn(|i| f32::from_bits(words[i]))
    }
}

impl Packed for [f32; 3] {
    const WORDS: usize = 3;
    fn pack(&self, words: &mut Vec<u32>) {
        words.extend(self.map(f32::to_bits));
    }
    fn unpack(words: &[u32]) -> Self {
        std::array::from_fn(|i| f32::from_bits(words[i]))
    }
}

impl Packed for [u32; 3] {
    const WORDS: usize = 3;
    fn pack(&self, words: &mut Vec<u32>) {
        words.extend(self);
    }
    fn unpack(words: &[u32]) -> Self {
        std::array::from_fn(|i| words[i])
    }
}

/// Serde's `with` attribute passes the field itself, a `Vec`.
pub(super) fn serialize<T: Packed, S: Serializer>(
    values: &Vec<T>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut words = Vec::with_capacity(values.len() * T::WORDS);
    for value in values {
        value.pack(&mut words);
    }
    let bytes: Vec<u8> = words.into_iter().flat_map(u32::to_le_bytes).collect();
    serializer.serialize_str(&STANDARD.encode(bytes))
}

pub(super) fn deserialize<'de, T: Packed, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<T>, D::Error> {
    let text = String::deserialize(deserializer)?;
    let bytes = STANDARD.decode(text).map_err(D::Error::custom)?;
    let record = T::WORDS * size_of::<u32>();
    if bytes.len() % record != 0 {
        return Err(D::Error::custom(format!(
            "packed array of {} bytes is not a whole number of {record}-byte records",
            bytes.len()
        )));
    }
    let words: Vec<u32> = bytes
        .as_chunks::<{ size_of::<u32>() }>()
        .0
        .iter()
        .map(|word| u32::from_le_bytes(*word))
        .collect();
    Ok(words.chunks_exact(T::WORDS).map(T::unpack).collect())
}
