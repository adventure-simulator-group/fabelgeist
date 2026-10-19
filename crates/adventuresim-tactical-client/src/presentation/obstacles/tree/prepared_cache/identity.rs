//! Caller-selected identity of an independently verified complete producer payload.
use super::{PlantId, SceneEnvironment, TacticalSceneInput};
use serde::{Serialize, Serializer};
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TreeCacheContentId(blake3::Hash);

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
