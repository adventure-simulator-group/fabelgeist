use super::geometry::*;
use super::obstruction::Obstruction;
use crate::CollisionResult;
use crate::interior::InteriorResult;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::GeometryResult;
use crate::spatial_geometry::{Elevation, PositiveLength};
use crate::{BuildingPlan, CELL_SIZE_METRES, OpeningUse, SolidRole, Stair};
use bevy::math::Vec2;
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
struct FloorSupport {
    rect: FloorFootprint,
    elevation: Elevation<crate::Architectural>,
}
pub(super) struct Floor {
    pub level: crate::StoreyIndex,
    pub cells: BTreeSet<(i16, i16)>,
    pub obstacles: Vec<Rect>,
    pub reserved: Vec<Rect>,
    solids: Vec<Obstruction<crate::Architectural>>,
    supports: Vec<FloorSupport>,
}
impl Floor {
    pub fn new(plan: &BuildingPlan, level: crate::StoreyIndex) -> InteriorResult<Self> {
        let height = floor_height(plan, level)?;
        let native_height = height.metres();
        let solids = architectural_solids(plan)?;
        let low = Elevation::from_metres(native_height + FLOOR_CLEARANCE)?;
        let high = Elevation::from_metres(native_height + PERSON_HEIGHT)?;
        let mut obstacles = Vec::new();
        for solid in &solids {
            if let Some(rect) = solid.rectangle(low, high)? {
                obstacles.push(rect);
            }
        }
        let reserved = circulation_reservations(plan, height, &mut obstacles)?;
        let cells = plan
            .storeys
            .iter()
            .find(|s| crate::StoreyIndex::from_serialized(s.level) == level)
            .into_iter()
            .flat_map(|s| &s.rooms)
            .flat_map(|r| &r.cells)
            .map(|c| (c.x, c.z))
            .collect();
        let floor_solids = plan
            .resolved_geometry
            .solids
            .iter()
            .filter(|s| is_floor(plan, s))
            .filter(|s| {
                (s.centre.metres().y + s.size.metres().y * 0.5 - native_height).abs()
                    <= PERSON_RADIUS
            })
            .map(|s| {
                Ok(FloorSupport {
                    rect: FloorFootprint::from_solid(s)?,
                    elevation: Elevation::from_metres(
                        s.centre.metres().y + s.size.metres().y * 0.5,
                    )?,
                })
            })
            .collect::<InteriorResult<Vec<_>>>()?;
        Ok(Self {
            level,
            cells,
            obstacles,
            reserved,
            solids,
            supports: floor_solids,
        })
    }
    pub fn contains(&self, point: ArchitecturalPlanPoint) -> bool {
        let cell = (point.metres() / CELL_SIZE_METRES).floor().as_ivec2();
        self.cells.contains(&(cell.x as i16, cell.y as i16))
            && self.supports.iter().any(|s| s.rect.contains(point))
    }
    pub fn supports(
        &self,
        rect: Rect,
        height: Elevation<crate::Architectural>,
    ) -> GeometryResult<bool> {
        rect.all_samples(|point| {
            self.contains(point)
                && self.supports.iter().any(|s| {
                    (s.elevation.metres() - height.metres()).abs() < GEOMETRY_EPSILON
                        && s.rect.contains(point)
                })
        })
    }
    pub fn height_at(
        &self,
        point: ArchitecturalPlanPoint,
    ) -> Option<Elevation<crate::Architectural>> {
        self.supports
            .iter()
            .filter(|s| s.rect.contains(point))
            .map(|s| s.elevation)
            .max_by(|a, b| a.metres().total_cmp(&b.metres()))
    }
    pub fn placement_clear(
        &self,
        rect: Rect,
        bottom: Elevation<crate::Architectural>,
        height: PositiveLength,
    ) -> InteriorResult<bool> {
        let low = Elevation::from_metres(bottom.metres() + GEOMETRY_EPSILON)?;
        let high = Elevation::from_metres(bottom.metres() + height.metres())?;
        Ok(!crate::geometry_index::try_any(&self.solids, |s| {
            s.intersects(rect, low, high)
        })?)
    }
    pub fn walkable(&self, rect: Rect) -> GeometryResult<bool> {
        Ok(rect.all_samples(|p| self.contains(p))?
            && !self.obstacles.iter().any(|o| o.overlaps(rect)))
    }
}

