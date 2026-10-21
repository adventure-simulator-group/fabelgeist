//! Immutable bounded visual source bank on the existing ordered scene channel.

use std::{num::NonZeroUsize, sync::Arc};

use adventuresim_tactical_core::{prelude::SceneId, scene_input::MAX_SCENE_KEY_BYTES};
use bevy::prelude::*;
use bevy_replicon::{
    bytes::Bytes,
    postcard_utils,
    shared::message::ctx::{ClientReceiveCtx, ServerSendCtx},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use bounded::{LibraryBytes, SourceWords};

mod bounded;

pub const MAX_TREE_BANK_EVENT_BYTES: usize = 4 * 1024 * 1024;
const MAX_TREE_BANK_SOURCES: usize = 8;
const SCENE_DIGEST_HEX_BYTES: usize = 64;

type WireResult<T> = std::result::Result<T, TreeBankWireError>;

/// Whole encoded event size, excluding Replicon's tick and transport framing.
#[derive(Clone, Copy, Debug)]
struct EventPayloadBytes(NonZeroUsize);

/// Shared bytes are cloned cheaply for later enrolled clients of the same scene.
/// The complete bank remains untrusted biological content until client admission.
#[derive(Event, Clone, Debug)]
pub struct SceneTreeBankAsset {
    wire: Arc<BankWire>,
    encoded: EventPayloadBytes,
}

#[derive(Serialize, Deserialize, Debug)]
struct BankWire {
    #[serde(deserialize_with = "bounded::scene_key")]
    scene: SceneId,
    #[serde(deserialize_with = "bounded::scene_digest")]
    digest: String,
    sources: SourceWords,
    library: LibraryBytes,
}

#[derive(Debug, thiserror::Error)]
pub enum TreeBankWireError {
    #[error("invalid tree-bank scene key")]
    SceneKey,
    #[error("invalid tree-bank scene digest")]
    SceneDigest,
    #[error("tree-bank sources must be one to eight strictly increasing words")]
    Sources,
    #[error("tree-bank library byte count is outside its finite wire bound")]
    LibraryLength,
    #[error("complete tree-bank event exceeds its finite payload bound")]
    PayloadLength,
    #[error("tree-bank event buffer arithmetic overflow")]
    BufferOverflow,
    #[error("tree-bank event encoded length changed")]
    EncodedLength,
    #[error("tree-bank event has trailing payload bytes")]
    TrailingPayload,
    #[error(transparent)]
    Encoding(#[from] postcard::Error),
}

impl SceneTreeBankAsset {
    /// Native numeric source words and FORESTL3 bytes enter at this wire boundary.
    pub fn new(
        scene: SceneId,
        digest: String,
        sources: Vec<u64>,
        library: Box<[u8]>,
    ) -> WireResult<Self> {
        Self::admit(BankWire {
            scene,
            digest,
            sources: SourceWords(sources.into_boxed_slice()),
            library: LibraryBytes(library),
        })
    }

    pub fn scene(&self) -> &SceneId {
        &self.wire.scene
    }

    pub fn scene_digest(&self) -> &str {
        &self.wire.digest
    }

    pub fn source_words(&self) -> &[u64] {
        &self.wire.sources.0
    }

    pub fn library_bytes(&self) -> &[u8] {
        &self.wire.library.0
    }

    fn admit(wire: BankWire) -> WireResult<Self> {
        bounded::validate_scene_key(&wire.scene.0)?;
        bounded::validate_scene_digest(&wire.digest)?;
        bounded::validate_sources(&wire.sources.0)?;
        bounded::validate_library_length(wire.library.0.len())?;
        // This scans the immutable DTO once. Repeated sends retain the checked size.
        let length = postcard::experimental::serialized_size(&wire)?;
        if length > MAX_TREE_BANK_EVENT_BYTES {
            return Err(TreeBankWireError::PayloadLength);
        }
        let encoded =
            EventPayloadBytes(NonZeroUsize::new(length).ok_or(TreeBankWireError::PayloadLength)?);
        Ok(Self {
            wire: Arc::new(wire),
            encoded,
        })
    }
}

impl Serialize for SceneTreeBankAsset {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        self.wire.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SceneTreeBankAsset {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        use serde::de::Error as _;
        Self::admit(BankWire::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

pub(crate) fn serialize(
    _ctx: &mut ServerSendCtx,
    event: &SceneTreeBankAsset,
    output: &mut Vec<u8>,
) -> bevy::prelude::Result {
    let start = output.len();
    let length = event.encoded.0.get();
    let end = start
        .checked_add(length)
        .ok_or(TreeBankWireError::BufferOverflow)?;
    output.try_reserve(length)?;
    if let Err(error) = postcard_utils::to_extend_mut(event, output) {
        output.truncate(start);
        return Err(error.into());
    }
    if output.len() != end {
        output.truncate(start);
        return Err(TreeBankWireError::EncodedLength.into());
    }
    Ok(())
}

pub(crate) fn deserialize(
    _ctx: &mut ClientReceiveCtx,
    input: &mut Bytes,
) -> bevy::prelude::Result<SceneTreeBankAsset> {
    if input.is_empty() || input.len() > MAX_TREE_BANK_EVENT_BYTES {
        return Err(TreeBankWireError::PayloadLength.into());
    }
    let mut cursor = input.clone();
    let event = postcard_utils::from_buf(&mut cursor)?;
    if !cursor.is_empty() {
        return Err(TreeBankWireError::TrailingPayload.into());
    }
    *input = cursor;
    Ok(event)
}
