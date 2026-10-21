//! One retained immutable bank; population-bound storage is never overwritten.

use std::sync::Arc;

use bevy::prelude::*;
use bevy::render::storage::ShaderBuffer;
use serde::Serialize;

use super::{TreeLayout, grids, upload::TreeStores};
use crate::prepared::{LibraryId, PopulationId, Result, SharedForest, TreeBank};
use crate::render::{
    benchmark::ForestBenchmark,
    records::{ForestBuffers, Records, Stored},
};

/// Force reupload only for a same-process-design residency benchmark control.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "viewer", derive(clap::ValueEnum))]
pub enum TreeResidencyPolicy {
    #[default]
    Resident,
    ReuploadDiagnostic,
}

#[derive(Resource, Default)]
pub(super) struct ResidentBank(Option<Arc<TreeBankAssets>>);

pub(super) struct TreeBankAssets {
    source: Arc<TreeBank>,
    pub grids: grids::UploadedGrids,
    pub records: ForestBuffers,
    record_bytes: usize,
}

pub(super) struct BankSelection {
    pub bank: Arc<TreeBankAssets>,
    transfer: BankTransfer,
}

/// Application upload evidence, not device allocator copies or physical VRAM.
#[derive(Clone, Debug, Serialize)]
pub struct TreeResidencyEvidence {
    bank: LibraryId,
    population: PopulationId,
    transfer: BankTransfer,
    grids: Vec<GridIdentity>,
    records: RecordIdentity,
    placement: AssetId<ShaderBuffer>,
    application_grid_upload_bytes: usize,
    application_bank_record_upload_bytes: usize,
    application_placement_upload_bytes: usize,
}

#[derive(Clone, Debug, Serialize)]
struct GridIdentity {
    key: grids::GridKey,
    levels: [AssetId<Mesh>; 3],
}

#[derive(Clone, Debug, Serialize)]
struct RecordIdentity {
    species: AssetId<ShaderBuffer>,
    branches: AssetId<ShaderBuffer>,
    plants: AssetId<ShaderBuffer>,
    leaves: AssetId<ShaderBuffer>,
}

#[derive(Clone, Copy, Debug, Serialize)]
enum BankTransfer {
    Uploaded,
    Reused,
}

impl TreeStores<'_> {
    pub(super) fn bank(
        &mut self,
        source: Arc<TreeBank>,
        layout: TreeLayout,
        probe: Option<&ForestBenchmark>,
    ) -> Result<BankSelection> {
        if *self.residency_policy == TreeResidencyPolicy::Resident
            && let Some(bank) = &self.resident.0
            && bank.source.id() == source.id()
            && bank.grids.stats.layout == layout
        {
            if let Some(probe) = probe {
                bank.grids.observe(probe);
            }
            return Ok(BankSelection {
                bank: Arc::clone(bank),
                transfer: BankTransfer::Reused,
            });
        }
        let library = source.library();
        let grids = grids::upload(library, layout, &mut self.meshes, probe)?;
        let records = Records::from_parts(
            &library.species,
            &library.branches,
            &library.plants,
            &library.leaves,
        );
        let record_bytes = records.byte_len();
        let mut stored = ForestBuffers::new(&mut self.buffers);
        stored.write(records, &mut self.buffers, &mut self.writes);
        let bank = Arc::new(TreeBankAssets {
            source,
            grids,
            records: stored,
            record_bytes,
        });
        self.resident.0 = Some(Arc::clone(&bank));
        Ok(BankSelection {
            bank,
            transfer: BankTransfer::Uploaded,
        })
    }
}

impl BankSelection {
    pub(super) fn evidence(
        &self,
        forest: &SharedForest,
        placement: &Stored,
        placement_bytes: usize,
    ) -> TreeResidencyEvidence {
        let bank = &self.bank;
        let records = &bank.records;
        TreeResidencyEvidence {
            bank: bank.source.id(),
            population: forest.id(),
            transfer: self.transfer,
            grids: bank
                .grids
                .handles
                .iter()
                .map(|(&key, levels)| GridIdentity {
                    key,
                    levels: levels.each_ref().map(Handle::id),
                })
                .collect(),
            records: RecordIdentity {
                species: records.species().id(),
                branches: records.branches().id(),
                plants: records.plants().id(),
                leaves: records.leaves().id(),
            },
            placement: placement.handle.id(),
            application_grid_upload_bytes: match self.transfer {
                BankTransfer::Uploaded => {
                    bank.grids.stats.vertex_bytes + bank.grids.stats.index_bytes
                }
                BankTransfer::Reused => 0,
            },
            application_bank_record_upload_bytes: match self.transfer {
                BankTransfer::Uploaded => bank.record_bytes,
                BankTransfer::Reused => 0,
            },
            application_placement_upload_bytes: placement_bytes,
        }
    }
}
