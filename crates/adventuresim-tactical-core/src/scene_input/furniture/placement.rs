use super::*;
use crate::scene::GroundSubstrate;
use candidates::Candidate;
use fabelgeist_determinism::mix64;

const MAX_GROUPS: usize = 96;
const MAX_MARKET_GROUPS: usize = 24;
const MAX_SUPPORT_ERROR_METRES: f32 = 0.045;
const MAX_GROUP_GRADE: f32 = 0.08;
const INSTANCE_DOMAIN: u64 = 0x6675_726e_6974_656d;

pub(super) fn generate(
    input: &TacticalSceneInput,
    buildings: &[GeneratedBuilding],
    terrain: &SceneTerrain,
    ground: &SceneGround,
    obstacles: &[GeneratedObstacle],
) -> FurnitureLayout {
    let mut layout = FurnitureLayout {
        reserved_routes: reservations::routes(input, buildings),
        ..Default::default()
    };
    let mut occupied = reservations::obstacles(input, terrain, buildings, obstacles);
    for candidate in candidates::market(input) {
        if layout.groups.len() >= MAX_MARKET_GROUPS {
            break;
        }
        accept(
            input,
            terrain,
            ground,
            candidate,
            &mut occupied,
            &mut layout,
        );
    }
    let mut ordered = buildings.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|building| building.placement.id);
    for building in ordered {
        if layout.groups.len() >= MAX_GROUPS {
            break;
        }
        for candidate in candidates::building(input, building) {
            if accept(
                input,
                terrain,
                ground,
                candidate,
                &mut occupied,
                &mut layout,
            ) {
                break;
            }
        }
    }
    layout
}

fn accept(
    input: &TacticalSceneInput,
    terrain: &SceneTerrain,
    ground: &SceneGround,
    candidate: Candidate,
    occupied: &mut Vec<FurnitureFootprint>,
    layout: &mut FurnitureLayout,
) -> bool {
    let footprint = candidate.footprint;
    if occupied
        .iter()
        .chain(&layout.reserved_routes)
        .any(|other| footprint.intersects(*other))
    {
        return false;
    }
    let corners = footprint.corners();
    if candidate
        .market
        .is_some_and(|market| !corners.into_iter().all(|corner| market.contains(corner)))
    {
        return false;
    }
    let samples = corners
        .into_iter()
        .chain([footprint.centre_metres])
        .map(|point| {
            if input
                .landform
                .is_some_and(|landform| landform.transition_collar().contains(point))
            {
                return None;
            }
            if ground.ground_at(point).is_none_or(|surface| {
                matches!(
                    surface.substrate,
                    GroundSubstrate::Water | GroundSubstrate::Mud
                )
            }) {
                return None;
            }
            terrain.height_at(point)
        })
        .collect::<Option<Vec<_>>>();
    let Some(samples) = samples else {
        return false;
    };
    let min = samples.iter().copied().fold(f32::INFINITY, f32::min);
    let max = samples.into_iter().fold(f32::NEG_INFINITY, f32::max);
    if max - min > footprint.half_extents_metres.length() * 2.0 * MAX_GROUP_GRADE {
        return false;
    }
    let Some(instances) = supported_instances(&candidate, terrain) else {
        return false;
    };
    occupied.push(footprint);
    layout.groups.push(FurnitureGroup {
        id: candidate.id,
        kind: candidate.kind,
        anchor: candidate.anchor,
        footprint,
    });
    layout.instances.extend(instances);
    true
}

fn supported_instances(
    candidate: &Candidate,
    terrain: &SceneTerrain,
) -> Option<Vec<GeneratedFurniture>> {
    candidate
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let centre = candidate.footprint.centre_metres
                + candidate.footprint.orientation.local_to_world(item.offset);
            let supports = &item.key.recipe().support_points_metres;
            if supports.is_empty() {
                return None;
            }
            let heights = supports
                .iter()
                .map(|point| {
                    let position = centre
                        + candidate
                            .footprint
                            .orientation
                            .local_to_world(Vec2::new(point.x, point.z));
                    terrain.height_at(position).map(|height| height - point.y)
                })
                .collect::<Option<Vec<_>>>()?;
            let min = heights.iter().copied().fold(f32::INFINITY, f32::min);
            let max = heights.into_iter().fold(f32::NEG_INFINITY, f32::max);
            if max - min > MAX_SUPPORT_ERROR_METRES {
                return None;
            }
            Some(GeneratedFurniture {
                scene: SceneFurniture {
                    id: FurnitureInstanceId(mix64(candidate.id.0 ^ INSTANCE_DOMAIN ^ index as u64)),
                    key: item.key,
                    group_id: candidate.id,
                },
                position_metres: Vec3::new(centre.x, max, centre.y),
                orientation: candidate.footprint.orientation,
            })
        })
        .collect()
}
