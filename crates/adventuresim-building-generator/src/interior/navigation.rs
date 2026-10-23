use super::architecture::Floor;
use super::geometry::*;
use super::{FurnitureAccessPath, InteriorLayoutError, InteriorPlacement, InteriorWaypoint};
use crate::BuildingPlan;
use crate::interior::InteriorResult as Result;
use crate::spatial_geometry::PlanExtents;
use crate::{StoreyIndex, plan_geometry::ArchitecturalPlanPoint};
use bevy::math::Vec2;
use std::collections::{BTreeMap, VecDeque};

mod occupancy;
mod portals;
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
    pub fn new(plan: &BuildingPlan) -> Result<Self> {
        let mut nav = Self::lattice(plan)?;
        nav.enter_front_door(plan)?;
        nav.add_doors(plan)?;
        nav.add_stairs(plan)?;
        for storey in &plan.storeys {
            for room in &storey.rooms {
                RoomBounds::from_room(room, StoreyIndex::from_serialized(storey.level))?;
                let mut ids = Vec::new();
                for (index, node) in nav.nodes.iter().enumerate() {
                    if node.storey == StoreyIndex::from_serialized(storey.level)
                        && Rect::new(
                            node.position_metres,
                            PlanExtents::from_metres(Vec2::splat(PERSON_RADIUS))?,
                        )?
                        .inside_room(room)?
                    {
                        ids.push(index);
                    }
                }
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
    fn lattice(plan: &BuildingPlan) -> Result<Self> {
        let floors = plan
            .storeys
            .iter()
            .map(|s| Floor::new(plan, StoreyIndex::from_serialized(s.level)))
            .collect::<Result<Vec<_>>>()?;
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
                        if floor.walkable(Rect::new(
                            ArchitecturalPlanPoint::from_metres(point)?,
                            PlanExtents::from_metres(Vec2::splat(PERSON_RADIUS))?,
                        )?)? {
                            lookup.insert((floor.level, gx, gz), nodes.len());
                            nodes.push(InteriorWaypoint {
                                storey: floor.level,
                                position_metres: ArchitecturalPlanPoint::try_from(point)?,
                            });
                        }
                    }
                }
            }
        }
        let mut edges = vec![Vec::new(); nodes.len()];
        for (&(level, x, z), &index) in &lookup {
            let floor = floors
                .iter()
                .find(|f| f.level == level)
                .ok_or(InteriorLayoutError::MissingStoreyFloor { storey: level })?;
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if let Some(&other) = lookup.get(&(level, x + dx, z + dz)) {
                    let a = nodes[index].position_metres.metres();
                    let b = nodes[other].position_metres.metres();
                    if floor.walkable(Rect::new(
                        ArchitecturalPlanPoint::from_metres((a + b) * 0.5)?,
                        PlanExtents::from_metres((a - b).abs() * 0.5 + Vec2::splat(PERSON_RADIUS))?,
                    )?)? {
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
    pub fn flood(&self, placements: &[InteriorPlacement]) -> Result<Flood> {
        let mut occupancy = Occupancy::new(self)?;
        occupancy.add(placements)?;
        Ok(occupancy.flood())
    }
    pub fn verify_rooms(&self, flood: &Flood) -> Result<()> {
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
    ) -> Result<Vec<FurnitureAccessPath>> {
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
                            && access.contains(node.position_metres)
                    })
                    .min_by(|(_, a), (_, b)| {
                        a.position_metres
                            .metres()
                            .distance_squared(access.centre.metres())
                            .total_cmp(
                                &b.position_metres
                                    .metres()
                                    .distance_squared(access.centre.metres()),
                            )
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
