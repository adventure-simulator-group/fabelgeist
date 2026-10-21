//! One collision-aware assisted movement plan owns both arrival and actuation.

use super::*;
use adventuresim_building_generator::spatial_geometry::{PlanDirection, Position, RigidRotation};
use adventuresim_tactical_core::scene_coordinates::Scene;
use fabelgeist_geometry::SignedLength;

pub(super) struct EntityMeleeLungeRequest {
    pub(super) attacker: Entity,
    pub(super) target: Entity,
    pub(super) weapon_reach_metres: SignedLength,
}

#[derive(Clone, Copy)]
pub(crate) struct MeleeLungeRequest<'a> {
    attacker_position: Position<Scene>,
    attacker_collider: &'a Collider,
    attacker_dimensions: CharacterDimensions,
    target_position: Position<Scene>,
    target_rotation: RigidRotation,
    target_collider: &'a Collider,
    weapon_reach_metres: SignedLength,
    quickstep_distance_metres: SignedLength,
}

#[derive(Clone, Copy)]
pub(super) struct MeleeMovementPlan {
    geometry: MeleeLungeGeometry,
    pub(super) movement: Option<MeleeLungeMovement>,
}

#[derive(Clone, Copy)]
pub(super) struct PlannedMeleeLunge {
    pub(super) arrival: MeleeLungeArrival,
    movement: Option<MeleeLungeMovement>,
    reach: SignedLength,
    closure: SignedLength,
    maximum_travel: SignedLength,
}

impl<'a> MeleeLungeRequest<'a> {
    /// Admit native ECS poses once before passing a scene-frame request.
    pub(crate) fn observe(
        attacker_transform: &Transform,
        attacker_collider: &'a Collider,
        attacker_dimensions: CharacterDimensions,
        target_transform: &Transform,
        target_collider: &'a Collider,
        weapon_reach_metres: SignedLength,
        quickstep_distance_metres: SignedLength,
    ) -> MeleeGeometryResult<Self> {
        Ok(Self {
            attacker_position: Position::from_metres(attacker_transform.translation)?,
            attacker_collider,
            attacker_dimensions,
            target_position: Position::from_metres(target_transform.translation)?,
            target_rotation: RigidRotation::from_quaternion(target_transform.rotation)?,
            target_collider,
            weapon_reach_metres,
            quickstep_distance_metres,
        })
    }
}

impl MeleeMovementPlan {
    pub(super) fn admit(request: MeleeLungeRequest<'_>) -> MeleeGeometryResult<Self> {
        let geometry = MeleeLungeGeometry {
            interaction_range: melee_interaction_range(
                request.attacker_dimensions.arm_reach_metres,
                request.weapon_reach_metres,
            )?,
            maximum_travel: request.quickstep_distance_metres.min(melee_collision_clearance(
                request.attacker_position,
                request.attacker_collider,
                request.target_position,
                request.target_rotation,
                request.target_collider,
            )?),
            separation: melee_surface_measure(request.attacker_position, request.target_position)?,
        };
        // Scene X/Z direction is normalized by the original native kernel;
        // admission retains that represented direction without renormalizing.
        let direction = (request.target_position.metres() - request.attacker_position.metres())
            .xz().normalize_or_zero();
        let movement = if direction == Vec2::ZERO {
            None
        } else {
            let (distance_metres, quickstep) = match melee_lunge(geometry)? {
                MeleeLunge::None => return Ok(Self { geometry, movement: None }),
                MeleeLunge::Forward { distance_metres } => (distance_metres, false),
                MeleeLunge::Quickstep { distance_metres } => (distance_metres, true),
            };
            Some(MeleeLungeMovement {
                origin: request.attacker_position,
                direction: PlanDirection::from_normalized(direction)?,
                distance_metres,
                quickstep,
            })
        };
        Ok(Self { geometry, movement })
    }
}

impl PlannedMeleeLunge {
    fn admit(request: MeleeLungeRequest<'_>, config: &TacticalCombatConfig) -> MeleeGeometryResult<Self> {
        let plan = MeleeMovementPlan::admit(request)?;
        let closure = SignedLength::from_metres((plan.geometry.separation.metres()
            - plan.geometry.interaction_range.metres()).max(0.0))?;
        let lunge = match plan.movement {
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
            _ => plan.movement,
        };
        Ok(Self {
            arrival,
            movement,
            reach: plan.geometry.interaction_range,
            closure,
            maximum_travel: plan.geometry.maximum_travel,
        })
    }

