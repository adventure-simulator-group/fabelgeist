use super::*;
use adventuresim_tactical_core::scene_input::VistaLod;
fn source(count: usize) -> VistaSample {
    VistaSample {
        lods: (0..count)
            .map(|level| VistaLod {
                level: level as u8,
                spacing_metres: 10.0 * (level + 1) as f32,
                width: 5,
                depth: 7,
                origin_east_metres: 50.0,
                origin_north_metres: -20.0,
                heights_metres: vec![0.0; 35],
                environment: Vec::new(),
            })
            .collect(),
    }
}
fn observed(contract: &VistaCaptureContract, count: usize) -> BTreeMap<u8, RingBounds> {
    contract
        .0
        .iter()
        .take(count)
        .map(|(level, b)| {
            (
                *level,
                RingBounds {
                    minimum: b.minimum,
                    maximum: b.maximum,
                },
            )
        })
        .collect()
}
#[test]
fn one_and_three_declared_rings_require_exact_levels_and_complete_extent() {
    for count in [1, 3] {
        let contract = VistaCaptureContract::from_source(&source(count));
        let bounds = observed(&contract, count);
        assert!(contract.matches_bounds(&bounds, 3));
        let mut missing = observed(&contract, count);
        missing.remove(&0);
        assert!(!contract.matches_bounds(&missing, 3));
        let mut truncated = observed(&contract, count);
        truncated.get_mut(&0).unwrap().maximum.x -= 0.1;
        assert!(!contract.matches_bounds(&truncated, 3));
        let mut wrong = observed(&contract, count);
        let b = wrong.remove(&0).unwrap();
        wrong.insert(8, b);
        assert!(!contract.matches_bounds(&wrong, 3));
    }
}
#[test]
fn renderer_lod_cap_is_explicit_and_missing_extra_or_moved_rings_reject() {
    let contract = VistaCaptureContract::from_source(&source(3));
    assert!(contract.matches_bounds(&observed(&contract, 1), 1));
    assert!(!contract.matches_bounds(&observed(&contract, 3), 1));
    let mut moved = observed(&contract, 3);
    moved.get_mut(&1).unwrap().minimum += Vec2::ONE;
    moved.get_mut(&1).unwrap().maximum += Vec2::ONE;
    assert!(!contract.matches_bounds(&moved, 3));
    let bounds =
        RingBounds::from_points([Vec3::new(30.0, 0.0, -50.0), Vec3::new(70.0, 10.0, 10.0)])
            .unwrap();
    assert!(bounds.matches(&contract.0[0].1));
}

#[test]
fn actual_renderer_bounds_include_chunk_translation_and_reject_missing_bounds() {
    use bevy::ecs::system::RunSystemOnce;
    let check = |bounds| {
        let mut world = World::new();
        let mut entity = world.spawn((
            VistaTerrain(0),
            VistaTerrainMesh(0),
            GlobalTransform::from_translation(Vec3::new(50.0, 100.0, -20.0)),
        ));
        if let Some(bounds) = bounds {
            entity.insert(bounds);
        }
        // Scattered rocks and trees share VistaTerrain, but their bounds must
        // not extend the declared terrain ring. Collision detection still
        // covers them, including objects with no renderer AABB.
        world.spawn((
            VistaTerrain(0),
            GlobalTransform::from_translation(Vec3::splat(1_000.0)),
            adventuresim_tactical_core::prelude::Collider::sphere(1.0),
        ));
        let contract = VistaCaptureContract::from_source(&source(1));
        world
            .run_system_once(
                move |vistas: VistaQuery,
                      mut cameras: Query<&mut GlobalTransform, With<TacticalGameplayCamera>>| {
                    for mut camera in &mut cameras {
                        *camera = GlobalTransform::default();
                    }
                    contract.observe(&vistas, 3)
                },
            )
            .unwrap()
    };
    let complete = check(Some(Aabb::from_min_max(
        Vec3::new(-20.0, -5.0, -30.0),
        Vec3::new(20.0, 5.0, 30.0),
    )));
    assert!(complete.matches_source);
    assert_eq!(complete.presented_lods, [0]);
    assert_eq!(complete.chunks, 1);
    assert_eq!(complete.colliders, 1);
    assert!(!check(None).matches_source);
    assert!(!check(Some(Aabb::from_min_max(-Vec3::ONE, Vec3::ONE))).matches_source);
}
