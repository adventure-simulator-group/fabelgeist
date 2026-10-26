//! Cameras follow the accepted appliance and roof intersection.
use adventuresim_building_generator::{
    BuildingPlan, DomesticHeatingPlan, GenerationError, HeatingConstructionError, HeatingPartKind,
    HeatingPassageKind, StoreyIndex,
    spatial_geometry::{Architectural, Displacement, PlanDirection, Position, SpatialBounds},
};
use bevy::math::Vec3;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum HeatingTarget {
    Hearth,
    Stove,
    Roof,
    Support,
    Floor(u16),
}
impl HeatingTarget {
    pub(super) fn anchor(
        self,
        plan: &BuildingPlan,
    ) -> Result<Position<Architectural>, GenerationError> {
        let h = plan
            .domestic_heating
            .as_ref()
            .ok_or(HeatingConstructionError::MissingProgramme)?;
        let (offset, height) = match self {
            Self::Hearth => {
                let passage = h
                    .passages
                    .iter()
                    .find(|p| {
                        p.kind == adventuresim_building_generator::HeatingPassageKind::HearthMouth
                    })
                    .ok_or(HeatingConstructionError::MissingPassage {
                        kind: HeatingPassageKind::HearthMouth,
                    })?;
                let bounds = plan
                    .resolved_geometry
                    .voids
                    .iter()
                    .find(|v| v.id == passage.void)
                    .ok_or(HeatingConstructionError::MissingVoid { void: passage.void })?
                    .bounds;
                return Ok(bounds.centre()?);
            }
            Self::Support => {
                let id = h
                    .parts
                    .iter()
                    .find(|p| p.kind == HeatingPartKind::SupportPier)
                    .ok_or(HeatingConstructionError::MissingPart {
                        kind: HeatingPartKind::SupportPier,
                    })?
                    .solid;
                return Ok(plan
                    .resolved_geometry
                    .solids
                    .iter()
                    .find(|s| s.id == id)
                    .ok_or(HeatingConstructionError::MissingSolid { solid: id })?
                    .centre);
            }
            Self::Floor(level) => {
                let floor = h
                    .floors
                    .iter()
                    .find(|f| {
                        f.storey_level
                            == adventuresim_building_generator::StoreyIndex::from_serialized(level)
                    })
                    .ok_or(HeatingConstructionError::MissingFloor {
                        storey: StoreyIndex::new(usize::from(level)),
                    })?;
                let min = floor.core.min().metres();
                let max = floor.core.max().metres();
                return Ok(Position::from_metres(Vec3::new(
                    (min.x + max.x) * 0.5,
                    max.y,
                    (min.z + max.z) * 0.5,
                ))?);
            }
            Self::Stove => (-0.5, h.floor_height_metres.metres() + 0.9),
            Self::Roof => {
                let bounds = flue_bounds(plan, h)?;
                let min = bounds.min().metres();
                let max = bounds.max().metres();
                return Ok(Position::from_metres(Vec3::new(
                    (min.x + max.x) * 0.5,
                    max.y - 0.6,
                    (min.z + max.z) * 0.5,
                ))?);
            }
        };
        let p = h.centre_metres.metres() + h.kitchen_axis.vector() * offset;
        Ok(Position::from_metres(Vec3::new(p.x, height, p.y))?)
    }
    pub(super) fn offset(
        self,
        plan: &BuildingPlan,
        offset: Displacement<Architectural>,
    ) -> Result<Displacement<Architectural>, GenerationError> {
        let heating = plan
            .domestic_heating
            .as_ref()
            .ok_or(HeatingConstructionError::MissingProgramme)?;
        let axis = match self {
            Self::Roof => {
                let face = plan
                    .roof_assemblies
                    .iter()
                    .flat_map(|r| &r.faces)
                    .find(|f| f.id == heating.roof.face)
                    .ok_or(HeatingConstructionError::MissingFace {
                        face: heating.roof.face,
                    })?;
                PlanDirection::<Architectural>::from_vector(bevy::math::Vec2::new(
                    face.plane.normal.x,
                    face.plane.normal.z,
                ))?
                .vector()
            }
            Self::Hearth | Self::Stove | Self::Support | Self::Floor(_) => {
                heating.kitchen_axis.vector()
            }
        };
        let offset = offset.metres();
        let tangent = bevy::math::Vec2::new(-axis.y, axis.x);
        let p = tangent * offset.x + axis * offset.z;
        Ok(Displacement::from_metres(Vec3::new(p.x, offset.y, p.y))?)
    }
}

fn flue_bounds(
    plan: &BuildingPlan,
    heating: &DomesticHeatingPlan,
) -> Result<SpatialBounds<Architectural>, GenerationError> {
    let mut bounds: Option<SpatialBounds<Architectural>> = None;
    for part in heating
        .parts
        .iter()
        .filter(|part| part.kind == HeatingPartKind::Flue)
    {
        let solid = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|solid| solid.id == part.solid)
            .ok_or(HeatingConstructionError::MissingSolid { solid: part.solid })?;
        let next = solid.cuboid_bounds()?;
        bounds = Some(bounds.map_or(next, |bounds| bounds.union(next)));
    }
    Ok(bounds.ok_or(HeatingConstructionError::MissingPart {
        kind: HeatingPartKind::Flue,
    })?)
}
