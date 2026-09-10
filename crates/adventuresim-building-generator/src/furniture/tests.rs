use super::*;
use crate::BuildingLodMaterial;
use bevy::math::{Quat, Vec2};

#[test]
fn stall_stock_is_supported_inside_counter_and_broad_display_has_more_goods() {
    let mut grain_spans = Vec::new();
    for variant in FurnitureVariant::ALL {
        let recipe = FurnitureKey {
            kind: FurnitureKind::CanvasStall,
            variant,
        }
        .recipe();
        let counter_half_width = match variant {
            FurnitureVariant::Compact => 1.1,
            FurnitureVariant::Broad => 1.55,
        };
        let stock = recipe
            .colliders
            .iter()
            .filter(|collider| {
                let min_y = collider.centre.y - collider.size.y * 0.5;
                let max_y = collider.centre.y + collider.size.y * 0.5;
                min_y >= 0.849 && max_y < 1.5
            })
            .collect::<Vec<_>>();
        assert!(stock.len() >= 8, "stall counter must carry visible stock");
        for collider in stock {
            for point in corners(collider) {
                assert!(point.x.abs() <= counter_half_width);
                assert!((-0.645..=-0.095).contains(&point.z));
                assert!(point.y >= 0.849, "goods sunk through counter");
            }
        }
        let grain = recipe
            .meshes
            .iter()
            .find(|mesh| mesh.material == BuildingLodMaterial::Grain)
            .unwrap();
        let mut minimum = Vec3::splat(f32::INFINITY);
        let mut maximum = Vec3::splat(f32::NEG_INFINITY);
        for vertex in &grain.vertices {
            minimum = minimum.min(vertex.position);
            maximum = maximum.max(vertex.position);
            assert!(recipe.colliders.iter().any(|collider| contains(
                collider,
                vertex.position,
                0.002
            )));
        }
        assert!(minimum.y >= 0.88 && maximum.y > 1.08);
        assert!(
            all_members_grounded(recipe),
            "stock is floating above its counter"
        );
        grain_spans.push(maximum.x - minimum.x);
    }
    assert!(grain_spans[1] > grain_spans[0] + 0.8);
}

fn rotation(cuboid: &CollisionCuboid) -> Quat {
    Quat::from_rotation_y(cuboid.yaw_radians)
        * Quat::from_rotation_x(cuboid.crossfall_radians)
        * Quat::from_rotation_z(cuboid.longfall_radians)
}

fn corners(cuboid: &CollisionCuboid) -> [Vec3; 8] {
    std::array::from_fn(|index| {
        let sign = Vec3::new(
            if index & 1 == 0 { -1.0 } else { 1.0 },
            if index & 2 == 0 { -1.0 } else { 1.0 },
            if index & 4 == 0 { -1.0 } else { 1.0 },
        );
        cuboid.centre + rotation(cuboid) * (cuboid.size * 0.5 * sign)
    })
}

fn contains(cuboid: &CollisionCuboid, point: Vec3, tolerance: f32) -> bool {
    let local = rotation(cuboid).inverse() * (point - cuboid.centre);
    (cuboid.size * 0.5 + Vec3::splat(tolerance) - local.abs()).min_element() >= 0.0
}

/// Independent corner-projection oracle checks the pitched canopy members as oriented boxes.
fn touches(a: &CollisionCuboid, b: &CollisionCuboid) -> bool {
    let axes_a = [Vec3::X, Vec3::Y, Vec3::Z].map(|axis| rotation(a) * axis);
    let axes_b = [Vec3::X, Vec3::Y, Vec3::Z].map(|axis| rotation(b) * axis);
    let points_a = corners(a);
    let points_b = corners(b);
    let mut axes = axes_a.into_iter().chain(axes_b).collect::<Vec<_>>();
    axes.extend(axes_a.into_iter().flat_map(|a| axes_b.map(|b| a.cross(b))));
    axes.into_iter()
        .filter_map(Vec3::try_normalize)
        .all(|axis| {
            let interval = |points: [Vec3; 8]| {
                points
                    .map(|point| point.dot(axis))
                    .into_iter()
                    .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
                        (min.min(value), max.max(value))
                    })
            };
            let first = interval(points_a);
            let second = interval(points_b);
            first.0 <= second.1 + 0.012 && second.0 <= first.1 + 0.012
        })
}

