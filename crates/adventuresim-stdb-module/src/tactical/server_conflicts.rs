use super::*;

pub(super) fn has_server_for_group(ctx: &ReducerContext, hostile_group_id: &str) -> bool {
    ctx.db
        .tactical_server_request_authority()
        .iter()
        .any(|request| {
            ctx.db
                .mission_authority()
                .id()
                .find(&request.mission_id)
                .is_some_and(|mission| {
                    mission.hostile_group_id.as_deref() == Some(hostile_group_id)
                })
        })
        || ctx.db.tactical_server_authority().iter().any(|server| {
            ctx.db
                .mission_authority()
                .id()
                .find(&server.mission_id)
                .is_some_and(|mission| {
                    mission.hostile_group_id.as_deref() == Some(hostile_group_id)
                })
        })
}
