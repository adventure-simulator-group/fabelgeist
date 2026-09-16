//! Select a ground-floor kitchen/Stube bay against the complete structure.
use crate::{BuildingPlan, ResolvedBounds, RoofFace, RoomKind, SolidRole, WallAssemblyId};
use bevy::math::{Vec2, Vec3};
use geo::{Area, BooleanOps, Intersects};

pub(super) const FIRE_WALL_PATCH_HEIGHT_METRES: f32 = 2.0;
pub(super) const CORE_WIDTH_METRES: f32 = 0.96;
pub(super) const CORE_HALF_DEPTH_METRES: f32 = 0.9;
pub(super) const TIMBER_CLEARANCE_METRES: f32 = 0.08;
const STATION_STEP_METRES: f32 = 0.05;
pub(super) const SHAFT_HALF_WIDTH_METRES: f32 = 0.3;
#[derive(Clone, Copy, Debug)]
pub(super) enum HearthSection {
    Compact,
    Deep,
}
impl HearthSection {
    pub fn shaft_offset(self) -> f32 {
        match self {
            Self::Compact => 0.55,
            Self::Deep => 1.2,
        }
    }
    pub fn front(self) -> f32 {
        match self {
            Self::Compact => CORE_HALF_DEPTH_METRES,
            Self::Deep => 1.55,
        }
    }
}
const OPERATING_DEPTH_METRES: f32 = 0.75;
const PARTITION_END_RESERVE_METRES: f32 = 0.16;

#[derive(Clone, Copy, Debug)]
pub(super) struct Placement {
    pub wall: WallAssemblyId,
    pub section: HearthSection,
    pub kitchen: u16,
    pub stube: u16,
    pub centre: Vec2,
    pub kitchen_axis: Vec2,
    pub roof: crate::RoofAssemblyId,
    pub face: crate::ResolvedItemId,
}
impl Placement {
    pub fn bounds(self, min: Vec3, max: Vec3) -> ResolvedBounds {
        let tangent = Vec2::new(-self.kitchen_axis.y, self.kitchen_axis.x);
        let point = |v: Vec3| {
            let p = self.centre + tangent * v.x + self.kitchen_axis * v.z;
            Vec3::new(p.x, v.y, p.y)
        };
        let a = point(min);
        let b = point(max);
        ResolvedBounds {
            min: a.min(b),
            max: a.max(b),
        }
    }
    pub fn body(self) -> ResolvedBounds {
        self.bounds(
            Vec3::new(-CORE_WIDTH_METRES * 0.5, 0.0, -CORE_HALF_DEPTH_METRES),
            Vec3::new(CORE_WIDTH_METRES * 0.5, 2.2, self.section.front()),
        )
    }
    pub fn operating_space(self) -> ResolvedBounds {
        self.bounds(
            Vec3::new(-CORE_WIDTH_METRES * 0.5, 0.02, self.section.front()),
            Vec3::new(
                CORE_WIDTH_METRES * 0.5,
                2.0,
                self.section.front() + OPERATING_DEPTH_METRES,
            ),
        )
    }
    pub fn shaft(self, top: f32) -> ResolvedBounds {
        let centre = self.centre + self.kitchen_axis * self.section.shaft_offset();
        ResolvedBounds {
            min: Vec3::new(
                centre.x - SHAFT_HALF_WIDTH_METRES,
                2.1,
                centre.y - SHAFT_HALF_WIDTH_METRES,
            ),
            max: Vec3::new(
                centre.x + SHAFT_HALF_WIDTH_METRES,
                top,
                centre.y + SHAFT_HALF_WIDTH_METRES,
            ),
        }
    }
    pub fn flue_top(self, face: &RoofFace) -> f32 {
        const OUTLET_ABOVE_UPSLOPE_ROOF_METRES: f32 = 0.8;
        let shaft = self.shaft(0.0);
        [shaft.min.x, shaft.max.x]
            .into_iter()
            .flat_map(|x| [shaft.min.z, shaft.max.z].map(|z| Vec2::new(x, z)))
            .map(|point| roof_height(face, point))
            .fold(0.0_f32, f32::max)
            + OUTLET_ABOVE_UPSLOPE_ROOF_METRES
    }
    fn clear(self, plan: &BuildingPlan, bounds: ResolvedBounds, replace_wall: bool) -> bool {
        let wall = plan
            .wall_assemblies
            .iter()
            .find(|w| w.id == self.wall)
            .unwrap();
        let margin = Vec3::new(TIMBER_CLEARANCE_METRES, 0.0, TIMBER_CLEARANCE_METRES);
        !plan.resolved_geometry.solids.iter().any(|solid| {
            !(replace_wall && wall.host_solids.contains(&solid.id))
                && !matches!(solid.role, SolidRole::FrameFloor | SolidRole::InteriorFloor)
                && crate::solid_overlap::overlaps_bounds(
                    solid,
                    (bounds.min - margin, bounds.max + margin),
                    0.001,
                )
        })
    }
    fn clear_doors(self, plan: &BuildingPlan) -> bool {
        let body = self.body();
        plan.opening_assemblies
            .iter()
            .filter(|o| {
                matches!(
                    o.use_kind,
                    crate::OpeningUse::Door | crate::OpeningUse::Gate
                )
            })
            .all(|o| {
                let width = o.profile.interior_width_metres();
                let half = o.frame.tangent.abs() * (width * 0.5 + 0.3)
                    + o.frame.outward.abs() * (width + 0.3);
                let min = o.frame.origin - half;
                let max = o.frame.origin + half;
                body.max.x <= min.x
                    || body.min.x >= max.x
                    || body.max.z <= min.y
                    || body.min.z >= max.y
            })
    }
}

