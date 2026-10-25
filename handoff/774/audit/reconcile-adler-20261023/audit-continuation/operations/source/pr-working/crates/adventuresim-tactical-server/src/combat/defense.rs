use super::*;

/// A resolved response whose independent trade effect is not committed yet.
pub(super) enum PreparedMeleeDefense {
    Response(DefenderResponse),
    FinishTrade,
}

impl PreparedMeleeDefense {
    pub(super) fn response(&self) -> DefenderResponse {
        match self {
            Self::Response(response) => *response,
            Self::FinishTrade => DefenderResponse::None,
        }
    }

    pub(super) fn commit(self, authority: Option<&mut MeleeAttackAuthority>) -> DefenderResponse {
        match self {
            Self::Response(response) => response,
            Self::FinishTrade => {
                if let Some(authority) = authority {
                    authority.preserve_attack_for_trade();
                }
                DefenderResponse::None
            }
        }
    }
}

pub(super) fn resolve_passive_block(
    pending: Option<&PendingDefenderResponse>,
    time: &Time<()>,
    defender_view: &TacticalPlayerView,
    defender_skeleton: &SkeletonState,
    config: &DefenseAuthorityConfig,
) -> ElapsedResult<DefenderResponse> {
    if let Some(response) = resolve_requested_dodge(pending, time, config)? {
        return Ok(response);
    }
    Ok(passive_block_response(
        defender_skeleton.weapon_guard(),
        defender_skeleton.action_kind(),
        !defender_view.weapon_is_unarmed() || defender_view.shield_block_bonus() > 0.0,
    ))
}

#[expect(
    clippy::too_many_arguments,
    reason = "the defense boundary compares both combatants' authoritative attack state"
)]
pub(super) fn prepare_melee_defender_response(
    pending: Option<&PendingDefenderResponse>,
    time: &Time<()>,
    defender_view: &TacticalPlayerView,
    defender_skeleton: &SkeletonState,
    defender_attack: Option<&MeleeAttackAuthority>,
    defender_incapacitation: f32,
    defender_incapacitation_performance: UnitInterval,
    attacker: Entity,
    incoming_started_at: CombatInstant,
    config: &DefenseAuthorityConfig,
) -> ElapsedResult<PreparedMeleeDefense> {
    if let Some(response) = resolve_requested_melee_dodge(pending, incoming_started_at, config) {
        return Ok(PreparedMeleeDefense::Response(response));
    }
    let can_intercept =
        !defender_view.weapon_is_unarmed() || defender_view.shield_block_bonus() > 0.0;
    if can_intercept
        && defender_skeleton.action_kind() == SkeletonAction::Attack
        && let Some(authority) = defender_attack
        && let Some(opportunity) = authority.reciprocal_attack_opportunity(
            attacker,
            incoming_started_at,
            CombatInstant::from_elapsed(time),
            CombatDuration::from_elapsed(config.reflex_window_seconds)?,
        )
    {
        if defender_view.shield_block_bonus() <= 0.0 {
            let handling = effective_weapon_handling_skill(defender_view);
            let instinct = defender_view.raw_single_body_part_attr(SimpleAttribute::Instinct);
            let committed = choose_committed_threat_response(CommittedThreatFacts {
                own_contact_after_incoming_seconds: opportunity.own_contact_after_incoming_seconds,
                own_windup_seconds: opportunity.own_windup_seconds,
                expected_intercept_engagement: (opportunity.input_reflex
                    * ((handling + instinct) / 10.0).clamp(0.0, 1.0)
                    * defender_incapacitation_performance.fraction())
                .clamp(0.0, 1.0),
                incapacitation: defender_incapacitation,
                weapon_moment_of_inertia_kg_m2: defender_view.weapon_moment_of_inertia(),
                weapon_recovery_seconds: defender_view.weapon_recovery_secs()?,
                consecutive_intercepts: opportunity.consecutive_intercepts,
                decision_sample: opportunity.decision_sample,
            });
            if committed.choice == CommittedThreatChoice::FinishTrade {
                return Ok(PreparedMeleeDefense::FinishTrade);
            }
        }
        return Ok(PreparedMeleeDefense::Response(
            reciprocal_intercept_response(
                opportunity.input_reflex,
                opportunity.precision.get(),
                defender_view.shield_block_bonus(),
            ),
        ));
    }
    Ok(PreparedMeleeDefense::Response(passive_block_response(
        defender_skeleton.weapon_guard(),
        defender_skeleton.action_kind(),
        !defender_view.weapon_is_unarmed() || defender_view.shield_block_bonus() > 0.0,
    )))
}

pub(super) fn roll_dodge_reflex(input_reflex: f32, effectiveness: f32) -> f32 {
    input_reflex.clamp(0.0, 1.0) * effectiveness
}

