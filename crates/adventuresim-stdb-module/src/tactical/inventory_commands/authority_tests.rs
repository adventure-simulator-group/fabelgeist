//! Guarded fixtures for real registered-connection inventory acceptance.

use adventuresim_core::{
    item_catalog::{EquipmentChannel, EquipmentLocation, OccupancyRequirement, ParentRequirement},
    physical_object::CarriedInventoryScope,
};

use super::*;
use crate::{
    FoodLot, InventoryItem,
    food::food_lot,
    inventory_container::{container_liquid, insert_personal_object, object_for_row},
    repair::item_condition,
    strategic::{PartyMember, party_member},
    weapon_instance::weapon_instance,
};

const PARENT: &str = "authority-command-parent";
const CHILD: &str = "authority-command-child";
const BLOCKER: &str = "authority-command-blocker";
const COMBINED: &str = "authority-command-combined";
const BOX: &str = "authority-command-box";
const TIGHT_BOX: &str = "authority-command-tight-box";
const SOLID: &str = "authority-command-solid";
const COUNTED: &str = "authority-projection-counted";
const LOT: &str = "authority-projection-lot";
const WATER_CONTAINER: &str = "authority-projection-container";

#[derive(SpacetimeType)]
pub enum InventoryCommandFixtureStage {
    FoodQuality,
    RepartitionMaterial,
    Condition,
    Recipe,
    CountedInstance,
    RevokeMembership,
    RestoreMembership,
}

#[reducer]
pub fn authority_test_inventory_party(
    ctx: &ReducerContext,
    bootstrap_token: String,
    leader: u64,
    member: u64,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    crate::inventory_effects::require_non_live_actor(ctx, leader)?;
    crate::inventory_effects::require_non_live_actor(ctx, member)?;
    crate::strategic::attach_seeded_party_member(ctx, leader, member, "Companion")
}

#[reducer]
pub fn authority_test_inventory_command_seed(
    ctx: &ReducerContext,
    bootstrap_token: String,
    character_id: u64,
) -> Result<(), String> {
    require_fixture_actor(ctx, &bootstrap_token, character_id)?;
    if ctx
        .db
        .inventory_item()
        .character_and_item_id()
        .filter((character_id, PARENT))
        .next()
        .is_some()
    {
        return Err("Command fixture is already seeded".into());
    }
    for item in crate::character::equipped_wearable_ids(ctx, character_id) {
        crate::character::unequip_wearable(ctx, item);
    }
    for definition in definitions() {
        if ctx.db.item().id().find(&definition.id).is_none() {
            ctx.db.item().insert(definition);
        }
    }
    for id in [
        PARENT, CHILD, CHILD, BLOCKER, COMBINED, BOX, TIGHT_BOX, SOLID,
    ] {
        let row = ctx.db.inventory_item().insert(InventoryItem {
            id: 0,
            character_id,
            item_id: id.into(),
            quantity: 1,
        });
        insert_personal_object(ctx, &row)?;
        if id == PARENT {
            crate::repair::initialize_item_condition(ctx, &row);
        }
    }
    Ok(())
}

