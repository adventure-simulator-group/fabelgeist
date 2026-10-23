use super::*;

mod outcome;

struct RangedPerformances {
    attacker: UnitInterval,
    defender: UnitInterval,
}

struct PreparedRangedDefense<'v> {
    defender_view: TacticalPlayerView<'v, 'v, 'v>,
    response: DefenderResponse,
    performance: RangedPerformances,
    flanking: f32,
}

struct RangedAmmunition {
    entity: Entity,
    remaining: Option<CoreItemQuantity>,
}

impl RangedPerformances {
    fn admit(
        attacker: &TacticalCombatState,
        defender: &TacticalCombatState,
    ) -> adventuresim_core::unit_interval::FractionResult<Self> {
        Ok(Self {
            attacker: combat_incapacitation_performance(attacker.incapacitation)?,
            defender: combat_incapacitation_performance(defender.incapacitation)?,
        })
    }
}

impl<'v> PreparedRangedDefense<'v> {
    fn prepare(
        shot: &authority::ValidatedRangedShot,
        viewer: &'v TacticalPlayerViewer<'_, '_>,
        states: &Query<&TacticalCombatState>,
        skeletons: &Query<&SkeletonState>,
        pending: &Query<&PendingDefenderResponse>,
        time: &Time<()>,
        config: &DefenseAuthorityConfig,
    ) -> Option<Self> {
        let ValidatedRangedImpact::Hit {
            target, target_yaw, ..
        } = shot.impact()
        else {
            return None;
        };
        let defender_view = viewer.get(target).ok()?;
        let defender_skeleton = skeletons.get(target).ok()?;
        let flanking = ranged_flanking(shot.attacker_yaw(), target_yaw);
        let response = match resolve_passive_block(
            pending.get(target).ok(),
            time,
            &defender_view,
            defender_skeleton,
            config,
        ) {
            Ok(response) => response,
            Err(error) => {
                warn!(%error, "cannot resolve defense timing");
                return None;
            }
        };
        let (Ok(attacker), Ok(defender)) = (states.get(shot.attacker()), states.get(target)) else {
            return None;
        };
        let performance = match RangedPerformances::admit(attacker, defender) {
            Ok(performance) => performance,
            Err(error) => {
                warn!(%error, "invalid ranged combat performance");
                return None;
            }
        };
        Some(Self {
            defender_view,
            response,
            performance,
            flanking,
        })
    }
}

impl RangedAmmunition {
    fn find(
        attacker: Entity,
        ammunition: &Query<(Entity, &ItemOf, &ItemProperties, &TacticalItemQuantity)>,
    ) -> Option<Self> {
        let (entity, _, _, quantity) =
            ammunition.iter().find(|(_, owner, properties, quantity)| {
                owner.0 == attacker && properties.id == ARROW_ID && quantity.0.get() > 0
            })?;
        Some(Self {
            entity,
            remaining: remaining_ammo_after_shot(quantity.0),
        })
    }

