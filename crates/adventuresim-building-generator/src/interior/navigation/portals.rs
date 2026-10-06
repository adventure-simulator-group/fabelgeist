//! Admit doorway and stair approaches before connecting the navigation graph.
use super::*;
use crate::interior::InteriorResult as Result;
use crate::spatial_geometry::PlanExtents;
use crate::spatial_geometry::{Elevation, PlanDirection};
use crate::{Architectural, OpeningUse, Stair};

impl Navigation {
    pub(super) fn enter_front_door(&mut self, plan: &BuildingPlan) -> Result<()> {
        let entrance = Entrance::from_plan(plan)?.ok_or(InteriorLayoutError::MissingFrontDoor)?;
        let threshold = entrance.threshold.metres();
        let start = entrance.approach.metres();
        let swept = Rect::new(
            ArchitecturalPlanPoint::from_metres((threshold + start) * 0.5)?,
            PlanExtents::from_metres((threshold - start).abs() * 0.5 + Vec2::splat(PERSON_RADIUS))?,
        )?;
        if self
            .floors
            .iter()
            .find(|f| f.level == StoreyIndex::GROUND)
            .is_none_or(|f| f.obstacles.iter().any(|o| o.overlaps(swept)))
        {
            return Err(InteriorLayoutError::MissingFrontDoor);
        }
        let inside = self
            .insert_portal(
                StoreyIndex::GROUND,
                ArchitecturalPlanPoint::from_metres(start)?,
            )?
            .ok_or(InteriorLayoutError::MissingFrontDoor)?;
        self.entry = self.ensure_node(
            StoreyIndex::GROUND,
            ArchitecturalPlanPoint::from_metres(threshold)?,
        )?;
        self.connect(self.entry, inside);
        Ok(())
    }
    pub(super) fn add_doors(&mut self, plan: &BuildingPlan) -> Result<()> {
        for opening in plan.opening_assemblies.iter().filter(|o| {
            matches!(o.use_kind, OpeningUse::Door | OpeningUse::Gate)
                && o.frame.outside_room.is_some()
        }) {
            let level =
                represented_storey(plan, Elevation::from_metres(opening.sill_elevation_metres)?)?;
            let half_wall = plan
                .wall_assemblies
                .iter()
                .find(|w| w.id == opening.host_wall)
                .map_or(crate::WALL_THICKNESS_METRES * 0.5, |w| {
                    w.thickness_metres * 0.5
                });
            let approach = half_wall + PERSON_RADIUS + GRID_STEP;
            for offset in [0.0, -approach, approach] {
                self.insert_portal(
                    level,
                    ArchitecturalPlanPoint::from_metres(
                        opening.frame.origin + opening.frame.outward * offset,
                    )?,
                )?;
            }
        }
        Ok(())
    }
    pub(super) fn add_stairs(&mut self, plan: &BuildingPlan) -> Result<()> {
        for (index, stair) in plan.stairs.iter().enumerate() {
            if let Stair::Straight {
                start,
                direction,
                base_height_metres,
                rise_metres,
                run_metres,
                ..
            } = *stair
            {
                let lower = represented_storey(plan, Elevation::from_metres(base_height_metres)?)?;
                let upper = represented_storey(
                    plan,
                    Elevation::from_metres(base_height_metres + rise_metres)?,
                )?;
                let axis = direction.offset().as_vec2();
                let a = self
                    .stair_landing(
                        lower,
                        ArchitecturalPlanPoint::from_metres(start)?,
                        PlanDirection::from_normalized(-axis)?,
                    )?
                    .ok_or(InteriorLayoutError::InvalidStair { index })?;
                let b = self
                    .stair_landing(
                        upper,
                        ArchitecturalPlanPoint::from_metres(start + axis * run_metres)?,
                        PlanDirection::from_normalized(axis)?,
                    )?
                    .ok_or(InteriorLayoutError::InvalidStair { index })?;
                self.edges[a].push(b);
                self.edges[b].push(a);
            } else {
                let mut landings = Vec::new();
                for landing in crate::spiral_stairs::landings(plan, index) {
                    let point = ArchitecturalPlanPoint::from_metres(landing.position_metres)?;
                    if plan
                        .storeys
                        .iter()
                        .filter(|s| s.level == landing.storey)
                        .flat_map(|s| &s.rooms)
                        .any(|r| room_contains(r, point))
                    {
                        landings.push(landing);
                    }
                }
                let mut portals = Vec::new();
                for landing in landings {
                    portals.push(
                        self.insert_portal(
                            StoreyIndex::from_serialized(landing.storey),
                            ArchitecturalPlanPoint::from_metres(landing.position_metres)?,
                        )?
                        .ok_or(InteriorLayoutError::InvalidStair { index })?,
                    );
                }
                for pair in portals.windows(2) {
                    self.connect(pair[0], pair[1]);
                }
            }
        }
        Ok(())
    }
    fn ensure_node(&mut self, level: StoreyIndex, point: ArchitecturalPlanPoint) -> Result<usize> {
        let admitted = point;
        let point = point.metres();
        let key = NavigationPointKey {
            storey: level,
            x_bits: point.x.to_bits(),
            z_bits: point.y.to_bits(),
        };
        if let Some(&index) = self.node_lookup.get(&key) {
            return Ok(index);
        }
        let index = self.nodes.len();
        self.nodes.push(InteriorWaypoint {
            storey: level,
            position_metres: admitted,
        });
        self.edges.push(Vec::new());
        self.node_lookup.insert(key, index);
        Ok(index)
    }
    fn connect(&mut self, a: usize, b: usize) {
        if a != b && !self.edges[a].contains(&b) {
            self.edges[a].push(b);
            self.edges[b].push(a);
        }
    }
    fn insert_portal(
        &mut self,
        level: StoreyIndex,
        point: ArchitecturalPlanPoint,
    ) -> Result<Option<usize>> {
        let native_point = point.metres();
        let Some(floor) = self.floors.iter().find(|f| f.level == level) else {
            return Ok(None);
        };
        if !floor.walkable(Rect::new(
            point,
            PlanExtents::from_metres(Vec2::splat(PERSON_RADIUS))?,
        )?)? {
            return Ok(None);
        }
        let mut targets = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| {
                n.storey == level
                    && n.position_metres.metres().distance_squared(native_point)
                        > GEOMETRY_EPSILON * GEOMETRY_EPSILON
            })
            .map(|(index, n)| (index, n.position_metres.metres()))
            .collect::<Vec<_>>();
        targets.sort_by(|a, b| {
            a.1.distance_squared(native_point)
                .total_cmp(&b.1.distance_squared(native_point))
        });
        let mut connections = [None; 4];
        for (target, position) in targets {
            let delta = position - native_point;
            let quadrant = usize::from(delta.x >= 0.0) + 2 * usize::from(delta.y >= 0.0);
            if connections[quadrant].is_some() {
                continue;
            }
            for elbow in [
                Vec2::new(native_point.x, position.y),
                Vec2::new(position.x, native_point.y),
            ] {
                let first = Rect::new(
                    ArchitecturalPlanPoint::from_metres((native_point + elbow) * 0.5)?,
                    PlanExtents::from_metres(
                        (native_point - elbow).abs() * 0.5 + Vec2::splat(PERSON_RADIUS),
                    )?,
                )?;
                let second = Rect::new(
                    ArchitecturalPlanPoint::from_metres((position + elbow) * 0.5)?,
                    PlanExtents::from_metres(
                        (position - elbow).abs() * 0.5 + Vec2::splat(PERSON_RADIUS),
                    )?,
                )?;
                if floor.walkable(first)? && floor.walkable(second)? {
                    connections[quadrant] =
                        Some((target, ArchitecturalPlanPoint::from_metres(elbow)?));
                    break;
                }
            }
            if connections.iter().all(Option::is_some) {
                break;
            }
        }
        let index = self.ensure_node(level, point)?;
        for (target, elbow) in connections.into_iter().flatten() {
            let middle = self.ensure_node(level, elbow)?;
            self.connect(index, middle);
            self.connect(middle, target);
        }
        Ok(Some(index))
    }
    fn stair_landing(
        &mut self,
        level: StoreyIndex,
        end: ArchitecturalPlanPoint,
        outward: PlanDirection<Architectural>,
    ) -> Result<Option<usize>> {
        let end = end.metres();
        let outward = outward.vector();
        const LANDING_HALF_DEPTH_METRES: f32 = 0.45;
        const PORTAL_SEARCH_STEP_METRES: f32 = 0.01;
        let steps = ((LANDING_HALF_DEPTH_METRES - PERSON_RADIUS) / PORTAL_SEARCH_STEP_METRES).ceil()
            as usize;
        let mut accepted = Vec::new();
        for step in 0..=steps {
            let point = end + outward * (PERSON_RADIUS + step as f32 * PORTAL_SEARCH_STEP_METRES);
            if let Some(index) =
                self.insert_portal(level, ArchitecturalPlanPoint::from_metres(point)?)?
            {
                accepted.push(index);
            }
        }
        Ok(accepted
            .into_iter()
            .find(|&index| !self.edges[index].is_empty()))
    }
}

