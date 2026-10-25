//! Physical scenery must leave complete accepted bearings and approaches clear.
use super::*;

pub(super) fn remove(
    input: &TacticalSceneInput,
    terrain: &SceneTerrain,
    obstacles: &mut Vec<GeneratedObstacle>,
    repairs: &mut SceneRepairReport,
) {
    let Some(projection) = &input.grounding else {
        return;
    };
    let extent = Vec2::new(terrain.width(), terrain.depth());
    // The pedestrian's cylinder extends beyond an approach endpoint. Reserving
    // only obstacle centres, or only the bare support polygon, leaves its end
    // cap obstructed while advertising usable access. This is physical clearance,
    // not a larger grading region or a new movement allowance.
    let clearance = adventuresim_core::combat::HUMANOID_COLLISION_RADIUS_METRES
        + bevy_ahoy::CharacterController::default()
            .move_and_slide
            .skin_width;
    let before = obstacles.len();
    obstacles.retain(|obstacle| {
        let (x, z, radius) = match *obstacle {
            GeneratedObstacle::Tree { x, z } => (x, z, TREE_TRUNK_RADIUS_METRES),
            GeneratedObstacle::Rock { x, z, recipe } => (x, z, recipe.collision_radius_metres()),
        };
        let centre =
            Vec2::new(f32::from(x), f32::from(z)) * input.playable.spacing_metres - extent * 0.5;
        !projection
            .surfaces()
            .iter()
            .any(|surface| surface.intersects_disc(centre, radius + clearance))
    });
    repairs.removed_building_obstacles += (before - obstacles.len()) as u32;
}

#[cfg(test)]
mod tests {
    use crate::city_layout::grounding::PropertySupportSurface;
    use bevy::math::Vec2;

    #[test]
    fn kassel_1_approach_reserves_the_complete_rock_envelope() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../assets/tactical-grounding/kassel-1-access-obstacle.json"
        ))
        .unwrap();
        let surface: PropertySupportSurface =
            serde_json::from_value(fixture["surface"].clone()).unwrap();
        let centre = Vec2::new(26.0, 3.0);
        let radius = fixture["radius_metres"].as_f64().unwrap() as f32;
        assert_eq!(surface.property_id().0, 1);
        assert_eq!(
            surface.member_building_ids(),
            [1].map(crate::scene_input::SceneBuildingId)
        );
        assert!(
            !surface.contains(crate::scene_coordinates::ScenePlanPoint::try_from(centre).unwrap()),
            "fixture must expose the centre-only omission"
        );
        assert!(
            !surface.intersects_disc(centre, radius),
            "rock lies beyond the bare support polygon"
        );
        let clearance = adventuresim_core::combat::HUMANOID_COLLISION_RADIUS_METRES
            + bevy_ahoy::CharacterController::default()
                .move_and_slide
                .skin_width;
        assert!(surface.intersects_disc(centre, radius + clearance));
        assert!(!surface.intersects_disc(Vec2::new(50.0, 40.0), radius));
        assert!(!surface.intersects_disc(centre, 0.0));
        let before = serde_json::to_value(&surface).unwrap();
        let mut reversed = before.clone();
        reversed["regions"].as_array_mut().unwrap().reverse();
        reversed["clipping_outlines"]
            .as_array_mut()
            .unwrap()
            .reverse();
        let reversed: PropertySupportSurface = serde_json::from_value(reversed).unwrap();
        assert_eq!(
            surface.intersects_disc(centre, radius + clearance),
            reversed.intersects_disc(centre, radius + clearance)
        );
        assert_eq!(serde_json::to_value(surface).unwrap(), before);
    }
}
