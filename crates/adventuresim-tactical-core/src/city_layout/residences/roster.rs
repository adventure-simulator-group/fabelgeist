//! Fix property identities and housing supply before physical recipe packing.
use super::*;

pub(in crate::city_layout) struct SelectedRoster {
    pub members: Vec<CandidateLot>,
    pub unhoused_population: ResidentCount,
}
impl SelectedRoster {
    pub fn from_candidates(
        candidates: Vec<CandidateLot>,
        target_population: ResidentCount,
    ) -> Self {
        let mut represented_population = ResidentCount::ZERO;
        let mut market = adventuresim_core::settlement_property::HousingMarketReserve::default();
        let mut selected = Vec::new();
        for candidate in candidates.into_iter().take(MAX_CITY_LOTS) {
            if candidate.lot.service.is_none()
                && represented_population >= target_population
                && market.complete()
            {
                break;
            }
            let mut lot = candidate.lot;
            lot.id = CityPropertyId(selected.len() as u64 + 1);
            if lot.service.is_none()
                && market.reserve(lot.house_class.housing_tier())
                    == HomeSupplyRole::PopulationHousing
            {
                represented_population = represented_population
                    .saturating_add_capacity(lot.house_class.resident_capacity());
            }
            selected.push(CandidateLot { lot, ..candidate });
        }
        Self {
            members: selected,
            unhoused_population: target_population.saturating_sub(represented_population),
        }
    }
}
