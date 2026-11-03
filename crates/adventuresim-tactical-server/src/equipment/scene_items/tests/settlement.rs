//! Drops against the level changes of a compiled settlement: the natural
//! trimesh, solid foundation compounds and the cut faces between them.
use super::*;
use crate::compound_review_fixture::compound_review_terrain;
use adventuresim_tactical_core::physics::TACTICAL_MAXIMUM_STEP_HEIGHT_METRES;

const GRID: usize = 101;
/// A sloped source makes the compiled foundations step out of the terrain.
const SOURCE_GRADE: f32 = 0.12;
const PROBE_SPACING_M: f32 = 0.5;
const PROBE_REACH_M: f32 = 0.9;
const MINIMUM_FACE_RISE_M: f32 = 1.0;
/// The character cylinder's radius, inside which the ground must stay within
/// one step so the actor could really stand there.
const STANDING_CLEARANCE_M: f32 = 0.45;
const FACE_SITES: usize = 16;

struct FaceSite {
    feet: Vec2,
    facing: Vec2,
    rise_m: f32,
}

fn top_surface(colliders: &[Collider], position: Vec2) -> Option<f32> {
    const RAY_TOP_M: f32 = 500.0;
    colliders
        .iter()
        .filter_map(|collider| {
            collider
                .cast_ray(
                    Vec3::ZERO,
                    Rotation::default(),
                    Vec3::new(position.x, RAY_TOP_M, position.y),
                    Vec3::NEG_Y,
                    RAY_TOP_M * 2.0,
                    false,
                )
                .map(|(distance, _)| RAY_TOP_M - distance)
        })
        .max_by(f32::total_cmp)
}

/// The steepest places to stand facing a rise, spread across the settlement.
fn face_sites(colliders: &[Collider], low: Vec2, high: Vec2) -> Vec<FaceSite> {
    let mut sites = Vec::new();
    let mut x = low.x;
    while x <= high.x {
        let mut z = low.y;
        while z <= high.y {
            let feet = Vec2::new(x, z);
            if let Some(ground) = top_surface(colliders, feet) {
                for facing in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
                    let standable = [facing, facing.perp(), -facing.perp(), -facing]
                        .into_iter()
                        .all(|side| {
                            top_surface(colliders, feet + side * STANDING_CLEARANCE_M).is_some_and(
                                |height| {
                                    (height - ground).abs() <= TACTICAL_MAXIMUM_STEP_HEIGHT_METRES
                                },
                            )
                        });
                    if !standable {
                        continue;
                    }
                    let Some(ahead) = top_surface(colliders, feet + facing * PROBE_REACH_M) else {
                        continue;
                    };
                    let rise_m = ahead - ground;
                    if rise_m >= MINIMUM_FACE_RISE_M {
                        sites.push(FaceSite {
                            feet,
                            facing,
                            rise_m,
                        });
                    }
                }
            }
            z += PROBE_SPACING_M;
        }
        x += PROBE_SPACING_M;
    }
    sites.sort_by(|a, b| b.rise_m.total_cmp(&a.rise_m));
    let mut spread: Vec<FaceSite> = Vec::new();
    for site in sites {
        if spread
            .iter()
            .all(|kept| kept.feet.distance(site.feet) > 2.0)
        {
            spread.push(site);
        }
        if spread.len() == FACE_SITES {
            break;
        }
    }
    spread
}

#[test]
fn drops_against_settlement_level_changes_stay_recoverable() {
    let half = (GRID as f32 - 1.0) / 2.0;
    let heightmap = (0..GRID * GRID)
        .map(|index| ((index / GRID) as f32 - half) * SOURCE_GRADE)
        .collect();
    let (terrain, plans) = compound_review_terrain(heightmap);
    let colliders = terrain.colliders().unwrap();
    let (low, high) = plans
        .iter()
        .flat_map(|plan| plan.member_support())
        .map(|member| member.contact.centre_metres())
        .fold((Vec2::MAX, Vec2::MIN), |(low, high), centre| {
            (low.min(centre), high.max(centre))
        });
    let margin = Vec2::splat(8.0);
    let sites = face_sites(&colliders, low - margin, high + margin);
    assert_eq!(
        sites.len(),
        FACE_SITES,
        "the fixture lost its level changes"
    );

    for item_id in CATALOG_ITEMS {
        let surface = colliders.clone();
        let heights = Box::new(move |x: f32, z: f32| {
            top_surface(&surface, Vec2::new(x, z)).unwrap_or(f32::MIN)
        });
        let first = &sites[0];
        let mut scene = Scene::from_colliders(
            colliders.clone(),
            heights,
            item_id,
            first.feet,
            first.facing.extend(0.0).xzy(),
        );
        for site in &sites {
            let context = format!(
                "{item_id} at {} facing a {:.2} m rise toward {}",
                site.feet, site.rise_m, site.facing
            );
            scene.place_actor(site.feet);
            let actor = scene.actor;
            scene
                .app
                .world_mut()
                .get_mut::<Transform>(actor)
                .unwrap()
                .look_to(site.facing.extend(0.0).xzy(), Vec3::Y);
            let actor = scene.actor_translation();
            assert!(scene.drop_item(), "{context}: drop rejected");
            scene.settle(1);
            let released = scene.item_translation();
            assert!(
                released.xz().distance(actor.xz()) <= 1.6,
                "{context}: released at {released}, actor at {actor}"
            );
            scene.settle(SETTLE_TICKS);
            scene.assert_at_rest_above_terrain(&context);
            let rest = scene.item_translation();
            let recovered = [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y]
                .into_iter()
                .any(|side| {
                    scene.place_actor(rest.xz() + side * 0.5);
                    scene.pick_up_item()
                });
            assert!(recovered, "{context}: item at {rest} cannot be picked up");
        }
    }
}
