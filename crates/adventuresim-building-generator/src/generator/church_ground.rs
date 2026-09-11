//! Occupied cathedral rooms and continuous paving follow the resolved church envelope.
use super::*;
use geo::{Contains, Point};

const FLOOR_TILE_METRES: f32 = 0.15;
const FLOOR_THICKNESS_METRES: f32 = 0.20;
const FRAME_ROOM_SAMPLE_METRES: f32 = 0.30;
const FLOOR_BEARING_TOLERANCE_METRES: f32 = 0.015;
const FLOOR_SLOT_DOMAIN: u64 = 0x0d00_0000;
const ROOM_KINDS: [RoomKind; 5] = [
    RoomKind::Nave,
    RoomKind::Chancel,
    RoomKind::Chapel,
    RoomKind::Sacristy,
    RoomKind::EntranceHall,
];

pub(super) fn resolve(mut plan: BuildingPlan) -> BuildingPlan {
    let Some(church) = plan.church.as_ref() else {
        return plan;
    };
    let envelope = Envelope::new(&plan, church);
    let old_floors = church.floor_solids.iter().copied().collect::<BTreeSet<_>>();
    let source = plan
        .resolved_geometry
        .solids
        .iter()
        .find(|s| old_floors.contains(&s.id))
        .expect("church owns its ground paving")
        .clone();
    let tiles = envelope.tiles();
    let patches = merge_tiles(&tiles);
    plan.storeys = vec![occupied_storey(&tiles)];
    envelope.relabel_frames(&mut plan);
    replace_floor_solids(&mut plan, &source, &old_floors, &patches);
    plan
}

struct Envelope {
    ranges: [(Vec2, Vec2); 3],
    tower: (Vec2, Vec2),
    apse: Polygon<f32>,
}
impl Envelope {
    fn new(plan: &BuildingPlan, church: &crate::ChurchAssembly) -> Self {
        let ranges=[crate::ChurchRange::Nave,crate::ChurchRange::Choir,crate::ChurchRange::Transept].map(|range| {
            let vertices=plan.wall_assemblies.iter().filter(|wall|matches!(wall.source,crate::WallSourceId::ChurchExterior{range:r,..} if r==range)).flat_map(|wall|[wall.frame.origin-wall.frame.tangent*wall.length_metres*0.5,wall.frame.origin+wall.frame.tangent*wall.length_metres*0.5]);
            vertices.fold((Vec2::splat(f32::INFINITY),Vec2::splat(f32::NEG_INFINITY)),|(min,max),point|(min.min(point),max.max(point)))
        });
        let half = church.tower.footprint_size_metres * 0.5;
        let tower = (church.tower.centre - half, church.tower.centre + half);
        let mut vertices = Vec::new();
        for id in &church.choir.apse_facets {
            let wall = plan
                .wall_assemblies
                .iter()
                .find(|wall| wall.id == *id)
                .expect("apse facet belongs to church");
            if vertices.is_empty() {
                let p = wall.frame.origin - wall.frame.tangent * wall.length_metres * 0.5;
                vertices.push(Coord { x: p.x, y: p.y });
            }
            let p = wall.frame.origin + wall.frame.tangent * wall.length_metres * 0.5;
            vertices.push(Coord { x: p.x, y: p.y });
        }
        vertices.push(vertices[0]);
        Self {
            ranges,
            tower,
            apse: Polygon::new(LineString::new(vertices), Vec::new()),
        }
    }
    fn relabel_frames(&self, plan: &mut BuildingPlan) {
        // Cathedral assemblies are non-grid walls; obsolete Voronoi partitions
        // have no physical counterpart and are deliberately not regenerated.
        for wall in &mut plan.wall_assemblies {
            let offset =
                wall.frame.outward * (wall.thickness_metres * 0.5 + FRAME_ROOM_SAMPLE_METRES);
            wall.frame.inside_room = self.room(wall.frame.origin - offset);
            wall.frame.outside_room = self.room(wall.frame.origin + offset);
        }
        for opening in &mut plan.opening_assemblies {
            let wall = plan
                .wall_assemblies
                .iter()
                .find(|w| w.id == opening.host_wall)
                .expect("church opening belongs to a resolved wall");
            let offset =
                opening.frame.outward * (wall.thickness_metres * 0.5 + FRAME_ROOM_SAMPLE_METRES);
            opening.frame.inside_room = self.room(opening.frame.origin - offset);
            opening.frame.outside_room = self.room(opening.frame.origin + offset);
        }
    }
    fn room(&self, point: Vec2) -> Option<u16> {
        let inside = |(min, max): (Vec2, Vec2)| point.cmpge(min).all() && point.cmple(max).all();
        if inside(self.tower) {
            return Some(4);
        }
        if inside(self.ranges[1]) || self.apse.contains(&Point::new(point.x, point.y)) {
            return Some(1);
        }
        if inside(self.ranges[2]) {
            if point.y < self.ranges[1].0.y {
                return Some(2);
            }
            if point.y > self.ranges[1].1.y {
                return Some(3);
            }
            return Some(0);
        }
        inside(self.ranges[0]).then_some(0)
    }
    fn tiles(&self) -> BTreeMap<(i32, i32), u16> {
        let max = self
            .ranges
            .iter()
            .map(|(_, max)| *max)
            .chain(self.apse.exterior().0.iter().map(|p| Vec2::new(p.x, p.y)))
            .fold(self.tower.1, Vec2::max);
        let steps = (max / FLOOR_TILE_METRES).ceil().as_ivec2();
        let mut result = BTreeMap::new();
        for z in 0..steps.y {
            for x in 0..steps.x {
                let point = (Vec2::new(x as f32, z as f32) + Vec2::splat(0.5)) * FLOOR_TILE_METRES;
                if let Some(room) = self.room(point) {
                    result.insert((x, z), room);
                }
            }
        }
        result
    }
}

