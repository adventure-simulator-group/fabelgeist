use super::*;
use crate::furniture::{FurnitureRecipe, FurnitureVariant};
use bevy::math::Quat;

fn recipe(kind: FurnitureKind, variant: FurnitureVariant) -> FurnitureRecipe {
    let mut builder = Builder::default();
    assemble(&mut builder, FurnitureKey { kind, variant });
    builder.finish()
}

fn solid_at(recipe: &FurnitureRecipe, point: Vec3) -> bool {
    recipe.colliders.iter().any(|collider| {
        let rotation = Quat::from_rotation_y(collider.yaw_radians)
            * Quat::from_rotation_x(collider.crossfall_radians)
            * Quat::from_rotation_z(collider.longfall_radians);
        let local = rotation.inverse() * (point - collider.centre);
        (collider.size * 0.5 + Vec3::splat(0.000_01) - local.abs()).min_element() >= 0.0
    })
}

#[test]
fn joined_counters_have_continuous_level_tops_and_only_terminal_endcaps() {
    use FurnitureKind::*;
    for variant in FurnitureVariant::ALL {
        let pieces = [CounterLeftEnd, Counter, CounterRightEnd].map(|kind| recipe(kind, variant));
        let width = pieces[1].bounds.max.x - pieces[1].bounds.min.x;
        let height = pieces[1].bounds.max.y;
        let depth = pieces[1].bounds.max.z - pieces[1].bounds.min.z;
        for (index, piece) in pieces.iter().enumerate() {
            assert!((piece.bounds.max.y - height).abs() < 0.000_01);
            assert!((piece.bounds.max.x - piece.bounds.min.x - width).abs() < 0.000_01);
            for step in 0..=20 {
                let z = -depth * 0.5 + depth * step as f32 / 20.0;
                for sign in [-1.0, 1.0] {
                    let seam = Vec3::new(sign * width * 0.5, height - 0.01, z);
                    assert!(
                        solid_at(piece, seam),
                        "counter {index} has an unsupported top seam"
                    );
                }
            }
        }
        for join in 0..2 {
            let left_origin = Vec3::X * width * join as f32;
            let right_origin = left_origin + Vec3::X * width;
            let left_edge = left_origin.x + pieces[join].bounds.max.x;
            let right_edge = right_origin.x + pieces[join + 1].bounds.min.x;
            assert!((left_edge - right_edge).abs() < 0.000_01);
            for side in [-1.0, 1.0] {
                let world = Vec3::new(left_edge + side * 0.001, height - 0.01, 0.0);
                assert!(
                    solid_at(&pieces[join], world - left_origin)
                        || solid_at(&pieces[join + 1], world - right_origin)
                );
            }
        }
        let left_end = Vec3::new(-width * 0.5 + 0.01, height * 0.5, 0.0);
        let right_end = Vec3::new(width * 0.5 - 0.01, height * 0.5, 0.0);
        assert!(solid_at(&pieces[0], left_end));
        assert!(!solid_at(&pieces[0], right_end));
        assert!(!solid_at(&pieces[1], left_end));
        assert!(!solid_at(&pieces[1], right_end));
        assert!(!solid_at(&pieces[2], left_end));
        assert!(solid_at(&pieces[2], right_end));
    }
}

#[test]
fn counter_corner_joins_two_orthogonal_runs_without_a_top_step() {
    for variant in FurnitureVariant::ALL {
        let corner = recipe(FurnitureKind::CounterCorner, variant);
        let straight = recipe(FurnitureKind::Counter, variant);
        let half_width = straight.bounds.max.x;
        let half_corner = corner.bounds.max.x;
        let height = corner.bounds.max.y;
        assert!((height - straight.bounds.max.y).abs() < 0.000_01);
        for axis in [Vec3::X, Vec3::Z] {
            let rotation = if axis == Vec3::X {
                Quat::IDENTITY
            } else {
                Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)
            };
            let origin = axis * (half_corner + half_width);
            for fraction in [-0.9, 0.0, 0.9] {
                let tangent = Vec3::new(axis.z, 0.0, axis.x);
                let seam = axis * half_corner
                    + tangent * half_corner * fraction
                    + Vec3::Y * (height - 0.01);
                assert!(solid_at(&corner, seam - axis * 0.001));
                assert!(solid_at(
                    &straight,
                    rotation.inverse() * (seam + axis * 0.001 - origin)
                ));
            }
        }
    }
}

#[test]
fn trade_geometry_and_colliders_fit_the_placement_envelope() {
    use FurnitureKind::*;
    let kinds = [
        Workbench,
        CuttingTable,
        ToolRack,
        WeaponRack,
        ArmourStand,
        GrainBin,
        StorageCrate,
        Counter,
        CounterLeftEnd,
        CounterRightEnd,
        CounterCorner,
        DisplayCounter,
        DryingRack,
        KneadingTrough,
        ButchersBlock,
        CaskRack,
        HayRack,
        FeedTrough,
    ];
    for kind in kinds {
        for variant in FurnitureVariant::ALL {
            let key = FurnitureKey { kind, variant };
            let size = key.interior_spec().unwrap().size_metres;
            let recipe = recipe(kind, variant);
            let min = Vec3::new(-size.x * 0.5, 0.0, -size.z * 0.5);
            let max = Vec3::new(size.x * 0.5, size.y, size.z * 0.5);
            for point in recipe
                .meshes
                .iter()
                .flat_map(|mesh| &mesh.vertices)
                .map(|vertex| vertex.position)
            {
                assert!(
                    (point - min).min_element() >= -0.000_1
                        && (max - point).min_element() >= -0.000_1,
                    "{key:?}: vertex {point:?} escaped"
                );
            }
            for collider in &recipe.colliders {
                let rotation = Quat::from_rotation_y(collider.yaw_radians)
                    * Quat::from_rotation_x(collider.crossfall_radians)
                    * Quat::from_rotation_z(collider.longfall_radians);
                for x in [-1.0, 1.0] {
                    for y in [-1.0, 1.0] {
                        for z in [-1.0, 1.0] {
                            let point = collider.centre
                                + rotation * (collider.size * Vec3::new(x, y, z) * 0.5);
                            assert!(
                                (point - min).min_element() >= -0.000_1
                                    && (max - point).min_element() >= -0.000_1,
                                "{key:?}: collider escaped"
                            );
                        }
                    }
                }
            }
            assert!(recipe.support_points_metres.len() >= 4);
            for point in &recipe.support_points_metres {
                assert!(solid_at(&recipe, *point));
                assert!(point.y.abs() < 0.000_1);
            }
        }
    }
}

#[test]
fn bins_and_display_trays_remain_empty_above_their_supported_bottoms() {
    for kind in [
        FurnitureKind::GrainBin,
        FurnitureKind::KneadingTrough,
        FurnitureKind::FeedTrough,
        FurnitureKind::DisplayCounter,
    ] {
        for variant in FurnitureVariant::ALL {
            let recipe = recipe(kind, variant);
            let probe = Vec3::new(recipe.bounds.max.x * 0.5, recipe.bounds.max.y - 0.06, 0.0);
            assert!(
                !solid_at(&recipe, probe),
                "{kind:?} is solid-filled or contains stock"
            );
            assert!(
                recipe.colliders.iter().any(|collider| {
                    let point = Vec3::new(probe.x, collider.centre.y, probe.z);
                    point.y < probe.y && solid_at(&recipe, point)
                }),
                "{kind:?} has no supported bottom"
            );
        }
    }
}
