use super::*;

#[test]
fn spiral_flight_has_grounded_newel_and_exact_supported_storey_landings() {
    let stair = Stair::Spiral {
        centre: Vec2::new(8.0, 9.0),
        base_height_metres: 0.15,
        rise_metres: 10.05,
        inner_radius_metres: 0.25,
        outer_radius_metres: 1.25,
        turns: 2.8,
        clockwise: true,
        tread_count: required_treads(0.15, 10.05, 3.4),
    };
    let flight = compile_flight(stair, 3.4).unwrap();
    let Stair::Spiral { tread_count, .. } = stair else {
        unreachable!()
    };
    assert_eq!(
        flight
            .members
            .iter()
            .filter(|m| m.role == SolidRole::StairTread)
            .count(),
        usize::from(tread_count)
    );
    let newel = flight
        .members
        .iter()
        .find(|m| m.role == SolidRole::StairNewel)
        .unwrap();
    assert!((newel.centre.y - newel.size.y * 0.5).abs() < 0.0001);
    for landing in &flight.landings {
        let point = Vec3::new(
            landing.position_metres.x,
            landing.elevation_metres - 0.01,
            landing.position_metres.y,
        );
        assert!(
            flight
                .members
                .iter()
                .any(|m| m.role == SolidRole::Landing && contains(m, point))
        );
        for offset in [
            Vec2::new(-0.3, -0.3),
            Vec2::new(0.3, -0.3),
            Vec2::new(0.3, 0.3),
            Vec2::new(-0.3, 0.3),
        ] {
            let p = point + Vec3::new(offset.x, 0.0, offset.y);
            assert!(
                flight
                    .members
                    .iter()
                    .any(|m| m.role == SolidRole::Landing && contains(m, p))
            );
        }
        if landing.storey > 0 {
            assert!(
                flight
                    .members
                    .iter()
                    .any(|m| m.role == SolidRole::StairTread
                        && (m.centre.y + m.size.y * 0.5 - landing.elevation_metres).abs() < 0.0001)
            );
        }
    }
}

#[test]
fn spiral_authored_resolution_and_landing_headroom_are_preserved() {
    let count = required_treads(0.15, 10.05, 3.4);
    let make = |tread_count| Stair::Spiral {
        centre: Vec2::ZERO,
        base_height_metres: 0.15,
        rise_metres: 10.05,
        inner_radius_metres: 0.25,
        outer_radius_metres: 1.25,
        turns: 2.8,
        clockwise: true,
        tread_count,
    };
    assert!(compile_flight(make(count - 1), 3.4).is_none());
    for authored in [count, count + 7] {
        let flight = compile_flight(make(authored), 3.4).unwrap();
        let treads = flight
            .members
            .iter()
            .filter(|m| m.role == SolidRole::StairTread)
            .collect::<Vec<_>>();
        assert_eq!(treads.len(), usize::from(authored));
        let mut previous = 0.0;
        for tread in treads {
            let height = tread.centre.y + tread.size.y * 0.5;
            assert!(height - previous <= 0.1901);
            previous = height;
        }
        for landing in flight
            .members
            .iter()
            .filter(|m| m.role == SolidRole::Landing && m.centre.y > 1.0)
        {
            // The radial bridge joins only the tread outer end, leaving the
            // inner walking line beneath the arrival clear of an overhang.
            let radial = bevy::math::Quat::from_rotation_y(landing.yaw_radians) * Vec3::X;
            let point = radial * 0.75 + Vec3::Y * landing.centre.y;
            assert!(!contains(landing, point));
        }
    }
}

fn contains(member: &SpiralMember, point: Vec3) -> bool {
    let local = bevy::math::Quat::from_rotation_y(-member.yaw_radians) * (point - member.centre);
    (member.size * 0.5 + Vec3::splat(0.0001) - local.abs()).min_element() >= 0.0
}

#[test]
fn fortified_stairs_and_floor_holes_are_shared_by_detail_and_collision() {
    let plan = crate::generate(&crate::BuildingProgram::fixture(
        BuildingArchetype::WalledKeep,
        42,
    ))
    .unwrap_or_else(|error| match error {
        crate::GenerationError::StructuralContract { issues, .. } => panic!(
            "{} issues; first: {:?}",
            issues.len(),
            &issues[..issues.len().min(8)]
        ),
        other => panic!("{other}"),
    });
    let collision = crate::compile_building_collision(&plan);
    let detail = crate::compile_building_detail(&plan);
    for solid in plan.resolved_geometry.solids.iter().filter(|s| {
        matches!(
            s.role,
            SolidRole::StairTread | SolidRole::StairNewel | SolidRole::InteriorFloor
        ) || owns_landing(s)
    }) {
        assert!(collision.cuboids.iter().any(|c| c.source == solid.id));
        assert!(detail.meshes.iter().flat_map(|m| &m.vertices).any(|v| {
            let local =
                bevy::math::Quat::from_rotation_y(-solid.yaw_radians) * (v.position - solid.centre);
            (local.abs() - solid.size * 0.5).abs().max_element() < 0.001
        }));
    }
    for (index, stair) in plan.stairs.iter().enumerate() {
        let Stair::Spiral {
            centre,
            outer_radius_metres,
            ..
        } = stair
        else {
            continue;
        };
        if centre.cmpge(Vec2::ZERO).all() && centre.cmple(plan.dimensions_metres()).all() {
            assert!(!landings(&plan, index).is_empty());
        }
        for solid in plan
            .resolved_geometry
            .solids
            .iter()
            .filter(|s| s.role == SolidRole::InteriorFloor && s.centre.y > 1.0)
        {
            let separation = (Vec2::new(solid.centre.x, solid.centre.z) - *centre).abs()
                - Vec2::new(solid.size.x, solid.size.z) * 0.5;
            assert!(
                separation.max_element()
                    >= outer_radius_metres + flight::WELL_MARGIN_METRES - 0.001
            );
        }
    }
}

#[test]
fn auxiliary_castle_stairs_do_not_claim_unbuilt_occupied_floor_portals() {
    for archetype in [
        BuildingArchetype::CastleGatehouse,
        BuildingArchetype::CourtyardCastle,
    ] {
        let plan = crate::generate(&crate::BuildingProgram::fixture(archetype, 59)).unwrap_or_else(
            |error| match error {
                crate::GenerationError::StructuralContract { issues, .. } => panic!(
                    "{archetype:?}: {} issues; first {:?}",
                    issues.len(),
                    &issues[..issues.len().min(4)]
                ),
                other => panic!("{other}"),
            },
        );
        assert!(
            !plan
                .resolved_geometry
                .solids
                .iter()
                .any(|solid| solid.role == SolidRole::InteriorFloor)
        );
        for index in 0..plan.stairs.len() {
            assert!(landings(&plan, index).is_empty());
            let owner = GeometryOwnerId(FIRST_FLIGHT_OWNER + index as u32);
            assert!(
                plan.resolved_geometry
                    .solids
                    .iter()
                    .any(|solid| solid.owner == owner && solid.role == SolidRole::StairTread)
            );
            assert!(
                plan.resolved_geometry
                    .solids
                    .iter()
                    .any(|solid| solid.owner == owner && solid.role == SolidRole::Landing)
            );
        }
    }
}
