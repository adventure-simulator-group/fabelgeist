use super::*;

pub(super) struct OffensiveFacts {
    pub(super) weapon_reach: f32,
    pub(super) preferred_melee_measure: f32,
    pub(super) weapon_is_melee: bool,
    pub(super) use_ranged: bool,
    pub(super) strike_family: StrikeFamily,
    pub(super) melee_attack_available: bool,
    pub(super) melee_recovery_seconds: Option<RecoveryTime>,
    pub(super) dimensions: CharacterDimensions,
    pub(super) melee_lunge_delay: Option<MeleeLungeArrival>,
    pub(super) instinct: f32,
}

struct WeaponFacts {
    reach: f32,
    preferred: f32,
    melee: bool,
    ranged: bool,
    strike: StrikeFamily,
    available: bool,
    recovery: Option<RecoveryTime>,
}

pub(in crate::bot) fn ranged_weapon_needs_ammo_lookup(
    weapon_is_ranged: bool,
    weapon_reach: f32,
) -> bool {
    weapon_is_ranged && weapon_reach.is_finite() && weapon_reach > 0.0
}

pub(in crate::bot) fn compare_target(
    origin: &Transform,
    a_transform: &Transform,
    a: Entity,
    b_transform: &Transform,
    b: Entity,
) -> Ordering {
    let a_distance_squared = origin
        .translation
        .xz()
        .distance_squared(a_transform.translation.xz());
    let b_distance_squared = origin
        .translation
        .xz()
        .distance_squared(b_transform.translation.xz());
    a_distance_squared
        .total_cmp(&b_distance_squared)
        .then_with(|| a.to_bits().cmp(&b.to_bits()))
}

#[expect(
    clippy::too_many_arguments,
    reason = "facts span both actors and authored movement"
)]
pub(super) fn offensive_facts(
    entity: Entity,
    target: Entity,
    transform: &Transform,
    target_transform: &Transform,
    state: &TacticalCombatState,
    viewer: &TacticalPlayerViewer<'_, '_>,
    dimensions: &Query<&CharacterDimensions>,
    colliders: &Query<&Collider>,
    combat_config: &TacticalCombatConfig,
    config: &AiOffenseConfig,
) -> AnimationValueResult<OffensiveFacts> {
    let dimensions = dimensions.get(entity).copied().unwrap_or_default();
    let WeaponFacts {
        reach,
        preferred,
        melee,
        ranged,
        strike,
        available,
        recovery,
    } = weapon_facts(entity, state, viewer, dimensions.arm_reach_metres, config)?;
    let has_ammo = ranged_weapon_needs_ammo_lookup(ranged, reach)
        && viewer.inventory.get(entity).has_item_id(ARROW_ID);
    let instinct = viewer.get(entity).map_or(5.0, |view| {
        view.raw_single_body_part_attr(SimpleAttribute::Instinct)
    });
    let quickstep_distance = quickstep_target_displacement_metres(
        dimensions.leg_length_metres,
        &combat_config.movement.motor,
    );
    let melee_lunge_delay = lunge_delay(
        entity,
        target,
        transform,
        target_transform,
        dimensions,
        reach,
        quickstep_distance,
        colliders,
        combat_config,
    )?;
    Ok(OffensiveFacts {
        weapon_reach: reach,
        preferred_melee_measure: preferred,
        weapon_is_melee: melee,
        use_ranged: ranged && reach > 0.0 && has_ammo,
        strike_family: strike,
        melee_attack_available: available,
        melee_recovery_seconds: recovery,
        dimensions,
        melee_lunge_delay,
        instinct,
    })
}

fn weapon_facts(
    entity: Entity,
    state: &TacticalCombatState,
    viewer: &TacticalPlayerViewer<'_, '_>,
    arm_reach_metres: f32,
    config: &AiOffenseConfig,
) -> AnimationValueResult<WeaponFacts> {
    viewer.get(entity).map_or_else(
        |_| {
            Ok(WeaponFacts {
                reach: 0.0,
                preferred: 0.0,
                melee: false,
                ranged: false,
                strike: StrikeFamily::Thrust,
                available: false,
                recovery: None,
            })
        },
        |view| {
            let reach = view.weapon_reach();
            let grip = view.weapon_grip_to_tip();
            let head = view.weapon_striking_head_length();
            let distal = has_distal_striking_surface(
                grip,
                head,
                view.weapon_body_material(),
                view.weapon_striking_material(),
            );
            let recovery = incapacitation_adjusted_recovery_seconds(
                attack_recovery_secs(&view, view.weapon_preferred_melee_style(), false)?
                    .max(config.cooldown_seconds),
                combat_incapacitation_performance(state.incapacitation)?,
            )?;
            Ok(WeaponFacts {
                reach,
                preferred: preferred_melee_striking_measure(
                    melee_interaction_range(arm_reach_metres, reach),
                    grip,
                    head,
                    distal,
                    config.melee_measure_reach_fraction,
                ),
                melee: view.weapon_is_melee(),
                ranged: view.weapon_is_ranged(),
                strike: StrikeFamily::from_melee_style(view.weapon_preferred_melee_style()),
                available: melee_attack_capability(&view, &view).is_available(),
                recovery: Some(recovery),
            })
        },
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "lunge projection joins both collider facts"
)]
fn lunge_delay(
    entity: Entity,
    target: Entity,
    transform: &Transform,
    target_transform: &Transform,
    dimensions: CharacterDimensions,
    weapon_reach: f32,
    quickstep_distance: f32,
    colliders: &Query<&Collider>,
    combat_config: &TacticalCombatConfig,
) -> AnimationValueResult<Option<MeleeLungeArrival>> {
    let (Ok(attacker_collider), Ok(target_collider)) =
        (colliders.get(entity), colliders.get(target))
    else {
        return Ok(None);
    };
    crate::combat::melee_target_lunge_arrival(
        crate::combat::MeleeLungeRequest {
            attacker_position: transform.translation,
            attacker_collider,
            attacker_dimensions: dimensions,
            target_transform,
            target_collider,
            weapon_reach_metres: weapon_reach,
            quickstep_distance_metres: quickstep_distance,
        },
        combat_config,
    )
}
