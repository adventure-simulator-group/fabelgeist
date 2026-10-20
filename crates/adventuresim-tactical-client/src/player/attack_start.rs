//! Predicted attack admission, buffering and publication.

use super::*;

struct AttackWeaponFacts {
    reach: f32,
    ranged: bool,
    melee: bool,
    windup: adventuresim_core::elapsed::ElapsedResult<AnimationElapsed>,
    recovery: adventuresim_core::elapsed::ElapsedResult<AnimationElapsed>,
    curve: adventuresim_tactical_core::animation::AttackCurve,
    preferred_style: MeleeAttackStyle,
}

struct SelectedMeleeAttack {
    entity: Entity,
    hand: AttackHand,
    family: StrikeFamily,
    spec: AttackSpec,
    start: AnimationSampleTick,
    reach: f32,
}

struct PreparedMeleeAttack {
    spec: AttackSpec,
    timing: MeleePlaybackTiming,
    attack_state: AttackState,
    target: Option<AttackTarget>,
}

struct PredictedRangedAttack {
    entity: Entity,
    reach: f32,
    windup: AnimationElapsed,
    recovery: AnimationElapsed,
    curve: adventuresim_tactical_core::animation::AttackCurve,
    start: AnimationSampleTick,
}

struct ImmediateMeleeAttack {
    entity: Entity,
    hand: AttackHand,
    alternate_attack: bool,
    attacking: bool,
    reach: f32,
    preferred_style: MeleeAttackStyle,
    start: AnimationSampleTick,
}

impl AttackWeaponFacts {
    fn observe(
        viewer: &TacticalPlayerViewer,
        entity: Entity,
        hand: AttackHand,
        config: &TacticalCombatConfig,
    ) -> Option<Self> {
        viewer
            .get_for_attack(entity, hand)
            .ok()
            .map(|character| Self {
                reach: character.weapon_reach(),
                ranged: hand == AttackHand::Main && character.weapon_is_ranged(),
                melee: character.weapon_is_melee(),
                windup: character.weapon_ranged_windup_secs(),
                recovery: attack_recovery_secs(
                    &character,
                    character.weapon_preferred_melee_style(),
                    false,
                ),
                curve: configure_attack_curve(
                    AttackSpec::default(),
                    &character,
                    &config.presentation.attack_curve,
                )
                .curve,
                preferred_style: character.weapon_preferred_melee_style(),
            })
    }
}

impl PreparedMeleeAttack {
    fn admit(
        request: SelectedMeleeAttack,
        skeleton: &SkeletonState,
        viewer: &TacticalPlayerViewer,
        targeting: &CombatTargeting,
        config: &TacticalCombatConfig,
    ) -> AnimationValueResult<Option<Self>> {
        let Ok(character) = viewer.get_for_attack(request.entity, request.hand) else {
            return Ok(None);
        };
        let style = request.family.melee_style();
        let windup = attack_preparation_secs(&character, style)?;
        let recovery = attack_recovery_secs(&character, style, request.spec.continuation)?;
        let spec = AttackSpec {
            curve: configure_attack_curve(
                AttackSpec::default(),
                &character,
                &config.presentation.attack_curve,
            )
            .curve,
            ..request.spec
        };
        Self::prepare(
            SelectedMeleeAttack { spec, ..request },
            skeleton,
            targeting,
            config,
            windup,
            recovery,
        )
        .map(Some)
    }

    fn prepare(
        request: SelectedMeleeAttack,
        skeleton: &SkeletonState,
        targeting: &CombatTargeting,
        config: &TacticalCombatConfig,
        windup: AnimationElapsed,
        recovery: AnimationElapsed,
    ) -> AnimationValueResult<Self> {
        let range = targeting.melee_acquisition_range(request.entity, request.reach, config);
        let (target, position, direction) = targeting.acquire(
            request.entity,
            range,
            Some(MeleeAcquisitionTiming::try_from(&config.movement)?),
        )?;
        let arrival = target.map_or(MeleeLungeArrival::NotNeeded, |target| target.arrival);
        let contact = arrival.contact_timing(windup)?;
        let timing = MeleePlaybackTiming::delayed(
            skeleton,
            &request.spec,
            request.start,
            contact.delay,
            contact.contact,
            recovery,
        )?;
        let attack_state = AttackState::new(
            timing.contact_from_input_seconds,
            timing.contact_from_input_seconds,
            request.reach,
            false,
            target,
            position,
            direction,
        )?;
        Ok(Self {
            spec: request.spec,
            timing,
            attack_state,
            target,
        })
    }
}

