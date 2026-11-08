use super::*;

pub(super) fn quickstep_request(
    entity: Entity,
    tick: PlayerInputTick,
    validated: ValidatedPlayerInput,
    skeleton: &SkeletonState,
    push_active: bool,
    jump_requested: bool,
) {
    if jump_requested && let Some(direction) = validated.jump.quickstep {
        info!(
            target: "quickstep_trace",
            ?entity,
            simulation_tick = tick.encoded(),
            sequence = validated.jump.sequence,
            ?direction,
            requested_guard = ?validated.weapon_guard,
            server_guard = ?skeleton.weapon_guard(),
            server_body = ?skeleton.body(),
            server_action = ?skeleton.action_kind(),
            push_active = push_active,
            "[quickstep][server-input] received new edge"
        );
    }
}

pub(super) fn incapacitated_jump(
    entity: Entity,
    tick: PlayerInputTick,
    validated: ValidatedPlayerInput,
    jump_requested: bool,
) {
    if jump_requested && validated.jump.quickstep.is_some() {
        warn!(
            target: "quickstep_trace",
            ?entity,
            simulation_tick = tick.encoded(),
            sequence = validated.jump.sequence,
            "[quickstep][server-input] rejected: incapacitated"
        );
    }
}
