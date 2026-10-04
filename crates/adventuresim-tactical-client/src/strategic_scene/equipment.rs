//! Presentation-only equipment entities feed the shared tactical mesh generators.
use super::{RetainedScene, protocol::StrategicView};
use adventuresim_core::{equipment_presentation::*, item_catalog};
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub(super) struct SceneEquipment;

#[derive(Component, Default)]
pub(super) struct Wardrobe(Vec<EquipmentAppearance>, Vec<Entity>);

pub(super) fn sync_equipment(
    mut commands: Commands,
    view: Option<Res<StrategicView>>,
    scene: Res<RetainedScene>,
    wardrobes: Query<&Wardrobe>,
) {
    let Some(view) = view else {
        return;
    };
    if !view.is_changed() && !scene.is_changed() {
        return;
    }
    for person in &view.people {
        let Some(model) = scene.people.get(&person.id) else {
            continue;
        };
        let owner = model.entity;
        if let Ok(current) = wardrobes.get(owner) {
            if current.0 == person.equipment {
                continue;
            }
            for entity in &current.1 {
                commands.entity(*entity).despawn();
            }
        }
        let items: HashMap<_, _> = person
            .equipment
            .iter()
            .map(|item| (item.id.clone(), commands.spawn_empty().id()))
            .collect();
        for item in &person.equipment {
            spawn_item(&mut commands, owner, item, &items);
        }
        commands.entity(owner).insert(Wardrobe(
            person.equipment.clone(),
            items.into_values().collect(),
        ));
    }
}

fn spawn_item(
    commands: &mut Commands,
    owner: Entity,
    item: &EquipmentAppearance,
    items: &HashMap<PresentationId, Entity>,
) {
    let entity = items[&item.id];
    let Some(definition) = item_catalog::definition(&(&item.item).into()) else {
        commands.entity(entity).despawn();
        return;
    };
    let Some(equipment) = &definition.equipment else {
        commands.entity(entity).despawn();
        return;
    };
    let physical = equipment.physical;
    let mut entity = commands.entity(entity);
    entity.insert((
        SceneEquipment,
        ItemOf(owner),
        ChildOf(owner),
        Transform::IDENTITY,
        ItemProperties {
            id: item.item.clone(),
            weight: definition.weight_kg,
        },
        TacticalEquipmentPhysical {
            dimensions_m: Vec3::from_array(physical.dimensions_m),
            grip_to_tip_m: physical.grip_to_tip_m,
            // These entities present equipment; they have no striking volume.
            striking_head_length_m: 0.0,
            anchor_offset_m: Vec3::from_array(physical.anchor_offset_m),
        },
        EquipmentTopology {
            placement_id: Some(item.placement.clone()),
            occupancies: item
                .occupancies
                .iter()
                .enumerate()
                .filter_map(|(index, row)| {
                    Some(EquipmentTopologyOccupancy {
                        occupancy_id: format!("presentation:{}:{index}", item.id.0),
                        anchor: match &row.anchor {
                            AppearanceAnchor::Character(location) => {
                                TacticalEquipmentAnchor::CharacterLocation(*location)
                            }
                            AppearanceAnchor::Attachment { parent, point } => {
                                TacticalEquipmentAnchor::ItemAttachment {
                                    parent: *items.get(parent)?,
                                    attachment_point_id: point.clone(),
                                }
                            }
                        },
                        channel: row.channel,
                        order: row.order,
                        requirement_index: row.requirement_index,
                        capacity_index: row.capacity_index,
                    })
                })
                .collect(),
        },
    ));
    if let Some(recipe) = &item.weapon {
        entity.insert(WeaponAppearance {
            generator_version: recipe.generator_version,
            design_hash: recipe.design_hash,
            recipe: recipe.recipe.clone(),
        });
    }
    if let Some(recipe) = &item.holder {
        entity.insert(WeaponHolderAppearance {
            generator_version: recipe.generator_version,
            design_hash: recipe.design_hash,
            recipe: recipe.recipe.clone(),
        });
    }
    if let Some(slot) = held_slot(item) {
        entity.insert(slot);
    }
}

fn held_slot(item: &EquipmentAppearance) -> Option<EquipSlot> {
    use adventuresim_core::item_catalog::{EquipmentChannel, EquipmentLocation};
    item.occupancies.iter().find_map(|row| match row.anchor {
        AppearanceAnchor::Character(EquipmentLocation::LeftHand)
            if row.channel == EquipmentChannel::Held =>
        {
            Some(EquipSlot::HoldingLeft)
        }
        AppearanceAnchor::Character(EquipmentLocation::RightHand)
            if row.channel == EquipmentChannel::Held =>
        {
            Some(EquipSlot::HoldingRight)
        }
        _ => None,
    })
}
