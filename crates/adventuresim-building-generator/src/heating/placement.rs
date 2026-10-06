//! Select a kitchen/Stube bay against the complete structure.
use crate::GenerationResult;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::GeometryResult;
use crate::spatial_geometry::{Displacement, Elevation, PlanDirection, SignedLength};
use crate::{Architectural, RoomIndex, SpatialBounds, StoreyIndex};

/// Appliance Z points into the kitchen; Y is height above its floor.
/// X maps to `(-axis.y, axis.x)` in architectural X/Z, where `axis` is the
/// kitchen direction's plan vector.
#[derive(Clone, Copy, Debug, PartialEq, bevy::reflect::Reflect)]
pub(super) enum HearthLocal {}
impl crate::spatial_geometry::GeometryFrame for HearthLocal {}

#[derive(Clone, Copy, Debug)]
pub(super) struct Placement {
    pub site: PlacementCandidate,
    pub roof: crate::RoofAssemblyId,
    pub face: crate::ResolvedItemId,
}

use crate::{BuildingPlan, RoofFace, RoomKind, SolidRole, WallAssemblyId};
use bevy::math::{Vec2, Vec3};

pub(super) const FIRE_WALL_PATCH_HEIGHT_METRES: f32 = 2.0;
pub(super) const CORE_WIDTH_METRES: f32 = 0.96;
pub(super) const CORE_HALF_WIDTH_METRES: f32 = CORE_WIDTH_METRES * 0.5;
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
    pub fn shaft_offset(self) -> GeometryResult<SignedLength> {
        SignedLength::from_metres(match self {
            Self::Compact => 0.55,
            Self::Deep => 1.2,
            Self::Extended => 1.5,
        })
    }
    pub fn front(self) -> GeometryResult<SignedLength> {
        SignedLength::from_metres(match self {
            Self::Compact => CORE_HALF_DEPTH_METRES,
            Self::Deep => 1.55,
            Self::Extended => 1.85,
        })
    }
}
const OPERATING_DEPTH_METRES: f32 = 0.75;
const PARTITION_END_RESERVE_METRES: f32 = 0.16;