fn circulation_reservations(
    plan: &BuildingPlan,
    height: Elevation<crate::Architectural>,
    obstacles: &mut Vec<Rect>,
) -> InteriorResult<Vec<Rect>> {
    let mut reserved = door_reservations(plan, height)?;
    reserved.extend(super::church::nave_routes(plan, height)?);
    let height = height.metres();
    if let Some(heating) = &plan.domestic_heating {
        let space = heating.operating_space;
        if height < space.max().metres().y && height + PERSON_HEIGHT > space.min().metres().y {
            reserved.push(Rect::from_metres(
                Vec2::new(
                    space.min().metres().x + space.max().metres().x,
                    space.min().metres().z + space.max().metres().z,
                ) * 0.5,
                Vec2::new(
                    space.max().metres().x - space.min().metres().x,
                    space.max().metres().z - space.min().metres().z,
                ) * 0.5,
            )?);
        }
    }
    if let Some(workplace) = &plan.workplace {
        for passage in &workplace.passages {
            if passage.bounds.min().metres().y < height + PERSON_HEIGHT
                && passage.bounds.max().metres().y > height + FLOOR_CLEARANCE
            {
                reserved.push(Rect::from_metres(
                    Vec2::new(
                        passage.bounds.min().metres().x + passage.bounds.max().metres().x,
                        passage.bounds.min().metres().z + passage.bounds.max().metres().z,
                    ) * 0.5,
                    Vec2::new(
                        passage.bounds.max().metres().x - passage.bounds.min().metres().x,
                        passage.bounds.max().metres().z - passage.bounds.min().metres().z,
                    ) * 0.5,
                )?);
            }
        }
    }
    for stair in &plan.stairs {
        match *stair {
            Stair::Straight {
                start,
                direction,
                base_height_metres,
                rise_metres,
                width_metres,
                run_metres,
                ..
            } => {
                if height + FLOOR_CLEARANCE < base_height_metres
                    || height > base_height_metres + rise_metres + FLOOR_CLEARANCE
                {
                    continue;
                }
                let axis = direction.offset().as_vec2();
                let lateral = Vec2::new(-axis.y, axis.x);
                let flight = Rect::from_metres(
                    start + axis * run_metres * 0.5,
                    axis.abs() * run_metres * 0.5 + lateral.abs() * width_metres * 0.5,
                )?;
                reserved.push(flight.expanded(
                    crate::spatial_geometry::SignedLength::from_metres(PERSON_RADIUS)?,
                )?);
            }
            Stair::Spiral {
                centre,
                base_height_metres,
                rise_metres,
                outer_radius_metres,
                ..
            } => {
                if height >= base_height_metres - FLOOR_CLEARANCE
                    && height <= base_height_metres + rise_metres + FLOOR_CLEARANCE
                {
                    reserved.push(Rect::from_metres(
                        centre,
                        Vec2::splat(outer_radius_metres + PERSON_RADIUS),
                    )?);
                    // Spiral travel uses its physical landing links, never a
                    // horizontal shortcut through the flight at room height.
                    if let Some((min, max)) = crate::spiral_stairs::well_bounds(*stair) {
                        obstacles.push(Rect::from_metres((min + max) * 0.5, (max - min) * 0.5)?);
                    }
                }
            }
        }
    }
    Ok(reserved)
}

fn door_reservations(
    plan: &BuildingPlan,
    height: Elevation<crate::Architectural>,
) -> InteriorResult<Vec<Rect>> {
    let height = height.metres();
    plan.opening_assemblies
        .iter()
        .filter(|o| {
            matches!(o.use_kind, OpeningUse::Door | OpeningUse::Gate)
                && (o.sill_elevation_metres - height).abs() <= FLOOR_CLEARANCE
        })
        .map(|o| {
            let width = o.profile.interior_width_metres();
            let half = o.frame.tangent.abs() * (width * 0.5 + PERSON_RADIUS)
                + o.frame.outward.abs() * (width + PERSON_RADIUS);
            Ok(Rect::from_metres(o.frame.origin, half)?)
        })
        .collect()
}

fn architectural_solids(
    plan: &BuildingPlan,
) -> CollisionResult<Vec<Obstruction<crate::Architectural>>> {
    let floor_ids = plan
        .resolved_geometry
        .solids
        .iter()
        .filter(|s| is_floor(plan, s) || s.role == SolidRole::StairTread)
        .map(|s| s.id)
        .collect::<BTreeSet<_>>();
    let mut solids = crate::compile_building_collision(plan)?
        .cuboids
        .into_iter()
        .filter(|s| !floor_ids.contains(&s.source))
        .map(Obstruction::new)
        .collect::<CollisionResult<Vec<_>>>()?;
    solids.extend(
        plan.resolved_geometry
            .solids
            .iter()
            .filter(|s| {
                matches!(
                    s.role,
                    SolidRole::ChurchPier
                        | SolidRole::ChurchStairNewel
                        | SolidRole::FramePost
                        | SolidRole::FrameGirder
                        | SolidRole::FrameJoist
                        | SolidRole::FrameBrace
                        | SolidRole::RoofFraming
                        | SolidRole::ChurchArcade
                        | SolidRole::ChurchCrossingArch
                )
            })
            .map(|s| crate::collision::collision_parts(plan, s))
            .collect::<CollisionResult<Vec<_>>>()?
            .into_iter()
            .flatten()
            .map(Obstruction::new)
            .collect::<CollisionResult<Vec<_>>>()?,
    );
    Ok(solids)
}
pub(super) fn is_floor(plan: &BuildingPlan, solid: &crate::ResolvedSolid) -> bool {
    matches!(
        solid.role,
        SolidRole::FrameFloor
            | SolidRole::ChurchFloor
            | SolidRole::GalleryFloor
            | SolidRole::InteriorFloor
    ) || crate::spiral_stairs::owns_landing(solid)
        || plan
            .workplace
            .iter()
            .flat_map(|w| &w.parts)
            .any(|p| p.feature == crate::WorkplaceFeature::Floor && p.solid == solid.id)
}
