//! Exact rendered-record control. Simulation-only indices are not persisted.

use fabelgeist_botany::{
    identity::PlantId,
    viewer::sim::{BlendShapes, Litter, Snapshot},
};
use fabelgeist_math::units::Metres;
use serde::{Deserialize, Serialize};

use super::{
    Reader, UNIQUE_MAGIC, begin,
    headers::{self, FrameHeader, SpeciesHeader},
    records,
};
use crate::prepared::{ForestFrame, PreparedUnique, Result, TreeRoot};

#[derive(Serialize, Deserialize)]
struct Header {
    frame: FrameHeader,
    species: Vec<SpeciesHeader>,
    roots: Vec<RootHeader>,
    modules: usize,
    generation: u64,
}

#[derive(Serialize, Deserialize)]
struct RootHeader {
    id: u64,
    position: [Metres; 3],
}

pub fn encode_unique(unique: &PreparedUnique) -> Result<Vec<u8>> {
    unique.validate()?;
    let frame = ForestFrame::from_unique(unique);
    let snapshot = &unique.snapshot;
    let header = Header {
        frame: FrameHeader::new(&frame)?,
        species: snapshot.species.iter().map(SpeciesHeader::from).collect(),
        roots: unique
            .roots
            .iter()
            .map(|root| RootHeader {
                id: root.id.word(),
                position: root.position,
            })
            .collect(),
        modules: snapshot.modules,
        generation: snapshot.scene_generation,
    };
    let mut bytes = begin(UNIQUE_MAGIC, &header)?;
    headers::ground(&mut bytes, &frame);
    records(&mut bytes, &snapshot.plant_bounds);
    records(&mut bytes, &snapshot.branches);
    records(&mut bytes, &snapshot.leaves);
    Ok(bytes)
}

pub fn decode_unique(bytes: &[u8]) -> Result<PreparedUnique> {
    let mut reader = Reader(bytes);
    let header: Header = reader.header(UNIQUE_MAGIC)?;
    let frame = header.frame.read(&mut reader)?;
    let unique = PreparedUnique {
        roots: header
            .roots
            .into_iter()
            .map(|root| TreeRoot {
                id: PlantId::from_word(root.id),
                position: root.position,
            })
            .collect(),
        label: frame.label,
        detail: frame.detail,
        focus: frame.focus,
        snapshot: Snapshot {
            scene_generation: header.generation,
            time: frame.time,
            plants: frame.source.plants,
            modules: header.modules,
            step_ms: 0.0,
            species: header
                .species
                .into_iter()
                .map(SpeciesHeader::into_info)
                .collect::<Result<_>>()?,
            plant_bounds: reader.records()?,
            branches: reader.records()?,
            leaves: reader.records()?,
            // Prepared assets are renderer inputs, never resumable growth state.
            blend: BlendShapes::default(),
            fillets: false,
            fillet_size: 0.0,
            litter: Litter::default(),
            hierarchy: Default::default(),
            terrain: frame.terrain,
            structures: frame.structures,
            bounds: frame.bounds,
            biome: frame.biome,
        },
    };
    reader.finish()?;
    unique.validate()?;
    if unique.snapshot.branches.len() != frame.source.branches
        || unique.snapshot.leaves.len() != frame.source.leaves
    {
        return Err(super::PreparedError::Invalid("source record counts"));
    }
    Ok(unique)
}
