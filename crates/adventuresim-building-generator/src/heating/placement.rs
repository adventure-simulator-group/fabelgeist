//! Select a kitchen/Stube bay against the complete structure.
use crate::{Architectural, SpatialBounds};
use crate::{BuildingPlan, RoofFace, RoomKind, SolidRole, WallAssemblyId};
use bevy::math::{Vec2, Vec3};

pub(super) const FIRE_WALL_PATCH_HEIGHT_METRES: f32 = 2.0;
pub(super) const CORE_WIDTH_METRES: f32 = 0.96;
pub(super) const CORE_HALF_DEPTH_METRES: f32 = 0.9;
pub(super) const TIMBER_CLEARANCE_METRES: f32 = 0.08;
const STATION_STEP_METRES: f32 = 0.05;
pub(super) const SHAFT_HALF_WIDTH_METRES: f32 = 0.3;
const SHAFT_BASE_ABOVE_FLOOR_METRES: f32 = 2.1;
#[derive(Clone, Copy, Debug)]
pub(super) enum HearthSection {
    Compact,
    Deep,
    Extended,
}
impl HearthSection {
    pub fn shaft_offset(self) -> f32 {
        match self {
            Self::Compact => 0.55,
            Self::Deep => 1.2,
            Self::Extended => 1.5,
        }
    }
    pub fn front(self) -> f32 {
        match self {
            Self::Compact => CORE_HALF_DEPTH_METRES,
            Self::Deep => 1.55,
            Self::Extended => 1.85,
        }
    }
}
const OPERATING_DEPTH_METRES: f32 = 0.75;
const PARTITION_END_RESERVE_METRES: f32 = 0.16;