    pub(super) fn install(
        self,
        commands: &mut Commands,
        request: EntityMeleeLungeRequest,
        attack_key: MeleeAttackKey,
        start_tick: AnimationSampleTick,
    ) {
        commands.entity(request.attacker).remove::<MeleeLungeMovement>();
        let Some(movement) = self.movement else {
            let outcome = if self.closure.metres() <= MELEE_LUNGE_DISTANCE_EPSILON_METRES.metres() {
                "already_in_reach"
            } else {
                "unreachable_no_movement"
            };
            info!(attack_key = attack_key.encoded(), attacker = ?request.attacker, target = ?request.target, outcome, reach_metres = self.reach.metres(), closure_metres = self.closure.metres(), maximum_travel_metres = self.maximum_travel.metres(), "melee_lunge_planned");
            return;
        };
        info!(attack_key = attack_key.encoded(), attacker = ?request.attacker, target = ?request.target, outcome = if movement.quickstep { "quickstep" } else { "forward" }, reach_metres = self.reach.metres(), planned_distance_metres = movement.distance_metres.metres(), maximum_travel_metres = self.maximum_travel.metres(), "melee_lunge_planned");
        commands.entity(request.attacker).insert(movement);
        if movement.quickstep {
            let direction = movement.direction.vector();
            commands.entity(request.attacker).insert(QuickstepPush {
                start_tick,
                direction: Vec2::new(direction.x, -direction.y),
                orientation: RigidRotation::IDENTITY,
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
) -> MeleeGeometryResult<Option<PlannedMeleeLunge>> {
    let Ok([attacker_transform, target_transform]) = transforms.get_many([request.attacker, request.target]) else {
        return Ok(None);
    };
    let Ok([attacker_collider, target_collider]) = colliders.get_many([request.attacker, request.target]) else {
        return Ok(None);
    };
    let dimensions = dimensions.get(request.attacker).copied().unwrap_or_default();
    let observed = MeleeLungeRequest::observe(
        attacker_transform,
        attacker_collider,
        dimensions,
        target_transform,
        target_collider,
        request.weapon_reach_metres,
        quickstep_target_displacement_metres(dimensions.leg_length_metres, &config.movement.motor)?,
    )?;
    Ok(Some(PlannedMeleeLunge::admit(observed, config)?))
}

pub(crate) fn melee_surface_measure(
    attacker_position: Position<Scene>,
    target_position: Position<Scene>,
) -> MeleeGeometryResult<SignedLength> {
    Ok(SignedLength::from_metres((attacker_position.metres().xz().distance(target_position.metres().xz())
        - adventuresim_core::combat::HUMANOID_MELEE_MINIMUM_CENTER_SEPARATION_METRES).max(0.0))?)
}

pub(crate) fn melee_collision_clearance(
    attacker_position: Position<Scene>,
    attacker_collider: &Collider,
    target_position: Position<Scene>,
    target_rotation: RigidRotation,
    target_collider: &Collider,
) -> MeleeGeometryResult<SignedLength> {
    // Avian AABBs and glam X/Z distance form the complete native clearance kernel.
    let attacker = attacker_position.metres();
    let target = target_position.metres();
    let a = attacker_collider.aabb(attacker, Rotation::default());
    let b = target_collider.aabb(target, Rotation(target_rotation.quaternion()));
    let ar = ((a.max - a.min) * 0.5).xz().max_element();
    let br = ((b.max - b.min) * 0.5).xz().max_element();
    Ok(SignedLength::from_metres((attacker.xz().distance(target.xz()) - ar - br).max(0.0))?)
}

pub(crate) fn melee_target_reachable(request: MeleeLungeRequest<'_>) -> MeleeGeometryResult<bool> {
    let maximum_travel = request.quickstep_distance_metres.min(melee_collision_clearance(
        request.attacker_position,
        request.attacker_collider,
        request.target_position,
        request.target_rotation,
        request.target_collider,
    )?);
    Ok(melee_surface_measure(request.attacker_position, request.target_position)?.metres()
        <= melee_interaction_range(request.attacker_dimensions.arm_reach_metres, request.weapon_reach_metres)?.metres()
            + maximum_travel.metres() + MELEE_LUNGE_DISTANCE_EPSILON_METRES.metres())
}

pub(crate) fn melee_target_lunge_arrival(
    request: MeleeLungeRequest<'_>,
    config: &TacticalCombatConfig,
) -> MeleeGeometryResult<Option<MeleeLungeArrival>> {
    if !melee_target_reachable(request)? {
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
        let plan = |target_x| {
            MeleeMovementPlan::admit(MeleeLungeRequest::observe(
                &Transform::from_translation(Vec3::ZERO),
                &collider,
                dimensions,
                &Transform::from_xyz(target_x, 0.0, 0.0),
                &collider,
                SignedLength::from_metres(0.8).unwrap(),
                SignedLength::from_metres(1.0).unwrap(),
            ).unwrap()).unwrap().movement
        };

        assert!(plan(2.4).is_some(), "reachable target should plan movement");
        assert!(
            plan(3.3).is_none(),
            "collision-limited unreachable target must not lunge"
        );

        let fist = MeleeMovementPlan::admit(
            MeleeLungeRequest::observe(
                &Transform::from_translation(Vec3::ZERO),
                &collider,
                CharacterDimensions {
                    arm_reach_metres: SignedLength::from_metres(0.55).unwrap(),
                    ..dimensions
                },
                &Transform::from_xyz(1.25, 0.0, 0.0),
                &collider,
                SignedLength::from_metres(0.0).unwrap(),
                SignedLength::from_metres(1.0).unwrap(),
            ).unwrap(),
        ).unwrap().movement;
        assert!(
            fist.is_none(),
            "fist target is already within center-based reach"
        );
    }
}
