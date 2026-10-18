//! Strategic skyline admission shares the existing static city assembly.
use super::*;

pub(crate) fn queue_strategic_city(
    commands: &mut Commands,
    input: &TacticalSceneInput,
    occupied: &[adventuresim_tactical_core::scene_input::GeneratedBuilding],
) {
    let placements = input
        .distant_buildings
        .iter()
        .filter(|placement| {
            !occupied
                .iter()
                .any(|building| building.placement.id == placement.id)
        })
        .copied()
        .collect::<Vec<_>>();
    commands.insert_resource(StreamCityTraffic);
    commands.insert_resource(PendingCityBuildings::new(
        &placements,
        &input.establishments,
    ));
}