fn all_members_grounded(recipe: &FurnitureRecipe) -> bool {
    let members = recipe
        .members
        .iter()
        .chain(&recipe.colliders)
        .collect::<Vec<_>>();
    let mut grounded = members
        .iter()
        .map(|member| corners(member).iter().any(|point| point.y.abs() < 0.001))
        .collect::<Vec<_>>();
    loop {
        let mut changed = false;
        for index in 0..members.len() {
            if !grounded[index]
                && members
                    .iter()
                    .enumerate()
                    .any(|(other, member)| grounded[other] && touches(members[index], member))
            {
                grounded[index] = true;
                changed = true;
            }
        }
        if !changed {
            return grounded.into_iter().all(|value| value);
        }
    }
}

#[test]
fn cached_furniture_has_finite_ground_centred_footprints_and_real_support_contacts() {
    for key in FurnitureKey::ALL {
        let recipe = key.recipe();
        assert!(std::ptr::eq(recipe, key.recipe()));
        assert!(recipe.bounds.min.is_finite() && recipe.bounds.max.is_finite());
        assert!(recipe.bounds.min.y.abs() < 0.001);
        assert!(
            (recipe.bounds.min.x + recipe.bounds.max.x).abs() < 0.001
                && (recipe.bounds.min.z + recipe.bounds.max.z).abs() < 0.001
        );
        assert!(recipe.support_points_metres.len() >= 4);
        for point in &recipe.support_points_metres {
            assert!(point.y.abs() < 0.001);
            assert!(
                recipe
                    .meshes
                    .iter()
                    .flat_map(|mesh| &mesh.vertices)
                    .any(|vertex| vertex.position.distance(*point) < 0.001)
            );
            assert!(
                recipe
                    .colliders
                    .iter()
                    .any(|collider| contains(collider, *point, 0.002)),
                "{key:?} unsupported ground sample {point:?}"
            );
        }
        let support_min = recipe
            .support_points_metres
            .iter()
            .copied()
            .reduce(Vec3::min)
            .unwrap();
        let support_max = recipe
            .support_points_metres
            .iter()
            .copied()
            .reduce(Vec3::max)
            .unwrap();
        assert!(
            support_min.x < 0.0
                && support_max.x > 0.0
                && support_min.z < 0.0
                && support_max.z > 0.0,
            "{key:?} lacks a stable spread of ground contacts"
        );
        for collider in &recipe.colliders {
            assert!(collider.size.min_element() > 0.0);
            for point in corners(collider) {
                assert!(
                    (point - recipe.bounds.min).min_element() >= -0.025
                        && (recipe.bounds.max - point).min_element() >= -0.025,
                    "{key:?} collider outside visible bounds"
                );
            }
        }
        assert!(
            all_members_grounded(recipe),
            "{key:?} has an unsupported member"
        );
        for mesh in &recipe.meshes {
            for indices in mesh.indices.as_chunks::<3>().0 {
                let vertices = indices.map(|index| mesh.vertices[index as usize]);
                let normal = (vertices[1].position - vertices[0].position)
                    .cross(vertices[2].position - vertices[0].position);
                assert!(vertices.iter().all(|vertex| vertex.position.is_finite()
                    && vertex.normal.is_finite()
                    && vertex.uv.is_finite()));
                assert!(
                    normal.dot(vertices[0].normal) > 0.000_001,
                    "{key:?} has an inverted or degenerate face"
                );
                let uv_area = (vertices[1].uv - vertices[0].uv)
                    .perp_dot(vertices[2].uv - vertices[0].uv)
                    .abs();
                assert!(
                    uv_area > 0.000_000_1,
                    "{key:?} has a collapsed tangent basis"
                );
            }
        }
        let count = recipe
            .meshes
            .iter()
            .map(|mesh| mesh.indices.len() / 3)
            .sum::<usize>();
        eprintln!(
            "{key:?}: {count} triangles, {} colliders",
            recipe.colliders.len()
        );
        assert!(count > 40 && count < 1_200);
    }
}

