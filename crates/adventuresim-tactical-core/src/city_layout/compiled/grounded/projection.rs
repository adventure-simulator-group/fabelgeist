//! Compact accepted plans reconstruct exact support against a bound source.
use super::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub(super) struct GeographicGeometryDigest([u8; 32]);

impl GeographicGeometryDigest {
    pub(super) fn from_geographic(source: &GeographicSurface) -> Self {
        let mut digest = Sha256::new();
        for triangle in source.triangles() {
            for point in triangle {
                for value in point.to_array() {
                    digest.update(value.to_bits().to_le_bytes());
                }
            }
        }
        Self(digest.finalize().into())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct PlacementBindingDigest([u8; 32]);
impl PlacementBindingDigest {
    fn from_placements(
        placements: &[TacticalBuildingPlacement],
    ) -> Result<Self, serde_json::Error> {
        let mut ordered: Vec<_> = placements.iter().collect();
        ordered.sort_by_key(|p| p.id);
        let encoded = serde_json::to_vec(&ordered)?;
        Ok(Self(Sha256::digest(encoded).into()))
    }
}

/// The expanded closed terrain is a derived product, never a scene-input blob.
/// Source and placement digests bind plans to their exact geographic triangles
/// and occupied programmes/floors. Reconstruction publishes no partial surface.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ProjectionWire")]
pub struct CityGroundingProjection {
    source_digest: GeographicGeometryDigest,
    placement_digest: PlacementBindingDigest,
    embedment: FoundationEmbedment,
    surfaces: Vec<PropertySupportSurface>,
    property_bindings: Vec<ProjectionPropertyBinding>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectionPropertyBinding {
    property: CityPropertyId,
    members: PropertyMembers,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectionWire {
    source_digest: GeographicGeometryDigest,
    placement_digest: PlacementBindingDigest,
    embedment: FoundationEmbedment,
    surfaces: Vec<PropertySupportSurface>,
    property_bindings: Vec<ProjectionPropertyBinding>,
}
impl TryFrom<ProjectionWire> for CityGroundingProjection {
    type Error = CityGroundingProjectionError;
    fn try_from(wire: ProjectionWire) -> Result<Self, Self::Error> {
        let projection = Self {
            source_digest: wire.source_digest,
            placement_digest: wire.placement_digest,
            embedment: wire.embedment,
            surfaces: wire.surfaces,
            property_bindings: wire.property_bindings,
        };
        projection.validate_owned_surfaces()?;
        Ok(projection)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CityGroundingProjectionError {
    #[error("compact support differs from its accepted ordered property membership")]
    PropertyMismatch { owners: Vec<ProjectionOwnerContext> },
    #[error(
        "property {property:?}, member {building}, floor at {location:?}: bound {expected:?}, placement {actual:?}"
    )]
    FloorMismatch {
        property: CityPropertyId,
        building: crate::scene_input::SceneBuildingId,
        location: crate::scene_coordinates::ScenePlanPoint,
        expected: SupportElevation,
        actual: SupportElevation,
    },
    #[error("compact support source differs from the accepted geographic triangles")]
    SourceMismatch { owners: Vec<ProjectionOwnerContext> },
    #[error("compact support programmes or placements differ from the accepted bindings")]
    PlacementMismatch { owners: Vec<ProjectionOwnerContext> },
    #[error("compact support embedment is invalid")]
    Embedment,
    #[error("occupied binding encoding failed: {0}")]
    Encoding(#[from] serde_json::Error),
    #[error("compact property {property:?} is invalid: {issue}")]
    Surface {
        property: CityPropertyId,
        issue: SupportSurfaceIssue,
    },
    #[error(transparent)]
    Binding(#[from] CityGroundingError),
    #[error(transparent)]
    Terrain(#[from] SettlementSupportError),
}

impl GroundedCitySceneLayout {
    /// Carries the accepted complete plans instead of expanded foundation cells.
    pub fn support_projection(
        &self,
    ) -> Result<CityGroundingProjection, CityGroundingProjectionError> {
        let placements = physical_placements(&self.layout);
        Ok(CityGroundingProjection {
            source_digest: self.source_digest,
            placement_digest: PlacementBindingDigest::from_placements(&placements)?,
            embedment: self.embedment,
            surfaces: self.surfaces.clone(),
            property_bindings: self
                .surfaces
                .iter()
                .map(|surface| {
                    Ok(ProjectionPropertyBinding {
                        property: surface.property_id(),
                        members: PropertyMembers::new(surface.member_building_ids().to_vec())
                            .map_err(|_| CityGroundingProjectionError::Surface {
                                property: surface.property_id(),
                                issue: SupportSurfaceIssue::Members,
                            })?,
                    })
                })
                .collect::<Result<Vec<_>, CityGroundingProjectionError>>()?,
        })
    }
}

/// A digest mismatch invalidates every owner bound by that digest. Retain the
/// complete affected set rather than naming one arbitrarily as its cause.
#[derive(Clone, Debug)]
pub struct ProjectionOwnerContext {
    pub property: CityPropertyId,
    pub members: Vec<crate::scene_input::SceneBuildingId>,
    pub locations: Vec<crate::scene_coordinates::ScenePlanPoint>,
    pub boundary: ProjectionBoundary,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionBoundary {
    GeographicTriangles,
    OccupiedBindings,
}

impl CityGroundingProjection {
    fn owner_contexts(&self, boundary: ProjectionBoundary) -> Vec<ProjectionOwnerContext> {
        self.surfaces
            .iter()
            .map(|surface| ProjectionOwnerContext {
                property: surface.property_id(),
                members: surface.member_building_ids().to_vec(),
                locations: surface
                    .support_regions()
                    .iter()
                    .map(|region| region.centre())
                    .collect(),
                boundary,
            })
            .collect()
    }
    fn validate_owned_surfaces(&self) -> Result<(), CityGroundingProjectionError> {
        let mut properties = BTreeSet::new();
        if self.property_bindings.len() != self.surfaces.len()
            || self
                .property_bindings
                .iter()
                .zip(&self.surfaces)
                .any(|(binding, surface)| {
                    !properties.insert(binding.property)
                        || binding.property != surface.property_id()
                        || binding.members.ids() != surface.member_building_ids()
                })
        {
            return Err(CityGroundingProjectionError::PropertyMismatch {
                owners: self.owner_contexts(ProjectionBoundary::OccupiedBindings),
            });
        }
        Ok(())
    }
    /// Scene descriptors independently bind compound property and front/rear roles.
    pub fn validate_compound_bindings(
        &self,
        compounds: &[CityCompound],
    ) -> Result<(), CityGroundingProjectionError> {
        for compound in compounds {
            let expected = [compound.front_building_id, compound.rear_building_id];
            if !self
                .property_bindings
                .iter()
                .any(|binding| binding.property == compound.id && binding.members.ids() == expected)
            {
                return Err(CityGroundingProjectionError::PropertyMismatch {
                    owners: self.owner_contexts(ProjectionBoundary::OccupiedBindings),
                });
            }
        }
        if self
            .property_bindings
            .iter()
            .filter(|b| b.members.ids().len() == 2)
            .count()
            != compounds.len()
        {
            return Err(CityGroundingProjectionError::PropertyMismatch {
                owners: self.owner_contexts(ProjectionBoundary::OccupiedBindings),
            });
        }
        Ok(())
    }
    /// Exact accepted owners and access regions, available for bounded diagnostics.
    pub fn surfaces(&self) -> &[PropertySupportSurface] {
        &self.surfaces
    }

    /// Validate a clean encoded projection before reconstructing closed solids.
    /// The exact source and complete occupied bindings are supplied explicitly.
    pub fn reconstruct(
        &self,
        source: &GeographicSurface,
        placements: &[TacticalBuildingPlacement],
    ) -> Result<BoundedSettlementTerrain, CityGroundingProjectionError> {
        if self.source_digest != GeographicGeometryDigest::from_geographic(source) {
            return Err(CityGroundingProjectionError::SourceMismatch {
                owners: self.owner_contexts(ProjectionBoundary::GeographicTriangles),
            });
        }
        self.validate_bindings(placements)?;
        Ok(BoundedSettlementTerrain::compile(
            &self.surfaces,
            source,
            self.embedment,
        )?)
    }

    /// Check complete immutable physical bindings without expanding terrain.
    /// Source triangles are checked separately during reconstruction.
    pub fn validate_bindings(
        &self,
        placements: &[TacticalBuildingPlacement],
    ) -> Result<(), CityGroundingProjectionError> {
        if self.placement_digest != PlacementBindingDigest::from_placements(placements)? {
            return Err(CityGroundingProjectionError::PlacementMismatch {
                owners: self.owner_contexts(ProjectionBoundary::OccupiedBindings),
            });
        }
        if !self.embedment.is_valid() {
            return Err(CityGroundingProjectionError::Embedment);
        }
        self.validate_owned_surfaces()?;
        for surface in &self.surfaces {
            surface
                .validate_encoded()
                .map_err(|issue| CityGroundingProjectionError::Surface {
                    property: surface.property_id(),
                    issue,
                })?;
        }
        let mut members = BTreeSet::new();
        let mut owners = BTreeMap::new();
        for surface in &self.surfaces {
            for building in surface.member_building_ids() {
                if let Some(first) = owners.insert(*building, surface.property_id()) {
                    return Err(SettlementSupportError::DuplicateMember {
                        building: *building,
                        first,
                        second: surface.property_id(),
                    }
                    .into());
                }
                members.insert(*building);
            }
        }
        let bound_floors: BTreeMap<_, _> = self
            .surfaces
            .iter()
            .flat_map(|surface| {
                surface
                    .floor_bearings()
                    .iter()
                    .map(move |floor| (floor.building(), (surface.property_id(), floor)))
            })
            .collect();
        for placement in placements {
            if let Some((property, floor)) = bound_floors.get(&placement.id)
                && floor.elevation() != placement.base_elevation_metres
            {
                return Err(CityGroundingProjectionError::FloorMismatch {
                    property: *property,
                    building: placement.id,
                    location: floor.regions()[0].outline().vertices()[0],
                    expected: floor.elevation(),
                    actual: placement.base_elevation_metres,
                });
            }
            if !members.remove(&placement.id) {
                return Err(CityGroundingError::UnboundBuilding {
                    building: placement.id,
                }
                .into());
            }
        }
        if let Some(building) = members.first() {
            return Err(CityGroundingError::UnboundBuilding {
                building: *building,
            }
            .into());
        }
        Ok(())
    }
}

fn physical_placements(layout: &CitySceneLayout) -> Vec<TacticalBuildingPlacement> {
    layout
        .playable
        .iter()
        .cloned()
        .chain(
            layout
                .distant
                .iter()
                .copied()
                .map(TacticalBuildingPlacement::from),
        )
        .collect()
}

#[cfg(test)]
mod tests;
