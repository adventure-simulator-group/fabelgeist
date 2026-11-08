use super::authority::{AuthorizedMeleeAttack, ValidatedMeleeAttack};
use super::*;

use active_defense::{commit_active_defense, commit_defense_during_attack, prepare_active_defense};
use outcome::emit_melee_outcome;
use geometry::{CompletionActors, CompletionAuthorization, MeleeCompletionGeometry};

mod active_defense;
mod outcome;
mod geometry;

/// Read-only contact admission; authority is consumed only after defense preparation.
struct MeleeCompletion {
    attack: ValidatedMeleeAttack,
    started_at: CombatInstant,
    cooldown: CombatDuration,
}

impl MeleeCompletion {
    #[expect(
        clippy::too_many_arguments,
        reason = "authority validation consumes spatial and timing facts"
    )]
    fn prepare(
        facts: MeleeIntentFacts,
        authority: &MeleeAttackAuthority,
        spatial: &SpatialQuery,
        scene_items: &Query<Entity, With<TacticalSceneItem>>,
        config: &TacticalCombatConfig,
        attack_key: Option<MeleeAttackKey>,
        surface_distance: SignedLength,
        reach: SignedLength,
    ) -> Option<Self> {
        let validated = match validate_melee_intent_cheap(facts) {
            Ok(validated) => validated,
            Err(reason) => {
                info!(attack_key = attack_key.map(MeleeAttackKey::encoded), attacker = ?facts.attacker, target = ?facts.target, body_part = ?facts.body_part, reason = ?reason, surface_distance_metres = surface_distance.metres(), reach_metres = reach.metres(), "melee_completion_rejected");
                info!(attack_key = attack_key.map(MeleeAttackKey::encoded), attacker = ?facts.attacker, target = ?facts.target, body_part = ?facts.body_part, outcome = "miss", reason = ?reason, "melee_attack_resolved");
                return None;
            }
        };
        let line_of_sight = authoritative_line_of_sight(
            spatial,
            scene_items,
            validated.attacker(),
            validated.target(),
            validated.attacker_position(),
            validated.target_position(),
        );
        if let Err(reason) = validate_melee_line_of_sight(line_of_sight) {
            info!(attack_key = attack_key.map(MeleeAttackKey::encoded), attacker = ?facts.attacker, target = ?facts.target, body_part = ?facts.body_part, reason = ?reason, surface_distance_metres = surface_distance.metres(), reach_metres = reach.metres(), "melee_completion_rejected");
            info!(attack_key = attack_key.map(MeleeAttackKey::encoded), attacker = ?facts.attacker, target = ?facts.target, body_part = ?facts.body_part, outcome = "miss", reason = ?reason, "melee_attack_resolved");
            return None;
        }
        let cooldown = match CombatDuration::from_elapsed(
            config.realtime_authority.melee.replay_cooldown_seconds,
        ) {
            Ok(cooldown) => cooldown,
            Err(error) => {
                warn!(%error, "attack cooldown exceeds native clock");
                return None;
            }
        };
        Some(Self {
            attack: validated,
            started_at: authority.started_at()?,
            cooldown,
        })
    }

    fn authorize(
        self,
        authority: &mut MeleeAttackAuthority,
        now: CombatInstant,
        attack_key: Option<MeleeAttackKey>,
        surface_distance: SignedLength,
        reach: SignedLength,
    ) -> Option<AuthorizedMeleeAttack> {
        let attack = self.attack;
        authority.authorize_attack(attack, now, self.cooldown).or_else(|| {
            info!(attack_key = attack_key.map(MeleeAttackKey::encoded), attacker = ?attack.attacker(), target = ?attack.target(), body_part = ?attack.body_part, reason = "authorization_consumed", surface_distance_metres = surface_distance.metres(), reach_metres = reach.metres(), "melee_completion_rejected");
            info!(attack_key = attack_key.map(MeleeAttackKey::encoded), attacker = ?attack.attacker(), target = ?attack.target(), body_part = ?attack.body_part, outcome = "miss", reason = "authorization_consumed", "melee_attack_resolved");
            None
        })
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "alignment compares complete attacker and defender views"
)]
fn weapon_defense_alignment(
    attacker: &TacticalPlayerView<'_, '_, '_>,
    defender: &TacticalPlayerView<'_, '_, '_>,
    attacker_side: BodySide,
    attack_style: MeleeAttackStyle,
    precision: ReportedPrecision,
    flanking: f32,
    response: DefenderResponse,
    sample: f32,
    contact_at_time: MeleeContactAtTime,
    contact_parameters: adventuresim_core::combat::WeaponContactParameters,
) -> Option<adventuresim_core::combat::WeaponDefenseAlignment> {
    response.is_weapon_contact().then(|| {
        let attack_value = adventuresim_core::combat::melee_attack_value_by_parts(
            attacker,
            attacker,
            attacker,
            attacker,
            attacker,
            attacker_side,
            attack_style,
            adventuresim_core::combat::melee_measure_adjusted_precision(
                precision.get(),
                contact_at_time,
            ),
            flanking,
            response,
            defender,
            defender,
            defender,
            defender,
            defender,
            contact_parameters,
        );
        resolve_weapon_defense_alignment(response, attack_value, sample)
    })
}