#[derive(Clone, Copy, Debug)]
pub(super) struct Placement {
    pub wall: WallAssemblyId,
    pub storey_level: u16,
    pub floor_height: f32,
    pub next_floor: Option<f32>,
    pub section: HearthSection,
    pub kitchen: u16,
    pub stube: u16,
    pub centre: Vec2,
    pub kitchen_axis: Vec2,
    pub roof: crate::RoofAssemblyId,
    pub face: crate::ResolvedItemId,
}
impl Placement {
    pub fn bounds(
        self,
        min: Vec3,
        max: Vec3,
    ) -> Result<SpatialBounds<Architectural>, crate::GenerationError> {
        let tangent = Vec2::new(-self.kitchen_axis.y, self.kitchen_axis.x);
        let point = |v: Vec3| {
            let p = self.centre + tangent * v.x + self.kitchen_axis * v.z;
            Vec3::new(p.x, v.y + self.floor_height, p.y)
        };
        let a = point(min);
        let b = point(max);
        Ok(SpatialBounds::<Architectural>::from_metres(
            a.min(b),
            a.max(b),
        )?)
    }
    pub fn support(self) -> Result<SpatialBounds<Architectural>, crate::GenerationError> {
        let bounds = self.body()?;
        let ledge = Vec3::new(
            super::floors::MASONRY_BEARING_METRES,
            0.0,
            super::floors::MASONRY_BEARING_METRES,
        );
        let mut min = bounds.min().metres() - ledge;
        let mut max = bounds.max().metres() + ledge;
        min.y = 0.0;
        max.y = self.floor_height;
        Ok(SpatialBounds::from_metres(min, max)?)
    }
    pub fn shaft_shoulder(
        self,
    ) -> Result<Option<SpatialBounds<Architectural>>, crate::GenerationError> {
        let Some(top) = self.next_floor else {
            return Ok(None);
        };
        let bounds = self.shaft(top)?;
        let ledge = Vec3::new(
            super::floors::MASONRY_BEARING_METRES,
            0.0,
            super::floors::MASONRY_BEARING_METRES,
        );
        let mut min = bounds.min().metres() - ledge;
        let max = bounds.max().metres() + ledge;
        min.y = self.floor_height + 2.2;
        Ok(Some(SpatialBounds::from_metres(min, max)?))
    }
    pub fn body(self) -> Result<SpatialBounds<Architectural>, crate::GenerationError> {
        self.bounds(
            Vec3::new(-CORE_WIDTH_METRES * 0.5, 0.0, -CORE_HALF_DEPTH_METRES),
            Vec3::new(CORE_WIDTH_METRES * 0.5, 2.2, self.section.front()),
        )
    }
    pub fn operating_space(self) -> Result<SpatialBounds<Architectural>, crate::GenerationError> {
        self.bounds(
            Vec3::new(-CORE_WIDTH_METRES * 0.5, 0.02, self.section.front()),
            Vec3::new(
                CORE_WIDTH_METRES * 0.5,
                2.0,
                self.section.front() + OPERATING_DEPTH_METRES,
            ),
        )
    }
    pub fn shaft(self, top: f32) -> Result<SpatialBounds<Architectural>, crate::GenerationError> {
        let centre = self.centre + self.kitchen_axis * self.section.shaft_offset();
        Ok(SpatialBounds::<Architectural>::from_metres(
            Vec3::new(
                centre.x - SHAFT_HALF_WIDTH_METRES,
                self.floor_height + SHAFT_BASE_ABOVE_FLOOR_METRES,
                centre.y - SHAFT_HALF_WIDTH_METRES,
            ),
            Vec3::new(
                centre.x + SHAFT_HALF_WIDTH_METRES,
                top,
                centre.y + SHAFT_HALF_WIDTH_METRES,
            ),
        )?)
    }
    /// The horizontal shaft footprint is an admitted zero-height section at
    /// its lower datum; it never constructs a reversed dummy vertical range.
    pub fn shaft_section(self) -> Result<SpatialBounds<Architectural>, crate::GenerationError> {
        self.shaft(self.floor_height + SHAFT_BASE_ABOVE_FLOOR_METRES)
    }
    pub fn flue_top(self, face: &RoofFace) -> Result<f32, crate::GenerationError> {
        const OUTLET_ABOVE_UPSLOPE_ROOF_METRES: f32 = 0.8;
        // Only the horizontal section is needed; use an admitted zero-height
        // section at the shaft base rather than a reversed vertical interval.
        let shaft = self.shaft_section()?;
        Ok([shaft.min().metres().x, shaft.max().metres().x]
            .into_iter()
            .flat_map(|x| [shaft.min().metres().z, shaft.max().metres().z].map(|z| Vec2::new(x, z)))
            .map(|point| roof_height(face, point))
            .fold(0.0_f32, f32::max)
            + OUTLET_ABOVE_UPSLOPE_ROOF_METRES)
    }
    fn clear(
        self,
        plan: &BuildingPlan,
        bounds: SpatialBounds<Architectural>,
        replace_wall: bool,
    ) -> Result<bool, crate::GenerationError> {
        let wall = plan
            .wall_assemblies
            .iter()
            .find(|w| w.id == self.wall)
            .ok_or(crate::HeatingConstructionError::MissingWall { wall: self.wall })?;
        let margin = Vec3::new(TIMBER_CLEARANCE_METRES, 0.0, TIMBER_CLEARANCE_METRES);
        Ok(!plan.resolved_geometry.solids.iter().any(|solid| {
            !(replace_wall && wall.host_solids.contains(&solid.id))
                && !matches!(solid.role, SolidRole::FrameFloor | SolidRole::InteriorFloor)
                && crate::solid_overlap::overlaps_bounds(
                    solid,
                    (
                        bounds.min().metres() - margin,
                        bounds.max().metres() + margin,
                    ),
                    0.001,
                )
        }))
    }
    fn clear_doors(self, plan: &BuildingPlan) -> Result<bool, crate::GenerationError> {
        let body = self.body()?;
        Ok(plan
            .opening_assemblies
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
                body.max().metres().x <= min.x
                    || body.min().metres().x >= max.x
                    || body.max().metres().z <= min.y
                    || body.min().metres().z >= max.y
            }))
    }
}

