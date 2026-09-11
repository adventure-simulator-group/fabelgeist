use super::*;

pub(super) fn build(dimensions: Dimensions) -> StoreyPlan {
    let width = dimensions.width_cells as i16;
    let nave_depth = dimensions.nave_cells as i16;
    let depth = (dimensions.nave_cells + dimensions.chancel_cells) as i16;
    let mut assignments = BTreeMap::new();
    for z in 0..depth {
        for x in 0..width {
            if z < nave_depth {
                assignments.insert(Cell::new(x, z), 0);
            } else if x > 0 && x < width - 1 {
                assignments.insert(Cell::new(x, z), 1);
            }
        }
    }
    let cells = assignments.keys().copied().collect::<Vec<_>>();
    let mut walls = derive_walls(&cells, &assignments);
    // The chancel connects through a real 4.5 m opening. Its upper bearing band is resolved separately.
    walls.retain(|wall| !(wall.outside_room.is_some() && (wall.cell.x - width / 2).abs() <= 1));
    let mut openings = Vec::new();
    for (index, wall) in walls.iter().enumerate() {
        if !wall.exterior() {
            continue;
        }
        let front_door =
            wall.direction == Direction::South && wall.cell.x == width / 2 && wall.cell.z == 0;
        let long_side_window = matches!(wall.direction, Direction::East | Direction::West)
            && wall.cell.z % 2 == 1
            && wall.cell.z < depth - 1;
        let east_window = wall.direction == Direction::North
            && wall.cell.x == width / 2
            && wall.cell.z == depth - 1;
        if front_door || long_side_window || east_window {
            openings.push(Opening {
                wall: index,
                kind: if front_door {
                    OpeningKind::Door
                } else {
                    OpeningKind::Window
                },
                width_metres: if front_door { 1.3 } else { 0.85 },
                sill_metres: if front_door { 0.0 } else { 1.1 },
                height_metres: if front_door {
                    2.4
                } else if dimensions.kind == SmallChurchKind::Parish && wall.cell.z < nave_depth {
                    3.1
                } else {
                    2.1
                },
            });
        }
    }
    let mut rooms = vec![Room {
        id: 0,
        kind: RoomKind::Nave,
        cells: assignments
            .iter()
            .filter_map(|(&cell, &room)| (room == 0).then_some(cell))
            .collect(),
    }];
    if dimensions.chancel_cells > 0 {
        rooms.push(Room {
            id: 1,
            kind: RoomKind::Chancel,
            cells: assignments
                .iter()
                .filter_map(|(&cell, &room)| (room == 1).then_some(cell))
                .collect(),
        });
    }
    StoreyPlan {
        level: 0,
        rooms,
        walls,
        openings,
    }
}