#[derive(Clone, Copy, Debug)]
pub(super) struct PlacementCandidate {
    pub wall: WallAssemblyId,
    pub storey_level: StoreyIndex,
    pub floor_height: Elevation<Architectural>,
    pub next_floor: Option<Elevation<Architectural>>,
    pub section: HearthSection,
    pub kitchen: RoomIndex,
    pub stube: RoomIndex,
    pub centre: ArchitecturalPlanPoint,
    pub kitchen_axis: PlanDirection<Architectural>,
}
impl PlacementCandidate {
    pub fn bounds(
        self,
        min: Displacement<HearthLocal>,
        max: Displacement<HearthLocal>,
    ) -> GenerationResult<SpatialBounds<Architectural>> {
        SpatialBounds::<HearthLocal>::from_metres(min.metres(), max.metres())?;
        let axis = self.kitchen_axis.vector();
        let tangent = Vec2::new(-axis.y, axis.x);
        let point = |v: Displacement<HearthLocal>| {
            let v = v.metres();
            let p = self.centre.metres() + tangent * v.x + axis * v.z;
            Vec3::new(p.x, v.y + self.floor_height.metres(), p.y)
        };
        let a = point(min);
        let b = point(max);
        Ok(SpatialBounds::<Architectural>::from_metres(
            a.min(b),
            a.max(b),
        )?)
    }
    pub fn support(self) -> GenerationResult<SpatialBounds<Architectural>> {
        let bounds = self.body()?;
        let ledge = Vec3::new(
            super::floors::MASONRY_BEARING_METRES,
            0.0,
            super::floors::MASONRY_BEARING_METRES,
        );
        let mut min = bounds.min().metres() - ledge;
        let mut max = bounds.max().metres() + ledge;
        min.y = 0.0;
        max.y = self.floor_height.metres();
        Ok(SpatialBounds::from_metres(min, max)?)
    }
    pub fn shaft_shoulder(self) -> GenerationResult<Option<SpatialBounds<Architectural>>> {
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
        min.y = self.floor_height.metres() + 2.2;
        Ok(Some(SpatialBounds::from_metres(min, max)?))
    }
    pub fn body(self) -> GenerationResult<SpatialBounds<Architectural>> {
        self.bounds(
            Displacement::from_metres(Vec3::new(
                -CORE_WIDTH_METRES * 0.5,
                0.0,
                -CORE_HALF_DEPTH_METRES,
            ))?,
            Displacement::from_metres(Vec3::new(
                CORE_WIDTH_METRES * 0.5,
                2.2,
                self.section.front()?.metres(),
            ))?,
        )
    }
    pub fn operating_space(self) -> GenerationResult<SpatialBounds<Architectural>> {
        self.bounds(
            Displacement::from_metres(Vec3::new(
                -CORE_WIDTH_METRES * 0.5,
                0.02,
                self.section.front()?.metres(),
            ))?,
            Displacement::from_metres(Vec3::new(
                CORE_WIDTH_METRES * 0.5,
                2.0,
                self.section.front()?.metres() + OPERATING_DEPTH_METRES,
            ))?,
        )
    }
    pub fn shaft(
        self,
        top: Elevation<Architectural>,
    ) -> GenerationResult<SpatialBounds<Architectural>> {
        let centre = self.centre.metres()
            + self.kitchen_axis.vector() * self.section.shaft_offset()?.metres();
        Ok(SpatialBounds::<Architectural>::from_metres(
            Vec3::new(
                centre.x - SHAFT_HALF_WIDTH_METRES,
                self.floor_height.metres() + SHAFT_BASE_ABOVE_FLOOR_METRES,
                centre.y - SHAFT_HALF_WIDTH_METRES,
            ),
            Vec3::new(
                centre.x + SHAFT_HALF_WIDTH_METRES,
                top.metres(),
                centre.y + SHAFT_HALF_WIDTH_METRES,
            ),
        )?)
    }
    /// The horizontal shaft footprint is an admitted zero-height section at
    /// its lower datum; it never constructs a reversed dummy vertical range.
    pub fn shaft_section(self) -> GenerationResult<SpatialBounds<Architectural>> {
        self.shaft(Elevation::from_metres(
            self.floor_height.metres() + SHAFT_BASE_ABOVE_FLOOR_METRES,
        )?)
    }
    pub fn flue_top(self, face: &RoofFace) -> GenerationResult<Elevation<Architectural>> {
        const OUTLET_ABOVE_UPSLOPE_ROOF_METRES: f32 = 0.8;
        // Only the horizontal section is needed; use an admitted zero-height
        // section at the shaft base rather than a reversed vertical interval.
        let shaft = self.shaft_section()?;
        Ok(Elevation::from_metres(
            [shaft.min().metres().x, shaft.max().metres().x]
                .into_iter()
                .flat_map(|x| {
                    [shaft.min().metres().z, shaft.max().metres().z].map(|z| Vec2::new(x, z))
                })
                .try_fold(0.0_f32, |high, point| {
                    Ok::<_, crate::GenerationError>(high.max(
                        roof_height(face, ArchitecturalPlanPoint::from_metres(point)?)?.metres(),
                    ))
                })?
                + OUTLET_ABOVE_UPSLOPE_ROOF_METRES,
        )?)
    }
    fn clear(
        self,
        plan: &BuildingPlan,
        bounds: SpatialBounds<Architectural>,
        replace_wall: bool,
    ) -> GenerationResult<bool> {
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
    fn clear_doors(self, plan: &BuildingPlan) -> GenerationResult<bool> {
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

pub(super) fn find(plan: &BuildingPlan) -> GenerationResult<Option<Placement>> {
    for wall in &plan.wall_assemblies {
        let (Some(inside), Some(outside)) = (wall.frame.inside_room, wall.frame.outside_room)
        else {
            continue;
        };
        let Some(pair) = kitchen_pair(plan, wall, inside, outside)? else {
            continue;
        };
        let KitchenPair {
            storey,
            kitchen,
            stube,
            axis,
        } = pair;
        if !wall.opening_ids.is_empty() {
            continue;
        }
        let limit = (wall.length_metres - CORE_WIDTH_METRES) * 0.5 - PARTITION_END_RESERVE_METRES;
        if limit < 0.0 {
            continue;
        }
        let stations = station_offsets(plan, wall, SignedLength::from_metres(limit)?)?;
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
            let centre = wall.frame.origin + wall.frame.tangent * station.metres();
            let site = PlacementCandidate {
                wall: wall.id,
                storey_level: StoreyIndex::from_serialized(storey.level),
                floor_height: Elevation::from_metres(
                    f32::from(storey.level) * plan.storey_height_metres,
                )?,
                next_floor: plan
                    .storeys
                    .iter()
                    .filter(|s| s.level > storey.level)
                    .map(|s| f32::from(s.level) * plan.storey_height_metres)
                    .min_by(f32::total_cmp)
                    .map(Elevation::from_metres)
                    .transpose()?,
                section,
                kitchen,
                stube,
                centre: ArchitecturalPlanPoint::from_metres(centre)?,
                kitchen_axis: axis,
            };
            let Some(selected) = super::roof_route::find(plan, site)? else {
                continue;
            };
            let face = selected.face;
            let candidate = Placement {
                site,
                roof: selected.roof,
                face: face.id,
            };
            let top = candidate.site.flue_top(face)?;
            if candidate.site.clear(plan, candidate.site.body()?, true)?
                && candidate
                    .site
                    .clear(plan, candidate.site.shaft(top)?, false)?
                && candidate
                    .site
                    .clear(plan, candidate.site.operating_space()?, false)?
                && candidate.site.clear_doors(plan)?
                && (candidate.site.storey_level == StoreyIndex::GROUND
                    || candidate
                        .site
                        .clear(plan, candidate.site.support()?, false)?)
                && crate::geometry_index::try_all(candidate.site.shaft_shoulder()?, |b| {
                    candidate.site.clear(plan, b, false)
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

pub(super) fn roof_height(
    face: &RoofFace,
    point: ArchitecturalPlanPoint,
) -> GenerationResult<Elevation<Architectural>> {
    let point = point.metres();
    Ok(Elevation::from_metres(
        -(face.plane.normal.x * point.x + face.plane.normal.z * point.y + face.plane.constant)
            / face.plane.normal.y,
    )?)
}

/// Include actual joist-bay centres; a fixed sampling step can miss a narrow valid bay.
fn station_offsets(
    plan: &BuildingPlan,
    wall: &crate::WallAssembly,
    limit: SignedLength,
) -> GenerationResult<Vec<SignedLength>> {
    let limit = limit.metres();
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
            .collect::<GenerationResult<Vec<_>>>()?;
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
    Ok(stations
        .into_iter()
        .map(SignedLength::from_metres)
        .collect::<GeometryResult<_>>()?)
}

struct KitchenPair<'a> {
    storey: &'a crate::StoreyPlan,
    kitchen: RoomIndex,
    stube: RoomIndex,
    axis: PlanDirection<Architectural>,
}
fn kitchen_pair<'a>(
    plan: &'a BuildingPlan,
    wall: &crate::WallAssembly,
    inside: u16,
    outside: u16,
) -> GenerationResult<Option<KitchenPair<'a>>> {
    let level = StoreyIndex::from_serialized(wall.storey_level);
    let storey = plan
        .storeys
        .iter()
        .find(|s| s.level == wall.storey_level)
        .ok_or(crate::HeatingConstructionError::MissingStorey {
            wall: wall.id,
            storey: level,
        })?;
    let room = |id| {
        storey.rooms.iter().find(|r| r.id == id).ok_or(
            crate::HeatingConstructionError::MissingRoom {
                wall: wall.id,
                storey: level,
                room: RoomIndex::from_serialized(id),
            },
        )
    };
    let a = room(inside)?;
    let b = room(outside)?;
    let (kitchen, stube, axis) = match (a.kind, b.kind) {
        (RoomKind::Kitchen, RoomKind::CommonRoom | RoomKind::GreatHall) => {
            (a.id, b.id, -wall.frame.outward)
        }
        (RoomKind::CommonRoom | RoomKind::GreatHall, RoomKind::Kitchen) => {
            (b.id, a.id, wall.frame.outward)
        }
        _ => return Ok(None),
    };
    Ok(Some(KitchenPair {
        storey,
        kitchen: RoomIndex::from_serialized(kitchen),
        stube: RoomIndex::from_serialized(stube),
        axis: PlanDirection::from_normalized(axis)?,
    }))
}
