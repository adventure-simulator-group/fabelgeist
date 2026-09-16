fn resolve_roof_assemblies(
    program: &BuildingProgram,
    roofs: &[RoofPiece],
    dormers: &[RoofDormer],
    towers: &[RoundTower],
    square_towers: &[SquareTower],
    stairs: &[Stair],
    walls: &[crate::WallAssembly],
    openings: &[crate::OpeningAssembly],
    geometry: &mut ResolvedGeometry,
) -> Result<Vec<RoofAssembly>, GenerationError> {
    let mut assemblies = Vec::new();
    for (index, roof) in roofs.iter().copied().enumerate() {
        let id = RoofAssemblyId(index as u64 + 1);
        let shed_high_side = match (program.church_program.is_some(), index, roof.kind) {
            (true, 1, RoofKind::Shed) => Some(Direction::North),
            (true, 2, RoofKind::Shed) => Some(Direction::South),
            (_, _, RoofKind::Shed) => Some(match roof.ridge_axis {
                RidgeAxis::Z => Direction::East,
                RidgeAxis::X => Direction::North,
            }),
            _ => None,
        };
        assemblies.push(resolve_one_roof(
            id,
            GeometryOwnerId(60_000 + index as u32),
            roof,
            Some(index),
            None,
            None,
            RoofPhase::Primary,
            shed_high_side,
            None,
            walls,
            geometry,
        ));
    }
    if let Some(parent) = assemblies
        .first()
        .map(|assembly| (assembly.id, assembly.owner))
    {
        for index in 1..roofs.len() {
            let child_recipe = roofs[index];
            let parent_recipe = roofs[0];
            let child_min = child_recipe.centre - child_recipe.size * 0.5;
            let child_max = child_recipe.centre + child_recipe.size * 0.5;
            let parent_min = parent_recipe.centre - parent_recipe.size * 0.5;
            let parent_max = parent_recipe.centre + parent_recipe.size * 0.5;
            let overlaps = child_min.x < parent_max.x
                && child_max.x > parent_min.x
                && child_min.y < parent_max.y
                && child_max.y > parent_min.y;
            if overlaps && child_recipe.base_height_metres > parent_recipe.base_height_metres + 0.5
            {
                let child_id = assemblies[index].id;
                assemblies[index].parent = Some(parent.0);
                assemblies[index].phase = RoofPhase::AttachedChild;
                let belfry_top = small_church::belfry_enclosure_top(program, index);
                let enclosure_material = if belfry_top.is_some()
                    || walls
                        .iter()
                        .any(|wall| wall.material == crate::WallMaterialClass::TimberInfill)
                {
                    RoofMaterial::TimberInfill
                } else {
                    RoofMaterial::MasonryInfill
                };
                let top = belfry_top.unwrap_or(child_recipe.base_height_metres);
                let parent_snapshot = assemblies[0].clone();
                roof_child_enclosure::append(
                    &mut assemblies[index],
                    &parent_snapshot,
                    child_recipe,
                    parent_recipe.base_height_metres,
                    top,
                    enclosure_material,
                );
                let cut_id = ResolvedItemId((0xF_u64 << 60) | child_id.0);
                let bounds = ResolvedBounds {
                    min: Vec3::new(
                        child_min.x,
                        parent_recipe.base_height_metres - 0.2,
                        child_min.y,
                    ),
                    max: Vec3::new(
                        child_max.x,
                        child_recipe.base_height_metres + 5.0,
                        child_max.y,
                    ),
                };
                geometry.voids.push(ResolvedVoid {
                    id: cut_id,
                    owner: parent.1,
                    bounds,
                    role: VoidRole::RoofOpening,
                    shape: crate::ResolvedVoidShape::Box,
                    subtracts_from: parent.1,
                });
                let child_supports = assemblies[index].support_nodes.clone();
                let child_copy = assemblies[index].clone();
                roof_internal_cut::split_internal_edges(&mut assemblies[0], bounds, geometry);
                trim_roof_edge_treatments_for_cut(assemblies[0].owner, bounds, geometry);
                trim_roof_boundary_edges_for_cut(&mut assemblies[0], bounds);
                let cut_edges =
                    cut_parent_roof_face(&mut assemblies[0], &child_copy, bounds, geometry);
                let valleys =
                    bind_child_valleys(&mut assemblies[0], &child_copy, &cut_edges, geometry);
                let flashing_ids = assemblies[0]
                    .edges
                    .iter()
                    .filter(|edge| cut_edges.contains(&edge.id))
                    .filter_map(|edge| edge.flashing)
                    .collect();
                assemblies[0].children.push(RoofChildAssembly {
                    child: child_id,
                    kind: RoofChildKind::CrossGable,
                    parent_cut: cut_id,
                    trimmer_nodes: child_supports,
                    valley_edges: valleys,
                    flashing_ids,
                    facade_wall: None,
                    split_eave_edges: Vec::new(),
                });
            }
        }
    }
    roof_dormers::append(&mut assemblies, dormers, walls, geometry)?;
    for (index, tower) in towers.iter().copied().enumerate() {
        if let Some(roof) = tower.roof {
            let id = RoofAssemblyId(2_000 + index as u64);
            assemblies.push(resolve_one_roof(
                id,
                GeometryOwnerId(62_000 + index as u32),
                roof,
                None,
                Some(index),
                None,
                RoofPhase::Primary,
                None,
                None,
                walls,
                geometry,
            ));
        }
    }
    for (index, tower) in square_towers.iter().copied().enumerate() {
        let id = RoofAssemblyId(3_000 + index as u64);
        assemblies.push(resolve_one_roof(
            id,
            GeometryOwnerId(63_000 + index as u32),
            tower.roof,
            None,
            Some(index),
            None,
            RoofPhase::Primary,
            None,
            None,
            walls,
            geometry,
        ));
    }
    // A tower piercing the principal roof is a true abutment, not two
    // overlapping independent meshes. Cut the main weather faces to the
    // tower footprint, flash every resulting edge, and bind the tower roof as
    // a child carried by its own masonry ring.
    if !square_towers.is_empty() && !assemblies.is_empty() {
        for (tower_index, tower) in square_towers.iter().copied().enumerate() {
            let Some(child_index) = assemblies
                .iter()
                .position(|roof| roof.id == RoofAssemblyId(3_000 + tower_index as u64))
            else {
                continue;
            };
            let child = assemblies[child_index].clone();
            let parent_id = assemblies[0].id;
            let cut_id = ResolvedItemId((0xF_u64 << 60) | child.id.0);
            // `SquareTower::size` locates the four authoritative wall
            // centrelines.  The parent roof must stop at the exterior shell
            // faces, not halfway through the masonry.
            let shell_half_thickness = walls
                .iter()
                .filter_map(|wall| match wall.source {
                    crate::WallSourceId::SquareTowerFace {
                        tower_index: source_tower,
                        ..
                    } if source_tower == tower_index => Some(wall.thickness_metres * 0.5),
                    _ => None,
                })
                .fold(0.0_f32, f32::max);
            let half = tower.size * 0.5 + Vec2::splat(shell_half_thickness);
            let bounds = ResolvedBounds {
                min: Vec3::new(
                    tower.centre.x - half.x,
                    assemblies[0]
                        .faces
                        .iter()
                        .flat_map(|face| face.polygon.iter().map(|point| point.y))
                        .fold(f32::INFINITY, f32::min)
                        - 0.2,
                    tower.centre.y - half.y,
                ),
                max: Vec3::new(
                    tower.centre.x + half.x,
                    tower.wall_height_metres + 8.0,
                    tower.centre.y + half.y,
                ),
            };
            geometry.voids.push(ResolvedVoid {
                id: cut_id,
                owner: assemblies[0].owner,
                bounds,
                role: VoidRole::RoofOpening,
                shape: crate::ResolvedVoidShape::Box,
                subtracts_from: assemblies[0].owner,
            });
            let cut_height = bounds.max.y;
            let mut vertical_cut = child.clone();
            let mut cut_face = child.faces[0].clone();
            cut_face.polygon = vec![
                Vec3::new(bounds.min.x, cut_height, bounds.min.z),
                Vec3::new(bounds.max.x, cut_height, bounds.min.z),
                Vec3::new(bounds.max.x, cut_height, bounds.max.z),
                Vec3::new(bounds.min.x, cut_height, bounds.max.z),
            ];
            cut_face.cutouts.clear();
            cut_face.plane = RoofPlaneEquation {
                normal: Vec3::Y,
                constant: -cut_height,
            };
            vertical_cut.faces = vec![cut_face];
            trim_roof_edge_treatments_for_cut(assemblies[0].owner, bounds, geometry);
            trim_roof_boundary_edges_for_cut(&mut assemblies[0], bounds);
            let cut_edges =
                cut_parent_roof_face(&mut assemblies[0], &vertical_cut, bounds, geometry);
            for edge in assemblies[0]
                .edges
                .iter_mut()
                .filter(|edge| cut_edges.contains(&edge.id))
            {
                edge.kind = RoofEdgeKind::TowerAbutment;
            }
            let flashing_ids = assemblies[0]
                .edges
                .iter()
                .filter(|edge| cut_edges.contains(&edge.id))
                .filter_map(|edge| edge.flashing)
                .collect::<Vec<_>>();
            assemblies[0].children.push(RoofChildAssembly {
                child: child.id,
                kind: RoofChildKind::Tower,
                parent_cut: cut_id,
                trimmer_nodes: child.support_nodes.clone(),
                valley_edges: cut_edges,
                flashing_ids,
                facade_wall: None,
                split_eave_edges: Vec::new(),
            });
            assemblies[child_index].parent = Some(parent_id);
            assemblies[child_index].phase = RoofPhase::AttachedChild;
        }
    }
    bind_coincident_primary_roof_edges(&mut assemblies, geometry);
    finalize_roof_drainage(program.archetype, &mut assemblies, geometry);
    supplement_split_eave_drainage(&assemblies, geometry);
    consolidate_roof_outlet_stations(
        program.archetype,
        &mut assemblies,
        stairs,
        walls,
        openings,
        geometry,
    );
    resolve_roof_abutment_contours(&mut assemblies, walls, geometry);
    refit_roof_edge_treatments(&mut assemblies, geometry);
    bind_roof_junctions(&assemblies, geometry);
    Ok(assemblies)
}
