use super::*;
use crate::furniture::{FurnitureVariant, builder::Builder};
use bevy::math::Quat;

const DOMESTIC: [FurnitureKind; 16] = [
    FurnitureKind::DiningTable,
    FurnitureKind::Bench,
    FurnitureKind::Chair,
    FurnitureKind::Stool,
    FurnitureKind::Bed,
    FurnitureKind::BunkBed,
    FurnitureKind::StorageChest,
    FurnitureKind::Cupboard,
    FurnitureKind::Shelving,
    FurnitureKind::WritingDesk,
    FurnitureKind::Lectern,
    FurnitureKind::ChurchBench,
    FurnitureKind::Altar,
    FurnitureKind::WardBed,
    FurnitureKind::BathTub,
    FurnitureKind::WashStand,
];

#[test]
fn lectern_support_post_does_not_pierce_the_writing_leaf() {
    for variant in FurnitureVariant::ALL {
        let mut builder = Builder::default();
        assemble(
            &mut builder,
            FurnitureKey {
                kind: FurnitureKind::Lectern,
                variant,
            },
        );
        let recipe = builder.finish();
        let leaf = recipe
            .colliders
            .iter()
            .find(|c| c.crossfall_radians.abs() > 0.1)
            .unwrap();
        let post = recipe
            .colliders
            .iter()
            .find(|c| c.size.y > 0.5 && c.size.x < 0.2 && c.size.z < 0.2)
            .unwrap();
        let leaf_inverse = Quat::from_rotation_x(leaf.crossfall_radians).inverse();
        for x in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                let top = post.centre + post.size * Vec3::new(x, 1.0, z) * 0.5;
                let in_leaf = leaf_inverse * (top - leaf.centre);
                assert!(
                    in_leaf.y < leaf.size.y * 0.5 - 0.001,
                    "{variant:?} post penetrates writing surface"
                );
            }
        }
    }
}

#[test]
fn domestic_models_fit_independent_envelopes_with_ground_contacts() {
    for kind in DOMESTIC {
        for variant in FurnitureVariant::ALL {
            let key = FurnitureKey { kind, variant };
            let size = key.interior_spec().unwrap().size_metres;
            let mut builder = Builder::default();
            assemble(&mut builder, key);
            let recipe = builder.finish();
            assert!(recipe.bounds.min.y.abs() < 0.001, "{key:?}");
            for mesh in &recipe.meshes {
                for vertex in &mesh.vertices {
                    let p = vertex.position;
                    assert!(p.is_finite() && vertex.normal.is_finite());
                    assert!(
                        p.x.abs() <= size.x * 0.5 + 0.001
                            && p.z.abs() <= size.z * 0.5 + 0.001
                            && p.y >= -0.001
                            && p.y <= size.y + 0.001,
                        "{key:?} vertex {p:?} outside {size:?}"
                    );
                }
            }
            assert!(recipe.support_points_metres.len() >= 4, "{key:?}");
            for collider in &recipe.colliders {
                assert!(collider.size.min_element() > 0.0);
                let rotation = Quat::from_rotation_y(collider.yaw_radians)
                    * Quat::from_rotation_x(collider.crossfall_radians)
                    * Quat::from_rotation_z(collider.longfall_radians);
                for corner in 0..8 {
                    let sign = Vec3::new(
                        if corner & 1 == 0 { -1.0 } else { 1.0 },
                        if corner & 2 == 0 { -1.0 } else { 1.0 },
                        if corner & 4 == 0 { -1.0 } else { 1.0 },
                    );
                    let p = collider.centre + rotation * (collider.size * 0.5 * sign);
                    assert!(
                        p.x.abs() <= size.x * 0.5 + 0.001
                            && p.z.abs() <= size.z * 0.5 + 0.001
                            && p.y >= -0.001
                            && p.y <= size.y + 0.001,
                        "{key:?} collider vertex {p:?} outside {size:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn bath_and_shelving_leave_their_usable_interiors_empty() {
    for kind in [FurnitureKind::BathTub, FurnitureKind::Shelving] {
        for variant in FurnitureVariant::ALL {
            let key = FurnitureKey { kind, variant };
            let mut builder = Builder::default();
            assemble(&mut builder, key);
            let recipe = builder.finish();
            let point = Vec3::Y * 0.3;
            assert!(
                !recipe.colliders.iter().any(|collider| {
                    let rotation = Quat::from_rotation_y(collider.yaw_radians)
                        * Quat::from_rotation_x(collider.crossfall_radians);
                    let local = rotation.inverse() * (point - collider.centre);
                    (collider.size * 0.5 - local.abs()).min_element() > 0.0
                }),
                "{key:?} has a solid fill collider"
            );
        }
    }
}
