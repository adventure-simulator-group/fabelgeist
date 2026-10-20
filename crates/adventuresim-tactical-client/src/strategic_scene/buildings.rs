//! Anchors in the full-size tactical buildings along the shared frontage.
use super::preparation::{PreparationError, PreparationResult};
use super::protocol::{PlaceId, PlaceKind};
use super::staging::ConversationPose;
use crate::presentation::venues::select_building;
use adventuresim_building_generator::spatial_geometry::Position;
use adventuresim_building_generator::{OpeningUse, interior::InteriorLayout};
use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_core::scene_coordinates::{Scene, ScenePlanPoint};
use bevy::prelude::*;

pub(super) const ROOM_LAYER: usize = 0;
#[derive(Clone)]
pub(super) struct Venue {
    pub anchor: Position<Scene>,
    pub approach: Position<Scene>,
    pub positions: Vec<ConversationPose>,
}

struct SelectedBuildings {
    promoted: Vec<GeneratedBuilding>,
    selected: std::collections::HashMap<
        PlaceId,
        adventuresim_tactical_core::scene_input::SceneBuildingId,
    >,
}

impl Venue {
    pub(super) fn from_building(
        building: &GeneratedBuilding,
        layout: &InteriorLayout,
    ) -> PreparationResult<Self> {
        let bounds = building.collision.bounds;
        let transform = building.transform()?;
        let positions = ConversationPose::in_building(building, layout)?;
        let anchor = positions
            .first()
            .ok_or(PreparationError::MissingConversationPosition {
                building: building.placement.id,
            })?
            .anchor;
        let door = building.plan.opening_assemblies.iter().find(|opening| {
            matches!(opening.use_kind, OpeningUse::Door | OpeningUse::Gate)
                && opening.frame.outside_room.is_none()
                && opening.sill_elevation_metres.abs() < 0.1
        });
        let outward = door.map_or(Vec3::Z, |door| {
            Vec3::new(door.frame.outward.x, 0.0, door.frame.outward.y)
        });
        let outward = transform.rotation * outward;
        // A standing-eye perspective from the real approach, with the entire facade in view.
        let distance = bounds.extent()?.metres().xz().max_element() * 0.8 + 4.0;
        let target = transform.translation;
        let eye = Vec3::new(
            target.x,
            building.placement.base_elevation_metres.metres() + 1.7,
            target.z,
        ) + outward * distance;
        Ok(Self {
            anchor,
            approach: Position::from_metres(eye)?,
            positions,
        })
    }
}

impl SelectedBuildings {
    fn prepare(
        input: &TacticalSceneInput,
        view: &super::protocol::StrategicView,
        generated: &mut crate::presentation::PreparedStrategicScene,
    ) -> PreparationResult<Self> {
        let mut promoted = Vec::new();
        let mut selected = std::collections::HashMap::new();
        for place in &view.places {
            let id = select_building(
                input,
                place.kind,
                view.people
                    .iter()
                    .filter(|p| p.place == place.id)
                    .map(|p| p.id.0),
            );
            if let Some(id) = id {
                selected.insert(place.id.clone(), id);
                if !generated
                    .buildings
                    .iter()
                    .chain(promoted.iter())
                    .any(|b| b.placement.id == id)
                {
                    let placement = input
                        .distant_buildings
                        .iter()
                        .find(|b| b.id == id)
                        .ok_or(PreparationError::MissingBuildingPlacement { building: id })?;
                    let program = placement.occupied_program();
                    let recipe = generated.building_recipes.get_or_generate(&program)?;
                    promoted.push(GeneratedBuilding {
                        placement: (*placement).into(),
                        plan: recipe.plan.clone(),
                        collision: recipe.collision.clone(),
                    });
                }
            }
        }
        Ok(Self { promoted, selected })
    }
}