#[test]
fn authored_access_and_working_clearances_are_not_blocked_by_their_own_furniture() {
    for key in FurnitureKey::ALL {
        let recipe = key.recipe();
        assert!(!recipe.clearances.is_empty());
        for clearance in &recipe.clearances {
            let bounds = clearance.bounds;
            assert!((bounds.max - bounds.min).min_element() > 0.0);
            let corridor = CollisionCuboid {
                source: crate::ResolvedItemId(0),
                centre: (bounds.min + bounds.max) * 0.5,
                size: bounds.max - bounds.min - Vec3::splat(0.04),
                yaw_radians: 0.0,
                crossfall_radians: 0.0,
                longfall_radians: 0.0,
            };
            assert!(
                recipe
                    .colliders
                    .iter()
                    .all(|collider| !touches(collider, &corridor)),
                "{key:?} blocks {:?}",
                clearance.kind
            );
        }
    }
}

#[test]
fn trough_water_and_sagging_canvas_are_visible_without_solid_fill_collision() {
    for variant in FurnitureVariant::ALL {
        let trough = FurnitureKey {
            kind: FurnitureKind::HitchingTrough,
            variant,
        }
        .recipe();
        let water = trough
            .meshes
            .iter()
            .find(|mesh| mesh.material == BuildingLodMaterial::ProcessLiquid)
            .unwrap();
        let min = water
            .vertices
            .iter()
            .map(|v| v.position)
            .reduce(Vec3::min)
            .unwrap();
        let max = water
            .vertices
            .iter()
            .map(|v| v.position)
            .reduce(Vec3::max)
            .unwrap();
        let centre = (min + max) * 0.5;
        assert!(
            !trough
                .colliders
                .iter()
                .any(|collider| contains(collider, centre, 0.0))
        );
        assert!(
            trough.colliders.iter().any(|collider| contains(
                collider,
                Vec3::new(centre.x, min.y - 0.01, centre.z),
                0.0
            )),
            "water has no physical basin bottom"
        );
        let stall = FurnitureKey {
            kind: FurnitureKind::CanvasStall,
            variant,
        }
        .recipe();
        let canvas = stall
            .meshes
            .iter()
            .find(|mesh| mesh.material == BuildingLodMaterial::UndyedCloth)
            .unwrap();
        let quarter = canvas
            .vertices
            .iter()
            .filter(|vertex| vertex.position.z.abs() < 0.001 && vertex.position.x > 0.1)
            .min_by(|a, b| a.position.x.total_cmp(&b.position.x))
            .unwrap()
            .position;
        let edge = canvas
            .vertices
            .iter()
            .filter(|vertex| (vertex.position.x - quarter.x).abs() < 0.001)
            .max_by(|a, b| a.position.z.total_cmp(&b.position.z))
            .unwrap()
            .position;
        assert!(
            edge.y - quarter.y > 0.08,
            "canvas must sag between supported end frames"
        );
        assert!(
            !stall
                .colliders
                .iter()
                .any(|collider| contains(collider, quarter, 0.0)),
            "canvas cloth became a solid roof slab"
        );
    }
}

#[test]
fn stability_check_rejects_detached_frames_and_barrels_have_a_bilged_coopered_outline() {
    let mut detached = FurnitureKey {
        kind: FurnitureKind::CanvasStall,
        variant: FurnitureVariant::Compact,
    }
    .recipe()
    .clone();
    let mut floating = detached.members[0];
    floating.centre.y += 5.0;
    detached.members.push(floating);
    assert!(!all_members_grounded(&detached));
    let barrel = FurnitureKey {
        kind: FurnitureKind::Barrel,
        variant: FurnitureVariant::Compact,
    }
    .recipe();
    let timber = barrel
        .meshes
        .iter()
        .find(|mesh| mesh.material == BuildingLodMaterial::InteriorTimber)
        .unwrap();
    let radius = |point: Vec3| Vec2::new(point.x, point.z).length();
    let bottom = timber
        .vertices
        .iter()
        .filter(|v| v.position.y.abs() < 0.001)
        .map(|v| radius(v.position))
        .fold(0.0_f32, f32::max);
    let waist = timber
        .vertices
        .iter()
        .map(|v| radius(v.position))
        .fold(0.0_f32, f32::max);
    assert!(waist > bottom * 1.15);
    assert!(
        barrel
            .meshes
            .iter()
            .any(|mesh| mesh.material == BuildingLodMaterial::Iron)
    );
}