fn resolve_requested_dodge(
    pending: Option<&PendingDefenderResponse>,
    time: &Time<()>,
    config: &DefenseAuthorityConfig,
) -> ElapsedResult<Option<DefenderResponse>> {
    let Some(pending) = pending else {
        return Ok(None);
    };
    let elapsed = CombatInstant::from_elapsed(time).elapsed_since(pending.set_at);
    let reflex_window = config.reflex_window_seconds.native_duration()?;
    if elapsed > reflex_window {
        return Ok(None);
    }
    let input_reflex = if reflex_window.is_zero() {
        1.0
    } else {
        (1.0 - elapsed.as_secs_f32() / reflex_window.as_secs_f32()).clamp(0.0, 1.0)
    };
    Ok(Some(match pending.choice {
        DefendRequest::Dodge { .. } => DefenderResponse::Dodge { input_reflex },
        DefendRequest::Roll => DefenderResponse::Dodge {
            input_reflex: roll_dodge_reflex(input_reflex, config.roll_dodge_effectiveness),
        },
    }))
}

fn resolve_requested_melee_dodge(
    pending: Option<&PendingDefenderResponse>,
    incoming_started_at: CombatInstant,
    config: &DefenseAuthorityConfig,
) -> Option<DefenderResponse> {
    let pending = pending?;
    if pending.set_at < incoming_started_at {
        return None;
    }
    let delay = pending
        .set_at
        .elapsed_since(incoming_started_at)
        .as_secs_f32();
    let window = config.reflex_window_seconds.seconds();
    if delay > window {
        return None;
    }
    let input_reflex = if window == 0.0 {
        1.0
    } else {
        (1.0 - delay / window).clamp(0.0, 1.0)
    };
    Some(match pending.choice {
        DefendRequest::Dodge { .. } => DefenderResponse::Dodge { input_reflex },
        DefendRequest::Roll => DefenderResponse::Dodge {
            input_reflex: roll_dodge_reflex(input_reflex, config.roll_dodge_effectiveness),
        },
    })
}

fn passive_block_response(
    guard: WeaponGuardState,
    action: SkeletonAction,
    has_blocking_item: bool,
) -> DefenderResponse {
    if guard == WeaponGuardState::Raised && action == SkeletonAction::None && has_blocking_item {
        DefenderResponse::Block { effectiveness: 1.0 }
    } else {
        DefenderResponse::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn dodge_reflex(response: Option<DefenderResponse>) -> Option<f32> {
        match response {
            Some(DefenderResponse::Dodge { input_reflex }) => Some(input_reflex),
            _ => None,
        }
    }

    #[test]
    fn melee_dodge_reflex_decreases_with_reaction_delay_and_rejects_pre_attack_input() {
        let config = TacticalCombatConfig::default().realtime_authority.defense;
        let start = CombatInstant::from_duration(Duration::from_secs(1));
        let pending = |milliseconds| PendingDefenderResponse {
            choice: DefendRequest::Roll,
            set_at: CombatInstant::from_duration(Duration::from_millis(milliseconds)),
        };
        let early = dodge_reflex(resolve_requested_melee_dodge(
            Some(&pending(1_050)),
            start,
            &config,
        ))
        .unwrap();
        let late = dodge_reflex(resolve_requested_melee_dodge(
            Some(&pending(1_300)),
            start,
            &config,
        ))
        .unwrap();
        assert!(early > late);
        assert!(resolve_requested_melee_dodge(Some(&pending(900)), start, &config).is_none());
        let after_window = 1_000 + (config.reflex_window_seconds.seconds() * 1_000.0) as u64 + 1;
        assert!(
            resolve_requested_melee_dodge(Some(&pending(after_window)), start, &config).is_none()
        );
    }

    #[test]
    fn raised_item_blocks_only_during_neutral_action_state() {
        assert_eq!(
            passive_block_response(WeaponGuardState::Raised, SkeletonAction::None, true),
            DefenderResponse::Block { effectiveness: 1.0 }
        );
        for (guard, action, has_blocking_item) in [
            (WeaponGuardState::Lowered, SkeletonAction::None, true),
            (WeaponGuardState::Raised, SkeletonAction::Attack, true),
            (WeaponGuardState::Raised, SkeletonAction::Dodge, true),
            (WeaponGuardState::Raised, SkeletonAction::None, false),
        ] {
            assert_eq!(
                passive_block_response(guard, action, has_blocking_item),
                DefenderResponse::None
            );
        }
    }

    #[test]
    fn offhand_shield_is_preferred_over_committed_sword_for_reciprocal_intercept() {
        assert!(matches!(
            reciprocal_intercept_response(0.8, 0.9, 1.5),
            DefenderResponse::Block { effectiveness } if effectiveness > 0.65
        ));
        assert!(matches!(
            reciprocal_intercept_response(0.8, 0.9, 0.0),
            DefenderResponse::Parry { .. }
        ));
    }
}
