//! Deterministic settlement properties compiled by the shared city generator.
use adventuresim_core::reputation::effective_population;
use adventuresim_tactical_core::city_layout::{CityCompileError, CitySceneLayout};
use adventuresim_tactical_core::prelude::*;
use adventuresim_world_schema::SettlementEconomyProfile;
use sha2::{Digest, Sha256};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementSceneProfile {
    pub id: String,
    pub population_level: i32,
    pub population_estimate: u32,
    pub economy: SettlementEconomyProfile,
}

impl SettlementSceneProfile {
    fn effective_population(&self) -> u32 {
        effective_population(self.population_level, self.population_estimate)
    }
}

pub fn place_settlement_buildings(
    settlement: &SettlementSceneProfile,
    playable_half_extent_metres: f32,
) -> Result<CitySceneLayout, CityCompileError> {
    let seed = settlement_seed(&settlement.id);
    CitySite::central_german_market_town()
        .generate(seed, settlement.effective_population(), &settlement.economy)
        .compile(seed)?
        .partition(Some(playable_half_extent_metres))
}

fn settlement_seed(id: &str) -> u64 {
    let digest = Sha256::digest(id.as_bytes());
    u64::from_le_bytes(digest[..8].try_into().expect("SHA-256 prefix"))
}

#[cfg(test)]
mod tests;
