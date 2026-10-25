use super::*;

pub(super) struct MeleeRecoverySchedule {
    pub(super) recovery: CombatDuration,
    pub(super) completion_allowance: CombatDuration,
}

pub(super) struct MeleeStartTiming {
    pub(super) animation: MeleeAnimationTiming,
    pub(super) contact_windup: CombatDuration,
    pub(super) work_duration: AnimationElapsed,
}

pub(super) struct StartedMeleePublication {
    pub(super) event: MeleeAttackStartedIntent,
    pub(super) body_part: Option<BodyPart>,
    pub(super) contact: super::super::contact::InitialMeleeContact,
    pub(super) observation: ObservedMeleeStart,
    pub(super) timing: MeleeAnimationTiming,
    pub(super) contact_windup: CombatDuration,
    pub(super) work_duration: AnimationElapsed,
    pub(super) scheduled_measure: SignedLength,
    pub(super) allowance: CombatDuration,
    pub(super) plan: Option<PlannedMeleeLunge>,
}

pub(super) struct StartedMeleePreparation {
    pub(super) event: MeleeAttackStartedIntent,
    pub(super) contact: super::super::contact::InitialMeleeContact,
    pub(super) observation: ObservedMeleeStart,
    pub(super) plan: Option<PlannedMeleeLunge>,
    pub(super) recovery: MeleeRecoverySchedule,
}

impl StartedMeleePreparation {
    pub(super) fn admit(
        self,
        skeleton: &SkeletonState,
        spec: &AttackSpec,
        transforms: &Query<&Transform>,
    ) -> MeleeGeometryResult<StartedMeleePublication> {
        let arrival = self
            .plan
            .map_or(MeleeLungeArrival::NotNeeded, |plan| plan.arrival);
        let body_part = if arrival == MeleeLungeArrival::CannotArrive {
            None
        } else {
            self.contact.body_part
        };
        let event = MeleeAttackStartedIntent {
            target: if arrival == MeleeLungeArrival::CannotArrive {
                None
            } else {
                self.event.target
            },
            ..self.event
        };
        let timing = MeleeStartTiming::admit(
            &event,
            self.observation.sample,
            arrival,
            self.recovery.recovery,
            skeleton,
            spec,
        )?;
        let scheduled_measure =
            scheduled_attack_measure(&event, self.contact.weapon_reach, transforms)?;
        Ok(StartedMeleePublication {
            event,
            body_part,
            contact: self.contact,
            observation: self.observation,
            timing: timing.animation,
            contact_windup: timing.contact_windup,
            work_duration: timing.work_duration,
            scheduled_measure,
            allowance: self.recovery.completion_allowance,
            plan: self.plan,
        })
    }
}

impl StartedMeleePublication {
    pub(super) fn publish(
        self,
        commands: &mut Commands,
        authority: &mut MeleeAttackAuthority,
        transforms: &Query<&Transform>,
        time: &Time<()>,
    ) {
        begin_started_attack_movement(
            commands,
            &self.event,
            self.body_part,
            self.contact.weapon_reach,
            self.observation.key,
            self.timing.start,
            self.timing.contact,
            transforms,
            self.plan,
        );
        authorize_started_attack(
            commands,
            authority,
            &self.event,
            self.body_part,
            self.contact.sample,
            self.contact.defense_alignment_sample,
            self.observation.key,
            self.contact_windup,
            self.scheduled_measure,
            time,
            self.allowance,
        );
    }
}

impl MeleeRecoverySchedule {
    pub(super) fn admit(
        event: &MeleeAttackStartedIntent,
        view: &TacticalPlayerView<'_, '_, '_>,
        strike_family: StrikeFamily,
        spec: &AttackSpec,
        states: &Query<&mut TacticalCombatState>,
        config: &TacticalCombatConfig,
    ) -> AnimationValueResult<Option<Self>> {
        let recovery = attack_recovery_secs(view, strike_family.melee_style(), spec.continuation)?;
        let RecoveryTime::After(recovery) =
            incapacitation_adjusted_attack_recovery(event.attacker, recovery, states)?
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            recovery: CombatDuration::from_elapsed(recovery)?,
            completion_allowance: CombatDuration::from_elapsed(
                config.realtime_authority.melee.completion_allowance_seconds,
            )?,
        }))
    }
}

