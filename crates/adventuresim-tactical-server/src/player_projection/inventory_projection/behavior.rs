//! Inventory refresh exercises the owning ECS relationships and live bodies.

use adventuresim_core::prelude::PlayerEquipment;
use adventuresim_stdb_client::{ConnectedInventory, ConnectedPlayerItem, TacticalActorTime};
use adventuresim_tactical_core::{
    inventory::{InventoryViewer, ItemProperties},
    prelude::*,
};
use bevy::{ecs::system::SystemState, prelude::*};

use super::refresh::refresh_snapshots;
use crate::stdb::ConnectedActorInventory;

fn actor(world: &mut World, id: u64) -> Entity {
    world
        .spawn((
            Player {
                name: format!("Actor {id}"),
            },
            CharacterId(id),
            Transform::from_xyz(7.0, 3.0, 2.0),
            EquipmentActionState::default(),
            TacticalCombatState {
                fatigue: 0.25,
                starting_blood_fraction: 0.75,
                ..default()
            },
        ))
        .id()
}

fn snapshot(character: u64, items: Vec<ConnectedPlayerItem>) -> ConnectedActorInventory {
    ConnectedActorInventory {
        character: CharacterId(character),
        time: TacticalActorTime::MissionSnapshot,
        inventory: ConnectedInventory::Available(items),
    }
}

fn load(world: &mut World, owner: Entity) -> f32 {
    let mut viewer = SystemState::<InventoryViewer>::new(world);
    viewer.get(world).unwrap().get(owner).inventory_weight()
}

fn row_entity(world: &mut World, row: u64) -> Entity {
    world
        .query::<(Entity, &TacticalInventoryItemId)>()
        .iter(world)
        .find_map(|(entity, id)| (id.0 == row).then_some(entity))
        .unwrap()
}

#[test]
fn refresh_preserves_bodies_and_entities_across_partial_use_transfer_and_removal() {
    let mut world = World::new();
    world.insert_resource(TacticalCombatConfig::default());
    let first = actor(&mut world, 1);
    let second = actor(&mut world, 2);
    let mut lot = item(11, 3, 2.0, 1.5);
    let mut container = item(12, 1, 1.0, 4.0);
    container.physical_object_id = Some(50);
    refresh_snapshots(
        &mut world,
        vec![
            snapshot(1, vec![lot.clone(), container.clone()]),
            snapshot(2, vec![]),
        ],
    )
    .unwrap();
    let lot_entity = row_entity(&mut world, 11);
    let container_entity = row_entity(&mut world, 12);
    assert_eq!(load(&mut world, first), 5.5);
    assert_eq!(
        world
            .get::<ItemProperties>(container_entity)
            .unwrap()
            .weight,
        1.0
    );

    lot.carried_mass_kg = 0.5;
    container.inventory_item_id = 22;
    container.carried_mass_kg = 3.0;
    let next = vec![
        snapshot(2, vec![container.clone()]),
        snapshot(1, vec![lot.clone()]),
    ];
    refresh_snapshots(&mut world, next).unwrap();
    assert_eq!(row_entity(&mut world, 11), lot_entity);
    assert_eq!(row_entity(&mut world, 22), container_entity);
    assert_eq!(world.get::<ItemOf>(container_entity).unwrap().0, second);
    assert_eq!(load(&mut world, first), 0.5);
    assert_eq!(load(&mut world, second), 3.0);
    for owner in [first, second] {
        assert_eq!(
            world.get::<Transform>(owner).unwrap().translation,
            Vec3::new(7.0, 3.0, 2.0)
        );
        let body = world.get::<TacticalCombatState>(owner).unwrap();
        assert_eq!(body.fatigue, 0.25);
        assert_eq!(body.starting_blood_fraction, 0.75);
        assert_eq!(
            world.get::<EquipmentActionState>(owner).unwrap().revision,
            EquipmentRevision::INITIAL.next()
        );
    }
    refresh_snapshots(
        &mut world,
        vec![snapshot(1, vec![lot]), snapshot(2, vec![container.clone()])],
    )
    .unwrap();
    assert_eq!(
        world.get::<EquipmentActionState>(first).unwrap().revision,
        EquipmentRevision::INITIAL.next()
    );
    refresh_snapshots(
        &mut world,
        vec![snapshot(1, vec![]), snapshot(2, vec![container])],
    )
    .unwrap();
    assert!(world.get_entity(lot_entity).is_err());
    assert_eq!(row_entity(&mut world, 22), container_entity);
    assert_eq!(load(&mut world, first), 0.0);
    assert_eq!(load(&mut world, second), 3.0);
}

