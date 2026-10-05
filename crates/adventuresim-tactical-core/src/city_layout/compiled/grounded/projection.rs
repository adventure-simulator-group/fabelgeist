//! Compact accepted plans reconstruct exact support against a bound source.
use super::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub(super) struct GroundingDigest([u8; 32]);

impl GroundingDigest {
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

    fn from_placements(placements: &[TacticalBuildingPlacement]) -> Self {
        let mut ordered: Vec<_> = placements.iter().collect();
        ordered.sort_by_key(|p| p.id);
        let encoded = serde_json::to_vec(&ordered)
            .expect("accepted building programmes and placements serialize");
        Self(Sha256::digest(encoded).into())
    }
}

/// The expanded closed terrain is a derived product, never a scene-input blob.
/// Source and placement digests bind plans to their exact geographic triangles
/// and occupied programmes/floors. Reconstruction publishes no partial surface.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CityGroundingProjection {
    source_digest: GroundingDigest,
    placement_digest: GroundingDigest,
    embedment: FoundationEmbedment,
    surfaces: Vec<PropertySupportSurface>,
}

#[derive(Debug, thiserror::Error)]
pub enum CityGroundingProjectionError {
    #[error("compact support source differs from the accepted geographic triangles")]
    SourceMismatch,
    #[error("compact support programmes or placements differ from the accepted bindings")]
    PlacementMismatch,
    #[error("compact support embedment is invalid")]
    Embedment,
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
    pub fn support_projection(&self) -> CityGroundingProjection {
        let placements = physical_placements(&self.layout);
        CityGroundingProjection {
            source_digest: self.source_digest,
            placement_digest: GroundingDigest::from_placements(&placements),
            embedment: self.embedment,
            surfaces: self.surfaces.clone(),
        }
    }
}

impl CityGroundingProjection {
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
        if self.source_digest != GroundingDigest::from_geographic(source) {
            return Err(CityGroundingProjectionError::SourceMismatch);
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
        if self.placement_digest != GroundingDigest::from_placements(placements) {
            return Err(CityGroundingProjectionError::PlacementMismatch);
        }
        if !self.embedment.is_valid() {
            return Err(CityGroundingProjectionError::Embedment);
        }
        for surface in &self.surfaces {
            surface
                .validate_encoded()
                .map_err(|issue| CityGroundingProjectionError::Surface {
                    property: surface.property_id(),
                    issue,
                })?;
        }
        let mut members = BTreeSet::new();
        for surface in &self.surfaces {
            members.extend(surface.member_building_ids().iter().copied());
        }
        for placement in placements {
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
