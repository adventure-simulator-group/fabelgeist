//! Atomic producer handoff couples named building floors to accepted support.
use super::*;
use crate::city_layout::grounding::*;
use std::collections::{BTreeMap, BTreeSet};
mod projection;
use projection::GeographicGeometryDigest;
pub use projection::{
    CityGroundingProjection, CityGroundingProjectionError, ProjectionBoundary,
    ProjectionOwnerContext,
};

#[derive(Debug, thiserror::Error)]
pub enum CityGroundingError {
    #[error("building {building} occurs twice in the physical scene")]
    DuplicateBuilding {
        building: crate::scene_input::SceneBuildingId,
    },
    #[error("building {building} lacks an exact property support binding")]
    UnboundBuilding {
        building: crate::scene_input::SceneBuildingId,
    },
    #[error(transparent)]
    Binding(#[from] CitySupportError),
    #[error(transparent)]
    Terrain(#[from] SettlementSupportError),
}

/// Selection retains the exact input layout and source. Compilation cannot
/// accidentally apply these floors to another city's programmes or geometry.
/// Compilation still must validate composition between different owners.
#[derive(Debug)]
pub struct SelectedCityGrounding {
    layout: CitySceneLayout,
    geographic: GeographicSurface,
    embedment: FoundationEmbedment,
    surfaces: Vec<PropertySupportSurface>,
    members: BTreeMap<crate::scene_input::SceneBuildingId, MemberSupport>,
}

/// One accepted projection contains both the seated placements and their
/// physical support. Consumers must install them together. Final garden and
/// enclosure collision, rendering and route access require separate validation.
#[derive(Debug)]
pub struct GroundedCitySceneLayout {
    source_digest: GeographicGeometryDigest,
    embedment: FoundationEmbedment,
    layout: CitySceneLayout,
    terrain: BoundedSettlementTerrain,
    surfaces: Vec<PropertySupportSurface>,
}

impl SelectedCityGrounding {
    pub fn select(
        layout: &CitySceneLayout,
        geographic: &GeographicSurface,
        policy: CompoundGradingPolicy,
    ) -> Result<Self, CityGroundingError> {
        validate_membership(layout)?;
        let compounds = layout.plan_compound_support(geographic, policy)?;
        let singles = layout.plan_single_property_support(
            geographic,
            SinglePropertyGradingPolicy {
                limits: policy.limits,
                stairs: policy.stairs,
                embedment: policy.embedment,
                doorway_apron: policy.street_apron,
            },
        )?;
        let members = compounds
            .iter()
            .flat_map(CompoundSupportPlan::member_support)
            .chain(singles.iter().map(|plan| plan.floor))
            .map(|member| (member.building_id, member))
            .collect();
        let mut surfaces = compounds
            .iter()
            .map(CompoundSupportPlan::support_surface)
            .collect::<Result<Vec<_>, _>>()
            .map_err(CitySupportError::Support)?;
        surfaces.extend(singles.into_iter().map(|plan| plan.surface));
        Ok(Self {
            layout: layout.clone(),
            geographic: geographic.clone(),
            embedment: policy.embedment,
            surfaces,
            members,
        })
    }

    /// Planned controls retain exact member identities before compilation.
    /// Different owners may still conflict until `compile` succeeds.
    pub fn member_support(&self) -> impl Iterator<Item = MemberSupport> + '_ {
        self.members.values().copied()
    }

    pub fn support_surfaces(&self) -> &[PropertySupportSurface] {
        &self.surfaces
    }

    /// Compose the complete declared ownership before publishing any changed
    /// floor. Failure returns the exact original support error and no projection.
    pub fn compile(self) -> Result<GroundedCitySceneLayout, SettlementSupportError> {
        let terrain =
            BoundedSettlementTerrain::compile(&self.surfaces, &self.geographic, self.embedment)?;
        let mut layout = self.layout;
        for placement in &mut layout.playable {
            placement.base_elevation_metres = self.members[&placement.id].elevation;
        }
        for placement in &mut layout.distant {
            placement.base_elevation_metres = self.members[&placement.id].elevation;
        }
        Ok(GroundedCitySceneLayout {
            source_digest: GeographicGeometryDigest::from_geographic(&self.geographic),
            embedment: self.embedment,
            layout,
            terrain,
            surfaces: self.surfaces,
        })
    }
}

impl GroundedCitySceneLayout {
    pub fn layout(&self) -> &CitySceneLayout {
        &self.layout
    }
    pub fn terrain(&self) -> &BoundedSettlementTerrain {
        &self.terrain
    }
    pub fn support_surfaces(&self) -> &[PropertySupportSurface] {
        &self.surfaces
    }

    /// Explicit projections for an owning producer; callers retain their common
    /// scene identity and must not publish the placements without their terrain.
    pub fn into_parts(
        self,
    ) -> (
        CitySceneLayout,
        BoundedSettlementTerrain,
        Vec<PropertySupportSurface>,
    ) {
        (self.layout, self.terrain, self.surfaces)
    }
}

fn validate_membership(layout: &CitySceneLayout) -> Result<(), CityGroundingError> {
    let mut ids: Vec<_> = layout
        .playable
        .iter()
        .map(|p| p.id)
        .chain(layout.distant.iter().map(|p| p.id))
        .collect();
    ids.sort_unstable();
    if let Some(pair) = ids.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(CityGroundingError::DuplicateBuilding { building: pair[0] });
    }
    let buildings: BTreeSet<_> = ids.into_iter().collect();
    let mut properties = BTreeSet::new();
    let mut members = BTreeMap::new();
    let mut owners: Vec<_> = layout
        .compounds
        .iter()
        .map(|p| (p.id, vec![p.front_building_id, p.rear_building_id]))
        .chain(
            layout
                .single_properties
                .iter()
                .map(|p| (p.id, vec![p.building_id])),
        )
        .collect();
    owners.sort_by_key(|(property, _)| *property);
    for (property, ids) in owners {
        if !properties.insert(property) {
            return Err(SettlementSupportError::DuplicateProperty { property }.into());
        }
        for building in ids {
            if !buildings.contains(&building) {
                return Err(CitySupportError::MissingBuilding { property, building }.into());
            }
            if let Some(first) = members.insert(building, property) {
                return Err(SettlementSupportError::DuplicateMember {
                    building,
                    first,
                    second: property,
                }
                .into());
            }
        }
    }
    for building in buildings {
        if !members.contains_key(&building) {
            return Err(CityGroundingError::UnboundBuilding { building });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
