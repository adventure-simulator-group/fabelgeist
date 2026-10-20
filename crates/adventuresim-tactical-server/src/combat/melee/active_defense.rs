use super::*;
use crate::combat::{authority::ValidatedMeleeAttack, defense::PreparedMeleeDefense};

struct DefenderCondition {
    incapacitation: f32,
    performance: UnitInterval,
}

#[expect(
    clippy::too_many_arguments,
    reason = "active defense bridges live authority, animation, and physiology"
)]
pub(super) fn prepare_active_defense(
    attack: &ValidatedMeleeAttack,
    incoming_started_at: CombatInstant,
    attacker_view: &TacticalPlayerView<'_, '_, '_>,
    defender_view: &TacticalPlayerView<'_, '_, '_>,
    attacker_performance: UnitInterval,
    attack_style: MeleeAttackStyle,
    contact_sample: f32,
    q_pending: &Query<&PendingDefenderResponse>,
    q_authorities: &Query<&mut MeleeAttackAuthority>,
    q_skeletons: &Query<&mut SkeletonState>,
    q_states: &Query<&mut TacticalCombatState>,
    time: &Time<()>,
    config: &TacticalCombatConfig,
) -> Option<PreparedMeleeDefense> {
    let defender_skeleton = q_skeletons.get(attack.target()).ok()?;
    let pending = q_pending.get(attack.target()).ok();
    let DefenderCondition {
        incapacitation,
        performance,
    } = match defender_condition(attack, q_states) {
        Ok(Some(condition)) => condition,
        Ok(None) => return None,
        Err(error) => {
            warn!(%error, "invalid defender performance");
            return None;
        }
    };
    let response_preparation = {
        let authority = q_authorities.get(attack.target()).ok();
        prepare_melee_defender_response(
            pending,
            time,
            defender_view,
            defender_skeleton,
            authority,
            incapacitation,
            performance,
            attack.attacker(),
            incoming_started_at,
            &config.realtime_authority.defense,
        )
    };
    let prepared_response = match response_preparation {
        Ok(response) => response,
        Err(error) => {
            warn!(%error, "cannot resolve defense timing");
            return None;
        }
    };
    let response = prepared_response
        .response()
        .scaled_for_performance(performance);
    let attacker_side = attacker_view.weapon_holding_side()?;
    let preview = attacker_view.melee_contact_location(
        attacker_side,
        attack_style,
        defender_view,
        attack.reported_precision().get() * attacker_performance.fraction(),
        contact_sample,
        config.resolution.contact,
    );
    let response = shield_aligned_response(response, defender_view.shield_holding_side(), preview);
    Some(match prepared_response {
        PreparedMeleeDefense::Response(_) => PreparedMeleeDefense::Response(response),
        PreparedMeleeDefense::FinishTrade => PreparedMeleeDefense::FinishTrade,
    })
}

pub(super) fn commit_active_defense(
    prepared: PreparedMeleeDefense,
    attack: &AuthorizedMeleeAttack,
    defender_view: &TacticalPlayerView<'_, '_, '_>,
    authorities: &mut Query<&mut MeleeAttackAuthority>,
    states: &mut Query<&mut TacticalCombatState>,
    config: &TacticalCombatConfig,
) -> DefenderResponse {
    let mut authority = authorities.get_mut(attack.target()).ok();
    let response = prepared.commit(authority.as_deref_mut());
    charge_defense_work(response, attack, defender_view, states, config);
    response
}

#[expect(
    clippy::too_many_arguments,
    reason = "committed defense bridges attack authority, animation, and implement ownership"
)]
pub(super) fn commit_defense_during_attack(
    cmd: &mut Commands,
    incoming: &AuthorizedMeleeAttack,
    attempted: DefenderResponse,
    effective: DefenderResponse,
    engagement: f32,
    defender_view: &TacticalPlayerView<'_, '_, '_>,
    authorities: &mut Query<&mut MeleeAttackAuthority>,
    skeletons: &mut Query<&mut SkeletonState>,
) {
    if matches!(attempted, DefenderResponse::Parry { .. })
        && let Ok(mut authority) = authorities.get_mut(incoming.target())
        && let Some(canceled_attack_key) = authority.commit_attack_to_defense()
    {
        if let Ok(mut skeleton) = skeletons.get_mut(incoming.target()) {
            skeleton.commit_attack_to_defense();
        }
        cmd.entity(incoming.target())
            .remove::<PendingMeleeContact>()
            .remove::<MeleeLungeMovement>();
        cmd.trigger(MeleeAttackCommittedToDefense {
            defender: incoming.target(),
            incoming_attacker: incoming.attacker(),
            canceled_attack_key,
            response: attempted,
            engagement,
        });
        return;
    }
    if defender_view.shield_block_bonus() > 0.0
        && let DefenderResponse::Block { effectiveness } = effective
        && let Ok(mut authority) = authorities.get_mut(incoming.target())
        && let Some(transformed) = authority.transform_attack_for_offhand_defense(effectiveness)
    {
        cmd.trigger(MeleeAttackTransformedByDefense {
            defender: incoming.target(),
            incoming_attacker: incoming.attacker(),
            attack_key: transformed.attack_key,
            retained_power: transformed.retained_power,
            response: attempted,
            engagement,
        });
    }
}

fn defender_condition(
    attack: &ValidatedMeleeAttack,
    states: &Query<&mut TacticalCombatState>,
) -> adventuresim_core::unit_interval::FractionResult<Option<DefenderCondition>> {
    let Ok(state) = states.get(attack.target()) else {
        return Ok(None);
    };
    Ok(Some(DefenderCondition {
        incapacitation: state.incapacitation,
        performance: combat_incapacitation_performance(state.incapacitation)?,
    }))
}

fn charge_defense_work(
    response: DefenderResponse,
    attack: &AuthorizedMeleeAttack,
    defender: &TacticalPlayerView<'_, '_, '_>,
    states: &mut Query<&mut TacticalCombatState>,
    config: &TacticalCombatConfig,
) {
    if !matches!(
        response,
        DefenderResponse::Block { .. } | DefenderResponse::Parry { .. }
    ) {
        return;
    }
    let Ok(mut state) = states.get_mut(attack.target()) else {
        return;
    };
    let state = &mut *state;
    let endurance = defender.raw_single_body_part_attr(SimpleAttribute::Endurance);
    let workload = combat_action_workload(
        CombatActionWork::WeaponDefense,
        config.realtime_authority.defense.reflex_window_seconds,
        defender.weapon_weight(),
        defender.weapon_moment_of_inertia(),
        defender.inventory_weight(),
        defender.body_weight(),
        config.resolution.fatigue,
    );
    state.charge_work(workload, endurance, config.resolution.fatigue);
}
