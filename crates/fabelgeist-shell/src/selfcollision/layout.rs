//! Power-of-two hash buckets and their native range-table layout.

use fabelgeist_gpu::prelude::{BufferByteLength, InvocationCount, PassParameter};
use fabelgeist_xpbd::ParticleCount;
use std::fmt;

/// A spatial-hash layout that cannot use native addresses.
#[derive(Debug)]
pub enum CollisionLayoutError {
    Buckets { particles: ParticleCount },
    AdjacencyOverflow,
}

/// Checked native offsets and flattened neighbor words, ready for upload.
pub(super) struct CollisionAdjacency {
    starts: Vec<u32>,
    neighbours: Vec<u32>,
}

#[derive(Clone, Copy)]
pub(super) struct CollisionBuckets(u32);

impl CollisionBuckets {
    pub(super) fn for_particles(particles: ParticleCount) -> Result<Self, CollisionLayoutError> {
        particles.native_elements().checked_mul(2)
            .and_then(u32::checked_next_power_of_two)
            .map(|buckets| Self(buckets.max(64)))
            .ok_or(CollisionLayoutError::Buckets { particles })
    }

    pub(super) fn range_bytes(self) -> BufferByteLength {
        BufferByteLength::from((u64::from(self.0) + 1) * 4)
    }

    pub(super) fn boundary_invocations(self) -> InvocationCount {
        // A checked power of two in u32 is at most 2^31, so its endpoint fits.
        InvocationCount::from(self.0 + 1)
    }
}

impl From<CollisionBuckets> for PassParameter {
    fn from(buckets: CollisionBuckets) -> Self {
        Self::Unsigned(buckets.0)
    }
}

impl CollisionAdjacency {
    pub(super) fn new(lists: &[Vec<u32>]) -> anyhow::Result<Self> {
        let words = lists.iter().try_fold(0u32, |total, list| {
            let words = u32::try_from(list.len())?;
            total.checked_add(words)
                .ok_or_else(|| anyhow::Error::new(CollisionLayoutError::AdjacencyOverflow))
        })?;
        let headers = lists.len().checked_add(1)
            .ok_or(CollisionLayoutError::AdjacencyOverflow)?;
        let mut starts = Vec::with_capacity(headers);
        let mut neighbours = Vec::with_capacity(words.max(1) as usize);
        for list in lists {
            starts.push(u32::try_from(neighbours.len())?);
            neighbours.extend_from_slice(list);
        }
        starts.push(words);
        if neighbours.is_empty() {
            // Empty ranges never read the one-word native allocation sentinel.
            neighbours.push(0);
        }
        Ok(Self { starts, neighbours })
    }

    pub(super) fn starts(&self) -> &[u32] {
        &self.starts
    }

    pub(super) fn neighbours(&self) -> &[u32] {
        &self.neighbours
    }
}

impl fmt::Display for CollisionLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Buckets { particles } => write!(formatter,
                "self-collision buckets for {particles} particles exceed native addresses"),
            Self::AdjacencyOverflow => formatter.write_str(
                "self-collision adjacency exceeds native addresses"),
        }
    }
}

impl std::error::Error for CollisionLayoutError {}
