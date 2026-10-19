//! Caller-selected identity of an independently verified complete producer payload.
use super::{Binding, PlantId, SceneEnvironment, TacticalSceneInput, playable_cache_key};
use anyhow::{Result, ensure};
use serde::{Serialize, Serializer};
use std::sync::Arc;
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TreeCacheContentId(blake3::Hash);

/// Complete checked producer binding retained after payload conversion.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PreparedTreeIdentity {
    content: TreeCacheContentId,
    binding: Arc<Binding>,
}

/// The caller pins previously source-verified producer content independently
/// of the file under review. Live input, environment and source are still
/// checked against the complete decoded binding during admission.
pub(crate) struct ExpectedTreeCache<'a> {
    pub(super) input: &'a TacticalSceneInput,
    pub(super) environment: &'a SceneEnvironment,
    pub(super) source: PlantId,
    pub(super) content: TreeCacheContentId,
}

impl TreeCacheContentId {
    pub(super) fn from_digest(digest: blake3::Hash) -> Self {
        Self(digest)
    }

    /// Native encoded-container boundary, before payload allocation.
    pub(super) fn matches_digest(self, bytes: &[u8]) -> bool {
        self.0.as_bytes() == bytes
    }
}

impl fmt::Display for TreeCacheContentId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0.to_hex(), formatter)
    }
}

impl FromStr for TreeCacheContentId {
    type Err = blake3::HexError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        blake3::Hash::from_hex(value).map(Self)
    }
}

impl Serialize for TreeCacheContentId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'a> ExpectedTreeCache<'a> {
    pub(crate) fn new(
        input: &'a TacticalSceneInput,
        environment: &'a SceneEnvironment,
        source: PlantId,
        content: TreeCacheContentId,
    ) -> Self {
        Self {
            input,
            environment,
            source,
            content,
        }
    }
}

impl PreparedTreeIdentity {
    pub(super) fn from_checked(binding: Binding, content: TreeCacheContentId) -> Self {
        Self {
            content,
            binding: Arc::new(binding),
        }
    }

    pub(crate) fn content_id(&self) -> TreeCacheContentId {
        self.content
    }

    pub(in crate::presentation::obstacles::tree) fn verify_environment(
        &self,
        environment: &SceneEnvironment,
    ) -> Result<()> {
        ensure!(
            self.binding.environment == serde_json::to_vec(environment)?,
            "prepared tree destination environment changed"
        );
        ensure!(
            self.binding.cache_key
                == playable_cache_key(self.binding.species, self.binding.seed, environment),
            "prepared tree destination site changed"
        );
        Ok(())
    }
}

impl fmt::Debug for PreparedTreeIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedTreeIdentity")
            .field("content", &self.content)
            .field("source", &self.binding.source)
            .field("cache_key", &self.binding.cache_key)
            .finish_non_exhaustive()
    }
}
