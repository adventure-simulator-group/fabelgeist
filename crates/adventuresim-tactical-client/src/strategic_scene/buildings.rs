//! Anchors in the full-size tactical buildings along the shared frontage.
use super::protocol::PlaceKind;
use adventuresim_building_generator::{OpeningUse, interior::InteriorLayout};
use adventuresim_tactical_core::prelude::*;
use adventuresim_world_schema::settlement_buildings::BuildingUse;
use bevy::prelude::*;

pub(super) const ROOM_LAYER: usize = 0;
#[derive(Clone)]
pub(super) struct Venue {
    pub anchor: Vec3,
    pub approach: Vec3,
    pub positions: Vec<Transform>,
}

pub(super) fn building_use(kind: PlaceKind) -> BuildingUse {
    match kind {
        PlaceKind::Square => BuildingUse::WeighHouse,
        PlaceKind::Market => BuildingUse::GeneralShop,
        PlaceKind::Residence | PlaceKind::Camp => BuildingUse::Dwelling,
        PlaceKind::Keep => BuildingUse::Guardhouse,
        PlaceKind::Smith => BuildingUse::Weaponsmith,
        PlaceKind::Armor => BuildingUse::Armorer,
        PlaceKind::Tailor => BuildingUse::Tailor,
        PlaceKind::Apothecary => BuildingUse::Herbalist,
        PlaceKind::Books => BuildingUse::Bookshop,
        PlaceKind::Inn => BuildingUse::Inn,
        PlaceKind::Church => BuildingUse::ParishChurch,
        PlaceKind::Guild => BuildingUse::Guildhall,
    }
}

pub(super) fn transform(building: &GeneratedBuilding) -> Transform {
    let bounds = building.collision.bounds;
    Transform::from_xyz(
        building.placement.centre_metres.x,
        building.pad_elevation_metres + bounds.centre().y - bounds.min.y,
        building.placement.centre_metres.y,
    )
    .with_rotation(Quat::from_rotation_y(
        building.placement.orientation.yaw_radians(),
    ))
}

impl Venue {
    pub(super) fn from_building(
        building: &GeneratedBuilding,
        layout: &InteriorLayout,
    ) -> Result<Self, String> {
        let bounds = building.collision.bounds;
        let transform = transform(building);
        let positions = super::staging::positions(building, layout);
        let anchor = positions
            .first()
            .ok_or("building has no clear conversation position")?
            .translation;
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
        let distance = (bounds.max - bounds.min).xz().max_element() * 0.8 + 4.0;
        let target = transform.translation;
        let eye =
            Vec3::new(target.x, building.pad_elevation_metres + 1.7, target.z) + outward * distance;
        Ok(Self {
            anchor,
            approach: eye,
            positions,
        })
    }
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
        let mut generated = input.generate().unwrap();
        let mut street = None;
        let venues = prepare_venues(&input, &view, &mut generated, &mut street).unwrap();
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
        let collision = adventuresim_building_generator::compile_building_collision(&plan);
        let building = GeneratedBuilding {
            placement,
            plan,
            collision,
            pad_elevation_metres: 0.0,
        };
        let layout = adventuresim_building_generator::interior::furnish(
            &building.plan,
            &building.placement.program,
        )
        .unwrap();
        let venue = Venue::from_building(&building, &layout).unwrap();
        let pose = transform(&building);
        let field = crate::presentation::interior_lighting::InteriorField::from_plan(
            &building.plan,
            Vec3::ZERO,
        );
        assert_eq!(pose.scale, Vec3::ONE);
        let local = pose
            .compute_affine()
            .inverse()
            .transform_point3(venue.anchor)
            + building.collision.bounds.centre();
        let bounds = building.collision.bounds;
        assert!(local.x >= bounds.min.x && local.x <= bounds.max.x);
        assert!(local.z >= bounds.min.z && local.z <= bounds.max.z);
        assert!(local.y >= -0.001 && local.y < 0.3, "floor: {local:?}");
        assert!(
            field.daylight_at(local + Vec3::Y) > 0.0,
            "selected room must receive actual daylight: {local:?}"
        );
        assert!((venue.approach.y - building.pad_elevation_metres - 1.7).abs() < 0.01);
    }
}

pub(super) fn prepare_venues(
    input: &TacticalSceneInput,
    view: &super::protocol::StrategicView,
    generated: &mut GeneratedTacticalScene,
    street: &mut Option<super::street::Street>,
) -> Result<std::collections::HashMap<super::protocol::PlaceId, Venue>, String> {
    let mut venues = std::collections::HashMap::new();
    let (promoted, selected) = select_buildings(input, view, generated)?;
    generated
        .furniture
        .furnish_interiors(&promoted)
        .map_err(|e| e.to_string())?;
    generated.buildings.extend(promoted);
    *street = view
        .places
        .iter()
        .any(|place| place.kind != PlaceKind::Camp)
        .then(|| super::street::Street::arrange(input, &view.places, &selected, generated));
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
                .ok_or("building has no prepared interior")?;
            Venue::from_building(building, &interior.layout)?
        } else {
            // Public squares and camps are real outdoor positions on the terrain.
            let anchor = Vec3::Y * generated.terrain.height_at(Vec2::ZERO).unwrap_or_default();
            Venue {
                anchor,
                positions: vec![Transform::from_translation(anchor)],
                approach: anchor + Vec3::new(0.0, 1.7, 12.0),
            }
        };
        if matches!(place.kind, PlaceKind::Square | PlaceKind::Camp) {
            let position = venue.approach;
            venue.anchor = Vec3::new(
                position.x,
                generated
                    .terrain
                    .height_at(position.xz())
                    .unwrap_or(venue.anchor.y),
                position.z,
            );
            venue.positions = vec![Transform::from_translation(venue.anchor)];
        }
        venues.insert(place.id.clone(), venue);
    }
    Ok(venues)
}

fn select_buildings(
    input: &TacticalSceneInput,
    view: &super::protocol::StrategicView,
    generated: &mut GeneratedTacticalScene,
) -> Result<
    (
        Vec<GeneratedBuilding>,
        std::collections::HashMap<super::protocol::PlaceId, u64>,
    ),
    String,
> {
    let mut promoted = Vec::new();
    let mut selected = std::collections::HashMap::new();
    for place in &view.places {
        let operator_building = input
            .establishments
            .iter()
            .find(|establishment| {
                view.people.iter().any(|person| {
                    person.place == place.id && person.id.0 == establishment.operator_character_id
                })
            })
            .map(|establishment| establishment.building_id);
        let usage = building_use(place.kind);
        let id = operator_building
            .or_else(|| {
                input
                    .buildings
                    .iter()
                    .find(|building| building.program.usage == Some(usage))
                    .map(|building| building.id)
            })
            .or_else(|| {
                input
                    .distant_buildings
                    .iter()
                    .find(|building| building.usage == Some(usage))
                    .map(|building| building.id)
            });
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
                    .ok_or("missing building placement")?;
                let program = placement.program();
                let recipe = generated
                    .building_recipes
                    .get_or_generate(&program)
                    .map_err(|e| e.to_string())?;
                promoted.push(GeneratedBuilding {
                    placement: TacticalBuildingPlacement {
                        id,
                        program,
                        centre_metres: placement.centre_metres,
                        orientation: placement.orientation,
                    },
                    plan: recipe.plan.clone(),
                    collision: recipe.collision.clone(),
                    pad_elevation_metres: placement.base_elevation_metres,
                });
            }
        }
    }
    Ok((promoted, selected))
}
