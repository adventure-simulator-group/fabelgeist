//! Bounded street-detail residency for a freely explored city showcase.
use super::*;
use std::collections::BTreeMap;
use traffic::{TrafficMask, TrafficNetwork, TrafficTile};

const DETAIL_RADIUS_METRES: f32 = 110.0;
const NETWORK_MARGIN_METRES: f32 = 96.0;

#[derive(Resource)]
pub(crate) struct StreamCityTraffic;

#[derive(Component)]
pub(crate) struct StreamedTrafficTile(pub(super) TrafficTile);

#[derive(Resource)]
pub(crate) struct CityTrafficResidency {
    streets: Vec<CityStreetPatch>,
    neutral: TrafficMask,
    masks: BTreeMap<TrafficTile, TrafficMask>,
}

impl CityTrafficResidency {
    pub(super) fn new(streets: Vec<CityStreetPatch>, neutral: TrafficMask) -> Self {
        Self {
            streets,
            neutral,
            masks: BTreeMap::new(),
        }
    }
}

pub(crate) fn update(
    residency: Option<ResMut<CityTrafficResidency>>,
    cameras: Query<&GlobalTransform, With<TacticalGameplayCamera>>,
    tiles: Query<(&StreamedTrafficTile, &MeshMaterial3d<CityGroundMaterial>)>,
    mut materials: ResMut<Assets<CityGroundMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(mut residency) = residency else {
        return;
    };
    let Some(camera) = cameras.iter().next() else {
        return;
    };
    let position = camera.translation();
    let mut wanted = tiles
        .iter()
        .map(|(tile, _)| tile.0)
        .filter(|tile| {
            let centre = tile.corners().into_iter().sum::<Vec2>() * 0.25;
            Vec3::new(centre.x, 0.0, centre.y).distance(position) < DETAIL_RADIUS_METRES
        })
        .collect::<Vec<_>>();
    wanted.sort();
    wanted.dedup();
    residency.masks.retain(|tile, _| wanted.contains(tile));
    // At most one tile per frame, with no whole-city wheel-stroke allocation.
    if let Some(tile) = wanted
        .into_iter()
        .filter(|tile| !residency.masks.contains_key(tile))
        .min_by(|a, b| {
            a.corners()[0]
                .distance_squared(position.xz())
                .total_cmp(&b.corners()[0].distance_squared(position.xz()))
        })
    {
        let streets = nearby_streets(&residency.streets, tile);
        let network = TrafficNetwork::new(&streets);
        residency
            .masks
            .insert(tile, TrafficMask::bake(&network, tile, &mut images));
    }
    for (tile, handle) in &tiles {
        let mask = residency.masks.get(&tile.0).unwrap_or(&residency.neutral);
        if materials
            .get(&handle.0)
            .is_some_and(|material| material.extension.traffic_mask != mask.image)
        {
            let mut material = materials
                .get_mut(&handle.0)
                .expect("existing city material");
            material.extension.traffic_mask = mask.image.clone();
            material.extension.traffic_transform = mask.transform;
        }
    }
}

fn nearby_streets(streets: &[CityStreetPatch], tile: TrafficTile) -> Vec<CityStreetPatch> {
    let corners = tile.corners();
    let minimum = corners[0] - Vec2::splat(NETWORK_MARGIN_METRES);
    let maximum = corners[2] + Vec2::splat(NETWORK_MARGIN_METRES);
    streets
        .iter()
        .copied()
        .filter(|street| {
            let (min, max) = match *street {
                CityStreetPatch::Corridor {
                    start_metres,
                    end_metres,
                    half_width_metres,
                    ..
                } => (
                    start_metres.min(end_metres) - Vec2::splat(half_width_metres),
                    start_metres.max(end_metres) + Vec2::splat(half_width_metres),
                ),
                CityStreetPatch::Market { corners_metres, .. } => (
                    corners_metres
                        .into_iter()
                        .fold(Vec2::splat(f32::INFINITY), Vec2::min),
                    corners_metres
                        .into_iter()
                        .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max),
                ),
            };
            min.cmple(maximum).all() && max.cmpge(minimum).all()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn street_detail_never_builds_the_whole_city_network() {
        let input: TacticalSceneInput = serde_json::from_str(include_str!(
            "../../../../../../assets/tactical-scenes/massive-city.json"
        ))
        .unwrap();
        let streets = nearby_streets(&input.streets, TrafficTile(0, 0));
        assert!(!streets.is_empty());
        assert!(streets.len() < input.streets.len() / 5);
        assert!(TrafficNetwork::new(&streets).stroke_count() < 300_000);
    }
}
