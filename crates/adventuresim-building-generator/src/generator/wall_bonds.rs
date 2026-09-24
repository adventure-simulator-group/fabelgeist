fn replace_storey_wall_sources_inside_round_towers(
    towers: &[RoundTower],
    walls: &mut [crate::WallAssembly],
    openings: &mut Vec<crate::OpeningAssembly>,
    geometry: &mut ResolvedGeometry,
) {
    let round_hosts = walls
        .iter()
        .filter_map(|wall| match wall.source {
            crate::WallSourceId::RoundTower { tower_index } => {
                Some((tower_index, wall.owner, wall.host_solids.first().copied()?))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut removed_owners = std::collections::HashSet::new();
    let mut replaced_wall_ids = std::collections::HashSet::new();
    for wall in walls.iter_mut() {
        if !matches!(wall.source, crate::WallSourceId::StoreyWall { .. }) {
            continue;
        }
        let Some((_, replacement_owner, replacement_host)) =
            round_hosts.iter().find(|(index, _, _)| {
                let tower = towers[*index];
                wall.frame.origin.distance(tower.centre_metres())
                    <= tower.radius_metres() + CELL_SIZE_METRES * 0.5
            })
        else {
            continue;
        };
        removed_owners.insert(wall.owner);
        replaced_wall_ids.insert(wall.id);
        wall.replaced_by_owner = Some(*replacement_owner);
        wall.host_solids = vec![*replacement_host];
        wall.opening_ids.clear();
    }
    let removed_opening_owners = openings
        .iter()
        .filter(|opening| replaced_wall_ids.contains(&opening.host_wall))
        .map(|opening| opening.owner)
        .collect::<std::collections::HashSet<_>>();
    openings.retain(|opening| !replaced_wall_ids.contains(&opening.host_wall));
    let removed = |owner: GeometryOwnerId| {
        removed_owners.contains(&owner) || removed_opening_owners.contains(&owner)
    };
    geometry.solids.retain(|solid| !removed(solid.owner));
    geometry.surfaces.retain(|surface| !removed(surface.owner));
    geometry.voids.retain(|void| !removed(void.owner));
    geometry
        .support_interfaces
        .retain(|interface| !removed(interface.owner));
    geometry
        .junction_bonds
        .retain(|bond| !bond.owners.iter().any(|owner| removed(*owner)));
}

fn resolve_gatehouse_tower_chord_bonds(
    towers: &[RoundTower],
    defenses: &[ProjectedDefenseAssembly],
    walls: &[crate::WallAssembly],
    geometry: &mut ResolvedGeometry,
) {
    for (tower_index, tower) in towers.iter().copied().enumerate() {
        let Some(round_wall) = walls.iter().find(|wall| {
            matches!(
                wall.source,
                crate::WallSourceId::RoundTower { tower_index: index } if index == tower_index
            )
        }) else {
            continue;
        };
        for (interface_index, interface) in tower.chord_interfaces().enumerate() {
            let toward = direction_vector(interface.toward_gate);
            let perpendicular = Vec2::new(-toward.y, toward.x);
            let radius = tower.radius_metres();
            let chord_offset = radius - interface.bearing_depth.metres();
            let half_chord = (radius * radius - chord_offset * chord_offset)
                .max(0.0)
                .sqrt();
            let point = tower.centre_metres() + toward * chord_offset;
            let defense = defenses.iter().find(|defense| {
                let ProjectedDefensePath::Linear { start, end, .. } = defense.path else {
                    return false;
                };
                let delta = end - start;
                let progress =
                    ((point - start).dot(delta) / delta.length_squared()).clamp(0.0, 1.0);
                point.distance(start + delta * progress) <= 0.08
            });
            let Some(defense) = defense else {
                continue;
            };
            let horizontal = toward.abs() * 0.035 + perpendicular.abs() * half_chord;
            for (slot, target_owner) in [defense.host_owner, defense.owner].into_iter().enumerate()
            {
                geometry.junction_bonds.push(JunctionBond {
                    id: ResolvedItemId(
                        (7_u64 << 60)
                            | (u64::from(round_wall.owner.0) << 20)
                            | ((interface_index as u64) << 4)
                            | (slot as u64 + 0x800),
                    ),
                    owners: [round_wall.owner, target_owner],
                    bounds: ResolvedBounds {
                        min: Vec3::new(point.x - horizontal.x, 0.0, point.y - horizontal.y),
                        max: Vec3::new(
                            point.x + horizontal.x,
                            tower.wall_height_metres,
                            point.y + horizontal.y,
                        ),
                    },
                    minimum_interface_area_square_metres: 0.08,
                    maximum_penetration_metres: 0.08,
                });
            }
        }
    }
}
