//! Serving-slot projections distinguish the gateway from individual hosts.

use super::*;

#[view(accessor = tactical_server_request, public)]
pub fn tactical_server_request(ctx: &ViewContext) -> Vec<TacticalServerRequest> {
    let is_gateway = ctx
        .db
        .strategic_gateway_authority()
        .id()
        .find(0)
        .is_some_and(|authority| authority.identity == ctx.sender());
    if is_gateway {
        {
            ctx.db
                .tactical_server_request_authority()
                .gateway_bucket()
                .filter(0u8)
                .collect()
        }
    } else {
        Default::default()
    }
}

#[view(accessor = tactical_server, public)]
pub fn tactical_server(ctx: &ViewContext) -> Vec<TacticalServer> {
    let is_gateway = ctx
        .db
        .strategic_gateway_authority()
        .id()
        .find(0)
        .is_some_and(|authority| authority.identity == ctx.sender());
    if is_gateway {
        ctx.db
            .tactical_server_authority()
            .gateway_bucket()
            .filter(0u8)
            .collect()
    } else {
        ctx.db
            .tactical_server_authority()
            .identity()
            .find(ctx.sender())
            .into_iter()
            .collect()
    }
}
