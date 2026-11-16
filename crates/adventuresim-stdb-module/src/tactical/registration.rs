//! Claim consumption and registration for both kinds of tactical scene.

use super::*;

#[reducer]
pub fn create_tactical_server_for_request(
    ctx: &ReducerContext,
    instance_id: String,
    claim: String,
    addr: String,
    cert_digest: String,
) -> Result<(), String> {
    let hosting_connection = ctx
        .connection_id()
        .ok_or("Scene registration requires a live connection")?;
    instance_id
        .parse::<adventuresim_core::tactical_session::TacticalInstanceId>()
        .map_err(|error| error.to_string())?;
    let claim_row = ctx
        .db
        .tactical_server_claim()
        .instance_id()
        .find(&instance_id)
        .ok_or("Tactical server request has no authorized claim")?;
    let actual = Sha256::digest(claim.as_bytes());
    if actual.len() != claim_row.claim_hash.len()
        || actual
            .iter()
            .zip(&claim_row.claim_hash)
            .fold(0u8, |difference, (left, right)| difference | (left ^ right))
            != 0
    {
        return Err("Tactical server claim is invalid".into());
    }
    let request = ctx
        .db
        .tactical_server_request_authority()
        .instance_id()
        .find(&instance_id)
        .ok_or("Tactical server request no longer exists")?;
    request.session.validate()?;
    if request.session.instance_id()?.to_string() != instance_id {
        return Err("Tactical request identity does not match its session".into());
    }
    if ctx
        .db
        .tactical_server_authority()
        .identity()
        .find(ctx.sender())
        .is_some()
    {
        return Err("This identity already hosts a tactical scene".into());
    }
    if ctx
        .db
        .tactical_server_authority()
        .instance_id()
        .find(&instance_id)
        .is_some()
    {
        return Err("This scene slot already has an active server".into());
    }
    ctx.db
        .tactical_server_request_authority()
        .instance_id()
        .delete(&instance_id);
    ctx.db
        .tactical_server_claim()
        .instance_id()
        .delete(&instance_id);
    let enemies = request
        .session
        .mission()
        .map(|mission| mission.enemy_character_ids.clone())
        .unwrap_or_default();
    ctx.db.tactical_server_authority().insert(TacticalServer {
        identity: ctx.sender(),
        hosting_connection,
        gateway_bucket: 0,
        instance_id,
        scene_key: request.scene_key,
        addr,
        cert_digest,
        session: request.session,
    });
    for enemy in enemies {
        enter_tactical_scene(ctx, enemy, ctx.sender())?;
    }
    Ok(())
}