impl MeleeStartTiming {
    pub(super) fn admit(
        event: &MeleeAttackStartedIntent,
        start: AnimationSampleTick,
        arrival: MeleeLungeArrival,
        recovery: CombatDuration,
        skeleton: &SkeletonState,
        spec: &AttackSpec,
    ) -> AnimationValueResult<Self> {
        let animation = MeleeAnimationTiming::for_attack(
            skeleton,
            spec,
            start,
            event.windup,
            arrival,
            recovery,
        )?;
        let contact_windup = super::super::contact::windup_duration(animation.contact, start)?;
        let work_duration = contact_windup.elapsed().checked_add(recovery.elapsed())?;
        Ok(Self {
            animation,
            contact_windup,
            work_duration,
        })
    }
}

pub(super) fn started_attack_lunge_plan(
    event: &MeleeAttackStartedIntent,
    weapon_reach: SignedLength,
    transforms: &Query<&Transform>,
    dimensions: &Query<&CharacterDimensions>,
    colliders: &Query<&Collider>,
    config: &TacticalCombatConfig,
) -> MeleeGeometryResult<Option<PlannedMeleeLunge>> {
    let Some(target) = event.target else {
        return Ok(None);
    };
    planned_melee_lunge_for_entities(
        EntityMeleeLungeRequest {
            attacker: event.attacker,
            target,
            weapon_reach_metres: weapon_reach,
        },
        transforms,
        dimensions,
        colliders,
        config,
    )
}

pub(super) fn scheduled_attack_measure(
    event: &MeleeAttackStartedIntent,
    weapon_reach: SignedLength,
    transforms: &Query<&Transform>,
) -> MeleeGeometryResult<SignedLength> {
    let Some(target) = event.target else { return Ok(weapon_reach); };
    let (Ok(attacker), Ok(target)) = (transforms.get(event.attacker), transforms.get(target)) else {
        return Ok(weapon_reach);
    };
    melee_surface_measure(
        Position::from_metres(attacker.translation)?,
        Position::from_metres(target.translation)?,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "the authority record retains the complete authored attack identity and timing"
)]
pub(super) fn authorize_started_attack(
    commands: &mut Commands,
    authority: &mut MeleeAttackAuthority,
    event: &MeleeAttackStartedIntent,
    selected_body_part: Option<BodyPart>,
    contact_sample: f32,
    defense_alignment_sample: f32,
    attack_key: MeleeAttackKey,
    contact_windup: CombatDuration,
    scheduled_measure_metres: SignedLength,
    time: &Time<()>,
    completion_allowance: CombatDuration,
) {
    let now = CombatInstant::from_elapsed(time);
    authority.observe(
        attack_key,
        event.target,
        selected_body_part,
        now,
        contact_windup,
        completion_allowance,
        scheduled_measure_metres,
        event.reported_precision,
    );
    commands.entity(event.attacker).insert(PendingMeleeContact {
        attack_key,
        target: event.target,
        body_part: selected_body_part,
        contact_sample,
        defense_alignment_sample,
        resolve_at: now + contact_windup,
        reported_precision: event.reported_precision,
        strike_family: event.strike_family,
        hand: event.hand,
    });
}

#[expect(
    clippy::too_many_arguments,
    reason = "movement planning receives authored attack timing and production collision queries"
)]
pub(super) fn begin_started_attack_movement(
    commands: &mut Commands,
    event: &MeleeAttackStartedIntent,
    selected_body_part: Option<BodyPart>,
    weapon_reach: SignedLength,
    attack_key: MeleeAttackKey,
    animation_start_tick: AnimationSampleTick,
    contact_tick: AnimationSampleTick,
    transforms: &Query<&Transform>,
    plan: Option<PlannedMeleeLunge>,
) {
    info!(attack_key = attack_key.encoded(), attacker = ?event.attacker, target = ?event.target, body_part = ?selected_body_part, strike_family = ?event.strike_family, hand = ?event.hand, "melee_attack_started");
    begin_attack_facing(
        commands,
        event.attacker,
        event.target,
        contact_tick,
        transforms,
    );
    if let (Some(target), Some(plan)) = (event.target, plan) {
        plan.install(
            commands,
            EntityMeleeLungeRequest {
                attacker: event.attacker,
                target,
                weapon_reach_metres: weapon_reach,
            },
            attack_key,
            animation_start_tick,
        );
    } else {
        commands
            .entity(event.attacker)
            .remove::<MeleeLungeMovement>();
        if event.target.is_some() {
            return;
        }
        info!(attack_key = attack_key.encoded(), attacker = ?event.attacker, target = ?event.target, body_part = ?selected_body_part, outcome = "untargeted_no_movement", "melee_lunge_planned");
    }
}
