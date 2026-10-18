//! Selected destination coordinates and optional terrain-route application.

use super::TravelDestination;
use crate::{
    routes::AppState,
    spacetimedb::{BackendCaseSitePin, CharacterView, SettlementView},
};

impl TravelDestination {
    pub(crate) async fn apply_selected_terrain(
        &mut self,
        state: &AppState,
        origin: &SettlementView,
        settlements: &[SettlementView],
        case_sites: &[BackendCaseSitePin],
        active_character: Option<&CharacterView>,
    ) {
        let goal = if let Some(site) = case_sites
            .iter()
            .find(|site| site.case_site_id.value == self.id)
        {
            super::super::wgs84_latitude_longitude_degrees(site.latitude_e_7, site.longitude_e_7)
                .ok()
        } else {
            settlements
                .iter()
                .find(|candidate| candidate.id == self.id)
                .map(|candidate| (candidate.latitude, candidate.longitude))
        };
        if let Some(goal) = goal {
            let terrain_profile = if let Some(character) = active_character {
                crate::routes::party_terrain_profile(state, character)
                    .await
                    .unwrap_or_default()
                    .0
            } else {
                adventuresim_terrain::TerrainSkillProfile::default()
            };
            // Existing planner port: latitude/longitude degrees in that order.
            super::apply_terrain_route(
                self,
                state.terrain.as_deref(),
                (origin.latitude, origin.longitude),
                goal,
                terrain_profile,
            )
            .await;
        }
    }
}