pub(super) fn find(plan: &BuildingPlan) -> Result<Option<Placement>, crate::GenerationError> {
    for wall in &plan.wall_assemblies {
        let Some(storey) = plan.storeys.iter().find(|s| s.level == wall.storey_level) else {
            continue;
        };
        let (Some(inside), Some(outside)) = (wall.frame.inside_room, wall.frame.outside_room)
        else {
            continue;
        };
        let Some(a) = storey.rooms.iter().find(|r| r.id == inside) else {
            continue;
        };
        let Some(b) = storey.rooms.iter().find(|r| r.id == outside) else {
            continue;
        };
        let (kitchen, stube, axis) = match (a.kind, b.kind) {
            (RoomKind::Kitchen, RoomKind::CommonRoom | RoomKind::GreatHall) => {
                (a.id, b.id, -wall.frame.outward)
            }
            (RoomKind::CommonRoom | RoomKind::GreatHall, RoomKind::Kitchen) => {
                (b.id, a.id, wall.frame.outward)
            }
            _ => continue,
        };
        if !wall.opening_ids.is_empty() {
            continue;
        }
        let limit = (wall.length_metres - CORE_WIDTH_METRES) * 0.5 - PARTITION_END_RESERVE_METRES;
        if limit < 0.0 {
            continue;
        }
        let stations = station_offsets(plan, wall, limit)?;
        for (section, station) in [
            HearthSection::Compact,
            HearthSection::Deep,
            HearthSection::Extended,
        ]
        .into_iter()
        .flat_map(|section| {
            stations
                .iter()
                .copied()
                .map(move |station| (section, station))
        }) {
            let centre = wall.frame.origin + wall.frame.tangent * station;
            let mut candidate = Placement {
                wall: wall.id,
                storey_level: storey.level,
                floor_height: f32::from(storey.level) * plan.storey_height_metres,
                next_floor: plan
                    .storeys
                    .iter()
                    .filter(|s| s.level > storey.level)
                    .map(|s| f32::from(s.level) * plan.storey_height_metres)
                    .min_by(f32::total_cmp),
                section,
                kitchen,
                stube,
                centre,
                kitchen_axis: axis,
                roof: crate::RoofAssemblyId(0),
                face: crate::ResolvedItemId(0),
            };
            let Some((roof, face)) = super::roof_route::find(plan, candidate)? else {
                continue;
            };
            candidate.roof = roof;
            candidate.face = face.id;
            let top = candidate.flue_top(face)?;
            if candidate.clear(plan, candidate.body()?, true)?
                && candidate.clear(plan, candidate.shaft(top)?, false)?
                && candidate.clear(plan, candidate.operating_space()?, false)?
                && candidate.clear_doors(plan)?
                && (candidate.storey_level == 0
                    || candidate.clear(plan, candidate.support()?, false)?)
                && crate::geometry_index::try_all(candidate.shaft_shoulder()?, |b| {
                    candidate.clear(plan, b, false)
                })?
                && super::floors::supported(plan, candidate)?
                && super::roof_route::weather_clear(plan, candidate, face)?
            {
                return Ok(Some(candidate));
            }
        }
    }
    Ok(None)
}

pub(super) fn roof_height(face: &RoofFace, point: Vec2) -> f32 {
    -(face.plane.normal.x * point.x + face.plane.normal.z * point.y + face.plane.constant)
        / face.plane.normal.y
}

/// Include actual joist-bay centres; a fixed sampling step can miss a narrow valid bay.
fn station_offsets(
    plan: &BuildingPlan,
    wall: &crate::WallAssembly,
    limit: f32,
) -> Result<Vec<f32>, crate::GenerationError> {
    let mut stations = (0..=((2.0 * limit / STATION_STEP_METRES).floor() as usize))
        .map(|step| -limit + step as f32 * STATION_STEP_METRES)
        .collect::<Vec<_>>();
    if wall.storey_level > 0 && wall.frame.tangent.x.abs() > 0.5 {
        let mut joists = plan
            .resolved_geometry
            .solids
            .iter()
            .filter(|s| s.role == SolidRole::FrameJoist)
            .map(|s| s.cuboid_bounds())
            .collect::<Result<Vec<_>, _>>()?;
        joists.sort_by(|a, b| a.min().metres().x.total_cmp(&b.min().metres().x));
        joists.dedup_by(|a, b| (a.min().metres().x - b.min().metres().x).abs() < 0.001);
        stations.extend(
            joists
                .windows(2)
                .map(|pair| {
                    ((pair[0].max().metres().x + pair[1].min().metres().x) * 0.5
                        - wall.frame.origin.x)
                        / wall.frame.tangent.x
                })
                .filter(|station| station.abs() <= limit),
        );
    }
    Ok(stations)
}
