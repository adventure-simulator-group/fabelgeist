//! Model payload and its content identity.

use fabelgeist_botany::identity::{PlantId, SpeciesId};
use serde::{Deserialize, Serialize};

use super::{LIBRARY_MAGIC, Reader, begin, headers::SpeciesHeader, identity, records};
use crate::{
    prepared::{LibraryId, RecordSpan, Result, TreeBounds, TreeLibrary, TreeModel},
    render::ForestDetail,
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    topology: TopologyRevision,
    detail: ForestDetail,
    species: Vec<SpeciesHeader>,
    models: Vec<ModelHeader>,
}

/// Exact canonical address/winding contract, included in the library identity.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TopologyRevision {
    CanonicalV1,
}

#[derive(Serialize, Deserialize)]
struct ModelHeader {
    source: u64,
    species: u16,
    bounds: TreeBounds,
    branches: RecordSpan,
    leaves: RecordSpan,
}

pub fn encode_library(library: &TreeLibrary) -> Result<(Vec<u8>, LibraryId)> {
    library.validate()?;
    let header = Header {
        topology: TopologyRevision::CanonicalV1,
        detail: library.detail,
        species: library.species.iter().map(SpeciesHeader::from).collect(),
        models: library
            .models
            .iter()
            .map(|model| ModelHeader {
                source: model.source.word(),
                species: model.species.word(),
                bounds: model.bounds,
                branches: model.branches,
                leaves: model.leaves,
            })
            .collect(),
    };
    let mut bytes = begin(LIBRARY_MAGIC, &header)?;
    records(&mut bytes, &library.plants);
    records(&mut bytes, &library.branches);
    records(&mut bytes, &library.leaves);
    let id = identity(&bytes);
    Ok((bytes, id))
}

#[cfg(not(target_family = "wasm"))]
pub fn decode_library(bytes: &[u8]) -> Result<(TreeLibrary, LibraryId)> {
    let library = decode_records(bytes)?;
    Ok((library, identity(bytes)))
}

#[cfg(target_family = "wasm")]
pub async fn decode_library(bytes: &[u8]) -> Result<(TreeLibrary, LibraryId)> {
    let library = decode_records(bytes)?;
    let id = super::browser_identity::library_id(bytes).await?;
    Ok((library, id))
}

fn decode_records(bytes: &[u8]) -> Result<TreeLibrary> {
    let mut reader = Reader(bytes);
    let header: Header = reader.header(LIBRARY_MAGIC)?;
    let models: Vec<_> = header
        .models
        .into_iter()
        .map(|model| TreeModel {
            source: PlantId::from_word(model.source),
            species: SpeciesId::from_word(model.species),
            bounds: model.bounds,
            branches: model.branches,
            leaves: model.leaves,
        })
        .collect();
    // Header-owned dimensions are bounded before allocating source arrays.
    super::super::TopologyRequirements::from_models(&models, &header.detail)?;
    let plants = reader.records()?;
    let branches = reader.records()?;
    let leaves = reader.records()?;
    reader.finish()?;
    let library = TreeLibrary {
        detail: header.detail,
        species: header
            .species
            .into_iter()
            .map(SpeciesHeader::into_info)
            .collect::<Result<_>>()?,
        plants,
        branches,
        leaves,
        models,
    };
    library.validate()?;
    Ok(library)
}
