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
    pub const fn new(
        width: adventuresim_building_generator::spatial_geometry::PositiveLength,
        run: adventuresim_building_generator::spatial_geometry::PositiveLength,
    ) -> Self {
        Self(Vec2::new(width.metres(), run.metres()))
    }
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

#[derive(Clone, Copy, Debug)]
pub enum SupportBindingRole {
    GroundBearing,
    Threshold(adventuresim_building_generator::Direction),
}

#[derive(Debug, thiserror::Error)]
pub enum CitySupportError {
    #[error("physical building {building} occurs twice in support inputs")]
    DuplicateBuilding {
        building: crate::scene_input::SceneBuildingId,
    },
    #[error(transparent)]
    Geometry(#[from] adventuresim_building_generator::spatial_geometry::GeometryError),
    #[error(transparent)]
    Compilation(#[from] CityCompileError),
    #[error("property {property:?} lacks exact member {building}")]
    MissingBuilding {
        property: CityPropertyId,
        building: crate::scene_input::SceneBuildingId,
    },
    #[error("property {property:?}, member {building} has invalid bearing: {issue}")]
    InvalidBearing {
        property: CityPropertyId,
        building: crate::scene_input::SceneBuildingId,
        issue: adventuresim_building_generator::plan_geometry::PlanGeometryError,
    },
    #[error("building {building} support recipe changed its occupied programme")]
    ProgrammeChanged {
        building: crate::scene_input::SceneBuildingId,
    },
    #[error("property {property:?}, member {building} lacks unique {binding:?} support binding")]
    MissingBinding {
        property: CityPropertyId,
        building: crate::scene_input::SceneBuildingId,
        binding: SupportBindingRole,
    },
    #[error("property {property:?}, members {members:?}: source sample absent at {point:?}")]
    SourceSample {
        property: CityPropertyId,
        members: [crate::scene_input::SceneBuildingId; 2],
        point: crate::scene_coordinates::ScenePlanPoint,
    },
    #[error("property {property:?}, members {members:?} lacks unique gate binding at {location:?}")]
    GateBinding {
        property: CityPropertyId,
        members: [crate::scene_input::SceneBuildingId; 2],
        location: SupportDiagnosticLocation,
    },
    #[error("bounded support rejected: {0:?}")]
    Support(SupportDiagnostic),
}

impl CitySceneLayout {
    pub(super) fn validate_physical_support_members(&self) -> Result<(), CitySupportError> {
        let mut members = std::collections::BTreeSet::new();
        for building in self
            .playable
            .iter()
            .map(|p| p.id)
            .chain(self.distant.iter().map(|p| p.id))
        {
            if !members.insert(building) {
                return Err(CitySupportError::DuplicateBuilding { building });
            }
        }
        Ok(())
    }
    /// Plan without changing input terrain or placement. Installing selected
    /// floors and terrain is a separate producer operation. Complete membership
    /// is retained across playable/distant partitioning; no nearest match is used.
    pub fn plan_compound_support(
        &self,
        geographic: &GeographicSurface,
        policy: CompoundGradingPolicy,
    ) -> Result<Vec<CompoundSupportPlan>, CitySupportError> {
        self.validate_physical_support_members()?;
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
    buildings: &std::collections::BTreeMap<
        crate::scene_input::SceneBuildingId,
        TacticalBuildingPlacement,
    >,
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
    let (mut front, recipe) = member(
        property,
        front_placement,
        adventuresim_building_generator::Direction::North,
        recipes,
        geographic,
    )?;
    let threshold = recipe
        .door_point(
            front_placement,
            adventuresim_building_generator::Direction::South,
        )?
        .ok_or(CitySupportError::MissingBinding {
            property: property.id,
            building: front.building_id,
            binding: SupportBindingRole::Threshold(
                adventuresim_building_generator::Direction::South,
            ),
        })?;
    front.elevation = sample(property, geographic, threshold)?;
    let (rear, _) = member(
        property,
        building(property.rear_building_id)?,
        adventuresim_building_generator::Direction::South,
        recipes,
        geographic,
    )?;
    let gate =
        crate::scene_coordinates::ScenePlanPoint::try_from(property.boundary.gate.centre_metres)
            .map_err(|cause| {
                CitySupportError::Support(SupportDiagnostic::gate_position(property, cause))
            })?;
    let mut routes = property
        .access
        .iter()
        .filter(|r| r.contains_centreline(gate));
    let passage =
        routes
            .next()
            .filter(|_| routes.next().is_none())
            .ok_or(CitySupportError::GateBinding {
                property: property.id,
                members: [front.building_id, rear.building_id],
                location: SupportDiagnosticLocation::from_attempt_metres(
                    property.boundary.gate.centre_metres,
                ),
            })?;
    street::select(
        property,
        CompoundSupportLevels {
            front,
            rear,
            court: sample(property, geographic, property.court.centre())?,
            gate: sample(property, geographic, gate)?,
            street: sample(property, geographic, passage.start())?,
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
    outward: adventuresim_building_generator::Direction,
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
        binding: SupportBindingRole::Threshold(outward),
    };
    let contact = recipe
        .collision
        .ground_floor_contact_bounds()
        .map_err(|issue| CitySupportError::InvalidBearing {
            property: property.id,
            building: placement.id,
            issue,
        })?
        .ok_or_else(binding)?;
    let threshold = recipe.door_point(placement, outward)?.ok_or_else(binding)?;
    Ok((
        MemberSupport {
            building_id: placement.id,
            contact: CityPlotBounds::new(
                crate::scene_coordinates::ScenePlanPoint::try_from(
                    placement.centre_metres.metres()
                        + placement.orientation.local_to_world(
                            contact.centre()?.metres().xz()
                                - recipe.collision.bounds.centre()?.metres().xz(),
                        ),
                )?,
                adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                    contact.plan_half_extents()?.metres() * 2.0,
                )?,
                placement.orientation,
            )?,
            court_threshold_metres: threshold,
            elevation: sample(property, geographic, threshold)?,
        },
        recipe,
    ))
}

fn sample(
    property: &CityCompound,
    geographic: &GeographicSurface,
    point: crate::scene_coordinates::ScenePlanPoint,
) -> Result<SupportElevation, CitySupportError> {
    geographic
        .elevation_at(point)
        .ok_or(CitySupportError::SourceSample {
            property: property.id,
            members: [property.front_building_id, property.rear_building_id],
            point,
        })
}
