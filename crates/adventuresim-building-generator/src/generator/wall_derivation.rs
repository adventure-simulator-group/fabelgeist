const RNG_BUILDING_WINDOW_PLACEMENT: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("building.window-placement");
fn derive_walls(
    footprint: &[Cell],
    assignments: &BTreeMap<Cell, RoomIndex>,
) -> Result<Vec<crate::WallSegment>, GenerationError> {
    let occupied = footprint.iter().copied().collect::<HashSet<_>>();
    let mut walls = Vec::new();
    for cell in footprint.iter().copied() {
        let assignment = |cell| {
            assignments
                .get(&cell)
                .copied()
                .ok_or(GenerationError::RoomAllocation(
                    room_allocation::AllocationFailure::MissingCellAssignment { cell },
                ))
        };
        let inside_room = assignment(cell)?.serialized_ordinal();
        for direction in Direction::ALL {
            let neighbour = cell.neighbour(direction);
            if !occupied.contains(&neighbour) {
                walls.push(crate::WallSegment {
                    cell,
                    direction,
                    inside_room,
                    outside_room: None,
                });
            } else if matches!(direction, Direction::North | Direction::East) {
                let other_room = assignment(neighbour)?.serialized_ordinal();
                if inside_room != other_room {
                    walls.push(crate::WallSegment {
                        cell,
                        direction,
                        inside_room,
                        outside_room: Some(other_room),
                    });
                }
            }
        }
    }
    Ok(walls)
}

fn derive_openings(
    walls: &[crate::WallSegment],
    requirements: &[RoomRequirement],
    archetype: BuildingArchetype,
    seed: u64,
    level: StoreyIndex,
    straight_stair_core: Option<&StraightStairCore>,
) -> Result<Vec<Opening>, GenerationError> {
    let storey = level;
    let serialized_level = storey.serialized_ordinal()?;
    let mut openings = Vec::new();
    let exterior_extent = GridEnvelope::from_walls(walls);
    let mut occupied_walls = HashSet::new();
    let mut required_room_connections = Vec::new();

    if let Some(core) = straight_stair_core.filter(|core| core.serves(serialized_level)) {
        let landing = core.landing_opening(walls, requirements, storey)?;
        occupied_walls.insert(landing.opening.wall);
        required_room_connections.push(room_connections::RoomConnection {
            left: landing.stair_hall,
            right: landing.neighbour,
        });
        openings.push(landing.opening);
    }

    if level == StoreyIndex::GROUND {
        entrances::append(
            walls,
            requirements,
            archetype,
            &mut openings,
            &mut occupied_walls,
        )?;
    }

    let edges = room_connections::shared_boundaries(walls, requirements, storey)?;
    let mut sets = DisjointSets::new(requirements.len());
    for connection in required_room_connections {
        sets.union(connection.left.index(), connection.right.index());
    }
    for edge in edges {
        if sets.union(edge.connection.left.index(), edge.connection.right.index()) {
            let wall_index = edge.candidates[edge.candidates.len() / 2];
            openings.push(Opening {
                wall: wall_index,
                kind: OpeningKind::Door,
                width_metres: 0.95,
                sill_metres: 0.0,
                height_metres: 2.1,
            });
            occupied_walls.insert(wall_index);
        }
    }
    if sets.component_count() != 1 {
        return Err(GenerationError::DisconnectedStorey { level });
    }

    for (wall_index, wall) in walls.iter().enumerate() {
        if !wall.exterior() || occupied_walls.contains(&wall_index) {
            continue;
        }
        // The two-post HallHouse MVP keeps its roof-carrying transverse
        // frames uninterrupted. The large hall doors remain opening-first;
        // optional ordinary lights are deferred rather than allowing a
        // seed-dependent window to cut a roof brace.
        if archetype == BuildingArchetype::HallHouse {
            continue;
        }
        // A one-cell opening at a perimeter corner consumes the return pier:
        // its jamb/reveal then occupies the perpendicular facade's frame
        // plane. Keep corner cells solid; nearby bays still provide light.
        let corner_cell = exterior_extent.is_some_and(|extent| extent.is_corner(wall.cell));
        if corner_cell {
            continue;
        }
        let room_kind = room_connections::requirement(
            requirements,
            RoomIndex::from_serialized(wall.inside_room),
            storey,
        )?
        .kind;
        if matches!(
            room_kind,
            RoomKind::Storage | RoomKind::Pantry | RoomKind::Passage
        ) || cell_random(
            seed,
            wall.direction as u64,
            wall.cell,
            RNG_BUILDING_WINDOW_PLACEMENT,
        )
        .index(3)
            == 0
        {
            continue;
        }
        let fortified = matches!(
            archetype,
            BuildingArchetype::CastleGatehouse
                | BuildingArchetype::CourtyardCastle
                | BuildingArchetype::WalledKeep
                | BuildingArchetype::ArtilleryRondelCastle
        );
        openings.push(Opening {
            wall: wall_index,
            kind: if fortified {
                OpeningKind::ArrowSlit
            } else {
                OpeningKind::Window
            },
            width_metres: if fortified { 0.18 } else { 0.85 },
            sill_metres: if fortified { 1.2 } else { 0.9 },
            height_metres: if fortified { 0.9 } else { 1.15 },
        });
    }

    openings.sort_by_key(|opening| opening.wall);
    Ok(openings)
}

