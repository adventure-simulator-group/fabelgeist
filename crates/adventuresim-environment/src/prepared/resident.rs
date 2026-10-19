//! Immutable admitted content: a digest can never outlive its decoded bank.

use std::sync::Arc;

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::{LibraryId, Result, TreeLibrary, TreePopulation, decode_library, decode_population};

/// Decoded model bank with its validated complete-payload identity.
/// Raw libraries remain available to offline builders, but cannot enter the
/// resident renderer paired with an independently supplied claimed identity.
pub struct TreeBank {
    id: LibraryId,
    library: TreeLibrary,
}

/// SHA-256 of the complete encoded descriptors and ground, separate from bank ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PopulationId([u8; 32]);

/// One immutable admitted population bound to its exact shared model bank.
pub struct SharedForest {
    bank: Arc<TreeBank>,
    population: TreePopulation,
    id: PopulationId,
}

impl TreeBank {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let (library, id) = decode_library(bytes)?;
        Ok(Self { id, library })
    }

    pub fn id(&self) -> LibraryId {
        self.id
    }

    pub fn library(&self) -> &TreeLibrary {
        &self.library
    }
}

impl SharedForest {
    pub fn from_population_bytes(bank: Arc<TreeBank>, bytes: &[u8]) -> Result<Self> {
        let population = decode_population(bytes, bank.library(), bank.id())?;
        Ok(Self {
            bank,
            population,
            id: PopulationId(Sha256::digest(bytes).into()),
        })
    }

    pub fn bank(&self) -> &Arc<TreeBank> {
        &self.bank
    }

    pub fn population(&self) -> &TreePopulation {
        &self.population
    }

    pub fn id(&self) -> PopulationId {
        self.id
    }
}
