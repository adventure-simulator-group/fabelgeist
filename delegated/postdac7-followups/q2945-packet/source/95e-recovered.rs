//! Enclosure geometry retains the property, physical members and gate datum.
use super::*;
use sha2::{Digest, Sha256};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "BindingWire")]
pub struct BoundaryOwnerBinding {
    property: CityPropertyId,
    members: PropertyMembers,
    gate: SupportElevation,
    descriptor_digest: BoundaryDescriptorDigest,
    elements: Vec<BoundarySupportElement>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
struct BoundaryDescriptorDigest([u8; 32]);
impl BoundaryDescriptorDigest {
    fn of(boundary: &crate::city_layout::CityBoundary) -> Self {
        // Descriptor hashing is an explicit little-endian scalar boundary;
        // every authored field participates, including wall order and hinge.
        let mut digest = Sha256::new();
        digest.update((boundary.walls.len() as u64).to_le_bytes());
        for wall in &boundary.walls {
            for scalar in [
                wall.start_metres.x,
                wall.start_metres.y,
                wall.end_metres.x,
                wall.end_metres.y,
                wall.height_metres,
                wall.thickness_metres,
            ] {
                digest.update(scalar.to_bits().to_le_bytes());
            }
        }
        let gate = boundary.gate;
        digest.update([match gate.hinge {
            crate::city_layout::PropertySide::Left => 0,
            crate::city_layout::PropertySide::Right => 1,
        }]);
        for scalar in [
            gate.centre_metres.x,
            gate.centre_metres.y,
            gate.orientation.yaw_radians(),
            gate.width_metres,
            gate.height_metres,
        ] {
            digest.update(scalar.to_bits().to_le_bytes());
        }
        Self(digest.finalize().into())
    }
}
#[derive(Deserialize)]
struct BindingWire {
    property: CityPropertyId,
    members: PropertyMembers,
    gate: SupportElevation,
    descriptor_digest: BoundaryDescriptorDigest,
    elements: Vec<BoundarySupportElement>,
}
impl TryFrom<BindingWire> for BoundaryOwnerBinding {
    type Error = SupportGeometryIssue;
    fn try_from(wire: BindingWire) -> Result<Self, Self::Error> {
        let binding = Self {
            property: wire.property,
            members: wire.members,
            gate: wire.gate,
            descriptor_digest: wire.descriptor_digest,
            elements: wire.elements,
        };
        binding.validate()?;
        Ok(binding)
    }
}
impl BoundaryOwnerBinding {
    pub(super) fn from_property(
        property: &CityCompound,
        gate: SupportElevation,
    ) -> Result<Self, BoundaryAdmissionError> {
        let members =
            PropertyMembers::new(vec![property.front_building_id, property.rear_building_id])
                .map_err(|_| BoundaryAdmissionError::Owner {
                    property: property.id,
                    front: property.front_building_id,
                })?;
        let elements = property
            .boundary
            .fixed_members()
            .map_err(|cause| BoundaryAdmissionError::Geometry {
                property: property.id,
                cause,
            })?
            .into_iter()
            .map(|member| member.pose.element())
            .collect();
        let binding = Self {
            property: property.id,
            members,
            gate,
            descriptor_digest: BoundaryDescriptorDigest::of(&property.boundary),
            elements,
        };
        binding
            .validate()
            .map_err(|issue| BoundaryAdmissionError::Support {
                property: property.id,
                issue,
            })?;
        Ok(binding)
    }
    fn validate(&self) -> Result<(), SupportGeometryIssue> {
        if self.property.0 == 0
            || self.members.ids().len() != 2
            || self.elements.is_empty()
            || self.elements.iter().enumerate().any(|(index, element)| {
                self.elements[..index].contains(element)
                    || matches!(
                        element,
                        BoundarySupportElement::Owner | BoundarySupportElement::GateLanding
                    )
            })
        {
            return Err(SupportGeometryIssue::Members);
        }
        Ok(())
    }
    pub fn property(&self) -> CityPropertyId {
        self.property
    }
    pub fn members(&self) -> &PropertyMembers {
        &self.members
    }
    pub fn gate(&self) -> SupportElevation {
        self.gate
    }
    pub fn validate_scene(
        &self,
        property: CityPropertyId,
        front: crate::scene_input::SceneBuildingId,
        boundary: &crate::city_layout::CityBoundary,
    ) -> Result<(), BoundaryAdmissionError> {
        if property != self.property || self.members.ids().first() != Some(&front) {
            return Err(BoundaryAdmissionError::Owner { property, front });
        }
        let elements: Vec<_> = boundary
            .fixed_members()
            .map_err(|cause| BoundaryAdmissionError::Geometry { property, cause })?
            .into_iter()
            .map(|member| member.pose.element())
            .collect();
        if elements != self.elements {
            return Err(BoundaryAdmissionError::Descriptor { property });
        }
        if BoundaryDescriptorDigest::of(boundary) != self.descriptor_digest {
            return Err(BoundaryAdmissionError::Descriptor { property });
        }
        Ok(())
    }
    pub(super) fn validate_cells(
        &self,
        cells: &[BoundarySupportCell],
    ) -> Result<(), SupportGeometryIssue> {
        if cells.len() > u32::MAX as usize / (6 * 8)
            || cells
                .iter()
                .any(|cell| !self.elements.contains(&cell.element))
            || self
                .elements
                .iter()
                .any(|element| !cells.iter().any(|cell| cell.element == *element))
        {
            return Err(SupportGeometryIssue::Topology);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, thiserror::Error)]
pub enum BoundaryAdmissionError {
    #[error(
        "property {property:?}, front building {front} disagrees with its fixed enclosure owner"
    )]
    Owner {
        property: CityPropertyId,
        front: crate::scene_input::SceneBuildingId,
    },
    #[error("property {property:?} enclosure descriptor disagrees with its fixed geometry")]
    Descriptor { property: CityPropertyId },
    #[error("property {property:?} gate datum {actual:?} differs from bound datum {expected:?}")]
    Datum {
        property: CityPropertyId,
        actual: SupportElevation,
        expected: SupportElevation,
    },
    #[error("property {property:?} descriptor geometry: {cause}")]
    Geometry {
        property: CityPropertyId,
        #[source]
        cause: crate::city_layout::BoundaryGeometryError,
    },
    #[error("property {property:?} fixed support: {issue}")]
    Support {
        property: CityPropertyId,
        #[source]
        issue: SupportGeometryIssue,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct BoundarySupportProjection {
    pub mesh: BoundarySupportMesh,
    pub gate_elevation: SupportElevation,
}
