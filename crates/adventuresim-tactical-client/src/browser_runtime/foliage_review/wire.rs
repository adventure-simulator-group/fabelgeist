//! Checked browser request envelope; camera geometry has one shared owner.

use adventuresim_environment::{
    prepared::{LibraryId, PopulationId},
    render::view::{ForestView, ForestViewDocument},
};
use anyhow::{Result, ensure};
use bevy::prelude::*;
use serde::Deserialize;

pub(super) const MAX_DOCUMENT_BYTES: usize = 8192;
pub(super) const MAX_BANK_BYTES: usize = 64 * 1024 * 1024;
pub(super) const MAX_POPULATION_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone)]
pub(crate) struct Document {
    pub(super) bank: LibraryId,
    pub(super) population: PopulationId,
    pub(super) transfer: BankTransfer,
    pub(super) camera: ForestView,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireDocument {
    bank: LibraryId,
    population: PopulationId,
    transfer: BankTransfer,
    camera: ForestViewDocument,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum BankTransfer {
    Download,
    Resident,
}

impl Document {
    pub fn parse(json: &str) -> Result<Self> {
        ensure!(
            json.len() <= MAX_DOCUMENT_BYTES,
            "oversized review document"
        );
        let wire: WireDocument = serde_json::from_str(json)?;
        let camera = wire.camera.admit(DistanceFog {
            color: super::scene::SKY,
            ..default()
        })?;
        Ok(Self {
            bank: wire.bank,
            population: wire.population,
            transfer: wire.transfer,
            camera,
        })
    }
}