#[test]
fn invalid_snapshot_cannot_partly_replace_an_existing_inventory() {
    let mut world = World::new();
    world.insert_resource(TacticalCombatConfig::default());
    let owner = actor(&mut world, 1);
    refresh_snapshots(&mut world, vec![snapshot(1, vec![item(11, 1, 2.0, 2.0)])]).unwrap();
    let entity = row_entity(&mut world, 11);
    let count = world.entities().len();
    let mut bad = item(12, 1, 1.0, f32::NAN);
    assert!(
        refresh_snapshots(
            &mut world,
            vec![snapshot(1, vec![item(11, 1, 2.0, 0.5), bad.clone()])]
        )
        .is_err()
    );
    assert_eq!(world.entities().len(), count);
    assert_eq!(load(&mut world, owner), 2.0);
    assert_eq!(row_entity(&mut world, 11), entity);
    bad.carried_mass_kg = 1.0;
    bad.quantity = 0;
    assert!(refresh_snapshots(&mut world, vec![snapshot(1, vec![bad])]).is_err());
    assert_eq!(load(&mut world, owner), 2.0);
    assert_eq!(
        world.get::<EquipmentActionState>(owner).unwrap().revision,
        EquipmentRevision::INITIAL
    );
}

#[test]
fn retained_owner_receives_inventory_without_recreating_a_live_body() {
    let mut world = World::new();
    world.insert_resource(TacticalCombatConfig::default());
    let owner = actor(&mut world, 1);
    refresh_snapshots(&mut world, vec![snapshot(1, vec![item(11, 1, 2.0, 2.0)])]).unwrap();
    let entity = row_entity(&mut world, 11);
    world
        .entity_mut(owner)
        .remove::<(Player, CharacterId)>()
        .insert(crate::player_projection::DisconnectedPlayer {
            character_id: CharacterId(1),
            reconnect_token: adventuresim_tactical_netcode::prelude::ReconnectToken([1; 32]),
            remaining_secs: 10.0,
            claimed: false,
        });
    refresh_snapshots(&mut world, vec![snapshot(1, vec![item(11, 1, 2.0, 0.5)])]).unwrap();
    assert_eq!(row_entity(&mut world, 11), entity);
    assert_eq!(load(&mut world, owner), 0.5);
    assert!(world.get::<Player>(owner).is_none());
    assert_eq!(
        world.get::<TacticalCombatState>(owner).unwrap().fatigue,
        0.25
    );
}

#[test]
fn snapshot_for_an_unknown_or_retired_visit_cannot_change_inventory() {
    use adventuresim_stdb_client::{
        CalendarMinuteFraction, LocalSceneVisitClock, PersonalClock, StrategicMinute,
    };
    let mut world = World::new();
    world.insert_resource(TacticalCombatConfig::default());
    let owner = actor(&mut world, 1);
    refresh_snapshots(&mut world, vec![snapshot(1, vec![item(11, 1, 2.0, 2.0)])]).unwrap();
    let entity = row_entity(&mut world, 11);
    let mut stale = snapshot(1, vec![]);
    stale.time = TacticalActorTime::LocalVisit(LocalSceneVisitClock {
        visit_id: 99,
        entered_at: spacetimedb_sdk::Timestamp::UNIX_EPOCH,
        personal_clock: PersonalClock {
            minute: StrategicMinute { minutes: 0 },
            fraction: CalendarMinuteFraction { numerator: 0 },
        },
        food_balance_kcal: 0.0,
        water_balance_ml: 0.0,
    });
    refresh_snapshots(&mut world, vec![stale]).unwrap();
    assert_eq!(row_entity(&mut world, 11), entity);
    assert_eq!(load(&mut world, owner), 2.0);
    assert_eq!(
        world.get::<EquipmentActionState>(owner).unwrap().revision,
        EquipmentRevision::INITIAL
    );
}

