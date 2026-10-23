fn floor_for_artillery_level(level: crate::ArtilleryStationLevel) -> f32 {
    if level == crate::ArtilleryStationLevel::LowerCasemate {
        0.20
    } else {
        5.86
    }
}

fn retaining_support_node(
    curtains: &[crate::ArtilleryCurtainAssembly],
    geometry: &ResolvedGeometry,
) -> Result<StructuralNodeId, crate::GenerationError> {
    let curtain = crate::ArtilleryCurtainId(1);
    let owner = curtains
        .iter()
        .find(|candidate| candidate.id == curtain)
        .ok_or(crate::GenerationError::MissingArtilleryCurtain { curtain })?
        .owner;
    geometry
        .structural_nodes
        .iter()
        .find(|node| {
            node.owner == owner && node.kind == StructuralNodeKind::ArtilleryRetainingBearing
        })
        .map(|node| node.id)
        .ok_or(crate::GenerationError::MissingBearing {
            owner,
            kind: StructuralNodeKind::ArtilleryRetainingBearing,
        })
}

fn resolve_artillery_gun_opening(
    rondel_index: usize,
    station_index: usize,
    facing: Vec2,
    level: crate::ArtilleryStationLevel,
    opening_id: crate::OpeningAssemblyId,
    wall: &mut crate::WallAssembly,
    openings: &mut Vec<crate::OpeningAssembly>,
    geometry: &mut ResolvedGeometry,
) -> Result<crate::ArtilleryFireStation, crate::GenerationError> {
    let owner = GeometryOwnerId(83_000 + (rondel_index * 3 + station_index) as u32);
    let tangent = Vec2::new(-facing.y, facing.x);
    let radius = 6.0_f32;
    let centre = wall
        .radial_frame
        .ok_or(crate::GenerationError::MissingRadialFrame { wall: wall.id })?
        .centre;
    let origin = centre + facing * radius;
    let floor = if level == crate::ArtilleryStationLevel::LowerCasemate {
        0.20
    } else {
        5.86
    };
    let sill = floor
        + if level == crate::ArtilleryStationLevel::LowerCasemate {
            0.82
        } else {
            0.25
        };
    let thickness = wall.thickness_metres;
    let exterior_width = 0.28_f32;
    let interior_width = 1.10_f32;
    let exterior_height = 0.56_f32;
    let interior_height = 1.20_f32;
    let node_base = 43_000_000 + (rondel_index * 24 + station_index * 6) as u64;
    let jamb_nodes = [StructuralNodeId(node_base), StructuralNodeId(node_base + 1)];
    let head_node = StructuralNodeId(node_base + 2);
    let spandrel_node = StructuralNodeId(node_base + 3);
    opening_jambs::OpeningJambSetOut::from_metres(
        jamb_nodes,
        owner,
        wall.support_node,
        origin,
        tangent,
        interior_width,
        floor,
    )?
    .append_nodes(geometry)?;
    geometry.structural_nodes.extend(
        opening_jambs::OpeningHeadSetOut::from_metres(
            [head_node, spandrel_node],
            owner,
            jamb_nodes,
            origin,
            sill + interior_height,
            sill + interior_height + 0.2,
        )?
        .nodes()?,
    );
    wall.frame = crate::WallLocalFrame {
        origin,
        tangent,
        outward: facing,
        inside_room: None,
        outside_room: None,
    };
    wall.base_elevation_metres = floor;
    wall.height_metres = 2.45;
    let yaw = -tangent.y.atan2(tangent.x);
    let side_width = (wall.length_metres - exterior_width) * 0.5;
    let jamb = [-1.0_f32, 1.0]
        .into_iter()
        .enumerate()
        .map(|(index, side)| {
            let p = origin + tangent * side * (exterior_width + side_width) * 0.5;
            let id = wall_solid(
                geometry,
                owner,
                index as u64,
                Vec3::new(p.x, floor + wall.height_metres * 0.5, p.y),
                Vec3::new(side_width, wall.height_metres, thickness),
                SolidRole::OpeningJamb,
                crate::ResolvedSolidShape::SplayedReveal {
                    exterior_width_metres: exterior_width,
                    interior_width_metres: interior_width,
                    side: if side < 0.0 { -1 } else { 1 },
                    exterior_depth_sign: if tangent.x.abs() > 0.5 {
                        if facing.y >= 0.0 { 1 } else { -1 }
                    } else if facing.x <= 0.0 {
                        1
                    } else {
                        -1
                    },
                },
                if side < 0.0 {
                    jamb_nodes[0]
                } else {
                    jamb_nodes[1]
                },
            )?;
            geometry.solid_mut(id)?.yaw_radians = Radians::new(yaw)?;
            Ok(id)
        })
        .collect::<Result<Vec<_>, crate::GenerationError>>()?;
    let jamb = [jamb[0], jamb[1]];
    let depth_sign = if tangent.x.abs() > 0.5 {
        if facing.y >= 0.0 { 1 } else { -1 }
    } else if facing.x <= 0.0 {
        1
    } else {
        -1
    };
    let head_bottom = sill + exterior_height;
    let head_top = sill + interior_height + 0.20;
    let head = wall_solid(
        geometry,
        owner,
        2,
        Vec3::new(origin.x, (head_bottom + head_top) * 0.5, origin.y),
        Vec3::new(interior_width + 0.20, head_top - head_bottom, thickness),
        SolidRole::OpeningHead,
        crate::ResolvedSolidShape::SplayedHead {
            exterior_clear_height_metres: exterior_height,
            interior_clear_height_metres: interior_height,
            exterior_depth_sign: depth_sign,
        },
        head_node,
    )?;
    let spandrel = wall_solid(
        geometry,
        owner,
        3,
        Vec3::new(origin.x, head_top + 0.10, origin.y),
        Vec3::new(interior_width + 0.20, 0.24, thickness),
        SolidRole::OpeningSpandrel,
        crate::ResolvedSolidShape::Cuboid,
        spandrel_node,
    )?;
    for id in [head, spandrel] {
        geometry.solid_mut(id)?.yaw_radians = Radians::new(yaw)?;
    }
    let half_t = tangent.abs() * interior_width * 0.5;
    let half_d = facing.abs() * thickness * 0.6;
    let void_id = wall_void(
        geometry,
        owner,
        0,
        SpatialBounds::<Architectural>::from_metres(
            Vec3::new(
                origin.x - half_t.x - half_d.x,
                sill,
                origin.y - half_t.y - half_d.y,
            ),
            Vec3::new(
                origin.x + half_t.x + half_d.x,
                sill + interior_height,
                origin.y + half_t.y + half_d.y,
            ),
        )?,
        opening_id,
        exterior_width,
        interior_width,
        exterior_height,
        interior_height,
        depth_sign,
    );
    let left = wall_shaped_surface(
        geometry,
        owner,
        10,
        SpatialBounds::<Architectural>::from_metres(
            Vec3::new(
                origin.x - half_t.x - half_d.x,
                sill,
                origin.y - half_t.y - half_d.y,
            ),
            Vec3::new(
                origin.x + half_t.x + half_d.x,
                sill + interior_height,
                origin.y + half_t.y + half_d.y,
            ),
        )?,
        SurfaceRole::LeftJambReveal,
        crate::ResolvedSurfaceShape::SplayedJamb {
            side: -1,
            exterior_width_metres: exterior_width,
            interior_width_metres: interior_width,
            exterior_depth_sign: 1,
        },
    );
    let right = wall_shaped_surface(
        geometry,
        owner,
        11,
        SpatialBounds::<Architectural>::from_metres(
            Vec3::new(
                origin.x - half_t.x - half_d.x,
                sill,
                origin.y - half_t.y - half_d.y,
            ),
            Vec3::new(
                origin.x + half_t.x + half_d.x,
                sill + interior_height,
                origin.y + half_t.y + half_d.y,
            ),
        )?,
        SurfaceRole::RightJambReveal,
        crate::ResolvedSurfaceShape::SplayedJamb {
            side: 1,
            exterior_width_metres: exterior_width,
            interior_width_metres: interior_width,
            exterior_depth_sign: depth_sign,
        },
    );
    if let Some(surface) = geometry
        .surfaces
        .iter_mut()
        .find(|surface| surface.id == left)
    {
        surface.shape = crate::ResolvedSurfaceShape::SplayedJamb {
            side: -1,
            exterior_width_metres: exterior_width,
            interior_width_metres: interior_width,
            exterior_depth_sign: depth_sign,
        };
    }
    let weather = wall_shaped_surface(
        geometry,
        owner,
        12,
        SpatialBounds::<Architectural>::from_metres(
            Vec3::new(
                origin.x - half_t.x - half_d.x,
                sill - 0.04,
                origin.y - half_t.y - half_d.y,
            ),
            Vec3::new(
                origin.x + half_t.x + half_d.x,
                sill + 0.02,
                origin.y + half_t.y + half_d.y,
            ),
        )?,
        SurfaceRole::WeatherSill,
        crate::ResolvedSurfaceShape::WeatherSill {
            interior_elevation_metres: sill,
            exterior_elevation_metres: sill - 0.04,
            drip_depth_metres: 0.03,
        },
    );
    let intrados = wall_shaped_surface(
        geometry,
        owner,
        13,
        SpatialBounds::<Architectural>::from_metres(
            Vec3::new(
                origin.x - half_t.x - half_d.x,
                sill + exterior_height,
                origin.y - half_t.y - half_d.y,
            ),
            Vec3::new(
                origin.x + half_t.x + half_d.x,
                sill + interior_height,
                origin.y + half_t.y + half_d.y,
            ),
        )?,
        SurfaceRole::Intrados,
        crate::ResolvedSurfaceShape::Planar,
    );
    let exterior_plan = origin + facing * thickness * 0.5;
    let interior_plan = origin - facing * thickness * 0.5;
    let throat = wall_shaped_surface(
        geometry,
        owner,
        14,
        SpatialBounds::<Architectural>::from_metres(
            Vec3::new(
                exterior_plan.x - tangent.x.abs() * exterior_width * 0.5 - facing.x.abs() * 0.006,
                sill,
                exterior_plan.y - tangent.y.abs() * exterior_width * 0.5 - facing.y.abs() * 0.006,
            ),
            Vec3::new(
                exterior_plan.x + tangent.x.abs() * exterior_width * 0.5 + facing.x.abs() * 0.006,
                sill + exterior_height,
                exterior_plan.y + tangent.y.abs() * exterior_width * 0.5 + facing.y.abs() * 0.006,
            ),
        )?,
        SurfaceRole::ExteriorThroat,
        crate::ResolvedSurfaceShape::Planar,
    );
    let mouth = wall_shaped_surface(
        geometry,
        owner,
        15,
        SpatialBounds::<Architectural>::from_metres(
            Vec3::new(
                interior_plan.x - tangent.x.abs() * interior_width * 0.5 - facing.x.abs() * 0.006,
                sill,
                interior_plan.y - tangent.y.abs() * interior_width * 0.5 - facing.y.abs() * 0.006,
            ),
            Vec3::new(
                interior_plan.x + tangent.x.abs() * interior_width * 0.5 + facing.x.abs() * 0.006,
                sill + interior_height,
                interior_plan.y + tangent.y.abs() * interior_width * 0.5 + facing.y.abs() * 0.006,
            ),
        )?,
        SurfaceRole::InteriorMouth,
        crate::ResolvedSurfaceShape::Planar,
    );
    let stance_plan = origin - facing * (thickness * 0.5 + 1.55);
    let stance = projected_surface(
        geometry,
        owner,
        SpatialBounds::<Architectural>::from_metres(
            Vec3::new(stance_plan.x - 0.5, floor, stance_plan.y - 0.5),
            Vec3::new(stance_plan.x + 0.5, floor + 0.03, stance_plan.y + 0.5),
        )?,
        SurfaceRole::ArtilleryStance,
    );
    let mount_pos = origin - facing * (thickness * 0.5 + 0.85);
    let mount = projected_solid(
        geometry,
        owner,
        Vec3::new(mount_pos.x, sill + 0.18, mount_pos.y),
        Vec3::splat(0.22),
        0.0,
        SolidRole::WeaponMount,
        vec![wall.support_node],
    )?;
    let mut ray_indices = Vec::new();
    let mut rays = Vec::new();
    let eye = Vec3::new(
        origin.x - facing.x * (thickness * 0.5 + 0.02),
        sill + 0.30,
        origin.y - facing.y * (thickness * 0.5 + 0.02),
    );
    for (range, distance) in [
        (ProjectedDefenseRange::Near, 4.0),
        (ProjectedDefenseRange::Middle, 12.0),
        (ProjectedDefenseRange::Far, 24.0),
    ] {
        let southern_gate_flank = rondel_index < 2 && station_index == 0;
        let (target, target_kind) = if southern_gate_flank {
            let (z, kind) = match range {
                ProjectedDefenseRange::Near => (-13.5, crate::ArtilleryTargetKind::GateThreshold),
                ProjectedDefenseRange::Middle => (-17.0, crate::ArtilleryTargetKind::Bridge),
                ProjectedDefenseRange::Far => (-25.0, crate::ArtilleryTargetKind::Approach),
            };
            (Vec3::new(6.0, 0.20, z), kind)
        } else {
            (
                Vec3::new(
                    eye.x + facing.x * distance,
                    0.20,
                    eye.z + facing.y * distance,
                ),
                if station_index < 2 {
                    crate::ArtilleryTargetKind::CurtainFoot
                } else {
                    crate::ArtilleryTargetKind::DitchCorner
                },
            )
        };
        ray_indices.push(geometry.projected_defense_rays.len());
        geometry.projected_defense_rays.push(ProjectedDefenseRay {
            owner,
            throat: void_id,
            stance: Vec3::new(centre.x, floor, centre.y),
            origin: eye,
            target,
            range,
        });
        rays.push(crate::ArtilleryFireRay {
            target_id: crate::ArtilleryTargetId(u64::MAX),
            origin: eye,
            target,
            target_kind,
            range,
        });
    }
    let interfaces = [-1.0_f32, 1.0]
        .into_iter()
        .enumerate()
        .map(|(index, side)| {
            let slot = 50 + index as u64;
            let p = origin + tangent * side * (exterior_width * 0.5 + side_width * 0.5);
            let ext = tangent.abs() * side_width * 0.5 + facing.abs() * thickness * 0.5;
            let id = ResolvedItemId((4_u64 << 60) | (u64::from(owner.0) << 32) | slot);
            geometry
                .support_interfaces
                .push(crate::SupportInterface::new(
                    id,
                    owner,
                    head_node,
                    SpatialBounds::<Architectural>::from_metres(
                        Vec3::new(p.x - ext.x, sill + interior_height - 0.16, p.y - ext.y),
                        Vec3::new(p.x + ext.x, sill + interior_height + 0.02, p.y + ext.y),
                    )?,
                ));
            Ok(id)
        })
        .collect::<Result<Vec<_>, crate::GenerationError>>()?;
    let interfaces = [interfaces[0], interfaces[1]];
    let above = ResolvedItemId((4_u64 << 60) | (u64::from(owner.0) << 32) | 52);
    geometry
        .support_interfaces
        .push(crate::SupportInterface::new(
            above,
            owner,
            spandrel_node,
            SpatialBounds::<Architectural>::from_metres(
                Vec3::new(
                    origin.x - tangent.x.abs() * 0.75 - facing.x.abs() * 0.6,
                    sill + interior_height + 0.17,
                    origin.y - tangent.y.abs() * 0.75 - facing.y.abs() * 0.6,
                ),
                Vec3::new(
                    origin.x + tangent.x.abs() * 0.75 + facing.x.abs() * 0.6,
                    sill + interior_height + 0.26,
                    origin.y + tangent.y.abs() * 0.75 + facing.y.abs() * 0.6,
                ),
            )?,
        ));
    let opening = crate::OpeningAssembly {
        id: opening_id,
        owner,
        host_wall: wall.id,
        host_source: wall.source,
        frame: crate::WallLocalFrame {
            origin,
            tangent,
            outward: facing,
            inside_room: None,
            outside_room: None,
        },
        use_kind: crate::OpeningUse::GunLoop,
        profile: crate::OpeningProfile::GunLoop {
            exterior_width_metres: exterior_width,
            interior_width_metres: interior_width,
            exterior_height_metres: exterior_height,
            interior_height_metres: interior_height,
            mount: crate::WeaponMountClass::LightSwivelGun,
            traverse_degrees: 38.0,
            recoil_metres: 1.8,
            crew_clearance_metres: 2.5,
        },
        sill_elevation_metres: sill,
        closure: crate::ClosurePolicy {
            layers: vec![crate::ClosureKind::OpenMilitary],
            state: crate::ClosureState::Open,
            thickness_metres: 0.0,
            swing_clearance_metres: 0.0,
        },
        head_kind: crate::OpeningHeadKind::StoneLintel,
        void_id,
        jamb_solids: jamb,
        sill_solid: None,
        head_solid: head,
        spandrel_solid: spandrel,
        reveal_surfaces: vec![left, right, weather, intrados, throat, mouth],
        closure_solids: Vec::new(),
        jamb_nodes,
        head_node,
        spandrel_node,
        tracery_node: None,
        stance_surface: Some(stance),
        mount_solid: Some(mount),
        ray_indices,
        sectional_void: (0..=8)
            .map(|i| {
                let t = i as f32 / 8.0;
                crate::OpeningVoidSlice {
                    depth_fraction: t,
                    width_metres: exterior_width + (interior_width - exterior_width) * t,
                    height_metres: exterior_height + (interior_height - exterior_height) * t,
                }
            })
            .collect(),
        head_bearing_interfaces: interfaces,
        wall_above_interface: above,
    };
    wall.opening_ids.push(opening_id);
    openings.push(opening);
    let vent = (level == crate::ArtilleryStationLevel::LowerCasemate)
        .then(|| {
            Ok::<_, crate::GenerationError>(projected_void(
                geometry,
                owner,
                SpatialBounds::<Architectural>::from_metres(
                    Vec3::new(centre.x - 0.15, 2.45, centre.y - 0.15),
                    Vec3::new(centre.x + 0.15, 3.25, centre.y + 0.15),
                )?,
                VoidRole::ArtillerySmokeVent,
            ))
        })
        .transpose()?;
    let recoil_centre = origin - facing * (thickness * 0.5 + 2.0);
    let recoil_half = facing.abs() * 2.0 + tangent.abs() * 1.25;
    Ok(crate::ArtilleryFireStation {
        id: crate::ArtilleryStationId((rondel_index * 3 + station_index) as u64),
        rondel: crate::ArtilleryRondelId(rondel_index as u64),
        level,
        facing,
        opening: opening_id,
        stance_surface: stance,
        mount_solid: mount,
        recoil_envelope: SpatialBounds::<Architectural>::from_metres(
            Vec3::new(
                recoil_centre.x - recoil_half.x,
                floor,
                recoil_centre.y - recoil_half.y,
            ),
            Vec3::new(
                recoil_centre.x + recoil_half.x,
                floor + 2.1,
                recoil_centre.y + recoil_half.y,
            ),
        )?,
        smoke_vent: vent,
        rays,
    })
}
