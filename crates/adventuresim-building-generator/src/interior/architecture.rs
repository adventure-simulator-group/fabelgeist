use super::geometry::*;
use super::obstruction::Obstruction;
use crate::{BuildingPlan, CELL_SIZE_METRES, OpeningUse, SolidRole, Stair};
use bevy::math::Vec2;
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
struct FloorSupport {
    rect: FloorFootprint,
    elevation: f32,
}
pub(super) struct Floor {
    pub level: u16,
    pub cells: BTreeSet<(i16, i16)>,
    pub obstacles: Vec<Rect>,
    pub reserved: Vec<Rect>,
    solids: Vec<Obstruction>,
    supports: Vec<FloorSupport>,
}
impl Floor {
    pub fn new(plan: &BuildingPlan, level: u16) -> Self {
        let height = floor_height(plan, level);
        let solids = architectural_solids(plan);
        let mut obstacles = solids
            .iter()
            .filter_map(|s| s.projection(height + FLOOR_CLEARANCE, height + PERSON_HEIGHT))
            .collect::<Vec<_>>();
        let reserved = circulation_reservations(plan, height, &mut obstacles);
        let cells = plan
            .storeys
            .iter()
            .find(|s| s.level == level)
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
            .filter(|s| (s.centre.y + s.size.y * 0.5 - height).abs() <= PERSON_RADIUS)
            .map(|s| FloorSupport {
                rect: FloorFootprint::from_solid(s),
                elevation: s.centre.y + s.size.y * 0.5,
            })
            .collect::<Vec<_>>();
        Self {
            level,
            cells,
            obstacles,
            reserved,
            solids,
            supports: floor_solids,
        }
    }
    pub fn contains(&self, point: Vec2) -> bool {
        let cell = (point / CELL_SIZE_METRES).floor().as_ivec2();
        self.cells.contains(&(cell.x as i16, cell.y as i16))
            && self.supports.iter().any(|s| s.rect.contains(point))
    }
    pub fn supports(&self, rect: Rect, height: f32) -> bool {
        rect.all_samples(|point| {
            self.contains(point)
                && self.supports.iter().any(|s| {
                    (s.elevation - height).abs() < GEOMETRY_EPSILON && s.rect.contains(point)
                })
        })
    }
    pub fn height_at(&self, point: Vec2) -> Option<f32> {
        self.supports
            .iter()
            .filter(|s| s.rect.contains(point))
            .map(|s| s.elevation)
            .max_by(f32::total_cmp)
    }
    pub fn placement_clear(&self, rect: Rect, bottom: f32, height: f32) -> bool {
        !self
            .solids
            .iter()
            .any(|s| s.intersects(rect, bottom + GEOMETRY_EPSILON, bottom + height))
    }
    pub fn walkable(&self, rect: Rect) -> bool {
        rect.all_samples(|p| self.contains(p)) && !self.obstacles.iter().any(|o| o.overlaps(rect))
    }
}

fn circulation_reservations(
    plan: &BuildingPlan,
    height: f32,
    obstacles: &mut Vec<Rect>,
) -> Vec<Rect> {
    let mut reserved = door_reservations(plan, height);
    if let Some(workplace) = &plan.workplace {
        for passage in &workplace.passages {
            if passage.min.y < height + PERSON_HEIGHT && passage.max.y > height + FLOOR_CLEARANCE {
                reserved.push(Rect::new(
                    Vec2::new(passage.min.x + passage.max.x, passage.min.z + passage.max.z) * 0.5,
                    Vec2::new(passage.max.x - passage.min.x, passage.max.z - passage.min.z) * 0.5,
                ));
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
                let flight = Rect::new(
                    start + axis * run_metres * 0.5,
                    axis.abs() * run_metres * 0.5 + lateral.abs() * width_metres * 0.5,
                );
                reserved.push(flight.expanded(PERSON_RADIUS));
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
                    reserved.push(Rect::new(
                        centre,
                        Vec2::splat(outer_radius_metres + PERSON_RADIUS),
                    ));
                    // Spiral travel uses its physical landing links, never a
                    // horizontal shortcut through the flight at room height.
                    if let Some((min, max)) = crate::spiral_stairs::well_bounds(*stair) {
                        obstacles.push(Rect::new((min + max) * 0.5, (max - min) * 0.5));
                    }
                }
            }
        }
    }
    reserved
}

fn door_reservations(plan: &BuildingPlan, height: f32) -> Vec<Rect> {
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
            Rect::new(o.frame.origin, half)
        })
        .collect()
}

fn architectural_solids(plan: &BuildingPlan) -> Vec<Obstruction> {
    let floor_ids = plan
        .resolved_geometry
        .solids
        .iter()
        .filter(|s| is_floor(plan, s) || s.role == SolidRole::StairTread)
        .map(|s| s.id)
        .collect::<BTreeSet<_>>();
    let mut solids = crate::compile_building_collision(plan)
        .cuboids
        .into_iter()
        .filter(|s| !floor_ids.contains(&s.source))
        .map(|s| {
            Obstruction::new(
                s.centre,
                s.size,
                s.yaw_radians,
                s.crossfall_radians,
                s.longfall_radians,
            )
        })
        .collect::<Vec<_>>();
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
            .flat_map(|s| crate::collision::collision_parts(plan, s))
            .map(|s| {
                Obstruction::new(
                    s.centre,
                    s.size,
                    s.yaw_radians,
                    s.crossfall_radians,
                    s.longfall_radians,
                )
            }),
    );
    solids
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
