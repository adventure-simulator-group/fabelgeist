use super::architecture::Floor;
use super::geometry::*;
use super::{FurnitureAccessPath, InteriorLayoutError, InteriorPlacement, InteriorWaypoint};
use crate::{BuildingPlan, OpeningUse, Stair};
use bevy::math::Vec2;
use std::collections::{BTreeMap, VecDeque};

mod occupancy;
pub(super) use occupancy::Occupancy;

pub(super) struct Navigation {
    pub floors: Vec<Floor>,
    pub nodes: Vec<InteriorWaypoint>,
    edges: Vec<Vec<usize>>,
    entry: usize,
    room_nodes: Vec<RoomNodes>,
    node_lookup: BTreeMap<NavigationPointKey, usize>,
}
struct RoomNodes {
    storey: crate::StoreyIndex,
    room: crate::RoomIndex,
    nodes: Vec<usize>,
}
/// Bit representations retain the existing exact node lookup policy.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct NavigationPointKey {
    storey: crate::StoreyIndex,
    x_bits: u32,
    z_bits: u32,
}
pub(super) struct Flood {
    pub parents: Vec<Option<usize>>,
    pub entry: usize,
}
impl Navigation {
    pub fn new(plan: &BuildingPlan) -> Result<Self, InteriorLayoutError> {
        let mut nav = Self::lattice(plan)?;
        nav.enter_front_door(plan)?;
        nav.add_doors(plan)?;
        nav.add_stairs(plan)?;
        for storey in &plan.storeys {
            for room in &storey.rooms {
                let ids = nav
                    .nodes
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| {
                        n.storey == crate::StoreyIndex::from_serialized(storey.level)
                            && Rect::new(n.position_metres.metres(), Vec2::splat(PERSON_RADIUS))
                                .inside_room(room)
                    })
                    .map(|(i, _)| i)
                    .collect();
                nav.room_nodes.push(RoomNodes {
                    storey: crate::StoreyIndex::from_serialized(storey.level),
                    room: crate::RoomIndex::from_serialized(room.id),
                    nodes: ids,
                });
            }
        }
        let empty = nav.flood(&[])?;
        nav.verify_rooms(&empty)?;
        Ok(nav)
    }
    fn lattice(plan: &BuildingPlan) -> Result<Self, InteriorLayoutError> {
        let floors = plan
            .storeys
            .iter()
            .map(|s| Floor::new(plan, s.level))
            .collect::<Result<Vec<_>, _>>()?;
        let mut nodes = Vec::new();
        let mut lookup = BTreeMap::new();
        for floor in &floors {
            for &(cx, cz) in &floor.cells {
                let cell_steps = (crate::CELL_SIZE_METRES / GRID_STEP) as i32;
                for x in 0..cell_steps {
                    for z in 0..cell_steps {
                        let gx = i32::from(cx) * cell_steps + x;
                        let gz = i32::from(cz) * cell_steps + z;
                        let point = Vec2::new(gx as f32, gz as f32) * GRID_STEP;
                        if floor.walkable(Rect::new(point, Vec2::splat(PERSON_RADIUS))) {
                            lookup.insert((floor.level, gx, gz), nodes.len());
                            nodes.push(InteriorWaypoint {
                                storey: crate::StoreyIndex::from_serialized(floor.level),
                                position_metres:
                                    crate::plan_geometry::ArchitecturalPlanPoint::try_from(point)?,
                            });
                        }
                    }
                }
            }
        }
        let mut edges = vec![Vec::new(); nodes.len()];
        for (&(level, x, z), &index) in &lookup {
            let floor = floors.iter().find(|f| f.level == level).ok_or(
                InteriorLayoutError::MissingStoreyFloor {
                    storey: crate::StoreyIndex::new(usize::from(level)),
                },
            )?;
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if let Some(&other) = lookup.get(&(level, x + dx, z + dz)) {
                    let a = nodes[index].position_metres.metres();
                    let b = nodes[other].position_metres.metres();
                    if floor.walkable(Rect::new(
                        (a + b) * 0.5,
                        (a - b).abs() * 0.5 + Vec2::splat(PERSON_RADIUS),
                    )) {
                        edges[index].push(other);
                    }
                }
            }
        }
        let node_lookup = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| {
                (
                    NavigationPointKey {
                        storey: n.storey,
                        x_bits: n.position_metres.metres().x.to_bits(),
                        z_bits: n.position_metres.metres().y.to_bits(),
                    },
                    i,
                )
            })
            .collect();
        Ok(Self {
            floors,
            nodes,
            edges,
            entry: 0,
            room_nodes: Vec::new(),
            node_lookup,
        })
    }
    fn enter_front_door(&mut self, plan: &BuildingPlan) -> Result<(), InteriorLayoutError> {
        let (threshold, start) =
            entrance_position(plan).ok_or(InteriorLayoutError::MissingFrontDoor)?;
        let swept = Rect::new(
            (threshold + start) * 0.5,
            (threshold - start).abs() * 0.5 + Vec2::splat(PERSON_RADIUS),
        );
        if self
            .floors
            .iter()
            .find(|f| f.level == 0)
            .is_none_or(|f| f.obstacles.iter().any(|o| o.overlaps(swept)))
        {
            return Err(InteriorLayoutError::MissingFrontDoor);
        }
        let inside = self
            .insert_portal(0, start)?
            .ok_or(InteriorLayoutError::MissingFrontDoor)?;
        self.entry = self.ensure_node(0, threshold)?;
        self.connect(self.entry, inside);
        Ok(())
    }
    fn add_doors(&mut self, plan: &BuildingPlan) -> Result<(), InteriorLayoutError> {
        for opening in plan.opening_assemblies.iter().filter(|o| {
            matches!(o.use_kind, OpeningUse::Door | OpeningUse::Gate)
                && o.frame.outside_room.is_some()
        }) {
            let level = represented_storey(plan, opening.sill_elevation_metres)?;
            let half_wall = plan
                .wall_assemblies
                .iter()
                .find(|w| w.id == opening.host_wall)
                .map_or(crate::WALL_THICKNESS_METRES * 0.5, |w| {
                    w.thickness_metres * 0.5
                });
            let approach = half_wall + PERSON_RADIUS + GRID_STEP;
            for offset in [0.0, -approach, approach] {
                self.insert_portal(level, opening.frame.origin + opening.frame.outward * offset)?;
            }
        }
        Ok(())
    }
    fn add_stairs(&mut self, plan: &BuildingPlan) -> Result<(), InteriorLayoutError> {
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
                let lower = represented_storey(plan, base_height_metres)?;
                let upper = represented_storey(plan, base_height_metres + rise_metres)?;
                let axis = direction.offset().as_vec2();
                let a = self
                    .stair_landing(lower, start, -axis)?
                    .ok_or(InteriorLayoutError::InvalidStair { index })?;
                let b = self
                    .stair_landing(upper, start + axis * run_metres, axis)?
                    .ok_or(InteriorLayoutError::InvalidStair { index })?;
                self.edges[a].push(b);
                self.edges[b].push(a);
            } else {
                let landings = crate::spiral_stairs::landings(plan, index)
                    .into_iter()
                    .filter(|landing| {
                        plan.storeys
                            .iter()
                            .filter(|s| s.level == landing.storey)
                            .flat_map(|s| &s.rooms)
                            .any(|r| room_contains(r, landing.position_metres))
                    })
                    .collect::<Vec<_>>();
                let mut portals = Vec::new();
                for landing in landings {
                    portals.push(
                        self.insert_portal(landing.storey, landing.position_metres)?
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
    fn ensure_node(&mut self, level: u16, point: Vec2) -> Result<usize, InteriorLayoutError> {
        let admitted = crate::plan_geometry::ArchitecturalPlanPoint::try_from(point)?;
        let key = NavigationPointKey {
            storey: crate::StoreyIndex::from_serialized(level),
            x_bits: point.x.to_bits(),
            z_bits: point.y.to_bits(),
        };
        if let Some(&index) = self.node_lookup.get(&key) {
            return Ok(index);
        }
        let index = self.nodes.len();
        self.nodes.push(InteriorWaypoint {
            storey: crate::StoreyIndex::from_serialized(level),
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
        level: u16,
        point: Vec2,
    ) -> Result<Option<usize>, InteriorLayoutError> {
        crate::plan_geometry::ArchitecturalPlanPoint::try_from(point)?;
        let Some(floor) = self.floors.iter().find(|f| f.level == level) else {
            return Ok(None);
        };
        if !floor.walkable(Rect::new(point, Vec2::splat(PERSON_RADIUS))) {
            return Ok(None);
        }
        let mut targets = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| {
                n.storey == crate::StoreyIndex::from_serialized(level)
                    && n.position_metres.metres().distance_squared(point)
                        > GEOMETRY_EPSILON * GEOMETRY_EPSILON
            })
            .map(|(index, n)| (index, n.position_metres.metres()))
            .collect::<Vec<_>>();
        targets.sort_by(|a, b| {
            a.1.distance_squared(point)
                .total_cmp(&b.1.distance_squared(point))
        });
        let mut connections = [None; 4];
        for (target, position) in targets {
            let delta = position - point;
            let quadrant = usize::from(delta.x >= 0.0) + 2 * usize::from(delta.y >= 0.0);
            if connections[quadrant].is_some() {
                continue;
            }
            if let Some(elbow) = [
                Vec2::new(point.x, position.y),
                Vec2::new(position.x, point.y),
            ]
            .into_iter()
            .find(|&elbow| {
                let first = Rect::new(
                    (point + elbow) * 0.5,
                    (point - elbow).abs() * 0.5 + Vec2::splat(PERSON_RADIUS),
                );
                let second = Rect::new(
                    (position + elbow) * 0.5,
                    (position - elbow).abs() * 0.5 + Vec2::splat(PERSON_RADIUS),
                );
                floor.walkable(first) && floor.walkable(second)
            }) {
                connections[quadrant] = Some((target, elbow));
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
        level: u16,
        end: Vec2,
        outward: Vec2,
    ) -> Result<Option<usize>, InteriorLayoutError> {
        const LANDING_HALF_DEPTH_METRES: f32 = 0.45;
        const PORTAL_SEARCH_STEP_METRES: f32 = 0.01;
        let steps = ((LANDING_HALF_DEPTH_METRES - PERSON_RADIUS) / PORTAL_SEARCH_STEP_METRES).ceil()
            as usize;
        let mut accepted = Vec::new();
        for step in 0..=steps {
            let point = end + outward * (PERSON_RADIUS + step as f32 * PORTAL_SEARCH_STEP_METRES);
            if let Some(index) = self.insert_portal(level, point)? {
                accepted.push(index);
            }
        }
        Ok(accepted
            .into_iter()
            .find(|&index| !self.edges[index].is_empty()))
    }
    pub fn flood(&self, placements: &[InteriorPlacement]) -> Result<Flood, InteriorLayoutError> {
        let mut occupancy = Occupancy::new(self);
        occupancy.add(placements)?;
        Ok(occupancy.flood())
    }
    pub fn verify_rooms(&self, flood: &Flood) -> Result<(), InteriorLayoutError> {
        for room in &self.room_nodes {
            if !room.nodes.iter().any(|&n| flood.parents[n].is_some()) {
                return Err(InteriorLayoutError::DisconnectedRoom {
                    storey: room.storey,
                    room_id: room.room,
                });
            }
        }
        Ok(())
    }
    pub fn access_paths(
        &self,
        placements: &[InteriorPlacement],
        flood: &Flood,
    ) -> Result<Vec<FurnitureAccessPath>, InteriorLayoutError> {
        let mut paths = Vec::new();
        for (index, p) in placements.iter().enumerate() {
            for &face in p.key.interior_spec()?.required_faces {
                let access = p.access_rect(face)?;
                let target = self
                    .nodes
                    .iter()
                    .enumerate()
                    .filter(|(n, node)| {
                        node.storey == p.storey
                            && flood.parents[*n].is_some()
                            && access.contains(node.position_metres.metres())
                    })
                    .min_by(|(_, a), (_, b)| {
                        a.position_metres
                            .metres()
                            .distance_squared(access.centre)
                            .total_cmp(&b.position_metres.metres().distance_squared(access.centre))
                    })
                    .map(|(n, _)| n)
                    .ok_or(InteriorLayoutError::InaccessibleFurniture { index })?;
                let mut cursor = target;
                let mut points = vec![self.nodes[cursor]];
                while cursor != flood.entry {
                    cursor =
                        flood.parents[cursor].ok_or(InteriorLayoutError::BrokenAccessPath {
                            placement_index: index,
                            node: cursor,
                        })?;
                    points.push(self.nodes[cursor]);
                }
                points.reverse();
                paths.push(FurnitureAccessPath {
                    placement_index: index,
                    face,
                    points,
                });
            }
        }
        Ok(paths)
    }
}

fn entrance_position(plan: &BuildingPlan) -> Option<(Vec2, Vec2)> {
    if let Some(workplace) = &plan.workplace {
        let dimensions = plan.dimensions_metres();
        return workplace.passages.iter().find_map(|p| {
            let centre = Vec2::new(
                p.bounds.min().metres().x + p.bounds.max().metres().x,
                p.bounds.min().metres().z + p.bounds.max().metres().z,
            ) * 0.5;
            let inset = PERSON_RADIUS + crate::WALL_THICKNESS_METRES;
            if p.bounds.min().metres().z <= GEOMETRY_EPSILON
                && p.bounds.max().metres().x - p.bounds.min().metres().x >= PERSON_RADIUS * 2.0
            {
                Some((Vec2::new(centre.x, 0.0), Vec2::new(centre.x, inset)))
            } else if p.bounds.min().metres().x <= GEOMETRY_EPSILON
                && p.bounds.max().metres().z - p.bounds.min().metres().z >= PERSON_RADIUS * 2.0
            {
                Some((Vec2::new(0.0, centre.y), Vec2::new(inset, centre.y)))
            } else if p.bounds.max().metres().z >= dimensions.y - GEOMETRY_EPSILON
                && p.bounds.max().metres().x - p.bounds.min().metres().x >= PERSON_RADIUS * 2.0
            {
                Some((
                    Vec2::new(centre.x, dimensions.y),
                    Vec2::new(centre.x, dimensions.y - inset),
                ))
            } else if p.bounds.max().metres().x >= dimensions.x - GEOMETRY_EPSILON
                && p.bounds.max().metres().z - p.bounds.min().metres().z >= PERSON_RADIUS * 2.0
            {
                Some((
                    Vec2::new(dimensions.x, centre.y),
                    Vec2::new(dimensions.x - inset, centre.y),
                ))
            } else {
                None
            }
        });
    }
    plan.opening_assemblies
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
            (
                o.frame.origin,
                o.frame.origin
                    - o.frame.outward
                        * (PERSON_RADIUS
                            + half_wall.max(crate::WALL_THICKNESS_METRES)
                            + GEOMETRY_EPSILON),
            )
        })
}

fn represented_storey(
    plan: &BuildingPlan,
    elevation_metres: f32,
) -> Result<u16, InteriorLayoutError> {
    use crate::spatial_geometry::{Elevation, PositiveLength};
    let level = crate::StoreyIndex::from_elevation(
        Elevation::from_metres(elevation_metres)?,
        PositiveLength::from_metres(plan.storey_height_metres)?,
    )?;
    Ok(level.serialized_ordinal()?)
}
