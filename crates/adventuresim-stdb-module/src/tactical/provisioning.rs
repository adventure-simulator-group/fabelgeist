//! Retryable operational failures do not produce a combat outcome.

use super::{TacticalProvisioningStatus, tactical_server_request_authority};
use spacetimedb::{ReducerContext, reducer};

#[reducer]
pub fn record_tactical_scene_provisioning_failure(
    ctx: &ReducerContext,
    instance_id: String,
) -> Result<(), String> {
    crate::strategic::require_strategic_gateway(ctx)?;
    instance_id
        .parse::<adventuresim_core::tactical_session::TacticalInstanceId>()
        .map_err(|error| error.to_string())?;
    let Some(mut request) = ctx
        .db
        .tactical_server_request_authority()
        .instance_id()
        .find(&instance_id)
    else {
        return Ok(());
    };
    if request.provisioning != TacticalProvisioningStatus::Failed {
        request.provisioning = TacticalProvisioningStatus::Failed;
        ctx.db
            .tactical_server_request_authority()
            .instance_id()
            .update(request);
    }
    Ok(())
}
