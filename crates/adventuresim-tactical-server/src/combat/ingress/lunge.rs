//! One collision-aware assisted movement plan owns both arrival and actuation.

use super::*;

pub(super) struct EntityMeleeLungeRequest {
    pub(super) attacker: Entity,
    pub(super) target: Entity,
    pub(super) weapon_reach_metres: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct MeleeLungeRequest<'a> {
    pub(crate) attacker_position: Vec3,
    pub(crate) attacker_collider: &'a Collider,
    pub(crate) attacker_dimensions: CharacterDimensions,
    pub(crate) target_transform: &'a Transform,
    pub(crate) target_collider: &'a Collider,
    pub(crate) weapon_reach_metres: f32,
    pub(crate) quickstep_distance_metres: f32,
}

#[derive(Clone, Copy)]
pub(super) struct PlannedMeleeLunge {
    pub(super) arrival: MeleeLungeArrival,
    movement: Option<MeleeLungeMovement>,
    reach: f32,
    closure: f32,
    maximum_travel: f32,
}

impl PlannedMeleeLunge {
    fn admit(
        request: MeleeLungeRequest<'_>,
        config: &TacticalCombatConfig,
    ) -> AnimationValueResult<Self> {
        let reach = melee_interaction_range(
            request.attacker_dimensions.arm_reach_metres,
            request.weapon_reach_metres,
        );
        let maximum_travel = request
            .quickstep_distance_metres
            .min(melee_collision_clearance(
                request.attacker_position,
                request.attacker_collider,
                request.target_transform,
                request.target_collider,
            ));
        let closure = (melee_surface_measure(
            request.attacker_position,
            request.target_transform.translation,
        ) - reach)
            .max(0.0);
        let movement = planned_melee_lunge(request, config);
        let lunge = match movement {
            Some(movement) if movement.quickstep => MeleeLunge::Quickstep {
                distance_metres: movement.distance_metres,
            },
            Some(movement) => MeleeLunge::Forward {
                distance_metres: movement.distance_metres,
            },
            None => MeleeLunge::None,
        };
        let arrival = melee_lunge_arrival(
            lunge,
            config.movement.speeds_metres_per_second.run,
            conservative_forward_lunge_acceleration(&config.movement.motor)?,
            config.movement.maneuvers.quickstep_duration_seconds,
        )?;
        let movement = match arrival {
            MeleeLungeArrival::CannotArrive => None,
            _ => movement,
        };
        Ok(Self {
            arrival,
            movement,
            reach,
            closure,
            maximum_travel,
        })
    }

    pub(super) fn install(
        self,
        commands: &mut Commands,
        request: EntityMeleeLungeRequest,
        start_tick: AnimationSampleTick,
    ) {
        commands
            .entity(request.attacker)
            .remove::<MeleeLungeMovement>();
        let Some(movement) = self.movement else {
            let outcome = if self.closure <= 1.0e-5 {
                "already_in_reach"
            } else {
                "unreachable_no_movement"
            };
            info!(attack_key = start_tick.encoded(), attacker = ?request.attacker, target = ?request.target, outcome, reach_metres = self.reach, closure_metres = self.closure, maximum_travel_metres = self.maximum_travel, "melee_lunge_planned");
            return;
        };
        info!(attack_key = start_tick.encoded(), attacker = ?request.attacker, target = ?request.target, outcome = if movement.quickstep { "quickstep" } else { "forward" }, reach_metres = self.reach, planned_distance_metres = movement.distance_metres, maximum_travel_metres = self.maximum_travel, "melee_lunge_planned");
        commands.entity(request.attacker).insert(movement);
        if movement.quickstep {
            commands.entity(request.attacker).insert(QuickstepPush {
                start_tick,
                direction: Vec2::new(movement.direction.x, -movement.direction.y),
                orientation: Quat::IDENTITY,
                origin: movement.origin,
                active: true,
            });
        }
    }
}

pub(super) fn planned_melee_lunge_for_entities(
    request: EntityMeleeLungeRequest,
    transforms: &Query<&Transform>,
    dimensions: &Query<&CharacterDimensions>,
    colliders: &Query<&Collider>,
    config: &TacticalCombatConfig,
) -> AnimationValueResult<Option<PlannedMeleeLunge>> {
    let Ok([attacker_transform, target_transform]) =
        transforms.get_many([request.attacker, request.target])
    else {
        return Ok(None);
    };
    let Ok([attacker_collider, target_collider]) =
        colliders.get_many([request.attacker, request.target])
    else {
        return Ok(None);
    };
    let dimensions = dimensions
        .get(request.attacker)
        .copied()
        .unwrap_or_default();
    Ok(Some(PlannedMeleeLunge::admit(
        MeleeLungeRequest {
            attacker_position: attacker_transform.translation,
            attacker_collider,
            attacker_dimensions: dimensions,
            target_transform,
            target_collider,
            weapon_reach_metres: request.weapon_reach_metres,
            quickstep_distance_metres: quickstep_target_displacement_metres(
                dimensions.leg_length_metres,
                &config.movement.motor,
            ),
        },
        config,
    )?))
}

