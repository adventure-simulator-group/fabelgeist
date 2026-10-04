//! Release of unconsumed dispatcher startup claims.
use super::*;

/// Release an unconsumed claim after the trusted dispatcher fails to start
/// its child process. Consumed claims and active servers have no row to revoke.
#[reducer]
pub fn revoke_tactical_server_claim(
    ctx: &ReducerContext,
    mission_id: String,
) -> Result<(), String> {
    crate::strategic::require_strategic_gateway(ctx)
        .map_err(|error: crate::strategic::GatewayAdmissionError| error.to_string())?;
    MissionId::new(mission_id.clone()).map_err(|error| error.to_string())?;
    if ctx
        .db
        .tactical_server_request_authority()
        .mission_id()
        .find(&mission_id)
        .is_none()
    {
        return Err("Tactical server request is no longer pending".into());
    }
    ctx.db
        .tactical_server_claim()
        .mission_id()
        .delete(&mission_id);
    Ok(())
}
