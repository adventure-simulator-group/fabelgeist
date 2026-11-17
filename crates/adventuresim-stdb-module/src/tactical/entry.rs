//! Server-owned scene enrollment and durable identity assignment.

use super::*;

/// Put a character in an existing [`TacticalServer`].
#[reducer]
pub fn enter_tactical_scene(
    ctx: &ReducerContext,
    character_id: u64,
    server: Identity,
) -> Result<(), String> {
    if ctx.sender() != server {
        return Err("Only the owning tactical server can enroll characters".into());
    }
    crate::character::require_living_character(ctx, character_id)?;
    // Check character exists
    let mut character = ctx
        .db
        .character()
        .id()
        .find(character_id)
        .ok_or_else(|| format!("Character {character_id} not found"))?;
    // An unassigned character may join. An assignment to this server is an
    // idempotent retry, and a stale assignment whose server no longer exists
    // may be reclaimed. Never steal a character from another active server.
    if character.has_tactical_server_assignment()
        && character.server != server
        && ctx
            .db
            .tactical_server_authority()
            .identity()
            .find(character.server)
            .is_some()
    {
        return Err("Character is already assigned to another active tactical server".into());
    }

    let server = ctx
        .db
        .tactical_server_authority()
        .identity()
        .find(server)
        .ok_or_else(|| format!("Server {server} not found"))?;

    match &server.session {
        TacticalSession::Mission(authority) => {
            if !character.temporary {
                let enemy = authority.enemy_character_ids.contains(&character_id);
                let character_party = character.party_id.as_deref();
                if !enemy {
                    if !authority
                        .authorized_party_member_ids
                        .contains(&character_id)
                    {
                        return Err(
                            "Character was not authorized when this tactical mission was requested"
                                .into(),
                        );
                    }
                    if character_party != Some(authority.party_id.as_str()) {
                        return Err("Character is not a member of this mission's party".into());
                    }
                }
                let party = ctx
                    .db
                    .party_authority()
                    .id()
                    .find(&authority.party_id)
                    .ok_or("Mission party no longer exists")?;
                let mission = ctx
                    .db
                    .mission_authority()
                    .id()
                    .find(&authority.mission_id)
                    .ok_or("Mission authority no longer exists")?;
                if mission.party_id != party.id {
                    return Err("Mission authority changed before enrollment".into());
                }
            }
        }
        TacticalSession::Settlement(settlement) => {
            if crate::tactical::residents::has_npc_authority(ctx, character_id) {
                return Err(
                    "Settlement residents use scene-owned admission, not player connections".into(),
                );
            }
            super::settlement::require_settlement_admission(
                ctx,
                &character,
                settlement,
                server.identity,
            )?;
            super::local_time::LocalSceneVisit::begin(
                ctx,
                character_id,
                server.identity,
                super::local_time::LocalSceneTimeOwner::Player,
            )?;
        }
    }

    character.server = server.identity;
    ctx.db.character().id().update(character);

    Ok(())
}

/// Take out character from an existing [`TacticalServer`].
#[reducer]
pub fn leave_tactical_scene(ctx: &ReducerContext, character_id: u64) -> Result<(), String> {
    let server = ctx
        .db
        .tactical_server_authority()
        .identity()
        .find(ctx.sender())
        .ok_or("Only a registered tactical server can remove characters")?;
    if matches!(server.session, TacticalSession::Settlement(_)) {
        return Err("Ordinary scene departure requires its visit receipt".into());
    }
    let character = ctx
        .db
        .character()
        .id()
        .find(character_id)
        .ok_or_else(|| format!("Character {character_id} not found"))?;
    leave_tactical_scene_for_server(ctx, character, server.identity)
}

pub(super) fn leave_tactical_scene_for_server(
    ctx: &ReducerContext,
    mut character: Character,
    server: Identity,
) -> Result<(), String> {
    if !character.has_tactical_server_assignment() || character.server != server {
        return Err("Only the character's owning tactical server can remove it".into());
    }
    let character_id = character.id;
    super::local_time::require_unleased_time(ctx, character_id)?;
    if character.temporary {
        log::info!("Leaving scene for character #{character_id}: removing temporary character..");
        crate::character::delete_temporary_character(ctx, character)?;
    } else {
        log::info!("Leaving scene for character #{character_id}: resetting server info..");
        character.server = Identity::ZERO;
        ctx.db.character().id().update(character);
    }

    Ok(())
}
