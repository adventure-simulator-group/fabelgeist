//! The registered server's admitted character and equipment projection.

use super::*;

#[derive(SpacetimeType, Clone, Debug)]
pub struct ConnectedPlayer {
    pub character: Character,
    pub role: TacticalActorRole,
    pub time: super::local_time::TacticalActorTime,
    pub items: Vec<ConnectedPlayerItem>,
    pub skills: CharacterSkills,
    pub stats: CharacterStats,
    pub attrs: CharacterAttributes,
    pub limbs: CharacterLimbs,
    pub enemy_difficulty: i32,
    pub enemy_combat_scale_bps: u32,
    pub countermeasure_multiplier_bps: u32,
    pub party_has_surprise: bool,
    pub body_weight_kg: f32,
    pub current_blood_ml: f32,
    pub maximum_blood_ml: f32,
    pub strategic_incapacitation: f32,
    pub strategic_pain: f32,
    pub strategic_blood_loss: f32,
    pub strategic_fear: f32,
    pub strategic_fatigue: f32,
    pub strategic_hunger: f32,
    pub strategic_thirst: f32,
    pub strategic_thermal: f32,
}

#[derive(SpacetimeType, Clone, Debug)]
pub struct ConnectedPlayerItem {
    pub inventory_item_id: u64,
    pub quantity: u32,
    pub item: Item,
    pub selected_placement_id: Option<String>,
    pub occupancies: Vec<ConnectedEquipmentOccupancy>,
    pub protected_body_parts: Vec<adventuresim_core::item_catalog::EquipmentBodyPart>,
    pub condition: Option<ItemCondition>,
    pub weapon_appearance: Option<crate::weapon_instance::ConnectedWeaponAppearance>,
    pub weapon_holder_appearance: Option<crate::weapon_instance::ConnectedWeaponAppearance>,
}

#[derive(SpacetimeType, Clone, Debug)]
pub struct ConnectedEquipmentOccupancy {
    pub id: String,
    pub anchor_kind: crate::character::EquipmentAnchorKind,
    pub location: Option<adventuresim_core::item_catalog::EquipmentLocation>,
    pub parent_inventory_item_id: Option<u64>,
    pub attachment_point_id: Option<String>,
    pub channel: adventuresim_core::item_catalog::EquipmentChannel,
    pub order: u16,
    pub requirement_index: u16,
    pub capacity_index: u16,
}

#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TacticalActorRole {
    Player,
    Enemy,
}

/// View of [`ConnectedPlayer`] for this [`TacticalServer`].
#[view(accessor = connected_players, public)]
pub fn connected_players(ctx: &ViewContext) -> Vec<ConnectedPlayer> {
    ctx.db
        .character()
        .server()
        .filter(ctx.sender())
        .filter_map(|character| {
            let limbs = ctx.db.character_limbs().character_id().find(character.id)?;
            let attrs = ctx
                .db
                .character_attributes()
                .character_id()
                .find(character.id)?;
            let skills = ctx
                .db
                .character_skills()
                .character_id()
                .find(character.id)?;
            let stats = ctx.db.character_stats().character_id().find(character.id)?;
            let condition = ctx
                .db
                .character_condition()
                .character_id()
                .find(character.id)?;
            let strategic = ctx
                .db
                .character_strategic_condition()
                .character_id()
                .find(character.id)?;
            let items = connected_player_items(ctx, character.id).collect();

            let server = ctx
                .db
                .tactical_server_authority()
                .identity()
                .find(ctx.sender())?;
            let time = super::local_time::actor_time(ctx, character.id, &server)?;
            let mission = match &server.session {
                TacticalSession::Mission(mission) => Some(mission),
                TacticalSession::Settlement(_) => None,
            };
            let role = mission
                .filter(|mission| mission.enemy_character_ids.contains(&character.id))
                .map_or(TacticalActorRole::Player, |_| TacticalActorRole::Enemy);
            Some(ConnectedPlayer {
                character,
                role,
                time,
                items,
                skills,
                limbs,
                attrs,
                stats,
                enemy_difficulty: mission.map_or(1, |server| server.enemy_difficulty),
                enemy_combat_scale_bps: mission.map_or(
                    u32::from(adventuresim_world_schema::BASIS_POINTS_PER_WHOLE),
                    |server| server.enemy_combat_scale_bps,
                ),
                countermeasure_multiplier_bps: mission.map_or(
                    u32::from(adventuresim_world_schema::BASIS_POINTS_PER_WHOLE),
                    |server| server.countermeasure_multiplier_bps,
                ),
                party_has_surprise: mission.is_some_and(|server| server.party_has_surprise),
                body_weight_kg: condition.body_weight_kg,
                current_blood_ml: condition.current_blood_ml,
                maximum_blood_ml: condition.maximum_blood_ml,
                strategic_incapacitation: strategic.incapacitation,
                strategic_pain: strategic.pain,
                strategic_blood_loss: strategic.blood_loss,
                strategic_fear: strategic.fear,
                strategic_fatigue: strategic.fatigue,
                strategic_hunger: strategic.hunger,
                strategic_thirst: strategic.thirst,
                strategic_thermal: strategic.thermal,
            })
        })
        .collect()
}