    fn consume(
        self,
        commands: &mut Commands,
        ids: &Query<&CharacterId>,
        consequences: &mut TacticalConsequenceAccumulator,
        shot: &authority::AuthorizedRangedShot,
    ) {
        if let Some(remaining) = self.remaining {
            commands
                .entity(self.entity)
                .insert(TacticalItemQuantity(remaining));
        } else {
            commands.entity(self.entity).despawn();
        }
        if shot.attacker_side() == TacticalCombatSide::Party
            && let Ok(player_id) = ids.get(shot.attacker())
        {
            record_party_ammunition_use(consequences, *player_id);
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "this domain boundary names each independent input explicitly"
)]
pub(super) fn resolve_ranged_attack(
    event: On<RangedAttackIntent>,
    mut cmd: Commands,
    viewer: TacticalPlayerViewer,
    spatial: SpatialQuery,
    q_character: Query<(&CharacterLook, &Transform)>,
    q_sides: Query<&TacticalCombatSide>,
    q_states: Query<&TacticalCombatState>,
    q_skeletons: Query<&SkeletonState>,
    mut q_authorities: Query<&mut RangedAttackAuthority>,
    q_pending: Query<&PendingDefenderResponse>,
    q_scene_items: Query<Entity, With<TacticalSceneItem>>,
    q_ammo: Query<(Entity, &ItemOf, &ItemProperties, &TacticalItemQuantity)>,
    q_ids: Query<&CharacterId>,
    mut consequences: ResMut<TacticalConsequenceAccumulator>,
    time: Res<Time<()>>,
    config: Res<TacticalCombatConfig>,
) {
    let attacker = event.attacker;
    let Ok(attacker_view) = viewer.get(attacker) else {
        return;
    };
    let Ok((attacker_look, attacker_transform)) = q_character.get(attacker) else {
        return;
    };
    let target_character = event.target.and_then(|target| q_character.get(target).ok());
    let now = CombatInstant::from_elapsed(&time);
    let Ok(mut authority) = q_authorities.get_mut(attacker) else {
        return;
    };
    let facts = RangedIntentFacts {
        attacker,
        target: event.target,
        attacker_side: q_sides.get(attacker).ok().copied(),
        target_side: event
            .target
            .and_then(|target| q_sides.get(target).ok().copied()),
        attacker_incapacitated: q_states
            .get(attacker)
            .ok()
            .map(TacticalCombatState::is_incapacitated),
        target_incapacitated: event.target.and_then(|target| {
            q_states
                .get(target)
                .ok()
                .map(TacticalCombatState::is_incapacitated)
        }),
        reported_precision: event.reported_precision,
        weapon_is_ranged: attacker_view.weapon_is_ranged(),
        weapon_range: attacker_view.weapon_reach(),
        range_latency_tolerance: config
            .realtime_authority
            .ranged
            .range_latency_tolerance_metres,
        separation: target_character.map(|(_, transform)| {
            attacker_transform
                .translation
                .distance(transform.translation)
        }),
        target_in_aim_cone: target_character.map(|(_, transform)| {
            ranged_target_in_aim_cone(
                attacker_look.yaw,
                attacker_transform.translation,
                transform.translation,
                config.realtime_authority.ranged.aim_half_angle_degrees,
            )
        }),
        authority_permits: authority.permits(now),
        body_part: event.body_part,
        attacker_position: attacker_transform.translation,
        target_position: target_character.map(|(_, transform)| transform.translation),
        attacker_yaw: attacker_look.yaw,
        target_yaw: target_character.map(|(look, _)| look.yaw),
    };
    let validated = match validate_ranged_intent(facts) {
        Ok(validated) => validated,
        Err(reason) => {
            if !matches!(reason, RangedIntentRejection::Windup) {
                debug!("Rejected ranged intent from {attacker:?}: {reason:?}");
            }
            return;
        }
    };
    if let ValidatedRangedImpact::Hit {
        target,
        target_position,
        ..
    } = validated.impact()
    {
        let line_of_sight = authoritative_line_of_sight(
            &spatial,
            &q_scene_items,
            validated.attacker(),
            target,
            validated.attacker_position(),
            target_position,
        );
        if !line_of_sight {
            debug!(
                "Rejected ranged intent from {attacker:?}: {:?}",
                RangedIntentRejection::BlockedLineOfSight
            );
            return;
        }
    }
    let cooldown =
        match CombatDuration::from_elapsed(config.realtime_authority.ranged.cooldown_seconds) {
            Ok(cooldown) => cooldown,
            Err(error) => {
                warn!(%error, "attack cooldown exceeds native clock");
                return;
            }
        };
    let ammunition = RangedAmmunition::find(validated.attacker(), &q_ammo);
    let defense = if ammunition.is_some()
        && matches!(validated.impact(), ValidatedRangedImpact::Hit { .. })
    {
        let Some(defense) = PreparedRangedDefense::prepare(
            &validated,
            &viewer,
            &q_states,
            &q_skeletons,
            &q_pending,
            &time,
            &config.realtime_authority.defense,
        ) else {
            return;
        };
        Some(defense)
    } else {
        None
    };
    let Some(shot) = authority.authorize_shot(validated, now, cooldown) else {
        return;
    };
    // Dry fire consumes only cooldown; misses consume ammunition without defense.
    let Some(ammunition) = ammunition else {
        return;
    };
    ammunition.consume(&mut cmd, &q_ids, &mut consequences, &shot);
    let Some(PreparedRangedDefense {
        defender_view,
        response: defender_response,
        performance,
        flanking,
    }) = defense
    else {
        return;
    };
    let ValidatedRangedImpact::Hit {
        target, body_part, ..
    } = shot.impact()
    else {
        return;
    };
    cmd.entity(target).remove::<PendingDefenderResponse>();
    let defender_response = defender_response.scaled_for_performance(performance.defender);
    let result = attacker_view.resolve_ranged_attack(
        config.resolution,
        &defender_view,
        defender_response,
        shot.reported_precision().get() * performance.attacker.fraction(),
        flanking,
        body_part,
    );
    let defender_blocking_slot = defender_blocking_slot(
        defender_response,
        defender_view.shield_holding_side(),
        defender_view.weapon_holding_side(),
    );
    let attacker_weapon_slot = weapon_slot_or_right(attacker_view.weapon_holding_side());
    let (_, target_transform) = q_character
        .get(target)
        .expect("validated ranged target still has a transform");
    outcome::RangedContact {
        shot: &shot,
        target_transform,
        target,
        body_part,
        result,
        defender_response,
        flanking,
        attacker_weapon_slot,
        defender_blocking_slot,
    }
    .publish(
        &mut cmd,
        &viewer,
        attacker_transform,
        &attacker_view,
        &defender_view,
        &config,
    );
}

fn ranged_flanking(attacker_yaw: f32, target_yaw: f32) -> f32 {
    let (a2, a1) = attacker_yaw.sin_cos();
    let (d2, d1) = target_yaw.sin_cos();
    flanking_from_dir((a1, a2), (d1, d2))
}