#[reducer]
pub fn authority_test_inventory_command_material(
    ctx: &ReducerContext,
    bootstrap_token: String,
    character_id: u64,
    stage: InventoryCommandFixtureStage,
) -> Result<(), String> {
    require_fixture_actor(ctx, &bootstrap_token, character_id)?;
    match stage {
        InventoryCommandFixtureStage::FoodQuality => {
            let mut lot = fixture_lot(ctx, character_id)?;
            lot.quality += 1;
            ctx.db.food_lot().id().update(lot);
        }
        InventoryCommandFixtureStage::RepartitionMaterial => {
            let mut lot = fixture_lot(ctx, character_id)?;
            lot.mass_kg += 0.5;
            ctx.db.food_lot().id().update(lot);
            let row = fixture_row(ctx, character_id, WATER_CONTAINER)?;
            let object = object_for_row(ctx, CarriedInventoryScope::Personal, row.id)?
                .ok_or("Missing fixture water object")?;
            let mut liquid = ctx
                .db
                .container_liquid()
                .container_object_id()
                .find(object.id)
                .ok_or("Missing fixture water")?;
            liquid.water_ml = liquid
                .water_ml
                .checked_sub(500)
                .ok_or("Insufficient fixture water")?;
            ctx.db
                .container_liquid()
                .container_object_id()
                .update(liquid);
        }
        InventoryCommandFixtureStage::Condition => {
            let row = fixture_row(ctx, character_id, PARENT)?;
            let mut condition = ctx
                .db
                .item_condition()
                .inventory_item_id()
                .find(row.id)
                .ok_or("Missing fixture condition")?;
            condition.tier_1 = 0.75;
            condition.tier_2 = 0.25;
            ctx.db
                .item_condition()
                .inventory_item_id()
                .update(condition);
        }
        InventoryCommandFixtureStage::Recipe => replace_equal_mass_recipe(ctx, character_id)?,
        InventoryCommandFixtureStage::CountedInstance => individualize_counted(ctx, character_id)?,
        InventoryCommandFixtureStage::RevokeMembership => {
            for row in ctx
                .db
                .party_member()
                .character_id()
                .filter(character_id)
                .collect::<Vec<_>>()
            {
                ctx.db.party_member().id().delete(row.id);
            }
        }
        InventoryCommandFixtureStage::RestoreMembership => {
            let character = ctx
                .db
                .character()
                .id()
                .find(character_id)
                .ok_or("Missing fixture character")?;
            let party_id = character.party_id.ok_or("Missing fixture party pointer")?;
            ctx.db.party_member().insert(PartyMember {
                id: 0,
                party_id: party_id.clone(),
                character_id,
                role: Some("Companion".into()),
                recruitment_role_id: None,
            });
            crate::strategic::normalize_and_elect_party_leader(ctx, &party_id)?;
        }
    }
    Ok(())
}

fn require_fixture_actor(ctx: &ReducerContext, token: &str, character: u64) -> CommandResult<()> {
    crate::strategic::require_dev_bootstrap_token(token)?;
    let actor = CustodyCharacterId::try_new(character).map_err(|error| error.to_string())?;
    let admission = InventoryAdmission::for_actor(&ctx.as_read_only(), actor)
        .map_err(|error| error.to_string())?;
    if admission.role != TacticalActorRole::Player {
        return Err("Command fixtures require a currently admitted player".into());
    }
    Ok(())
}

fn fixture_row(ctx: &ReducerContext, actor: u64, item: &str) -> CommandResult<InventoryItem> {
    ctx.db
        .inventory_item()
        .character_and_item_id()
        .filter((actor, item))
        .next()
        .ok_or("Missing command fixture inventory row".into())
}

fn fixture_lot(ctx: &ReducerContext, actor: u64) -> CommandResult<FoodLot> {
    let row = fixture_row(ctx, actor, LOT)?;
    crate::food::personal_lot(ctx, row.id).ok_or("Missing command fixture food lot".into())
}

fn individualize_counted(ctx: &ReducerContext, actor: u64) -> CommandResult<()> {
    let row = fixture_row(ctx, actor, COUNTED)?;
    let remainder = row
        .quantity
        .checked_sub(1)
        .ok_or("Empty fixture counted row")?;
    if remainder > 0 {
        let character = ctx
            .db
            .character()
            .id()
            .find(actor)
            .ok_or("Missing fixture character")?;
        let party_id = character.party_id.ok_or("Missing fixture party")?;
        let party = ctx
            .db
            .party_authority()
            .id()
            .find(&party_id)
            .ok_or("Missing fixture party")?;
        let leader = ctx
            .db
            .character()
            .id()
            .find(party.leader_id)
            .ok_or("Missing fixture leader")?;
        crate::inventory_effects::PersonalInventoryTransfer {
            source: row,
            recipient: leader,
            quantity: ItemQuantity::new(remainder).ok_or("Fixture remainder must be positive")?,
        }
        .apply(ctx)?;
    }
    insert_personal_object(ctx, &fixture_row(ctx, actor, COUNTED)?)?;
    Ok(())
}

