//! Validate advertised sequence lengths before their first collection allocation.

use std::fmt;

use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, SeqAccess, Visitor},
};

use super::{
    MAX_SCENE_KEY_BYTES, MAX_TREE_BANK_EVENT_BYTES, MAX_TREE_BANK_SOURCES, SCENE_DIGEST_HEX_BYTES,
    SceneId, TreeBankWireError, WireResult,
};

#[derive(Serialize, Debug)]
pub(super) struct LibraryBytes(pub(super) Box<[u8]>);

#[derive(Serialize, Debug)]
pub(super) struct SourceWords(pub(super) Box<[u64]>);

struct LibraryVisitor;
struct SourcesVisitor;
struct SceneKeyVisitor;
struct DigestVisitor;

impl<'de> Deserialize<'de> for LibraryBytes {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        deserializer.deserialize_seq(LibraryVisitor)
    }
}

impl<'de> Deserialize<'de> for SourceWords {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        deserializer.deserialize_seq(SourcesVisitor)
    }
}

impl<'de> Visitor<'de> for LibraryVisitor {
    type Value = LibraryBytes;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a nonempty bounded tree-bank byte sequence")
    }
    fn visit_seq<A: SeqAccess<'de>>(
        self,
        mut sequence: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let length = sequence
            .size_hint()
            .ok_or_else(|| A::Error::custom("missing bounded library length"))?;
        validate_library_length(length).map_err(A::Error::custom)?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(length).map_err(A::Error::custom)?;
        for _ in 0..length {
            bytes.push(
                sequence
                    .next_element::<u8>()?
                    .ok_or_else(|| A::Error::custom("truncated library"))?,
            );
        }
        if sequence.next_element::<u8>()?.is_some() {
            return Err(A::Error::custom("extra library element"));
        }
        Ok(LibraryBytes(bytes.into_boxed_slice()))
    }
}

impl<'de> Visitor<'de> for SourcesVisitor {
    type Value = SourceWords;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("one to eight strictly increasing source words")
    }
    fn visit_seq<A: SeqAccess<'de>>(
        self,
        mut sequence: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let length = sequence
            .size_hint()
            .ok_or_else(|| A::Error::custom("missing bounded source length"))?;
        if length == 0 || length > MAX_TREE_BANK_SOURCES {
            return Err(A::Error::custom(TreeBankWireError::Sources));
        }
        let mut words = [0; MAX_TREE_BANK_SOURCES];
        for word in words.iter_mut().take(length) {
            *word = sequence
                .next_element::<u64>()?
                .ok_or_else(|| A::Error::custom("truncated source list"))?;
        }
        if sequence.next_element::<u64>()?.is_some() {
            return Err(A::Error::custom("extra source word"));
        }
        validate_sources(&words[..length]).map_err(A::Error::custom)?;
        Ok(SourceWords(words[..length].into()))
    }
}

impl<'de> Visitor<'de> for SceneKeyVisitor {
    type Value = SceneId;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an exact bounded tactical scene key")
    }
    fn visit_str<E: serde::de::Error>(self, value: &str) -> std::result::Result<Self::Value, E> {
        validate_scene_key(value).map_err(E::custom)?;
        Ok(SceneId(value.to_owned()))
    }
}

impl<'de> Visitor<'de> for DigestVisitor {
    type Value = String;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a lowercase hexadecimal tactical scene digest")
    }
    fn visit_str<E: serde::de::Error>(self, value: &str) -> std::result::Result<Self::Value, E> {
        validate_scene_digest(value).map_err(E::custom)?;
        Ok(value.to_owned())
    }
}

pub(super) fn scene_key<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<SceneId, D::Error> {
    deserializer.deserialize_str(SceneKeyVisitor)
}

pub(super) fn scene_digest<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<String, D::Error> {
    deserializer.deserialize_str(DigestVisitor)
}

pub(super) fn validate_scene_key(value: &str) -> WireResult<()> {
    if value.is_empty() || value.len() > MAX_SCENE_KEY_BYTES {
        return Err(TreeBankWireError::SceneKey);
    }
    Ok(())
}

pub(super) fn validate_scene_digest(value: &str) -> WireResult<()> {
    if value.len() != SCENE_DIGEST_HEX_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(TreeBankWireError::SceneDigest);
    }
    Ok(())
}

pub(super) fn validate_sources(words: &[u64]) -> WireResult<()> {
    if words.is_empty()
        || words.len() > MAX_TREE_BANK_SOURCES
        || !words.windows(2).all(|pair| pair[0] < pair[1])
    {
        return Err(TreeBankWireError::Sources);
    }
    Ok(())
}

pub(super) fn validate_library_length(length: usize) -> WireResult<()> {
    if length == 0 || length > MAX_TREE_BANK_EVENT_BYTES {
        return Err(TreeBankWireError::LibraryLength);
    }
    Ok(())
}
