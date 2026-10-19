//! Shared grid topology classes, distinct from tree identities and bounds.

use serde::{Deserialize, Serialize};

use super::{PreparedError, Result};
use crate::render::ForestPart;

/// Positive power-of-two record slots in one indexed grid. Padding is clipped
/// by the shader, using the placement's actual record count.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct RecordCapacity(u32);

/// Topology reuse does not depend on species or source model identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MeshClass {
    pub part: ForestPart,
    pub capacity: RecordCapacity,
}

pub struct TreeGrid {
    pub class: MeshClass,
    pub levels: [AddressMesh; 3],
}

/// Local integer vertex addresses, not positions or global botanical indices.
/// Bounds always come from the placed TreeModel, including for padded slots.
pub struct AddressMesh {
    pub vertices: Vec<u32>,
    pub indices: Vec<u32>,
}

impl RecordCapacity {
    pub fn covering(records: u32) -> Result<Self> {
        records
            .checked_next_power_of_two()
            .filter(|_| records > 0)
            .map(Self)
            .ok_or(PreparedError::Invalid(
                "record capacity overflow or empty grid",
            ))
    }
    pub fn records(self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for RecordCapacity {
    type Error = PreparedError;
    fn try_from(value: u32) -> Result<Self> {
        if !value.is_power_of_two() {
            return Err(PreparedError::Invalid(
                "grid capacity must be a positive power of two",
            ));
        }
        Ok(Self(value))
    }
}

impl From<RecordCapacity> for u32 {
    fn from(value: RecordCapacity) -> Self {
        value.0
    }
}

impl MeshClass {
    pub fn for_records(part: ForestPart, records: u32) -> Result<Option<Self>> {
        if records == 0 {
            return Ok(None);
        }
        Ok(Some(Self {
            part,
            capacity: RecordCapacity::covering(records)?,
        }))
    }
}
