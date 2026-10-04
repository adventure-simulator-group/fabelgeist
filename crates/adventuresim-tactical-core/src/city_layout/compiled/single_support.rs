//! Retained single-building reservations supply exact bounded support ownership.
use super::*;
use crate::city_layout::grounding::*;
use bevy::math::Vec3Swizzles;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A generated home or service property without a separate rear-range member.
/// Its complete court and passage reservation survives recipe compilation.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CitySingleProperty {
    pub id: CityPropertyId,
    pub building_id: u64,
    pub plot: CityPlotBounds,
}

#[derive(Clone, Copy, Debug)]
pub struct SinglePropertyGradingPolicy {
    pub limits: SupportLimits,
    pub stairs: CourtStairLimits,
    pub embedment: FoundationEmbedment,
    /// Validated clear width and maximum perpendicular doorway approach.
    pub doorway_apron: StreetApronDimensions,
}

impl CitySceneLayout {
    pub fn plan_single_property_support(
        &self,
        geographic: &GeographicSurface,
        policy: SinglePropertyGradingPolicy,
    ) -> Result<Vec<SingleBuildingSupportPlan>, CitySupportError> {
        let mut recipes = self.support_recipes.clone();
        let mut properties: Vec<_> = self.single_properties.iter().collect();
        properties.sort_by_key(|property| property.id);
        let buildings: BTreeMap<_, _> = self
            .playable
            .iter()
            .cloned()
            .chain(
                self.distant
                    .iter()
                    .copied()
                    .map(TacticalBuildingPlacement::from),
            )
            .map(|b| (b.id, b))
            .collect();
        properties
            .into_iter()
            .map(|property| {
                let placement = buildings.get(&property.building_id).ok_or(
                    CitySupportError::MissingBuilding {
                        property: property.id,
                        building: property.building_id,
                    },
                )?;
                let recipe = recipes.for_program(&placement.program)?;
                let contact = recipe.collision.ground_floor_contact_bounds().ok_or(
                    CitySupportError::MissingBinding {
                        property: property.id,
                        building: property.building_id,
                        outward: Vec2::ZERO,
                    },
                )?;
                let bearing = CityPlotBounds {
                    centre_metres: placement.centre_metres
                        + placement.orientation.local_to_world(
                            contact.centre().xz() - recipe.collision.bounds.centre().xz(),
                        ),
                    dimensions_metres: contact.plan_half_extents() * 2.0,
                    orientation: placement.orientation,
                };
                let footprint = recipe.collision.ground_floor_footprint().ok_or(
                    CitySupportError::MissingBinding {
                        property: property.id,
                        building: property.building_id,
                        outward: Vec2::ZERO,
                    },
                )?;
                let bearing_outline = footprint
                    .vertices()
                    .iter()
                    .map(|p| {
                        placement.centre_metres
                            + placement
                                .orientation
                                .local_to_world(*p - recipe.collision.bounds.centre().xz())
                    })
                    .collect();
                let thresholds: Vec<_> = recipe
                    .ground_entrances
                    .iter()
                    .map(|door| {
                        let local = door.threshold_metres - recipe.collision.bounds.centre().xz();
                        DoorwaySupportBinding {
                            entrance: door.id,
                            support: door.support,
                            threshold_metres: placement.centre_metres
                                + placement.orientation.local_to_world(local),
                            outward: placement.orientation.local_to_world(door.outward),
                        }
                    })
                    .collect();
                SingleBuildingSupportRequest {
                    property: *property,
                    bearing,
                    bearing_outline,
                    thresholds: &thresholds,
                    geographic,
                    streets: &self.streets,
                    policy,
                }
                .select()
                .map_err(CitySupportError::Support)
            })
            .collect()
    }
}