pub(super) fn prepare_venues(
    input: &TacticalSceneInput,
    view: &super::protocol::StrategicView,
    generated: &mut crate::presentation::PreparedStrategicScene,
    street: &mut Option<super::street::Street>,
    backdrop: &crate::presentation::StrategicBackdrop,
) -> PreparationResult<std::collections::HashMap<PlaceId, Venue>> {
    let mut venues = std::collections::HashMap::new();
    let SelectedBuildings { promoted, selected } =
        SelectedBuildings::prepare(input, view, generated)?;
    generated.furniture.furnish_interiors(&promoted)?;
    generated.buildings.extend(promoted);
    *street = if view
        .places
        .iter()
        .any(|place| place.kind != PlaceKind::Camp)
    {
        Some(super::street::Street::arrange(
            input,
            &view.places,
            &selected,
            generated,
            backdrop,
        )?)
    } else {
        None
    };
    for place in &view.places {
        let mut venue = if let Some(building) = selected
            .get(&place.id)
            .and_then(|id| generated.buildings.iter().find(|b| b.placement.id == *id))
        {
            let interior = generated
                .furniture
                .interiors
                .iter()
                .find(|interior| interior.building_id == building.placement.id)
                .ok_or(PreparationError::MissingInterior {
                    building: building.placement.id,
                })?;
            Venue::from_building(building, &interior.layout)?
        } else {
            // Public squares and camps are real outdoor positions on the terrain.
            let elevation = backdrop.required_height_at(ScenePlanPoint::ORIGIN)?;
            let anchor = Position::from_metres(Vec3::Y * elevation.metres())?;
            Venue {
                anchor,
                positions: vec![ConversationPose {
                    anchor,
                    facing: Quat::IDENTITY,
                }],
                approach: Position::from_metres(anchor.metres() + Vec3::new(0.0, 1.7, 12.0))?,
            }
        };
        if matches!(place.kind, PlaceKind::Square | PlaceKind::Camp) {
            let position = venue.approach;
            let elevation = backdrop.required_height_at(ScenePlanPoint::from(position))?;
            venue.anchor = Position::from_metres(Vec3::new(
                position.metres().x,
                elevation.metres(),
                position.metres().z,
            ))?;
            venue.positions = vec![ConversationPose {
                anchor: venue.anchor,
                facing: Quat::IDENTITY,
            }];
        }
        venues.insert(place.id.clone(), venue);
    }
    Ok(venues)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn camp_has_no_settlement_street() {
        let input: TacticalSceneInput = serde_json::from_str(include_str!(
            "../../../../assets/tactical-scenes/flat-dry-grassland.json"
        ))
        .unwrap();
        let view = serde_json::from_value(serde_json::json!({
            "revision": 1, "location": "camp", "places": [{"id": "camp", "kind": "camp"}],
            "people": [], "active_place": "camp", "selected": null, "street": null,
            "stage": null, "forge": null, "portraits": []
        }))
        .unwrap();
        let mut generated = crate::presentation::PreparedStrategicScene::generate(&input).unwrap();
        let backdrop =
            crate::presentation::StrategicBackdrop::new(&input, input.vista.lods.len()).unwrap();
        let mut street = None;
        let venues = prepare_venues(&input, &view, &mut generated, &mut street, &backdrop).unwrap();
        assert!(street.is_none());
        assert_eq!(venues.len(), 1);
    }

    #[test]
    fn interior_and_frontage_use_the_tactical_placement_without_scaling() {
        let input: adventuresim_tactical_core::prelude::TacticalSceneInput = serde_json::from_str(
            include_str!("../../../../assets/tactical-scenes/massive-city.json"),
        )
        .unwrap();
        let placement = input.buildings[0].clone();
        let plan = adventuresim_building_generator::generate(&placement.program).unwrap();
        let collision = adventuresim_building_generator::compile_building_collision(&plan).unwrap();
        let building = GeneratedBuilding {
            placement,
            plan,
            collision,
        };
        let layout = adventuresim_building_generator::interior::furnish(
            &building.plan,
            &building.placement.program,
        )
        .unwrap();
        let venue = Venue::from_building(&building, &layout).unwrap();
        let pose = building.transform().unwrap();
        let field = crate::presentation::interior_lighting::InteriorField::from_plan(
            &building.plan,
            Vec3::ZERO,
        );
        assert_eq!(pose.scale, Vec3::ONE);
        let local = pose
            .compute_affine()
            .inverse()
            .transform_point3(venue.anchor.metres())
            + building.collision.bounds.centre().unwrap().metres();
        let bounds = building.collision.bounds;
        assert!(local.x >= bounds.min().metres().x && local.x <= bounds.max().metres().x);
        assert!(local.z >= bounds.min().metres().z && local.z <= bounds.max().metres().z);
        assert!(local.y >= -0.001 && local.y < 0.3, "floor: {local:?}");
        assert!(
            field.daylight_at(local + Vec3::Y) > 0.0,
            "selected room must receive actual daylight: {local:?}"
        );
        assert!(
            (venue.approach.metres().y - building.placement.base_elevation_metres.metres() - 1.7)
                .abs()
                < 0.01
        );
    }
}
