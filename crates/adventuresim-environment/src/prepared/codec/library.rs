//! Model payload and its content identity.

use fabelgeist_botany::identity::{PlantId, SpeciesId};
use serde::{Deserialize, Serialize};

use super::{LIBRARY_MAGIC, Reader, begin, headers::SpeciesHeader, identity, mesh, records};
use crate::{
    prepared::{
        LibraryId, MeshClass, RecordSpan, Result, TreeBounds, TreeGrid, TreeLibrary, TreeModel,
        meshes,
    },
    render::{ForestDetail, levels::Level},
};

#[derive(Serialize, Deserialize)]
struct Header {
    detail: ForestDetail,
    species: Vec<SpeciesHeader>,
    models: Vec<ModelHeader>,
    grids: Vec<MeshClass>,
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
        grids: library.grids.iter().map(|grid| grid.class).collect(),
    };
    let mut bytes = begin(LIBRARY_MAGIC, &header)?;
    records(&mut bytes, &library.plants);
    records(&mut bytes, &library.branches);
    records(&mut bytes, &library.leaves);
    for grid in &library.grids {
        for level in &grid.levels {
            mesh(&mut bytes, level);
        }
    }
    let id = identity(&bytes);
    Ok((bytes, id))
}

pub fn decode_library(bytes: &[u8]) -> Result<(TreeLibrary, LibraryId)> {
    let mut reader = Reader(bytes);
    let header: Header = reader.header(LIBRARY_MAGIC)?;
    let plants = reader.records()?;
    let branches = reader.records()?;
    let leaves = reader.records()?;
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
    let classes = header.grids.iter().copied().collect();
    if meshes::classes(&models)? != classes || header.grids.len() != classes.len() {
        return Err(super::PreparedError::Invalid(
            "missing, unused or duplicate grid classes",
        ));
    }
    meshes::preflight(&classes, &header.detail)?;
    let grids = header
        .grids
        .into_iter()
        .map(|class| {
            let levels = Level::ALL
                .into_iter()
                .map(|level| reader.mesh(class.cardinality(&level.shape(&header.detail))?))
                .collect::<Result<Vec<_>>>()?;
            Ok(TreeGrid {
                class,
                levels: levels
                    .try_into()
                    .map_err(|_| super::PreparedError::Invalid("LOD cardinality"))?,
            })
        })
        .collect::<Result<_>>()?;
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
        grids,
    };
    library.validate()?;
    Ok((library, identity(bytes)))
}
