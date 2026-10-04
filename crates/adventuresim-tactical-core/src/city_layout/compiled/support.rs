//! Bind exact generated programmes to bounded geographic compound support.
use super::*;
use crate::city_layout::grounding::*;
use bevy::math::Vec3Swizzles;
mod policy;
mod street;
#[cfg(test)]
mod tests;

/// Positive finite width and run of a declared front street apron.
#[derive(Clone, Copy, Debug)]
pub struct StreetApronDimensions(Vec2);

impl StreetApronDimensions {
    pub fn dimensions_metres(self) -> Vec2 {
        self.0
    }

    pub fn from_metres(dimensions: Vec2) -> Option<Self> {
        (dimensions.is_finite() && dimensions.cmpgt(Vec2::ZERO).all()).then_some(Self(dimensions))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CompoundGradingPolicy {
    pub limits: SupportLimits,
    pub stairs: CourtStairLimits,
    pub embedment: FoundationEmbedment,
    pub street_apron: StreetApronDimensions,
}

#[derive(Debug, thiserror::Error)]
pub enum CitySupportError {
    #[error(transparent)]
    Compilation(#[from] CityCompileError),
    #[error("property {property:?} lacks exact member {building}")]
    MissingBuilding {
        property: CityPropertyId,
        building: u64,
    },
    #[error("building {building} support recipe changed its occupied programme")]
    ProgrammeChanged { building: u64 },
    #[error(
        "property {property:?}, member {building} lacks unique {outward:?} threshold or ground bearing"
    )]
    MissingBinding {
        property: CityPropertyId,
        building: u64,
        outward: Vec2,
    },
    #[error("property {property:?}, members {members:?}: source sample absent at {point:?}")]
    SourceSample {
        property: CityPropertyId,
        members: [u64; 2],
        point: Vec2,
    },
    #[error("property {property:?}, members {members:?} lacks unique gate binding at {location:?}")]
    GateBinding {
        property: CityPropertyId,
        members: [u64; 2],
        location: Vec2,
    },
    #[error("bounded support rejected: {0:?}")]
    Support(SupportDiagnostic),
}

impl CitySceneLayout {
    /// Plan without changing input terrain or placement. Installing selected
    /// floors and terrain is a separate producer operation. Complete membership
    /// is retained across playable/distant partitioning; no nearest match is used.
    pub fn plan_compound_support(
        &self,
        geographic: &GeographicSurface,
        policy: CompoundGradingPolicy,
    ) -> Result<Vec<CompoundSupportPlan>, CitySupportError> {
        let buildings: std::collections::BTreeMap<_, _> = self
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
        let mut compounds: Vec<_> = self.compounds.iter().collect();
        compounds.sort_by_key(|c| c.id);
        let mut recipes = self.support_recipes.clone();
        compounds
            .into_iter()
            .map(|property| {
                plan_property(
                    property,
                    &buildings,
                    &mut recipes,
                    geographic,
                    &self.streets,
                    policy,
                )
            })
            .collect()
    }
}

fn plan_property(
    property: &CityCompound,
    buildings: &std::collections::BTreeMap<u64, TacticalBuildingPlacement>,
    recipes: &mut CityRecipePalette,
    geographic: &GeographicSurface,
    streets: &[CityStreetPatch],
    policy: CompoundGradingPolicy,
) -> Result<CompoundSupportPlan, CitySupportError> {
    let building = |id| {
        buildings.get(&id).ok_or(CitySupportError::MissingBuilding {
            property: property.id,
            building: id,
        })
    };
    let front_placement = building(property.front_building_id)?;
    let (mut front, recipe) = member(property, front_placement, Vec2::Y, recipes, geographic)?;
    let threshold =
        recipe
            .door_point(front_placement, -Vec2::Y)
            .ok_or(CitySupportError::MissingBinding {
                property: property.id,
                building: front.building_id,
                outward: -Vec2::Y,
            })?;
    front.elevation = sample(property, geographic, threshold)?;
    let (rear, _) = member(
        property,
        building(property.rear_building_id)?,
        -Vec2::Y,
        recipes,
        geographic,
    )?;
    let mut routes = property
        .access
        .iter()
        .filter(|r| r.contains_centreline(property.boundary.gate.centre_metres));
    let passage =
        routes
            .next()
            .filter(|_| routes.next().is_none())
            .ok_or(CitySupportError::GateBinding {
                property: property.id,
                members: [front.building_id, rear.building_id],
                location: property.boundary.gate.centre_metres,
            })?;
    street::select(
        property,
        CompoundSupportLevels {
            front,
            rear,
            court: sample(property, geographic, property.court.centre_metres)?,
            gate: sample(property, geographic, property.boundary.gate.centre_metres)?,
            street: sample(property, geographic, passage.start_metres)?,
        },
        threshold,
        geographic,
        streets,
        policy,
    )
}

fn member(
    property: &CityCompound,
    placement: &TacticalBuildingPlacement,
    outward: Vec2,
    recipes: &mut CityRecipePalette,
    geographic: &GeographicSurface,
) -> Result<(MemberSupport, std::sync::Arc<recipes::Recipe>), CitySupportError> {
    let p = &placement.program;
    let recipe = recipes.for_program(p)?;
    if recipe.program != *p {
        return Err(CitySupportError::ProgrammeChanged {
            building: placement.id,
        });
    }
    let binding = || CitySupportError::MissingBinding {
        property: property.id,
        building: placement.id,
        outward,
    };
    let contact = recipe
        .collision
        .ground_floor_contact_bounds()
        .ok_or_else(binding)?;
    let threshold = recipe.door_point(placement, outward).ok_or_else(binding)?;
    Ok((
        MemberSupport {
            building_id: placement.id,
            contact: CityPlotBounds {
                centre_metres: placement.centre_metres
                    + placement.orientation.local_to_world(
                        contact.centre().xz() - recipe.collision.bounds.centre().xz(),
                    ),
                dimensions_metres: contact.plan_half_extents() * 2.0,
                orientation: placement.orientation,
            },
            court_threshold_metres: threshold,
            elevation: sample(property, geographic, threshold)?,
        },
        recipe,
    ))
}

fn sample(
    property: &CityCompound,
    geographic: &GeographicSurface,
    point: Vec2,
) -> Result<SupportElevation, CitySupportError> {
    geographic
        .elevation_at(point)
        .ok_or(CitySupportError::SourceSample {
            property: property.id,
            members: [property.front_building_id, property.rear_building_id],
            point,
        })
}
