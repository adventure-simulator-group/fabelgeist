//! Joint measured constraints retain the fixed roster within its street blocks.
use super::*;
use context::{ParcelFrontage, ParcelGeometry};
mod coupled;
mod propagation;
mod records;
mod search;
use records::*;

/// Operational cap per independent block, separate from land or access limits.
const MAX_BLOCK_PACKING_SEARCH_STATES: usize = 100_000;

/// Cheap authored candidates keep their previous independent allocation.
const MAX_AUTHORED_PACKING_SEARCH_STATES: usize = 5_000;

#[derive(Clone)]
struct PlacementDomain {
    owner: CityPropertyId,
    frontage: ParcelFrontage,
    proposed: ParcelGeometry,
    allowed: FrontageInterval,
    base_translation_metres: Vec2,
}
impl PlacementDomain {
    fn from_frontage(
        frontage: ParcelFrontage,
        layout: &CompiledCityLayout,
        envelopes: &BTreeMap<u64, MeasuredBuildingEnvelope>,
    ) -> Result<Self, CityCompileError> {
        let owner = CityPropertyId(frontage.lot.id);
        let geometry = frontage.geometry(layout, envelopes)?;
        let proposed = geometry;
        let allowed = frontage
            .available_displacement(proposed.reservation)
            .ok_or_else(|| CityCompileError::Packing {
                property: owner,
                issue: CityPackingIssue::NoFreeFrontage {
                    block: frontage.block.id.0,
                    envelope: proposed.reservation,
                    available_displacement_metres: None,
                    blocking_properties: Vec::new(),
                },
            })?;
        Ok(Self {
            owner,
            frontage,
            proposed,
            allowed,
            base_translation_metres: Vec2::ZERO,
        })
    }
    fn geometry_at_zero(&self) -> ParcelGeometry {
        self.proposed
            .translated(self.base_translation_metres, self.frontage.tangent())
    }
    fn delta_at(&self, displacement: FrontagePosition) -> PlanDisplacement {
        PlanDisplacement::from_metres(
            self.base_translation_metres + self.frontage.tangent() * displacement.metres() as f32,
        )
        .expect("finite constrained property displacement")
    }
}

pub(super) fn solve(
    layout: &CompiledCityLayout,
    context: &CityPackingContext,
    envelopes: &BTreeMap<u64, MeasuredBuildingEnvelope>,
) -> Result<BTreeMap<CityPropertyId, Vec2>, CityCompileError> {
    let mut blocks = BTreeMap::<BlockId, Vec<PlacementDomain>>::new();
    for &frontage in context.frontages.values() {
        blocks
            .entry(frontage.block.id)
            .or_default()
            .push(PlacementDomain::from_frontage(frontage, layout, envelopes)?);
    }
    let mut translations = BTreeMap::new();
    for domains in blocks.into_values() {
        let selected = search::BlockSearch::new(&domains).solve()?;
        translations.extend(selected.into_iter().map(|selected| {
            let domain = &domains[selected.domain.index()];
            (domain.owner, selected.displacement.metres())
        }));
    }
    Ok(translations)
}