pub(super) fn planned_melee_lunge(
    request: MeleeLungeRequest<'_>,
    _config: &TacticalCombatConfig,
) -> Option<MeleeLungeMovement> {
    let direction = (request.target_transform.translation - request.attacker_position)
        .xz()
        .normalize_or_zero();
    if direction == Vec2::ZERO {
        return None;
    }
    let arm_reach = request.attacker_dimensions.arm_reach_metres;
    let maximum_travel = request
        .quickstep_distance_metres
        .min(melee_collision_clearance(
            request.attacker_position,
            request.attacker_collider,
            request.target_transform,
            request.target_collider,
        ));
    let (distance_metres, quickstep) = match melee_lunge(
        melee_surface_measure(
            request.attacker_position,
            request.target_transform.translation,
        ),
        arm_reach,
        request.weapon_reach_metres,
        maximum_travel,
    ) {
        MeleeLunge::None => return None,
        MeleeLunge::Forward { distance_metres } => (distance_metres, false),
        MeleeLunge::Quickstep { distance_metres } => (distance_metres, true),
    };
    Some(MeleeLungeMovement {
        origin: request.attacker_position,
        direction,
        distance_metres,
        quickstep,
    })
}

pub(crate) fn melee_surface_measure(attacker_position: Vec3, target_position: Vec3) -> f32 {
    (attacker_position.xz().distance(target_position.xz())
        - adventuresim_core::combat::HUMANOID_MELEE_MINIMUM_CENTER_SEPARATION_METRES)
        .max(0.0)
}

pub(crate) fn melee_collision_clearance(
    attacker_position: Vec3,
    attacker_collider: &Collider,
    target_transform: &Transform,
    target_collider: &Collider,
) -> f32 {
    let a = attacker_collider.aabb(attacker_position, Rotation::default());
    let b = target_collider.aabb(
        target_transform.translation,
        Rotation(target_transform.rotation),
    );
    let ar = ((a.max - a.min) * 0.5).xz().max_element();
    let br = ((b.max - b.min) * 0.5).xz().max_element();
    (attacker_position
        .xz()
        .distance(target_transform.translation.xz())
        - ar
        - br)
        .max(0.0)
}

pub(crate) fn melee_target_reachable(request: MeleeLungeRequest<'_>) -> bool {
    let maximum_travel = request
        .quickstep_distance_metres
        .min(melee_collision_clearance(
            request.attacker_position,
            request.attacker_collider,
            request.target_transform,
            request.target_collider,
        ));
    melee_surface_measure(
        request.attacker_position,
        request.target_transform.translation,
    ) <= melee_interaction_range(
        request.attacker_dimensions.arm_reach_metres,
        request.weapon_reach_metres,
    ) + maximum_travel
        + 1.0e-5
}

pub(crate) fn melee_target_lunge_arrival(
    request: MeleeLungeRequest<'_>,
    config: &TacticalCombatConfig,
) -> AnimationValueResult<Option<MeleeLungeArrival>> {
    if !melee_target_reachable(request) {
        return Ok(None);
    }
    Ok(Some(PlannedMeleeLunge::admit(request, config)?.arrival))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn surface_gap_plans_forward_quickstep_and_unreachable_attack_movement() {
        let collider = Collider::cylinder(0.4, 1.9);
        let dimensions = CharacterDimensions::default();
        let config = TacticalCombatConfig::default();
        let plan = |target_x| {
            planned_melee_lunge(
                MeleeLungeRequest {
                    attacker_position: Vec3::ZERO,
                    attacker_collider: &collider,
                    attacker_dimensions: dimensions,
                    target_transform: &Transform::from_xyz(target_x, 0.0, 0.0),
                    target_collider: &collider,
                    weapon_reach_metres: 0.8,
                    quickstep_distance_metres: 1.0,
                },
                &config,
            )
        };

        assert!(plan(2.4).is_some(), "reachable target should plan movement");
        assert!(
            plan(3.3).is_none(),
            "collision-limited unreachable target must not lunge"
        );

        let fist = planned_melee_lunge(
            MeleeLungeRequest {
                attacker_position: Vec3::ZERO,
                attacker_collider: &collider,
                attacker_dimensions: CharacterDimensions {
                    arm_reach_metres: 0.55,
                    ..dimensions
                },
                target_transform: &Transform::from_xyz(1.25, 0.0, 0.0),
                target_collider: &collider,
                weapon_reach_metres: 0.0,
                quickstep_distance_metres: 1.0,
            },
            &config,
        );
        assert!(
            fist.is_none(),
            "fist target is already within center-based reach"
        );
    }
}
