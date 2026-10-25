use super::*;

pub(super) fn incapacitation_adjusted_attack_recovery(
    attacker: Entity,
    recovery: AnimationElapsed,
    combat_states: &Query<&mut TacticalCombatState>,
) -> adventuresim_tactical_core::animation::AnimationValueResult<RecoveryTime> {
    if recovery.is_zero() {
        return Ok(RecoveryTime::After(recovery));
    }
    let Ok(state) = combat_states.get(attacker) else {
        return Ok(RecoveryTime::After(recovery));
    };
    let performance = combat_incapacitation_performance(state.incapacitation)?;
    Ok(incapacitation_adjusted_recovery_seconds(
        recovery,
        performance,
    )?)
}

pub(super) fn charge_started_attack_work(
    attacker: Entity,
    hand: AttackHand,
    work_duration: AnimationElapsed,
    viewer: &TacticalPlayerViewer<'_, '_>,
    combat_states: &mut Query<&mut TacticalCombatState>,
    config: &TacticalCombatConfig,
) {
    let Ok(view) = viewer.get_for_attack(attacker, hand) else {
        return;
    };
    let Ok(mut state) = combat_states.get_mut(attacker) else {
        return;
    };
    let workload = combat_action_workload(
        CombatActionWork::Attack,
        work_duration,
        view.weapon_weight(),
        view.weapon_moment_of_inertia(),
        view.inventory_weight(),
        view.body_weight(),
        config.resolution.fatigue,
    );
    let state = &mut *state;
    state.charge_work(
        workload,
        view.raw_single_body_part_attr(SimpleAttribute::Endurance),
        config.resolution.fatigue,
    );
}
