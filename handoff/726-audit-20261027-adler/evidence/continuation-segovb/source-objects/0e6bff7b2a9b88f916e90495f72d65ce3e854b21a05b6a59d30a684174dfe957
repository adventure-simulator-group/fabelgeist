//! Complete checked contact geometry and native actor observation preparation.

use super::*;

#[derive(Clone, Copy)]
pub(super) struct CompletionActors<'a> {
    pub(super) attacker_look: &'a CharacterLook,
    pub(super) attacker_transform: &'a Transform,
    pub(super) defender_look: &'a CharacterLook,
    pub(super) defender_transform: &'a Transform,
    pub(super) dimensions: &'a CharacterDimensions,
}

#[derive(Clone, Copy)]
pub(super) struct CompletionAuthorization<'a> {
    pub(super) event: &'a MeleeAttackIntent,
    pub(super) authority: &'a MeleeAttackAuthority,
    pub(super) now: CombatInstant,
}

pub(super) struct MeleeCompletionGeometry {
    pub(super) surface_distance: SignedLength,
    pub(super) reach: SignedLength,
    pub(super) contact_at_time: MeleeContactAtTime,
    pub(super) facts: MeleeIntentFacts,
}

impl MeleeCompletionGeometry {
    pub(super) fn observe(
        actors: CompletionActors<'_>,
        authorization: CompletionAuthorization<'_>,
        attacker: &TacticalPlayerView<'_, '_, '_>,
        sides: &Query<&TacticalCombatSide>,
        states: &Query<&mut TacticalCombatState>,
        config: &TacticalCombatConfig,
    ) -> MeleeGeometryResult<Self> {
        let reach = melee_interaction_range(actors.dimensions.arm_reach_metres, attacker.weapon_reach())?;
        let center_separation = actors.attacker_transform.translation.xz()
            .distance(actors.defender_transform.translation.xz());
        let surface_distance = SignedLength::from_metres((center_separation
            - adventuresim_core::combat::HUMANOID_MELEE_MINIMUM_CENTER_SEPARATION_METRES).max(0.0))?;
        let contact_at_time = contact_at_completion(
            attacker, authorization.authority, surface_distance, reach,
            authorization.event.strike_family.melee_style(),
            config.ai.ordinary.offense.melee_measure_reach_fraction,
        )?;
        let facts = completion_intent_facts(
            actors, authorization, attacker, surface_distance, sides, states, config,
        )?;
        Ok(Self { surface_distance, reach, contact_at_time, facts })
    }
}

fn contact_at_completion(
    attacker: &TacticalPlayerView<'_, '_, '_>,
    authority: &MeleeAttackAuthority,
    surface_distance: SignedLength,
    reach: SignedLength,
    attack_style: MeleeAttackStyle,
    fallback_reach_fraction: f32,
) -> fabelgeist_geometry::GeometryResult<MeleeContactAtTime> {
    let grip_to_tip_metres = attacker.weapon_grip_to_tip();
    let striking_head_length_metres = attacker.weapon_striking_head_length();
    let body_material = attacker.weapon_body_material();
    let striking_material = attacker.weapon_striking_material();
    let distal_headed = adventuresim_core::combat::has_distal_striking_surface(
        grip_to_tip_metres,
        striking_head_length_metres,
        body_material,
        striking_material,
    );
    resolve_melee_contact_at_time(MeleeContactAtTimeFacts {
        scheduled_measure_metres: authority
            .scheduled_measure_metres()
            .unwrap_or(surface_distance),
        actual_measure_metres: surface_distance,
        ideal_measure_metres: adventuresim_core::combat::preferred_melee_striking_measure(
            reach,
            grip_to_tip_metres,
            striking_head_length_metres,
            distal_headed,
            fallback_reach_fraction,
        )?,
        effective_reach_metres: reach,
        grip_to_tip_metres,
        total_length_metres: attacker.weapon_total_length(),
        striking_head_length_metres,
        distal_headed,
        attack_style,
        body_material,
        striking_material,
    })
}

fn completion_intent_facts(
    actors: CompletionActors<'_>,
    authorization: CompletionAuthorization<'_>,
    attacker: &TacticalPlayerView<'_, '_, '_>,
    surface_distance: SignedLength,
    sides: &Query<&TacticalCombatSide>,
    states: &Query<&mut TacticalCombatState>,
    config: &TacticalCombatConfig,
) -> fabelgeist_geometry::GeometryResult<MeleeIntentFacts> {
    let CompletionActors { attacker_look, attacker_transform, defender_look, defender_transform,
        dimensions: attacker_dimensions } = actors;
    let CompletionAuthorization { event, authority, now } = authorization;
    Ok(MeleeIntentFacts {
        attacker: event.attacker,
        target: event.target,
        attacker_side: sides.get(event.attacker).ok().copied(),
        target_side: sides.get(event.target).ok().copied(),
        attacker_incapacitated: states
            .get(event.attacker)
            .ok()
            .map(TacticalCombatState::is_incapacitated),
        target_incapacitated: states
            .get(event.target)
            .ok()
            .map(TacticalCombatState::is_incapacitated),
        attack_capability: adventuresim_core::combat::melee_attack_capability(attacker, attacker),
        reported_precision: event.reported_precision,
        arm_reach: attacker_dimensions.arm_reach_metres,
        weapon_reach: attacker.weapon_reach(),
        range_latency_tolerance: config
            .realtime_authority
            .melee
            .range_latency_tolerance_metres,
        separation: surface_distance,
        authority_permits: authority.permits(event.target, event.body_part, now),
        body_part: event.body_part,
        attacker_position: Position::from_metres(attacker_transform.translation)?,
        target_position: Position::from_metres(defender_transform.translation)?,
        attacker_yaw: attacker_look.yaw,
        target_yaw: defender_look.yaw,
    })
}