/// Fine paving boundaries remain hidden inside the thick masonry; adjacent runs share exact edges.
fn merge_tiles(tiles: &BTreeMap<(i32, i32), u16>) -> Vec<(Vec2, Vec2)> {
    let max_z = tiles.keys().map(|(_, z)| *z).max().unwrap_or(0);
    let max_x = tiles.keys().map(|(x, _)| *x).max().unwrap_or(0);
    let mut active = BTreeMap::<(i32, i32), (i32, i32)>::new();
    let mut patches = Vec::new();
    for z in 0..=max_z + 1 {
        let mut runs = BTreeSet::new();
        let mut start = None;
        for x in 0..=max_x + 1 {
            if tiles.contains_key(&(x, z)) {
                start.get_or_insert(x);
            } else if let Some(first) = start.take() {
                runs.insert((first, x));
            }
        }
        let finished = active
            .keys()
            .copied()
            .filter(|key| !runs.contains(key))
            .collect::<Vec<_>>();
        for key in finished {
            let (first, last) = active.remove(&key).unwrap();
            patches.push((
                Vec2::new(key.0 as f32, first as f32) * FLOOR_TILE_METRES,
                Vec2::new(key.1 as f32, (last + 1) as f32) * FLOOR_TILE_METRES,
            ));
        }
        for run in runs {
            active.entry(run).and_modify(|v| v.1 = z).or_insert((z, z));
        }
    }
    patches
}

fn occupied_storey(tiles: &BTreeMap<(i32, i32), u16>) -> StoreyPlan {
    let tiles_per_cell = (CELL_SIZE_METRES / FLOOR_TILE_METRES).round() as i32;
    let mut votes = BTreeMap::<Cell, [usize; 5]>::new();
    for (&(x, z), &room) in tiles {
        votes
            .entry(Cell::new(
                (x / tiles_per_cell) as i16,
                (z / tiles_per_cell) as i16,
            ))
            .or_default()[usize::from(room)] += 1;
    }
    let mut cells: [Vec<Cell>; 5] = std::array::from_fn(|_| Vec::new());
    for (cell, votes) in votes {
        let room = votes
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| *count)
            .unwrap()
            .0;
        cells[room].push(cell);
    }
    StoreyPlan {
        level: 0,
        rooms: ROOM_KINDS
            .into_iter()
            .enumerate()
            .map(|(id, kind)| Room {
                id: id as u16,
                kind,
                cells: std::mem::take(&mut cells[id]),
            })
            .filter(|room| !room.cells.is_empty())
            .collect(),
        walls: Vec::new(),
        openings: Vec::new(),
    }
}

fn replace_floor_solids(
    plan: &mut BuildingPlan,
    source: &ResolvedSolid,
    old: &BTreeSet<ResolvedItemId>,
    patches: &[(Vec2, Vec2)],
) {
    let old_interfaces = old
        .iter()
        .map(|id| ResolvedItemId((4_u64 << 60) | ((id.0 & ((1_u64 << 60) - 1)) + 1)))
        .collect::<BTreeSet<_>>();
    plan.resolved_geometry
        .solids
        .retain(|s| !old.contains(&s.id));
    plan.resolved_geometry
        .support_interfaces
        .retain(|s| !old_interfaces.contains(&s.id));
    let mut floors = Vec::new();
    for (index, &(min, max)) in patches.iter().enumerate() {
        let id = ResolvedItemId(
            (1_u64 << 60) | (u64::from(source.owner.0) << 32) | FLOOR_SLOT_DOMAIN | index as u64,
        );
        let centre = (min + max) * 0.5;
        plan.resolved_geometry.solids.push(ResolvedSolid {
            id,
            owner: source.owner,
            centre: Vec3::new(centre.x, -FLOOR_THICKNESS_METRES * 0.5, centre.y),
            size: Vec3::new(max.x - min.x, FLOOR_THICKNESS_METRES, max.y - min.y),
            yaw_radians: 0.0,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
            role: SolidRole::ChurchFloor,
            shape: crate::ResolvedSolidShape::Cuboid,
            supported_by: source.supported_by.clone(),
        });
        for &node in &source.supported_by {
            plan.resolved_geometry
                .support_interfaces
                .push(SupportInterface {
                    id: ResolvedItemId(
                        (4_u64 << 60)
                            | (u64::from(source.owner.0) << 32)
                            | FLOOR_SLOT_DOMAIN
                            | index as u64,
                    ),
                    owner: source.owner,
                    node,
                    bounds: ResolvedBounds {
                        min: Vec3::new(
                            min.x,
                            -FLOOR_THICKNESS_METRES - FLOOR_BEARING_TOLERANCE_METRES,
                            min.y,
                        ),
                        max: Vec3::new(
                            max.x,
                            -FLOOR_THICKNESS_METRES + FLOOR_BEARING_TOLERANCE_METRES,
                            max.y,
                        ),
                    },
                });
        }
        floors.push(id);
    }
    let church = plan.church.as_mut().unwrap();
    church.floor_solids = floors.clone();
    church.choir.floor_solids = floors;
}