fn two_centred_arc_radius(width_metres: f32, rise_metres: f32) -> f32 {
    let half_span = width_metres * 0.5;
    half_span + (rise_metres * rise_metres - half_span * half_span) / (2.0 * half_span.max(0.01))
}

fn opening_profile_for(
    archetype: BuildingArchetype,
    opening: Opening,
) -> (
    crate::OpeningUse,
    crate::OpeningProfile,
    crate::OpeningHeadKind,
) {
    match opening.kind {
        OpeningKind::Door => door_profile::resolve(archetype, opening),
        OpeningKind::Gate => (
            crate::OpeningUse::Gate,
            crate::OpeningProfile::Segmental {
                width_metres: opening.width_metres,
                spring_height_metres: (opening.height_metres - 0.28).max(1.8),
                rise_metres: 0.28,
                intrados_depth_metres: 0.24,
            },
            crate::OpeningHeadKind::SegmentalArch,
        ),
        OpeningKind::Window if archetype == BuildingArchetype::ParishChurch => (
            crate::OpeningUse::Window,
            crate::OpeningProfile::PointedTwoCentred {
                width_metres: opening.width_metres,
                spring_height_metres: opening.height_metres - 0.55,
                apex_height_metres: opening.height_metres,
                arc_radius_metres: two_centred_arc_radius(opening.width_metres, 0.55),
            },
            crate::OpeningHeadKind::PointedVoussoir,
        ),
        OpeningKind::Window if archetype == BuildingArchetype::Cathedral => (
            crate::OpeningUse::Window,
            crate::OpeningProfile::PointedTwoCentred {
                width_metres: 1.12,
                spring_height_metres: 3.0,
                apex_height_metres: 4.55,
                arc_radius_metres: two_centred_arc_radius(1.12, 4.55 - 3.0),
            },
            crate::OpeningHeadKind::PointedVoussoir,
        ),
        OpeningKind::Window if archetype == BuildingArchetype::RenaissanceTownHall => (
            crate::OpeningUse::Window,
            crate::OpeningProfile::Segmental {
                width_metres: 0.95,
                spring_height_metres: 1.0,
                rise_metres: 0.28,
                intrados_depth_metres: 0.18,
            },
            crate::OpeningHeadKind::SegmentalArch,
        ),
        OpeningKind::Window => (
            crate::OpeningUse::Window,
            crate::OpeningProfile::Rectangular {
                width_metres: opening.width_metres,
                height_metres: opening.height_metres,
            },
            crate::OpeningHeadKind::TimberLintel,
        ),
        OpeningKind::ArrowSlit
            if matches!(
                archetype,
                BuildingArchetype::WalledKeep | BuildingArchetype::ArtilleryRondelCastle
            ) =>
        {
            (
                crate::OpeningUse::GunLoop,
                crate::OpeningProfile::GunLoop {
                    exterior_width_metres: 0.20,
                    interior_width_metres: 0.92,
                    exterior_height_metres: 0.48,
                    interior_height_metres: 1.10,
                    mount: crate::WeaponMountClass::LightArquebus,
                    traverse_degrees: 28.0,
                    recoil_metres: 0.85,
                    crew_clearance_metres: 1.25,
                },
                crate::OpeningHeadKind::StoneLintel,
            )
        }
        OpeningKind::ArrowSlit => (
            crate::OpeningUse::ArrowLoop,
            crate::OpeningProfile::ArrowLoop {
                exterior_width_metres: 0.14,
                interior_width_metres: 0.68,
                exterior_height_metres: opening.height_metres,
                interior_height_metres: 1.18,
            },
            crate::OpeningHeadKind::StoneLintel,
        ),
    }
}