fn connected_player_items(
    ctx: &ViewContext,
    character_id: u64,
) -> impl Iterator<Item = ConnectedPlayerItem> + use<'_> {
    ctx.db
        .inventory_item()
        .character_id()
        .filter(character_id)
        .filter_map(move |inventory_item| {
            let mut item = ctx.db.item().id().find(&inventory_item.item_id)?;
            let weapon_appearance = crate::weapon_instance::connected_appearance(
                ctx,
                inventory_item.id,
                &inventory_item.item_id,
            );
            let weapon_holder_appearance = crate::weapon_instance::connected_holder_appearance(
                ctx,
                inventory_item.id,
                &inventory_item.item_id,
            );
            let condition = ctx
                .db
                .item_condition()
                .inventory_item_id()
                .find(inventory_item.id);
            if let Some(condition) = &condition {
                let damage = condition.bins();
                item.precision = adventuresim_core::durability::effective_weapon_stat(
                    item.precision,
                    damage,
                    item.edge_sensitivity,
                );
                item.block = adventuresim_core::durability::effective_weapon_stat(
                    item.block,
                    damage,
                    item.handling_sensitivity,
                );
                item.range_of_motion = adventuresim_core::durability::effective_handling(
                    item.range_of_motion,
                    damage,
                    item.handling_sensitivity,
                );
            }
            let worn = ctx
                .db
                .character_equipped_item()
                .inventory_item_id()
                .find(inventory_item.id);
            let placement = worn.as_ref().and_then(|worn| {
                item.equipment_placements
                    .iter()
                    .find(|placement| placement.id == worn.placement_id)
            });
            let occupancies = ctx
                .db
                .equipment_occupancy()
                .inventory_item_id()
                .filter(inventory_item.id)
                .map(|row| ConnectedEquipmentOccupancy {
                    id: row.id,
                    anchor_kind: row.anchor_kind,
                    location: row.location,
                    parent_inventory_item_id: row.parent_inventory_item_id,
                    attachment_point_id: row.attachment_point_id,
                    channel: row.channel,
                    order: row.order,
                    requirement_index: row.requirement_index,
                    capacity_index: row.capacity_index,
                })
                .collect();
            let protected_body_parts = placement
                .map(|placement| placement.protection.clone())
                .unwrap_or_default();
            Some(ConnectedPlayerItem {
                inventory_item_id: inventory_item.id,
                quantity: inventory_item.quantity,
                item,
                selected_placement_id: worn.map(|row| row.placement_id),
                condition,
                occupancies,
                protected_body_parts,
                weapon_appearance,
                weapon_holder_appearance,
            })
        })
}