struct Entrance {
    threshold: ArchitecturalPlanPoint,
    approach: ArchitecturalPlanPoint,
}
impl Entrance {
    fn new(threshold: ArchitecturalPlanPoint, approach: ArchitecturalPlanPoint) -> Self {
        Self {
            threshold,
            approach,
        }
    }
    fn from_plan(plan: &BuildingPlan) -> Result<Option<Self>> {
        if let Some(workplace) = &plan.workplace {
            return Self::from_workplace(plan, workplace);
        }
        let entrance = plan
            .opening_assemblies
            .iter()
            .find(|o| {
                matches!(o.use_kind, OpeningUse::Door | OpeningUse::Gate)
                    && o.frame.outside_room.is_none()
                    && o.sill_elevation_metres.abs() < FLOOR_CLEARANCE
            })
            .filter(|o| o.profile.interior_width_metres() >= PERSON_RADIUS * 2.0)
            .map(|o| {
                let half_wall = plan
                    .wall_assemblies
                    .iter()
                    .find(|w| w.id == o.host_wall)
                    .map_or(crate::WALL_THICKNESS_METRES * 0.5, |w| {
                        w.thickness_metres * 0.5
                    });
                Ok(Self::new(
                    ArchitecturalPlanPoint::from_metres(o.frame.origin)?,
                    ArchitecturalPlanPoint::from_metres(
                        o.frame.origin
                            - o.frame.outward
                                * (PERSON_RADIUS
                                    + half_wall.max(crate::WALL_THICKNESS_METRES)
                                    + GEOMETRY_EPSILON),
                    )?,
                ))
            });
        entrance.transpose()
    }
    fn from_workplace(
        plan: &BuildingPlan,
        workplace: &crate::WorkplacePlan,
    ) -> Result<Option<Self>> {
        let dimensions = plan.dimensions_metres();
        for passage in &workplace.passages {
            let centre = Vec2::new(
                passage.bounds.min().metres().x + passage.bounds.max().metres().x,
                passage.bounds.min().metres().z + passage.bounds.max().metres().z,
            ) * 0.5;
            let inset = PERSON_RADIUS + crate::WALL_THICKNESS_METRES;
            let entrance = if passage.bounds.min().metres().z <= GEOMETRY_EPSILON
                && passage.bounds.max().metres().x - passage.bounds.min().metres().x
                    >= PERSON_RADIUS * 2.0
            {
                Some(Self::new(
                    ArchitecturalPlanPoint::from_metres(Vec2::new(centre.x, 0.0))?,
                    ArchitecturalPlanPoint::from_metres(Vec2::new(centre.x, inset))?,
                ))
            } else if passage.bounds.min().metres().x <= GEOMETRY_EPSILON
                && passage.bounds.max().metres().z - passage.bounds.min().metres().z
                    >= PERSON_RADIUS * 2.0
            {
                Some(Self::new(
                    ArchitecturalPlanPoint::from_metres(Vec2::new(0.0, centre.y))?,
                    ArchitecturalPlanPoint::from_metres(Vec2::new(inset, centre.y))?,
                ))
            } else if passage.bounds.max().metres().z >= dimensions.y - GEOMETRY_EPSILON
                && passage.bounds.max().metres().x - passage.bounds.min().metres().x
                    >= PERSON_RADIUS * 2.0
            {
                Some(Self::new(
                    ArchitecturalPlanPoint::from_metres(Vec2::new(centre.x, dimensions.y))?,
                    ArchitecturalPlanPoint::from_metres(Vec2::new(centre.x, dimensions.y - inset))?,
                ))
            } else if passage.bounds.max().metres().x >= dimensions.x - GEOMETRY_EPSILON
                && passage.bounds.max().metres().z - passage.bounds.min().metres().z
                    >= PERSON_RADIUS * 2.0
            {
                Some(Self::new(
                    ArchitecturalPlanPoint::from_metres(Vec2::new(dimensions.x, centre.y))?,
                    ArchitecturalPlanPoint::from_metres(Vec2::new(dimensions.x - inset, centre.y))?,
                ))
            } else {
                None
            };
            if let Some(entrance) = entrance {
                return Ok(Some(entrance));
            }
        }
        Ok(None)
    }
}

fn represented_storey(
    plan: &BuildingPlan,
    elevation: Elevation<Architectural>,
) -> Result<StoreyIndex> {
    use crate::spatial_geometry::PositiveLength;
    let level = crate::StoreyIndex::from_elevation(
        elevation,
        PositiveLength::from_metres(plan.storey_height_metres)?,
    )?;
    Ok(level)
}
