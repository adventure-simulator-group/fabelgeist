fn generate_unchecked(
    program: &BuildingProgram,
    edits: &[BuildingEdit],
) -> Result<BuildingPlan, GenerationError> {
    program.validate_workplace_edits(edits)?;
    let (storeys, straight_stair_core) = occupied_storeys::generate_storeys(program, edits)?;

    let roofs = derive_roofs(program);
    let roof_dormers = derive_roof_dormers(program);
    let curtain_walls = derive_curtain_walls(program);
    let gatehouse_assemblies = derive_gatehouse_assemblies(program);
    let towers = derive_towers(program, &gatehouse_assemblies, &curtain_walls);
    let square_towers = derive_square_towers(program);
    let mut stairs = derive_stairs(program, &storeys, &towers, straight_stair_core.as_ref());
    let battlements = derive_battlements(program);
    let wall_walks = derive_wall_walks(program, &battlements, &towers);
    let crowns = derive_crowns(program, &battlements, &towers);
    let defensive_junctions = derive_defensive_junctions(&wall_walks);
    let defensive_circuits = derive_defensive_circuits(program, &wall_walks);
    let tower_portals = derive_tower_portals(program, &towers, &wall_walks, &defensive_junctions);
    let mut resolved_geometry =
        resolve_crown_geometry(&crowns, &wall_walks, &stairs, &tower_portals);
    let gate_defenses = derive_gate_defenses(
        program,
        &gatehouse_assemblies,
        &towers,
        &curtain_walls,
        &wall_walks,
    );
    let bartizans = derive_bartizans(program);
    let projected_defenses = resolve_projected_defenses(
        program,
        &storeys,
        &battlements,
        &bartizans,
        &mut resolved_geometry,
    );
    let (mut wall_assemblies, mut opening_assemblies) = resolve_storey_wall_assemblies(
        program,
        &storeys,
        &projected_defenses,
        &mut resolved_geometry,
    );
    let workplace = crate::workplace::resolve_workplace(program, &mut wall_assemblies, &mut resolved_geometry);
    let small_church = small_church::resolve(program, &mut wall_assemblies, &mut resolved_geometry);
    if program.archetype == BuildingArchetype::Cathedral {
        suppress_cathedral_legacy_storey_walls(
            &mut wall_assemblies,
            &mut opening_assemblies,
            &mut resolved_geometry,
        );
        resolve_cathedral_bell_stage(
            &square_towers,
            &mut wall_assemblies,
            &mut opening_assemblies,
            &mut resolved_geometry,
        );
    }
    let mut church = if program.archetype == BuildingArchetype::Cathedral {
        Some(resolve_church_assembly(
            program,
            &mut wall_assemblies,
            &mut opening_assemblies,
            &mut stairs,
            &mut resolved_geometry,
        ))
    } else {
        None
    };
    fortified_envelope::resolve(program, &towers, &crowns, &projected_defenses, &mut wall_assemblies, &mut opening_assemblies, &mut resolved_geometry);
    let artillery_castle = resolve_artillery_castle(
        program,
        &towers,
        &mut wall_assemblies,
        &mut opening_assemblies,
        &mut resolved_geometry,
    );

    let mut roof_assemblies = resolve_roof_assemblies(
        program,
        &roofs,
        &roof_dormers,
        &towers,
        &square_towers,
        &stairs,
        &wall_assemblies,
        &opening_assemblies,
        &mut resolved_geometry,
    );
    resolve_roof_child_front_openings(
        program,
        &roof_dormers,
        &mut roof_assemblies,
        &mut wall_assemblies,
        &mut opening_assemblies,
        &mut resolved_geometry,
    );
    let timber_frame = resolve_timber_frame_assembly(
        program,
        edits,
        &mut wall_assemblies,
        &opening_assemblies,
        &roofs,
        &roof_dormers,
        &mut stairs,
        &mut roof_assemblies,
        &mut resolved_geometry,
    );
    // Corner bonds must be resolved against the final timber-infill depth,
    // after the semantic frame has replaced the exterior structural layer.
    resolve_storey_wall_corner_bonds(&wall_assemblies, &mut resolved_geometry);
    if let Some(church) = &mut church {
        church.roof_assemblies = roof_assemblies.iter().map(|roof| roof.id).collect();
    }

    Ok(church_ground::resolve(crate::spiral_stairs::resolve(BuildingPlan {
        archetype: program.archetype,
        workplace,
        seed: program.seed,
        footprint: program.footprint,
        storey_height_metres: program.storey_height_metres,
        wall_style: program.wall_style,
        wall_style_overrides: Vec::new(),
        timber_frame_style: program.timber_frame_style,
        upper_storey_projection_metres: program.upper_storey_projection_metres,
        storeys,
        wall_assemblies,
        opening_assemblies,
        roofs,
        roof_dormers,
        roof_assemblies,
        towers,
        square_towers,
        stairs,
        battlements,
        crowns,
        projected_defenses,
        resolved_geometry,
        wall_walks,
        defensive_junctions,
        defensive_circuits,
        tower_portals,
        curtain_walls,
        gate_defenses,
        gatehouse_assemblies,
        bartizans,
        church,
        small_church,
        timber_frame,
        castle_phase: if program.archetype == BuildingArchetype::ArtilleryRondelCastle {
            Some(crate::CastleConstructionPhase::ArtilleryRetrofit1544)
        } else {
            matches!(
                program.archetype,
                BuildingArchetype::CastleGatehouse
                    | BuildingArchetype::CourtyardCastle
                    | BuildingArchetype::WalledKeep
            )
            .then_some(crate::CastleConstructionPhase::InheritedMedieval)
        },
        artillery_castle,
    })))
}

