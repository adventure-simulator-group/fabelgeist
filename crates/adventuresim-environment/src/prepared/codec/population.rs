//! Compact regional descriptors: no per-placement branch or leaf expansion.

use fabelgeist_botany::identity::PlantId;
use serde::{Deserialize, Serialize};

use super::{
    POPULATION_MAGIC, Reader, begin,
    headers::{self, FrameHeader},
};
use crate::prepared::{
    LibraryId, Result, TreeLibrary, TreePlacement, TreePopulation, TreePose, VariantIndex,
};

#[derive(Serialize, Deserialize)]
struct Header {
    library: LibraryId,
    frame: FrameHeader,
    placements: Vec<PlacementHeader>,
}

#[derive(Serialize, Deserialize)]
struct PlacementHeader {
    id: u64,
    variant: VariantIndex,
    pose: TreePose,
}

pub fn encode_population(
    population: &TreePopulation,
    library: &TreeLibrary,
    id: LibraryId,
) -> Result<Vec<u8>> {
    population.validate(library, id)?;
    let header = Header {
        library: population.library,
        frame: FrameHeader::new(&population.frame)?,
        placements: population
            .placements
            .iter()
            .map(|p| PlacementHeader {
                id: p.id.word(),
                variant: p.variant,
                pose: p.pose,
            })
            .collect(),
    };
    let mut bytes = begin(POPULATION_MAGIC, &header)?;
    headers::ground(&mut bytes, &population.frame);
    Ok(bytes)
}

pub fn decode_population(
    bytes: &[u8],
    library: &TreeLibrary,
    id: LibraryId,
) -> Result<TreePopulation> {
    let mut reader = Reader(bytes);
    let header: Header = reader.header(POPULATION_MAGIC)?;
    let population = TreePopulation {
        library: header.library,
        frame: header.frame.read(&mut reader)?,
        placements: header
            .placements
            .into_iter()
            .map(|p| TreePlacement {
                id: PlantId::from_word(p.id),
                variant: p.variant,
                pose: p.pose,
            })
            .collect(),
    };
    reader.finish()?;
    population.validate(library, id)?;
    Ok(population)
}