pub(super) fn find(plan: &BuildingPlan) -> Option<Placement> {
    if plan.storeys.len() != 1 || !plan.stairs.is_empty() {
        return None;
    }
    let storey = plan.storeys.first()?;
    for wall in &plan.wall_assemblies {
        let (Some(inside), Some(outside)) = (wall.frame.inside_room, wall.frame.outside_room)
        else {
            continue;
        };
        let a = storey.rooms.iter().find(|r| r.id == inside)?;
        let b = storey.rooms.iter().find(|r| r.id == outside)?;
        let (kitchen, stube, axis) = match (a.kind, b.kind) {
            (RoomKind::Kitchen, RoomKind::CommonRoom | RoomKind::GreatHall) => {
                (a.id, b.id, -wall.frame.outward)
            }
            (RoomKind::CommonRoom | RoomKind::GreatHall, RoomKind::Kitchen) => {
                (b.id, a.id, wall.frame.outward)
            }
            _ => continue,
        };
        if !wall.opening_ids.is_empty() || wall.base_elevation_metres.abs() > 0.01 {
            continue;
        }
        let limit = (wall.length_metres - CORE_WIDTH_METRES) * 0.5 - PARTITION_END_RESERVE_METRES;
        if limit < 0.0 {
            continue;
        }
        for (section, step) in [HearthSection::Compact, HearthSection::Deep]
            .into_iter()
            .flat_map(|section| {
                (0..=((2.0 * limit / STATION_STEP_METRES).floor() as usize))
                    .map(move |step| (section, step))
            })
        {
            let centre = wall.frame.origin
                + wall.frame.tangent * (-limit + step as f32 * STATION_STEP_METRES);
            let mut candidate = Placement {
                wall: wall.id,
                section,
                kitchen,
                stube,
                centre,
                kitchen_axis: axis,
                roof: crate::RoofAssemblyId(0),
                face: crate::ResolvedItemId(0),
            };
            let probe = centre + axis * section.shaft_offset();
            let Some((roof, face)) = plan
                .roof_assemblies
                .iter()
                .filter(|r| r.parent.is_none())
                .find_map(|r| {
                    r.faces
                        .iter()
                        .find(|face| roof_fits(face, probe))
                        .map(|f| (r.id, f))
                })
            else {
                continue;
            };
            candidate.roof = roof;
            candidate.face = face.id;
            let top = candidate.flue_top(face);
            if candidate.clear(plan, candidate.body(), true)
                && candidate.clear(plan, candidate.shaft(top), false)
                && candidate.clear(plan, candidate.operating_space(), false)
                && candidate.clear_doors(plan)
            {
                return Some(candidate);
            }
        }
    }
    None
}
pub(super) fn roof_height(face: &RoofFace, point: Vec2) -> f32 {
    -(face.plane.normal.x * point.x + face.plane.normal.z * point.y + face.plane.constant)
        / face.plane.normal.y
}
fn roof_fits(face: &RoofFace, point: Vec2) -> bool {
    let polygon = |points: &[Vec3]| {
        geo::Polygon::new(
            geo::LineString::new(
                points
                    .iter()
                    .map(|v| geo::Coord { x: v.x, y: v.z })
                    .collect(),
            ),
            vec![],
        )
    };
    let footprint = super::roof::PenetrationFootprint::new(face, point);
    let rect = geo::Rect::new(
        geo::coord! {x:footprint.weather_min.x,y:footprint.weather_min.y},
        geo::coord! {x:footprint.weather_max.x,y:footprint.weather_max.y},
    )
    .to_polygon();
    rect.difference(&polygon(&face.polygon)).unsigned_area() < 0.00001
        && !face
            .cutouts
            .iter()
            .any(|cut| rect.intersects(&polygon(cut)))
}