fn wall_solid(
    geometry: &mut ResolvedGeometry,
    owner: GeometryOwnerId,
    slot: u64,
    centre: Vec3,
    size: Vec3,
    role: SolidRole,
    shape: crate::ResolvedSolidShape,
    support: StructuralNodeId,
) -> Result<ResolvedItemId, crate::GenerationError> {
    let id = ResolvedItemId((1_u64 << 60) | (u64::from(owner.0) << 32) | slot);
    geometry.solids.push(ResolvedSolid::new(
        CollisionCuboid::<Architectural>::from_metres(id, centre, size, 0.0, 0.0, 0.0)?,
        owner,
        role,
        shape,
        vec![support],
    ));
    geometry
        .support_interfaces
        .push(crate::SupportInterface::new(
            ResolvedItemId((4_u64 << 60) | (u64::from(owner.0) << 32) | slot),
            owner,
            support,
            SpatialBounds::<Architectural>::from_metres(
                Vec3::new(
                    centre.x - size.x * 0.5,
                    centre.y - size.y * 0.5 - 0.015,
                    centre.z - size.z * 0.5,
                ),
                Vec3::new(
                    centre.x + size.x * 0.5,
                    centre.y - size.y * 0.5 + 0.015,
                    centre.z + size.z * 0.5,
                ),
            )?,
        ));
    Ok(id)
}

fn wall_void(
    geometry: &mut ResolvedGeometry,
    owner: GeometryOwnerId,
    slot: u64,
    bounds: SpatialBounds<Architectural>,
    opening: crate::OpeningAssemblyId,
    exterior_width_metres: f32,
    interior_width_metres: f32,
    exterior_height_metres: f32,
    interior_height_metres: f32,
    exterior_depth_sign: i8,
) -> ResolvedItemId {
    let id = ResolvedItemId((3_u64 << 60) | (u64::from(owner.0) << 32) | slot);
    geometry.voids.push(ResolvedVoid {
        id,
        owner,
        bounds,
        role: VoidRole::WallOpening,
        shape: crate::ResolvedVoidShape::SectionalOpening {
            opening,
            exterior_width_metres,
            interior_width_metres,
            exterior_height_metres,
            interior_height_metres,
            exterior_depth_sign,
        },
        subtracts_from: owner,
    });
    id
}

fn wall_shaped_surface(
    geometry: &mut ResolvedGeometry,
    owner: GeometryOwnerId,
    slot: u64,
    bounds: SpatialBounds<Architectural>,
    role: SurfaceRole,
    shape: crate::ResolvedSurfaceShape,
) -> ResolvedItemId {
    let id = ResolvedItemId((9_u64 << 60) | (u64::from(owner.0) << 32) | slot);
    geometry.surfaces.push(ResolvedSurface {
        id,
        owner,
        bounds,
        role,
        shape,
    });
    id
}

fn wall_surface(
    geometry: &mut ResolvedGeometry,
    owner: GeometryOwnerId,
    slot: u64,
    bounds: SpatialBounds<Architectural>,
    role: SurfaceRole,
) -> ResolvedItemId {
    wall_shaped_surface(
        geometry,
        owner,
        slot,
        bounds,
        role,
        crate::ResolvedSurfaceShape::Planar,
    )
}

/// Integer cell topology, before metre geometry admission.
#[derive(Clone, Copy)]
struct GridEnvelope {
    min_x: i16,
    max_x: i16,
    min_z: i16,
    max_z: i16,
}
impl GridEnvelope {
    fn from_walls(walls: &[crate::WallSegment]) -> Option<Self> {
        walls
            .iter()
            .filter(|wall| wall.exterior())
            .fold(None, |extent, wall| {
                let cell = wall.cell;
                Some(extent.map_or(
                    Self {
                        min_x: cell.x,
                        max_x: cell.x,
                        min_z: cell.z,
                        max_z: cell.z,
                    },
                    |e: Self| Self {
                        min_x: e.min_x.min(cell.x),
                        max_x: e.max_x.max(cell.x),
                        min_z: e.min_z.min(cell.z),
                        max_z: e.max_z.max(cell.z),
                    },
                ))
            })
    }
    fn is_corner(self, cell: Cell) -> bool {
        (cell.x == self.min_x || cell.x == self.max_x)
            && (cell.z == self.min_z || cell.z == self.max_z)
    }
}
