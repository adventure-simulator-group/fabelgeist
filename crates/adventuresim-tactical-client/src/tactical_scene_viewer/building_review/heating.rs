//! Cameras follow the accepted appliance and roof intersection.
use adventuresim_building_generator::{BuildingPlan, HeatingPartKind};
use bevy::math::Vec3;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum HeatingTarget {
    Hearth,
    Stove,
    Roof,
}
impl HeatingTarget {
    pub(super) fn anchor(self, plan: &BuildingPlan) -> Vec3 {
        let h = plan
            .domestic_heating
            .as_ref()
            .expect("heating review requires a resolved programme");
        let (offset, height) = match self {
            Self::Hearth => {
                let passage = h
                    .passages
                    .iter()
                    .find(|p| {
                        p.kind == adventuresim_building_generator::HeatingPassageKind::HearthMouth
                    })
                    .unwrap();
                let bounds = plan
                    .resolved_geometry
                    .voids
                    .iter()
                    .find(|v| v.id == passage.void)
                    .unwrap()
                    .bounds;
                return (bounds.min + bounds.max) * 0.5;
            }
            Self::Stove => (-0.5, 0.9),
            Self::Roof => {
                let bounds = h
                    .parts
                    .iter()
                    .filter(|p| p.kind == HeatingPartKind::Flue)
                    .filter_map(|p| {
                        plan.resolved_geometry
                            .solids
                            .iter()
                            .find(|s| s.id == p.solid)
                    })
                    .map(|s| s.cuboid_bounds())
                    .reduce(|a, b| adventuresim_building_generator::ResolvedBounds {
                        min: a.min.min(b.min),
                        max: a.max.max(b.max),
                    })
                    .expect("heating flue");
                return Vec3::new(
                    (bounds.min.x + bounds.max.x) * 0.5,
                    bounds.max.y - 0.6,
                    (bounds.min.z + bounds.max.z) * 0.5,
                );
            }
        };
        let p = h.centre_metres + h.kitchen_axis * offset;
        Vec3::new(p.x, height, p.y)
    }
    pub(super) fn offset(self, plan: &BuildingPlan, offset: Vec3) -> Vec3 {
        let heating = plan.domestic_heating.as_ref().unwrap();
        let axis = match self {
            Self::Roof => {
                let face = plan
                    .roof_assemblies
                    .iter()
                    .flat_map(|r| &r.faces)
                    .find(|f| f.id == heating.roof.face)
                    .unwrap();
                bevy::math::Vec2::new(face.plane.normal.x, face.plane.normal.z).normalize()
            }
            Self::Hearth | Self::Stove => heating.kitchen_axis,
        };
        let tangent = bevy::math::Vec2::new(-axis.y, axis.x);
        let p = tangent * offset.x + axis * offset.z;
        Vec3::new(p.x, offset.y, p.y)
    }
}