impl PredictedRangedAttack {
    fn start(
        self,
        cmd: &mut Commands,
        skeleton: &mut SkeletonState,
        targeting: &CombatTargeting,
    ) -> AnimationValueResult<()> {
        let Self {
            entity,
            reach,
            windup,
            recovery,
            curve,
            start,
        } = self;
        let spec = AttackSpec { curve, ..default() };
        let (target, position, direction) = targeting.acquire(entity, (reach, 0.0), None)?;
        let attack_state =
            AttackState::new(windup, windup, reach, true, target, position, direction)?;
        let contact = start.saturating_add(animation_ticks(windup)?);
        let end = start.saturating_add(animation_ticks(windup.checked_add(recovery)?)?);
        if skeleton
            .begin_attack_timed(spec, start, contact, end)
            .is_err()
        {
            return Ok(());
        }
        cmd.entity(entity).insert(attack_state);
        cmd.client_trigger(RangedActionRequest::Start {
            target: target.map(|target| target.body),
        });
        Ok(())
    }
}

impl ImmediateMeleeAttack {
    fn start(
        self,
        cmd: &mut Commands,
        skeleton: &mut SkeletonState,
        viewer: &TacticalPlayerViewer,
        targeting: &CombatTargeting,
        combat_config: &TacticalCombatConfig,
    ) -> AnimationValueResult<()> {
        let Self {
            entity,
            hand,
            alternate_attack,
            attacking,
            reach,
            preferred_style,
            start,
        } = self;
        let preferred_family = StrikeFamily::from_melee_style(preferred_style);
        let requested_family = if alternate_attack && hand == AttackHand::Main {
            preferred_family.alternate()
        } else {
            preferred_family
        };
        let Some(strike_family) = skeleton.available_strike_family(requested_family) else {
            return Ok(());
        };
        let Some(spec) = (!attacking)
            .then(|| match hand {
                AttackHand::Main => skeleton.select_main_attack(strike_family),
                AttackHand::Offhand => skeleton.select_offhand_attack(strike_family),
            })
            .flatten()
        else {
            cmd.entity(entity).insert(BufferedMeleeAttack {
                family: strike_family,
                hand,
            });
            return Ok(());
        };
        let Some(prepared) = PreparedMeleeAttack::admit(
            SelectedMeleeAttack {
                entity,
                hand,
                family: strike_family,
                spec,
                start,
                reach,
            },
            skeleton,
            viewer,
            targeting,
            combat_config,
        )?
        else {
            return Ok(());
        };
        let PreparedMeleeAttack {
            spec,
            timing,
            attack_state,
            target,
        } = prepared;
        match skeleton.begin_attack_timed(spec, timing.start, timing.contact, timing.end) {
            Ok(()) => {}
            Err(ActionTransitionError::ActionBusy) => {
                cmd.entity(entity).insert(BufferedMeleeAttack {
                    family: strike_family,
                    hand,
                });
                return Ok(());
            }
            Err(ActionTransitionError::Downed | ActionTransitionError::PostureTransitionActive) => {
                return Ok(());
            }
        }
        cmd.entity(entity)
            .insert(attack_state)
            .remove::<BufferedMeleeAttack>();
        cmd.client_trigger(MeleeActionRequest {
            strike_family,
            hand,
            target: target.map(|target| target.body),
        });
        Ok(())
    }
}

