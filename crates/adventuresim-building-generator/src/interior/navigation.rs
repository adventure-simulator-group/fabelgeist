use super::architecture::Floor;
use super::geometry::*;
use super::{FurnitureAccessPath, InteriorLayoutError, InteriorPlacement, InteriorWaypoint};
use crate::{BuildingPlan, OpeningUse, Stair};
use bevy::math::Vec2;
use std::collections::{BTreeMap, VecDeque};

pub(super) struct Navigation {
    pub floors: Vec<Floor>,
    pub nodes: Vec<InteriorWaypoint>,
    edges: Vec<Vec<usize>>,
    entry: usize,
    room_nodes: Vec<(u16, u16, Vec<usize>)>,
    node_lookup: BTreeMap<(u16, u32, u32), usize>,
}
pub(super) struct Flood {
    pub parents: Vec<Option<usize>>,
    pub entry: usize,
}
impl Navigation {
    pub fn new(plan: &BuildingPlan) -> Result<Self, InteriorLayoutError> {
        let mut nav = Self::lattice(plan);
        nav.enter_front_door(plan)?;
        nav.add_doors(plan);
        nav.add_stairs(plan)?;
        for storey in &plan.storeys {
            for room in &storey.rooms {
                let ids = nav
                    .nodes
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| {
                        n.storey == storey.level
                            && Rect::new(n.position_metres, Vec2::splat(PERSON_RADIUS))
                                .inside_room(room)
                    })
                    .map(|(i, _)| i)
                    .collect();
                nav.room_nodes.push((storey.level, room.id, ids));
            }
        }
        let empty = nav.flood(&[]);
        nav.verify_rooms(&empty)?;
        Ok(nav)
    }
    fn lattice(plan: &BuildingPlan) -> Self {
        let floors = plan
            .storeys
            .iter()
            .map(|s| Floor::new(plan, s.level))
            .collect::<Vec<_>>();
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
                                storey: floor.level,
                                position_metres: point,
                            });
                        }
                    }
                }
            }
        }
        let mut edges = vec![Vec::new(); nodes.len()];
        for (&(level, x, z), &index) in &lookup {
            let floor = floors.iter().find(|f| f.level == level).unwrap();
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if let Some(&other) = lookup.get(&(level, x + dx, z + dz)) {
                    let a = nodes[index].position_metres;
                    let b = nodes[other].position_metres;
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
                    (
                        n.storey,
                        n.position_metres.x.to_bits(),
                        n.position_metres.y.to_bits(),
                    ),
                    i,
                )
            })
            .collect();
        Self {
            floors,
            nodes,
            edges,
            entry: 0,
            room_nodes: Vec::new(),
            node_lookup,
        }
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
            .insert_portal(0, start)
            .ok_or(InteriorLayoutError::MissingFrontDoor)?;
        self.entry = self.ensure_node(0, threshold);
        self.connect(self.entry, inside);
        Ok(())
    }
    fn add_doors(&mut self, plan: &BuildingPlan) {
        for opening in plan.opening_assemblies.iter().filter(|o| {
            matches!(o.use_kind, OpeningUse::Door | OpeningUse::Gate)
                && o.frame.outside_room.is_some()
        }) {
            let level = (opening.sill_elevation_metres / plan.storey_height_metres).round() as u16;
            let half_wall = plan
                .wall_assemblies
                .iter()
                .find(|w| w.id == opening.host_wall)
                .map_or(crate::WALL_THICKNESS_METRES * 0.5, |w| {
                    w.thickness_metres * 0.5
                });
            let approach = half_wall + PERSON_RADIUS + GRID_STEP;
            for offset in [0.0, -approach, approach] {
                self.insert_portal(level, opening.frame.origin + opening.frame.outward * offset);
            }
        }
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
                let lower = (base_height_metres / plan.storey_height_metres).round() as u16;
                let upper =
                    ((base_height_metres + rise_metres) / plan.storey_height_metres).round() as u16;
                let axis = direction.offset().as_vec2();
                let a = self
                    .stair_landing(lower, start, -axis)
                    .ok_or(InteriorLayoutError::InvalidStair { index })?;
                let b = self
                    .stair_landing(upper, start + axis * run_metres, axis)
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
                        self.insert_portal(landing.storey, landing.position_metres)
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
    fn ensure_node(&mut self, level: u16, point: Vec2) -> usize {
        let key = (level, point.x.to_bits(), point.y.to_bits());
        if let Some(&index) = self.node_lookup.get(&key) {
            return index;
        }
        let index = self.nodes.len();
        self.nodes.push(InteriorWaypoint {
            storey: level,
            position_metres: point,
        });
        self.edges.push(Vec::new());
        self.node_lookup.insert(key, index);
        index
    }
    fn connect(&mut self, a: usize, b: usize) {
        if a != b && !self.edges[a].contains(&b) {
            self.edges[a].push(b);
            self.edges[b].push(a);
        }
    }
    fn insert_portal(&mut self, level: u16, point: Vec2) -> Option<usize> {
        let floor = self.floors.iter().find(|f| f.level == level)?;
        if !floor.walkable(Rect::new(point, Vec2::splat(PERSON_RADIUS))) {
            return None;
        }
        let mut targets = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| {
                n.storey == level
                    && n.position_metres.distance_squared(point)
                        > GEOMETRY_EPSILON * GEOMETRY_EPSILON
            })
            .map(|(index, n)| (index, n.position_metres))
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
        let index = self.ensure_node(level, point);
        for (target, elbow) in connections.into_iter().flatten() {
            let middle = self.ensure_node(level, elbow);
            self.connect(index, middle);
            self.connect(middle, target);
        }
        Some(index)
    }
    fn stair_landing(&mut self, level: u16, end: Vec2, outward: Vec2) -> Option<usize> {
        const LANDING_HALF_DEPTH_METRES: f32 = 0.45;
        const PORTAL_SEARCH_STEP_METRES: f32 = 0.01;
        let steps = ((LANDING_HALF_DEPTH_METRES - PERSON_RADIUS) / PORTAL_SEARCH_STEP_METRES).ceil()
            as usize;
        let mut accepted = Vec::new();
        for step in 0..=steps {
            let point = end + outward * (PERSON_RADIUS + step as f32 * PORTAL_SEARCH_STEP_METRES);
            if let Some(index) = self.insert_portal(level, point) {
                accepted.push(index);
            }
        }
        accepted
            .into_iter()
            .find(|&index| !self.edges[index].is_empty())
    }
    pub fn flood(&self, placements: &[InteriorPlacement]) -> Flood {
        let blocked = self
            .nodes
            .iter()
            .map(|n| {
                placements.iter().any(|p| {
                    p.storey == n.storey
                        && p.footprint()
                            .expanded(PERSON_RADIUS)
                            .contains(n.position_metres)
                })
            })
            .collect::<Vec<_>>();
        let mut parents = vec![None; self.nodes.len()];
        let mut queue = VecDeque::new();
        if !blocked[self.entry] {
            parents[self.entry] = Some(self.entry);
            queue.push_back(self.entry);
        }
        while let Some(index) = queue.pop_front() {
            for &next in &self.edges[index] {
                if blocked[next] || parents[next].is_some() {
                    continue;
                }
                let a = self.nodes[index];
                let b = self.nodes[next];
                let swept = Rect::new(
                    (a.position_metres + b.position_metres) * 0.5,
                    (a.position_metres - b.position_metres).abs() * 0.5
                        + Vec2::splat(PERSON_RADIUS),
                );
                if a.storey == b.storey
                    && placements
                        .iter()
                        .any(|p| p.storey == a.storey && p.footprint().overlaps(swept))
                {
                    continue;
                }
                parents[next] = Some(index);
                queue.push_back(next);
            }
        }
        Flood {
            parents,
            entry: self.entry,
        }
    }
    pub fn verify_rooms(&self, flood: &Flood) -> Result<(), InteriorLayoutError> {
        for (storey, room_id, nodes) in &self.room_nodes {
            if !nodes.iter().any(|&n| flood.parents[n].is_some()) {
                return Err(InteriorLayoutError::DisconnectedRoom {
                    storey: *storey,
                    room_id: *room_id,
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
            for &face in p.key.interior_spec().unwrap().required_faces {
                let access = p.access_rect(face);
                let target = self
                    .nodes
                    .iter()
                    .enumerate()
                    .filter(|(n, node)| {
                        node.storey == p.storey
                            && flood.parents[*n].is_some()
                            && access.contains(node.position_metres)
                    })
                    .min_by(|(_, a), (_, b)| {
                        a.position_metres
                            .distance_squared(access.centre)
                            .total_cmp(&b.position_metres.distance_squared(access.centre))
                    })
                    .map(|(n, _)| n)
                    .ok_or(InteriorLayoutError::InaccessibleFurniture { index })?;
                let mut cursor = target;
                let mut points = vec![self.nodes[cursor]];
                while cursor != flood.entry {
                    cursor = flood.parents[cursor].unwrap();
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
            let centre = Vec2::new(p.min.x + p.max.x, p.min.z + p.max.z) * 0.5;
            let inset = PERSON_RADIUS + crate::WALL_THICKNESS_METRES;
            if p.min.z <= GEOMETRY_EPSILON && p.max.x - p.min.x >= PERSON_RADIUS * 2.0 {
                Some((Vec2::new(centre.x, 0.0), Vec2::new(centre.x, inset)))
            } else if p.min.x <= GEOMETRY_EPSILON && p.max.z - p.min.z >= PERSON_RADIUS * 2.0 {
                Some((Vec2::new(0.0, centre.y), Vec2::new(inset, centre.y)))
            } else if p.max.z >= dimensions.y - GEOMETRY_EPSILON
                && p.max.x - p.min.x >= PERSON_RADIUS * 2.0
            {
                Some((
                    Vec2::new(centre.x, dimensions.y),
                    Vec2::new(centre.x, dimensions.y - inset),
                ))
            } else if p.max.x >= dimensions.x - GEOMETRY_EPSILON
                && p.max.z - p.min.z >= PERSON_RADIUS * 2.0
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