#[test]
fn attachment_parent_resolves_after_child_and_survives_retained_owner_refresh() {
    use adventuresim_stdb_client::{
        ConnectedEquipmentOccupancy, EquipmentAnchorKind, EquipmentChannel,
    };
    let bag = adventuresim_core::item_catalog::definition("scabbard").unwrap();
    let point = bag
        .equipment
        .as_ref()
        .unwrap()
        .attachment_points
        .iter()
        .find(|point| point.id == "blade")
        .unwrap();
    let mut parent = item(12, 1, bag.weight_kg, bag.weight_kg);
    parent.item.id = bag.id.clone();
    parent.physical_object_id = Some(51);
    let design = adventuresim_weapon_model::default_design("longsword").unwrap();
    let derived = adventuresim_weapon_model::derive_properties(&design).unwrap();
    let mut child = item(11, 1, derived.mass_kg, derived.mass_kg);
    child.item.id = "longsword".into();
    child.item.kind = adventuresim_stdb_client::CatalogItemKind::Weapon;
    child.physical_object_id = Some(50);
    let weapon = adventuresim_core::item_catalog::definition("longsword").unwrap();
    let weapon_equipment = weapon.equipment.as_ref().unwrap();
    child.item.attachment_tags = weapon_equipment.attachment_tags.clone();
    assert!(
        point
            .accepts_tags
            .iter()
            .any(|tag| child.item.attachment_tags.contains(tag))
    );
    child.selected_placement_id = Some(
        weapon_equipment
            .placements
            .iter()
            .find(|placement| {
                placement.parents.iter().any(|parent| {
                    parent.channel == adventuresim_core::item_catalog::EquipmentChannel::Containment
                })
            })
            .unwrap()
            .id
            .clone(),
    );
    child.weapon_appearance = Some(adventuresim_stdb_client::ConnectedWeaponAppearance {
        generator_version: adventuresim_weapon_model::GENERATOR_VERSION,
        design_hash: adventuresim_weapon_model::design_hash(&design).0.to_vec(),
        recipe: adventuresim_weapon_model::encode(&design).unwrap(),
        mass_kg: derived.mass_kg,
        length_m: derived.length_m,
        grip_to_tip_m: derived.grip_to_tip_m,
    });
    child.occupancies.push(ConnectedEquipmentOccupancy {
        id: "fixture-container-attachment".into(),
        anchor_kind: EquipmentAnchorKind::ItemAttachment,
        location: None,
        parent_inventory_item_id: Some(12),
        attachment_point_id: Some(point.id.clone()),
        channel: EquipmentChannel::Containment,
        order: 0,
        requirement_index: 0,
        capacity_index: 0,
    });
    let mut world = World::new();
    world.insert_resource(TacticalCombatConfig::default());
    let owner = actor(&mut world, 1);
    refresh_snapshots(
        &mut world,
        vec![snapshot(1, vec![child.clone(), parent.clone()])],
    )
    .unwrap();
    let child_entity = row_entity(&mut world, 11);
    let parent_entity = row_entity(&mut world, 12);
    let retained = world
        .spawn(crate::player_projection::DisconnectedPlayer {
            character_id: CharacterId(1),
            reconnect_token: adventuresim_tactical_netcode::prelude::ReconnectToken([1; 32]),
            remaining_secs: 10.0,
            claimed: false,
        })
        .id();
    world.entity_mut(child_entity).insert(ItemOf(retained));
    world.entity_mut(parent_entity).insert(ItemOf(retained));
    world.despawn(owner);
    child.item.precision = 0.5;
    refresh_snapshots(&mut world, vec![snapshot(1, vec![parent, child])]).unwrap();
    assert_eq!(row_entity(&mut world, 11), child_entity);
    assert_eq!(row_entity(&mut world, 12), parent_entity);
    let topology = world.get::<EquipmentTopology>(child_entity).unwrap();
    assert!(matches!(&topology.occupancies[0].anchor,
        TacticalEquipmentAnchor::ItemAttachment { parent, .. } if *parent == parent_entity));
    assert_eq!(load(&mut world, retained), bag.weight_kg + derived.mass_kg);
}

fn item(row: u64, quantity: u32, unit_mass: f32, carried_mass: f32) -> ConnectedPlayerItem {
    use adventuresim_stdb_client::{
        CatalogItemKind, Item, MeleeAttackStyle, Slot, WeaponSkillDistribution,
    };
    ConnectedPlayerItem {
        inventory_item_id: row,
        quantity,
        physical_object_id: None,
        unit_mass_kg: unit_mass,
        carried_mass_kg: carried_mass,
        selected_placement_id: None,
        occupancies: vec![],
        protected_body_parts: vec![],
        condition: None,
        weapon_appearance: None,
        weapon_holder_appearance: None,
        item: Item {
            id: "steel_stock".into(),
            weight: unit_mass,
            exterior_volume_ml: 1,
            slot: Slot::None,
            kind: CatalogItemKind::Simple,
            equipment_placements: vec![],
            attachment_tags: vec![],
            attachment_points: vec![],
            repairable: false,
            preferred_melee_style: MeleeAttackStyle::Swing,
            reach: 0.0,
            block: 0.0,
            coverage: 0.0,
            precision: 0.0,
            resistance: 0.0,
            padding: 0.0,
            flexibility: 0.0,
            range_of_motion: 0.0,
            moment_of_inertia_kg_m_2: 0.0,
            balance: 0.0,
            melee: false,
            ranged: false,
            weapon_skills: WeaponSkillDistribution {
                polearm: 0.0,
                axe: 0.0,
                bludgeon: 0.0,
                sword: 0.0,
                knife: 0.0,
                bow: 0.0,
                crossbow: 0.0,
                firearm: 0.0,
                throw: 0.0,
            },
            base_value: None,
            nutrition_kcal: 0.0,
            water_capacity_ml: 0,
            container_capacity_ml: 0,
            alcohol_serving_ml: 0,
            alcohol_abv_basis_points: 0,
            alcohol_net_hydration_ml: 0,
            alcohol_disinfectant_effectiveness: 0,
            alcohol_disinfectant_focused: false,
            alcohol_potable: false,
            quality: 0,
            durability_yield: 0.0,
            durability_fracture: 0.0,
            durability_wear: 0.0,
            durability_failure_share: 0.0,
            edge_sensitivity: 0.0,
            handling_sensitivity: 0.0,
        },
    }
}