fn replace_equal_mass_recipe(ctx: &ReducerContext, actor: u64) -> CommandResult<()> {
    for row in ctx.db.inventory_item().character_id().filter(actor) {
        let Some(object) = object_for_row(ctx, CarriedInventoryScope::Personal, row.id)? else {
            continue;
        };
        let Some(before) = ctx
            .db
            .weapon_instance()
            .physical_object_id()
            .find(object.id)
        else {
            continue;
        };
        let mut design =
            adventuresim_weapon_model::decode(&before.recipe).map_err(|error| error.to_string())?;
        let component = design
            .recipe
            .components
            .iter_mut()
            .find(|component| component.id.is_some())
            .ok_or("Fixture recipe has no explicitly identified component")?;
        component.label = Some("Inventory authority fixture component".into());
        crate::weapon_instance::replace_design(ctx, object.id, &design)?;
        let after = ctx
            .db
            .weapon_instance()
            .physical_object_id()
            .find(object.id)
            .ok_or("Replaced fixture recipe is missing")?;
        if before.mass_grams != after.mass_grams || before.design_hash == after.design_hash {
            return Err("Fixture recipe edit must change identity without changing mass".into());
        }
        return Ok(());
    }
    Err("Fixture actor has no parametric weapon".into())
}

fn definitions() -> Vec<Item> {
    use crate::item::{PersistedEquipmentAttachmentPoint, PersistedEquipmentPlacement};
    let hand = PersistedEquipmentPlacement {
        id: "held".into(),
        occupancy: vec![OccupancyRequirement {
            location: EquipmentLocation::LeftHand,
            channel: EquipmentChannel::Held,
            order: 0,
            fit_zone: None,
        }],
        parents: Vec::new(),
        protection: Vec::new(),
    };
    let attached = PersistedEquipmentPlacement {
        id: "attached".into(),
        occupancy: Vec::new(),
        parents: vec![ParentRequirement {
            location: None,
            channel: EquipmentChannel::Mount,
            order: 0,
        }],
        protection: Vec::new(),
    };
    let combined = PersistedEquipmentPlacement {
        id: "combined".into(),
        occupancy: hand.occupancy.clone(),
        parents: attached.parents.clone(),
        protection: Vec::new(),
    };
    vec![
        Item {
            id: PARENT.into(),
            kind: CatalogItemKind::Simple,
            weight: 1.0,
            exterior_volume_ml: 100,
            repairable: true,
            equipment_placements: vec![hand.clone()],
            attachment_points: vec![PersistedEquipmentAttachmentPoint {
                id: "hook".into(),
                channel: EquipmentChannel::Mount,
                capacity: 1,
                order: 0,
                accepts_tags: vec!["command-fixture".into()],
            }],
            ..Item::default()
        },
        Item {
            id: CHILD.into(),
            kind: CatalogItemKind::Simple,
            weight: 1.0,
            exterior_volume_ml: 100,
            equipment_placements: vec![attached],
            attachment_tags: vec!["command-fixture".into()],
            ..Item::default()
        },
        Item {
            id: BLOCKER.into(),
            kind: CatalogItemKind::Simple,
            weight: 1.0,
            exterior_volume_ml: 100,
            equipment_placements: vec![hand],
            ..Item::default()
        },
        Item {
            id: COMBINED.into(),
            kind: CatalogItemKind::Simple,
            weight: 1.0,
            exterior_volume_ml: 100,
            equipment_placements: vec![combined],
            attachment_tags: vec!["command-fixture".into()],
            ..Item::default()
        },
        Item {
            id: BOX.into(),
            kind: CatalogItemKind::Container,
            weight: 1.0,
            exterior_volume_ml: 100,
            container_capacity_ml: 2000,
            ..Item::default()
        },
        Item {
            id: TIGHT_BOX.into(),
            kind: CatalogItemKind::Container,
            weight: 1.0,
            exterior_volume_ml: 1,
            container_capacity_ml: 1,
            ..Item::default()
        },
        Item {
            id: SOLID.into(),
            kind: CatalogItemKind::Simple,
            weight: 1.0,
            exterior_volume_ml: 100,
            ..Item::default()
        },
    ]
}