#[expect(
    clippy::too_many_arguments,
    reason = "contact resolution consumes the defended strike projection"
)]
fn contact_after_defense(
    attacker: &TacticalPlayerView<'_, '_, '_>,
    defender: &TacticalPlayerView<'_, '_, '_>,
    attacker_side: BodySide,
    attack_style: MeleeAttackStyle,
    response: DefenderResponse,
    precision: ReportedPrecision,
    flanking: f32,
    contact_sample: f32,
    contact_at_time: MeleeContactAtTime,
    config: &TacticalCombatConfig,
) -> (MeleeContactLocation, AttackResult) {
    super::contact::resolve_melee_contact(
        attacker,
        defender,
        config.resolution,
        attacker_side,
        attack_style,
        response,
        precision,
        flanking,
        contact_sample,
        None,
        contact_at_time,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects each observer resource and query as an independent system parameter"
)]
pub(super) fn resolve_melee_attack(
    event: On<MeleeAttackIntent>,
    mut cmd: Commands,
    viewer: TacticalPlayerViewer,
    spatial: SpatialQuery,
    q_character: Query<(&CharacterLook, &Transform)>,
    q_dimensions: Query<&CharacterDimensions>,
    q_sides: Query<&TacticalCombatSide>,
    mut q_states: Query<&mut TacticalCombatState>,
    mut q_skeletons: Query<&mut SkeletonState>,
    mut q_authorities: Query<&mut MeleeAttackAuthority>,
    q_pending: Query<&PendingDefenderResponse>,
    q_scene_items: Query<Entity, With<TacticalSceneItem>>,
    time: Res<Time<()>>,
    config: Res<TacticalCombatConfig>,
) {
    let attack_style = event.strike_family.melee_style();
    let entity = event.attacker;
    let hand = event.hand;
    let Ok(authority) = q_authorities.get(entity) else {
        info!(attacker = ?entity, reason = "missing_authority", "melee_completion_rejected");
        return;
    };
    let attack_key = authority.attack_key();

    let Ok(attacker_view) = viewer.get_for_attack(entity, hand).inspect_err(|err| {
        info!(attack_key = attack_key.map(MeleeAttackKey::encoded), attacker = ?entity, reason = %err, "melee_completion_rejected");
    }) else {
        return;
    };
    let Ok(defender_view) = viewer.get(event.target).inspect_err(|err| {
        info!(attack_key = attack_key.map(MeleeAttackKey::encoded), attacker = ?entity, target = ?event.target, reason = %err, "melee_completion_rejected");
    }) else {
        return;
    };

    let Ok([attacker_character, defender_character]) = q_character
        .get_many([entity, event.target])
        .inspect_err(|err| {
            info!(attack_key = attack_key.map(MeleeAttackKey::encoded), attacker = ?entity, target = ?event.target, reason = %err, "melee_completion_rejected");
        })
    else {
        return;
    };
    let (attacker_look, attacker_transform) = attacker_character;
    let (defender_look, defender_transform) = defender_character;

    let Ok(attacker_dimensions) = q_dimensions.get(entity) else {
        info!(attack_key = attack_key.map(MeleeAttackKey::encoded), attacker = ?entity, reason = "missing_attacker_dimensions", "melee_completion_rejected");
        return;
    };
    let now = CombatInstant::from_elapsed(&time);
    let geometry = match MeleeCompletionGeometry::observe(
        CompletionActors { attacker_look, attacker_transform, defender_look, defender_transform,
            dimensions: attacker_dimensions },
        CompletionAuthorization { event: &event, authority, now },
        &attacker_view, &q_sides, &q_states, &config,
    ) {
        Ok(geometry) => geometry,
        Err(error) => {
            warn!(?entity, %error, "cannot admit melee completion geometry");
            return;
        }
    };
    let MeleeCompletionGeometry { surface_distance, reach, contact_at_time, facts } = geometry;
    let contact_sample = event.contact_sample;
    let defense_alignment_sample = event.defense_alignment_sample;
    let Some(completion) = MeleeCompletion::prepare(
        facts,
        authority,
        &spatial,
        &q_scene_items,
        &config,
        attack_key,
        surface_distance,
        reach,
    ) else {
        return;
    };
    let (a2, a1) = completion.attack.attacker_yaw().sin_cos();
    let (d2, d1) = completion.attack.target_yaw().sin_cos();
    let flanking = flanking_from_dir((a1, a2), (d1, d2));
    let Some(attacker_side) = attacker_view.weapon_holding_side() else {
        info!(attack_key = attack_key.map(MeleeAttackKey::encoded), attacker = ?completion.attack.attacker(), target = ?completion.attack.target(), body_part = ?completion.attack.body_part, outcome = "failed", reason = "missing_striking_side", "melee_attack_resolved");
        return;
    };
    let attacker_has_weapon = super::contact::attacker_has_weapon(&viewer, entity, hand);
    let Ok(attacker_state) = q_states.get(completion.attack.attacker()) else {
        return;
    };
    let attacker_performance =
        match combat_incapacitation_performance(attacker_state.incapacitation) {
            Ok(performance) => performance,
            Err(error) => {
                warn!(%error, "invalid attacker performance");
                return;
            }
        };

    let Some(prepared_defense) = prepare_active_defense(
        &completion.attack,
        completion.started_at,
        &attacker_view,
        &defender_view,
        attacker_performance,
        attack_style,
        contact_sample,
        &q_pending,
        &q_authorities,
        &q_skeletons,
        &q_states,
        &time,
        &config,
    ) else {
        return;
    };
    let Ok(mut authority) = q_authorities.get_mut(entity) else {
        return;
    };
    let Some(attack) =
        completion.authorize(&mut authority, now, attack_key, surface_distance, reach)
    else {
        return;
    };
    info!(attack_key = attack.attack_key().encoded(), attacker = ?attack.attacker(), target = ?attack.target(), body_part = ?attack.body_part(), surface_distance_metres = surface_distance.metres(), reach_metres = reach.metres(), "melee_completion_accepted");
    let attempted_defender_response = commit_active_defense(
        prepared_defense,
        &attack,
        &defender_view,
        &mut q_authorities,
        &mut q_states,
        &config,
    );

    // Consume the pending response so it is not reused.
    cmd.entity(attack.target())
        .remove::<PendingDefenderResponse>();

    let impaired_precision =
        ReportedPrecision::new(attack.reported_precision().get() * attacker_performance.fraction())
            .expect("incapacitation preserves finite bounded precision");
    let defense_alignment = weapon_defense_alignment(
        &attacker_view,
        &defender_view,
        attacker_side,
        attack_style,
        impaired_precision,
        flanking,
        attempted_defender_response,
        defense_alignment_sample,
        contact_at_time,
        config.resolution.contact,
    );
    let effective_defender_response =
        defense_alignment.map_or(attempted_defender_response, |alignment| alignment.effective);
    commit_defense_during_attack(
        &mut cmd,
        &attack,
        attempted_defender_response,
        effective_defender_response,
        defense_alignment.map_or(1.0, |alignment| alignment.engagement),
        &defender_view,
        &mut q_authorities,
        &mut q_skeletons,
    );
    let (contact, result) = contact_after_defense(
        &attacker_view,
        &defender_view,
        attacker_side,
        attack_style,
        effective_defender_response,
        impaired_precision,
        flanking,
        contact_sample,
        contact_at_time,
        &config,
    );
    let result = result * attacker_performance.fraction() * attack.power_multiplier();
    emit_melee_outcome(
        &mut cmd,
        &attack,
        entity,
        hand,
        attacker_side,
        attempted_defender_response,
        defense_alignment,
        contact_at_time,
        contact,
        result,
        flanking,
        attacker_has_weapon,
        &attacker_view,
        &defender_view,
        attacker_transform,
        defender_transform,
        &viewer,
        &config,
    );
}
