//! Compile the fixed roster, programme selection and property-owned members.
use super::*;

impl GeneratedCityLayout {
    /// Packing reserves complete properties cheaply. This stage validates actual
    /// generated envelopes and access before exposing any placement to a consumer.
    pub(super) fn compile_properties(
        mut self,
        seed: fabelgeist_determinism::Seed,
    ) -> CityCompileResult<CompiledCityLayout> {
        if self.unhoused_population > ResidentCount::ZERO
            || !self.unplaced_services.is_empty()
            || !self.demand_shortfalls.is_empty()
        {
            return Err(CityCompileError::Capacity {
                residents: self.unhoused_population,
                services: self.unplaced_services.len() + self.demand_shortfalls.len(),
            });
        }
        self.lots.sort_by_key(|lot| lot.id);
        let mut palette = CityRecipePalette::default();
        let parishes = self.parish_layout()?;
        let mut clearance_cache = property::ClearanceCache::default();
        let mut buildings = Vec::new();
        let mut compounds = Vec::new();
        let mut single_properties = Vec::new();
        let mut gardens = Vec::new();
        let mut envelopes = Vec::new();
        let mut businesses = Vec::new();
        for lot in self.lots {
            let recipe = palette.front(seed, lot)?;
            let front =
                recipe.place(lot.front_building_id(), lot.centre_metres, lot.orientation)?;
            if let Some(key) = lot.service.and_then(BuildingDemand::business_key) {
                businesses.push(CityBusinessSite {
                    building_id: front.id,
                    key,
                });
            }
            if recipe.program.church_program.is_some() {
                church::validate(lot, &front, &recipe, &self.streets)?;
            }
            envelopes.push(gardens::envelope(&front, &recipe)?);
            if let Some(garden) = gardens::compile(seed, lot, &front, &recipe, &self.streets)? {
                gardens.push(garden);
            }
            if lot.has_rear_range() {
                let range = palette.range()?;
                let property::CompiledCompound { rear, compound } = property::compile(
                    lot,
                    &front,
                    &recipe,
                    &range,
                    &self.streets,
                    &mut clearance_cache,
                )?;
                envelopes.push(gardens::envelope(&rear, &range)?);
                buildings.push(rear);
                compounds.push(compound);
            }
            if !lot.has_rear_range() {
                single_properties.push(CitySingleProperty {
                    id: lot.id,
                    building_id: front.id,
                    plot: plots::reservation(lot)?,
                });
            }
            buildings.push(front);
        }
        gardens.retain(|garden| {
            envelopes
                .iter()
                .all(|envelope| garden.clears_building(*envelope))
        });
        let mut yards = self.yards;
        for garden in &gardens {
            for bed in &garden.beds {
                yards.push(CityYardPatch::from_bounds(
                    *bed,
                    CityYardSurface::KitchenGarden,
                )?);
            }
        }
        let compiled = CompiledCityLayout {
            prosperity: self.prosperity,
            gardens,
            parishes,
            buildings,
            compounds,
            single_properties,
            streets: self.streets,
            yards,
            businesses,
            support_recipes: palette,
        };
        Ok(compiled)
    }
}
