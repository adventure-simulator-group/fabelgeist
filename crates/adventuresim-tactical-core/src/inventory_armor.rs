use adventuresim_core::equipment::LayeredArmor;

use crate::inventory::{ArmorItem, ArmorLayerContact, InventoryView, body_part_index};

impl InventoryView<'_, '_, '_> {
    pub fn armor_layer_chain(
        &self,
        part: adventuresim_core::body::BodyPart,
        sample: f32,
    ) -> Vec<ArmorLayerContact> {
        let index = body_part_index(part);
        let mut layers = self
            .iter()
            .filter_map(|item| {
                let armor = item
                    .armor
                    .filter(|armor| armor.covered_parts[index] && armor.coverage > 0.0)?;
                Some((item, armor))
            })
            .collect::<Vec<_>>();
        layers.sort_by_key(|(_, armor)| std::cmp::Reverse(armor.layer_order));
        let selected = layers.iter().position(|(_, armor)| {
            armor.coverage_geometry[index]
                .expect("covered armor part has authored surface geometry")
                .contains(sample)
        });
        layers
            .into_iter()
            .enumerate()
            .map(|(layer_index, (item, armor))| {
                let geometry = armor.coverage_geometry[index]
                    .expect("covered armor part retains authored geometry");
                ArmorLayerContact {
                    item_id: item.properties.id.clone(),
                    inventory_item_id: item.inventory_item_id.map(|id| id.0),
                    material: armor.material,
                    geometry,
                    intersected: geometry.contains(sample),
                    selected: selected == Some(layer_index),
                    surface: adventuresim_core::equipment::ArmorSurface {
                        inventory_item_id: item.inventory_item_id.map(|id| id.0),
                        material: Some(armor.material),
                        resistance: armor.resistance,
                        padding: armor.padding,
                        flexibility: armor.flexibility,
                    },
                }
            })
            .collect()
    }
}

pub(super) fn fold_armor_layers<'a>(
    index: usize,
    armor: impl IntoIterator<Item = &'a ArmorItem>,
) -> LayeredArmor {
    let mut result = LayeredArmor {
        range_of_motion: 1.0,
        ..Default::default()
    };
    let mut weighted_flexibility = 0.0;
    for armor in armor.into_iter().filter(|armor| armor.covered_parts[index]) {
        result.coverage = 1.0 - (1.0 - result.coverage) * (1.0 - armor.coverage.clamp(0.0, 1.0));
        let resistance = armor.resistance.max(0.0);
        result.resistance += resistance;
        result.padding += armor.padding.max(0.0);
        weighted_flexibility += armor.flexibility.clamp(0.0, 1.0) * resistance;
        result.range_of_motion = result
            .range_of_motion
            .min(armor.range_of_motion.clamp(0.0, 1.0));
    }
    result.flexibility = if result.resistance > f32::EPSILON {
        weighted_flexibility / result.resistance
    } else {
        0.0
    };
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{ArmorSlot, InventoryItems, InventoryViewer, ItemOf, ItemProperties};
    use adventuresim_core::{
        body::BodyPart, combat::AuthoredArmorCoverage, item_catalog, item_catalog_schema::ItemKind,
        prelude::PlayerEquipment,
    };
    use bevy::{ecs::system::SystemState, prelude::*};

    fn armor(item: &str) -> ArmorItem {
        let definition = item_catalog::definition(item).unwrap();
        let equipment = definition.equipment.as_ref().unwrap();
        let placement = &equipment.placements[0];
        let ItemKind::Armor {
            coverage,
            resistance,
            padding,
            flexibility,
            range_of_motion,
            ..
        } = definition.kind
        else {
            panic!("armor fixture")
        };
        let mut covered_parts = [false; 7];
        let mut coverage_geometry = [None; 7];
        for part in &placement.protection {
            let part = adventuresim_core::equipment::equipment_body_part(*part);
            let index = body_part_index(part);
            covered_parts[index] = true;
            coverage_geometry[index] = Some(AuthoredArmorCoverage::from_placement(placement, part));
        }
        ArmorItem {
            material: equipment.material.unwrap(),
            coverage,
            resistance,
            padding,
            flexibility,
            range_of_motion,
            covered_parts,
            coverage_geometry,
            slot: ArmorSlot::Chest,
            layer_order: placement.outermost_channel().unwrap().order(),
        }
    }

    #[test]
    fn joint_contacts_select_mail_once_and_reveal_cloth_when_mail_is_removed() {
        let mut world = World::new();
        let owner = world.spawn(InventoryItems::default()).id();
        for item in ["arming_doublet", "breastplate"] {
            world.spawn((
                ItemOf(owner),
                ItemProperties {
                    id: item.into(),
                    weight: 1.0,
                },
                armor(item),
            ));
        }
        let mail = world
            .spawn((
                ItemOf(owner),
                ItemProperties {
                    id: "mail_voiders".into(),
                    weight: 1.0,
                },
                armor("mail_voiders"),
            ))
            .id();
        let mut viewer = SystemState::<InventoryViewer>::new(&mut world);
        {
            let inventory = viewer.get(&world).unwrap();
            let view = inventory.get(owner);
            for (part, sample, expected) in [
                (BodyPart::Chest, 0.5, "breastplate"),
                (BodyPart::Chest, 0.87, "mail_voiders"),
                (BodyPart::Chest, 0.90, "mail_voiders"),
                (BodyPart::LeftArm, 0.05, "mail_voiders"),
                (BodyPart::RightArm, 0.55, "mail_voiders"),
                (BodyPart::LeftArm, 0.25, "arming_doublet"),
                (BodyPart::RightArm, 0.90, "arming_doublet"),
            ] {
                let chain = view.armor_layer_chain(part, sample);
                assert_eq!(chain.iter().filter(|layer| layer.selected).count(), 1);
                assert_eq!(
                    chain.iter().find(|layer| layer.selected).unwrap().item_id,
                    expected
                );
            }
            let chain = view.armor_layer_chain(BodyPart::LeftArm, 0.55);
            let mail_layer = chain
                .iter()
                .find(|layer| layer.item_id == "mail_voiders")
                .unwrap();
            assert_eq!(mail_layer.geometry.segments().count(), 3);
            assert_eq!(
                view.armor_surface(BodyPart::LeftArm, 0.55)
                    .unwrap()
                    .resistance,
                armor("mail_voiders").resistance
            );
        }
        world.entity_mut(mail).remove::<ArmorItem>();
        {
            let inventory = viewer.get(&world).unwrap();
            for (part, sample) in [(BodyPart::Chest, 0.87), (BodyPart::LeftArm, 0.55)] {
                let chain = inventory.get(owner).armor_layer_chain(part, sample);
                assert_eq!(
                    chain.iter().find(|layer| layer.selected).unwrap().item_id,
                    "arming_doublet"
                );
            }
        }
        world.entity_mut(mail).insert(armor("mail_voiders"));
        let inventory = viewer.get(&world).unwrap();
        let chain = inventory
            .get(owner)
            .armor_layer_chain(BodyPart::Chest, 0.90);
        assert_eq!(
            chain.iter().find(|layer| layer.selected).unwrap().item_id,
            "mail_voiders"
        );
    }
}