pub(super) fn try_start_attack(
    request: AttackStartRequest,
    cmd: &mut Commands,
    q_character: &mut Query<(Has<AttackState>, &mut SkeletonState)>,
    viewer: &TacticalPlayerViewer,
    time: &Time,
    combat_config: &TacticalCombatConfig,
    targeting: &CombatTargeting,
) -> AnimationValueResult<()> {
    let AttackStartRequest {
        entity,
        alternate_attack,
        hand,
    } = request;
    let Ok((attacking, mut skeleton)) = q_character.get_mut(entity) else {
        return Ok(());
    };
    let Some(weapon) = AttackWeaponFacts::observe(viewer, entity, hand, combat_config) else {
        warn!("Trying to attack, but can't get weapon reach. Not holding any weapons ?");
        return Ok(());
    };
    let AttackWeaponFacts {
        reach,
        ranged,
        melee,
        windup: windup_secs,
        recovery: recovery_secs,
        curve,
        preferred_style,
    } = weapon;

    let arm_reach = targeting.arm_reach(entity);
    if !attack_reach_is_usable(ranged, melee, reach, arm_reach) {
        warn!("Trying to attack without a usable equipped weapon");
        return Ok(());
    }
    if targeting
        .combatants
        .get(entity)
        .is_ok_and(|(_, state, _, _)| state.is_incapacitated())
    {
        return Ok(());
    }
    let windup_secs = windup_secs?;
    let recovery_secs = recovery_secs?;
    let start = AnimationSampleTick::from(
        (time.elapsed_secs_f64() * locomotion_sample_rate().hertz() as f64).round() as u64,
    );
    if ranged {
        if attacking {
            return Ok(());
        }
        PredictedRangedAttack {
            entity,
            reach,
            windup: windup_secs,
            recovery: recovery_secs,
            curve,
            start,
        }
        .start(cmd, &mut skeleton, targeting)?;
    } else {
        ImmediateMeleeAttack {
            entity,
            hand,
            alternate_attack,
            attacking,
            reach,
            preferred_style,
            start,
        }
        .start(cmd, &mut skeleton, viewer, targeting, combat_config)?;
    }
    Ok(())
}

pub(super) fn flush_buffered_melee_attacks(
    mut cmd: Commands,
    mut characters: Query<
        (Entity, &BufferedMeleeAttack, &mut SkeletonState),
        With<ControlledPlayer>,
    >,
    viewer: TacticalPlayerViewer,
    time: Res<Time>,
    combat_config: Res<TacticalCombatConfig>,
    targeting: CombatTargeting,
) {
    for (entity, buffered, mut skeleton) in &mut characters {
        let attempt = (|| -> AnimationValueResult<()> {
            let Some(spec) = (match buffered.hand {
                AttackHand::Main => skeleton.select_main_attack(buffered.family),
                AttackHand::Offhand => skeleton.select_offhand_attack(buffered.family),
            }) else {
                return Ok(());
            };
            let Ok((reach, melee, windup_secs, recovery_secs, curve)) = viewer
                .get_for_attack(entity, buffered.hand)
                .map(|character| {
                    (
                        character.weapon_reach(),
                        character.weapon_is_melee(),
                        attack_preparation_secs(&character, buffered.family.melee_style()),
                        attack_recovery_secs(
                            &character,
                            buffered.family.melee_style(),
                            spec.continuation,
                        ),
                        configure_attack_curve(
                            AttackSpec::default(),
                            &character,
                            &combat_config.presentation.attack_curve,
                        )
                        .curve,
                    )
                })
            else {
                return Ok(());
            };
            if !attack_reach_is_usable(false, melee, reach, targeting.arm_reach(entity)) {
                return Ok(());
            }
            if targeting
                .combatants
                .get(entity)
                .is_ok_and(|(_, state, _, _)| state.is_incapacitated())
            {
                return Ok(());
            }
            let start = AnimationSampleTick::from(
                (time.elapsed_secs_f64() * locomotion_sample_rate().hertz() as f64).round() as u64,
            );
            let prepared = PreparedMeleeAttack::prepare(
                SelectedMeleeAttack {
                    entity,
                    hand: buffered.hand,
                    family: buffered.family,
                    spec: AttackSpec { curve, ..spec },
                    start,
                    reach,
                },
                &skeleton,
                &targeting,
                &combat_config,
                windup_secs?,
                recovery_secs?,
            )?;
            let PreparedMeleeAttack {
                spec,
                timing,
                attack_state,
                target,
            } = prepared;
            if skeleton
                .begin_attack_timed(spec, timing.start, timing.contact, timing.end)
                .is_err()
            {
                return Ok(());
            }
            cmd.entity(entity)
                .insert(attack_state)
                .remove::<BufferedMeleeAttack>();
            cmd.client_trigger(MeleeActionRequest {
                strike_family: buffered.family,
                hand: buffered.hand,
                target: target.map(|target| target.body),
            });
            Ok(())
        })();
        if let Err(error) = attempt {
            warn!(?entity, %error, "cannot schedule buffered melee attack");
        }
    }
}