fn apply_opening_edits(
    _storey_program: &crate::StoreyProgram,
    level: u16,
    walls: &[crate::WallSegment],
    openings: &mut Vec<Opening>,
    edits: &[BuildingEdit],
) -> Result<(), GenerationError> {
    for edit in edits {
        let selector = match edit {
            BuildingEdit::AddOpening { wall, .. } | BuildingEdit::RemoveOpening { wall } => *wall,
            BuildingEdit::SetWallStyle { .. }
            | BuildingEdit::SetWallMaterial { .. }
            | BuildingEdit::SetTimberFrameStyle { .. } => {
                continue;
            }
        };
        if selector.storey_level != level {
            continue;
        }
        let wall_index = walls
            .iter()
            .position(|wall| wall.cell == selector.cell && wall.direction == selector.direction)
            .ok_or_else(|| {
                GenerationError::EditTargetNotFound(format!(
                    "storey {} cell ({}, {}) {:?} wall",
                    level, selector.cell.x, selector.cell.z, selector.direction
                ))
            })?;
        match *edit {
            BuildingEdit::AddOpening {
                opening_kind: kind,
                width_metres,
                sill_metres,
                height_metres,
                ..
            } => {
                if matches!(
                    kind,
                    OpeningKind::Window | OpeningKind::Gate | OpeningKind::ArrowSlit
                ) && !walls[wall_index].exterior()
                {
                    return Err(GenerationError::UnsupportedEdit(format!(
                        "{kind:?} openings require an exterior grid wall"
                    )));
                }
                if openings.iter().any(|opening| opening.wall == wall_index) {
                    return Err(GenerationError::EditConflict(format!(
                        "wall already owns an opening on storey {level}"
                    )));
                }
                let dimensions_are_valid = match kind {
                    OpeningKind::Window => {
                        (0.35..=1.20).contains(&width_metres)
                            && (0.30..=2.20).contains(&sill_metres)
                            && (0.45..=1.80).contains(&height_metres)
                    }
                    OpeningKind::Door => {
                        (0.70..=1.40).contains(&width_metres)
                            && sill_metres.abs() <= 0.01
                            && (1.80..=2.60).contains(&height_metres)
                    }
                    OpeningKind::Gate => {
                        (1.50..=3.80).contains(&width_metres)
                            && sill_metres.abs() <= 0.01
                            && (2.20..=3.40).contains(&height_metres)
                    }
                    OpeningKind::ArrowSlit => {
                        (0.15..=0.45).contains(&width_metres)
                            && (0.80..=1.80).contains(&sill_metres)
                            && (0.70..=1.50).contains(&height_metres)
                    }
                };
                if !dimensions_are_valid {
                    return Err(GenerationError::UnsupportedEdit(format!(
                        "{kind:?} dimensions are outside the editor project envelope"
                    )));
                }
                openings.push(Opening {
                    wall: wall_index,
                    kind,
                    width_metres,
                    sill_metres,
                    height_metres,
                });
                openings.sort_by_key(|opening| opening.wall);
            }
            BuildingEdit::RemoveOpening { .. } => {
                let before = openings.len();
                openings.retain(|opening| opening.wall != wall_index);
                if openings.len() == before {
                    return Err(GenerationError::EditTargetNotFound(format!(
                        "opening on storey {level} wall {wall_index}"
                    )));
                }
            }
            BuildingEdit::SetWallStyle { .. }
            | BuildingEdit::SetWallMaterial { .. }
            | BuildingEdit::SetTimberFrameStyle { .. } => {}
        }
    }
    Ok(())
}
