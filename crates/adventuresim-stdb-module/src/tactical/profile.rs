//! The registered server's admitted character and equipment projection.

use super::*;

#[derive(SpacetimeType, Clone, Debug)]
pub struct ConnectedPlayer {
    pub character: Character,
    pub role: TacticalActorRole,
    pub time: super::local_time::TacticalActorTime,
    pub inventory: super::inventory::ConnectedInventory,
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

#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TacticalActorRole {
    Player,
    Enemy,
    Resident,
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
            let server = ctx
                .db
                .tactical_server_authority()
                .identity()
                .find(ctx.sender())?;
            let projection = super::local_time::actor_projection(ctx, character.id, &server)?;
            let inventory = super::inventory::connected_inventory(ctx, character.id);
            let mission = match &server.session {
                TacticalSession::Mission(mission) => Some(mission),
                TacticalSession::Settlement(_) => None,
            };
            Some(ConnectedPlayer {
                character,
                role: projection.role,
                time: projection.time,
                inventory,
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
