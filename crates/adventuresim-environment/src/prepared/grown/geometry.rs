//! grown numerical-kernel shapes; packed GPU words exist only at submission.

use anyhow::{Context, Result};
use fabelgeist_botany::{
    species::Foliage,
    viewer::sim::{BranchInstance, LeafInstance},
};

use crate::render::{ForestDetail, ForestPart, Level};

#[derive(Clone, Copy)]
pub(super) enum RecordGeometry {
    Tube { sides: u32, spans: u32 },
    Blades { count: u32, foliage: Foliage },
}

impl RecordGeometry {
    pub fn at_level(detail: &ForestDetail, foliage: Foliage, level: Level) -> [Self; 2] {
        // The renderer owns the admitted minimum sides/spans/blades.
        let shape = level.shape(detail);
        [
            Self::Tube {
                sides: shape.sides,
                spans: shape.spans,
            },
            Self::Blades {
                count: shape.blades,
                foliage,
            },
        ]
    }

    pub fn role(self) -> ForestPart {
        match self {
            Self::Tube { .. } => ForestPart::Branches,
            Self::Blades { .. } => ForestPart::Leaves,
        }
    }

    pub fn vertices_per_record(self) -> Result<u32> {
        match self {
            Self::Tube { sides, spans } => sides
                .checked_add(1)
                .and_then(|columns| {
                    spans
                        .checked_add(1)
                        .and_then(|rings| columns.checked_mul(rings))
                })
                .context("grown tube vertex overflow"),
            Self::Blades { count, .. } => {
                count.checked_mul(4).context("grown blade vertex overflow")
            }
        }
    }

    pub fn indices_per_record(self) -> Result<u32> {
        match self {
            Self::Tube { sides, spans } => sides
                .checked_mul(spans)
                .and_then(|quads| quads.checked_mul(6)),
            Self::Blades { count, .. } => count.checked_mul(6),
        }
        .context("grown record index overflow")
    }

    pub fn record_stride(self) -> u64 {
        match self {
            Self::Tube { .. } => size_of::<BranchInstance>() as u64,
            Self::Blades { .. } => size_of::<LeafInstance>() as u64,
        }
    }

    /// Explicit native uniform boundary; source roles do not travel as arrays.
    pub fn shader_words(self) -> Result<[u32; 4]> {
        let vertices = self.vertices_per_record()?;
        Ok(match self {
            Self::Tube { sides, spans } => [sides, spans, vertices, 0],
            Self::Blades { count, foliage } => [count, foliage as u32, vertices, 0],
        })
    }
}
