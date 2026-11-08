use super::*;

pub(in crate::combat) fn on_ranged_action_request(
    event: On<FromClient<RangedActionRequest>>,
    mut cmd: Commands,
    viewer: TacticalPlayerViewer,
    config: Res<TacticalCombatConfig>,
) {
    let Some(attacker) = event.client_id.entity() else {
        debug!(
            "Ignoring ranged action from unknown client: {:?}",
            event.client_id
        );
        return;
    };
    match **event {
        RangedActionRequest::Start { target } => {
            // Same per-weapon windup + jitter-tolerance treatment as the
            // melee path - see `on_melee_action_request`.
            let Ok(view) = viewer.get(attacker) else {
                debug!(
                    ?attacker,
                    "cannot start ranged attack without authored weapon timing"
                );
                return;
            };
            let windups = view
                .weapon_ranged_windup_secs()
                .and_then(CombatDuration::from_elapsed)
                .and_then(|authored| {
                    PlayerAttackWindups::admit(authored, &config.realtime_authority.melee)
                });
            let windups = match windups {
                Ok(windups) => windups,
                Err(error) => {
                    warn!(?attacker, %error, "invalid authored ranged windup");
                    return;
                }
            };
            cmd.trigger(RangedAttackStartedIntent {
                attacker,
                target,
                animation_windup: windups.animation,
                minimum_windup: windups.minimum,
            });
        }
        RangedActionRequest::CompleteMiss => {
            cmd.trigger(RangedAttackIntent {
                attacker,
                target: None,
                body_part: BodyPart::Chest,
                reported_precision: ReportedPrecision::new(0.0).expect("zero is finite"),
            });
        }
        RangedActionRequest::CompleteHit {
            target,
            body_part,
            reported_precision,
        } => {
            let Some(reported_precision) = ReportedPrecision::new(reported_precision) else {
                debug!("Ignoring non-finite ranged precision from {attacker:?}");
                return;
            };
            // Finite precision is deliberately trusted. Animation and
            // secondary physics remain client-owned and non-deterministic.
            cmd.trigger(RangedAttackIntent {
                attacker,
                target: Some(target),
                body_part,
                reported_precision,
            });
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects each observer resource and query as an independent system parameter"
)]
pub(in crate::combat) fn on_ranged_attack_started(
    event: On<RangedAttackStartedIntent>,
    mut commands: Commands,
    mut authorities: Query<&mut RangedAttackAuthority>,
    mut skeletons: Query<&mut SkeletonState>,
    transforms: Query<&Transform>,
    viewer: TacticalPlayerViewer,
    time: Res<Time<()>>,
    config: Res<TacticalCombatConfig>,
) {
    let Ok(mut authority) = authorities.get_mut(event.attacker) else {
        return;
    };
    let Ok(mut skeleton) = skeletons.get_mut(event.attacker) else {
        return;
    };
    let start = animation_tick(&time);
    let Ok(view) = viewer.get(event.attacker) else {
        return;
    };
    let spec = configure_attack_curve(
        AttackSpec::default(),
        &view,
        &config.presentation.attack_curve,
    );
    let recovery = match attack_recovery_secs(&view, view.weapon_preferred_melee_style(), false)
        .and_then(CombatDuration::from_elapsed)
    {
        Ok(recovery) => recovery,
        Err(error) => {
            warn!(attacker = ?event.attacker, %error, "invalid ranged recovery");
            return;
        }
    };
    let allowance = match CombatDuration::from_elapsed(
        config
            .realtime_authority
            .ranged
            .completion_allowance_seconds,
    ) {
        Ok(allowance) => allowance,
        Err(error) => {
            warn!(%error, "ranged allowance exceeds native clock");
            return;
        }
    };
    let timing = match MeleeAnimationTiming::ranged(start, event.animation_windup, recovery) {
        Ok(timing) => timing,
        Err(error) => {
            warn!(%error, "invalid ranged sample schedule");
            return;
        }
    };
    if skeleton
        .begin_attack_timed(spec, start, timing.contact, timing.end)
        .is_err()
    {
        return;
    }
    begin_attack_facing(
        &mut commands,
        event.attacker,
        event.target,
        timing.contact,
        &transforms,
    );
    authority.observe(
        CombatInstant::from_elapsed(&time),
        event.minimum_windup,
        allowance,
    );
}
