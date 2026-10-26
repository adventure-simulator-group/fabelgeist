use super::*;
use crate::scene_coordinates::ScenePlanPoint;
use bevy::math::Vec3Swizzles;
use candidates::Candidate;
use fabelgeist_determinism::StreamId;
use ground::PlacementGround;

const MAX_SUPPORT_ERROR_METRES: f32 = 0.045;
const MAX_GROUP_GRADE: f32 = 0.08;
const INSTANCE_DOMAIN: StreamId = StreamId::new("furniture.outdoor-identity");

pub(super) fn generate(
    input: &TacticalSceneInput,
    buildings: &[sites::FurnitureSite],
    terrain: &SceneTerrain,
    ground: &SceneGround,
    obstacles: &[GeneratedObstacle],
) -> Result<FurnitureLayout, super::super::SceneInputError> {
    let mut layout = FurnitureLayout {
        reserved_routes: reservations::routes(input, buildings)?,
        ..Default::default()
    };
    let mut occupied = occupancy::Occupancy::default();
    let support = PlacementGround::new(input, terrain, ground);
    let market = candidates::market(input)?;
    layout.reserved_routes.extend(market.aisles);
    for footprint in reservations::obstacles(input, terrain, buildings, obstacles)?
        .into_iter()
        .chain(layout.reserved_routes.iter().copied())
    {
        occupied.insert(footprint);
    }
    for candidate in market.groups {
        accept(input, &support, candidate, &mut occupied, &mut layout);
    }
    let mut ordered = buildings.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|building| building.placement.id);
    for building in ordered {
        let mut accepted = 0;
        for candidate in candidates::building(input, building)? {
            if accept(input, &support, candidate, &mut occupied, &mut layout) {
                accepted += 1;
                if accepted >= candidates::group_limit(building) {
                    break;
                }
            }
        }
    }
    Ok(layout)
}

fn accept(
    input: &TacticalSceneInput,
    support: &PlacementGround,
    candidate: Candidate,
    occupied: &mut occupancy::Occupancy,
    layout: &mut FurnitureLayout,
) -> bool {
    let footprint = candidate.footprint;
    let Some(playable) = support.scope(footprint) else {
        return false;
    };
    if occupied.intersects(footprint) {
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
        .map(ScenePlanPoint::from_metres)
        .chain([Some(footprint.centre())])
        .map(|point| {
            let point = point?;
            if input
                .landform
                .is_some_and(|landform| landform.transition_collar().contains(point))
                || !support.allows_activity(point)
            {
                return None;
            }
            support.height_at(point)
        })
        .collect::<Option<Vec<_>>>();
    let Some(samples) = samples else {
        return false;
    };
    let min = samples
        .iter()
        .map(|height| height.metres())
        .fold(f32::INFINITY, f32::min);
    let max = samples
        .into_iter()
        .map(|height| height.metres())
        .fold(f32::NEG_INFINITY, f32::max);
    if max - min > footprint.half_extents().metres().length() * 2.0 * MAX_GROUP_GRADE {
        return false;
    }
    let Some(instances) = supported_instances(&candidate, support) else {
        return false;
    };
    occupied.insert(footprint);
    layout.groups.push(FurnitureGroup {
        id: candidate.id,
        kind: candidate.kind,
        anchor: candidate.anchor,
        footprint,
    });
    if matches!(playable, ground::PlacementScope::Playable) {
        layout.instances.extend(instances);
    } else {
        layout.distant_instances.extend(instances);
    }
    true
}

fn supported_instances(
    candidate: &Candidate,
    support: &PlacementGround,
) -> Option<Vec<GeneratedFurniture>> {
    candidate
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let centre = ScenePlanPoint::from_metres(
                candidate.footprint.centre().metres()
                    + candidate
                        .footprint
                        .orientation()
                        .local_to_world(item.offset.metres().xz()),
            )?;
            let supports = &item.recipe.support_points_metres;
            if supports.is_empty() {
                return None;
            }
            let heights = supports
                .iter()
                .map(|point| {
                    let position = centre.metres()
                        + candidate
                            .footprint
                            .orientation()
                            .local_to_world(Vec2::new(point.metres().x, point.metres().z));
                    support
                        .height_at(ScenePlanPoint::from_metres(position)?)
                        .map(|height| height.metres() - point.metres().y)
                })
                .collect::<Option<Vec<_>>>()?;
            let min = heights.iter().copied().fold(f32::INFINITY, f32::min);
            let max = heights.into_iter().fold(f32::NEG_INFINITY, f32::max);
            if max - min > MAX_SUPPORT_ERROR_METRES {
                return None;
            }
            Some(GeneratedFurniture {
                scene: SceneFurniture {
                    id: FurnitureInstanceId(
                        INSTANCE_DOMAIN
                            .seed(candidate.id.0.into(), &[index as u64])
                            .to_u64(),
                    ),
                    key: item.key,
                    location: FurnitureLocation::Outdoor {
                        group_id: candidate.id,
                    },
                },
                position_metres:
                    adventuresim_building_generator::spatial_geometry::Position::from_metres(
                        Vec3::new(centre.metres().x, max, centre.metres().y),
                    )
                    .ok()?,
                orientation: candidate.footprint.orientation(),
            })
        })
        .collect()
}
