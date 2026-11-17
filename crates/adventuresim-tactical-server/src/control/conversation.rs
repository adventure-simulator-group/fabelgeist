//! Approach a resident through ordinary navigation; only physical arrival
//! produces a request to establish durable dialogue authority.

use super::*;
use crate::residents::ResidentActor;
use adventuresim_tactical_core::avian3d::prelude::Position as PhysicsPosition;
use bevy::ecs::system::SystemParam;

type TalkActors<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static CharacterControl,
        &'static mut CharacterOrder,
        &'static PhysicsPosition,
        &'static CharacterDimensions,
        &'static TacticalCombatSide,
        &'static TacticalCombatState,
        &'static mut AgentState,
        &'static mut AuthoritativeMovementIntent,
        Option<&'static TalkApproach>,
    ),
>;

type TalkTargets<'w, 's> = Query<
    'w,
    's,
    (
        &'static PhysicsPosition,
        &'static CharacterDimensions,
        &'static TacticalCombatSide,
        &'static TacticalCombatState,
    ),
    With<ResidentActor>,
>;

type ContactActors<'w, 's> = Query<
    'w,
    's,
    (
        &'static CharacterId,
        &'static CharacterControl,
        &'static CharacterOrder,
        &'static PhysicsPosition,
        &'static CharacterDimensions,
        &'static TacticalCombatSide,
        &'static TacticalCombatState,
        &'static TalkApproach,
    ),
>;

pub(crate) struct TalkParticipants {
    pub(crate) actor: CharacterId,
    pub(crate) resident: CharacterId,
}

#[derive(Component, Clone, Copy)]
pub(super) struct TalkApproach {
    target: Entity,
    generation: ControlGeneration,
    sequence: OrderSequence,
    target_anchor: Position<Scene>,
}

#[derive(Message, Clone, Copy, Debug)]
pub(crate) struct ReachedTalkTarget {
    pub(crate) actor: Entity,
    pub(crate) target: Entity,
    pub(crate) generation: ControlGeneration,
    pub(crate) sequence: OrderSequence,
}

struct TalkActor<'a> {
    entity: Entity,
    control: CharacterControl,
    order: CharacterOrder,
    position: &'a PhysicsPosition,
    dimensions: &'a CharacterDimensions,
    side: TacticalCombatSide,
    state: &'a TacticalCombatState,
}

struct TalkPlan {
    approach: TalkApproach,
    progress: TalkProgress,
}

#[derive(SystemParam)]
pub(super) struct TalkWorld<'w, 's> {
    targets: TalkTargets<'w, 's>,
    actors: Query<'w, 's, Entity, With<Player>>,
    items: Query<'w, 's, Entity, With<TacticalSceneItem>>,
    spatial: SpatialQuery<'w, 's>,
    config: Res<'w, TacticalCombatConfig>,
}

/// Read-only access to the same physical policy used by the approach motor.
#[derive(SystemParam)]
pub(crate) struct TalkContactWorld<'w, 's> {
    world: TalkWorld<'w, 's>,
    actors: ContactActors<'w, 's>,
    residents: Query<'w, 's, &'static CharacterId, With<ResidentActor>>,
}

enum TalkProgress {
    Travel(NavigationDestination),
    Reached,
}

impl TalkContactWorld<'_, '_> {
    pub(crate) fn confirm(
        &self,
        reached: &ReachedTalkTarget,
    ) -> Result<TalkParticipants, ControlRejection> {
        let (id, control, order, position, dimensions, side, state, approach) = self
            .actors
            .get(reached.actor)
            .map_err(|_| ControlRejection::Unauthorized)?;
        if control.mode != ControlMode::Indirect {
            return Err(ControlRejection::WrongMode);
        }
        if control.generation != reached.generation {
            return Err(ControlRejection::StaleGeneration);
        }
        if order.sequence != reached.sequence
            || order.goal
                != (CharacterGoal::Talk {
                    target: reached.target,
                })
            || order.status != OrderStatus::Completed
        {
            return Err(ControlRejection::StaleOrder);
        }
        let actor = TalkActor {
            entity: reached.actor,
            control: *control,
            order: *order,
            position,
            dimensions,
            side: *side,
            state,
        };
        let plan = self.world.plan(actor, reached.target, Some(approach))?;
        if !matches!(plan.progress, TalkProgress::Reached) {
            return Err(ControlRejection::InvalidTarget);
        }
        let resident = *self
            .residents
            .get(reached.target)
            .map_err(|_| ControlRejection::InvalidTarget)?;
        Ok(TalkParticipants {
            actor: *id,
            resident,
        })
    }
}

impl TalkWorld<'_, '_> {
    fn plan(
        &self,
        actor: TalkActor<'_>,
        target: Entity,
        previous: Option<&TalkApproach>,
    ) -> Result<TalkPlan, ControlRejection> {
        let (position, dimensions, side, state) = self
            .targets
            .get(target)
            .map_err(|_| ControlRejection::InvalidTarget)?;
        // Ahoy interpolates Transform for presentation. Contact uses the current
        // Avian body centre, adapted here to the existing navigation geometry.
        let transform = Transform::from_translation(position.0);
        if actor.entity == target
            || actor.state.is_incapacitated()
            || state.is_incapacitated()
            || actor.side.is_hostile_to(*side)
        {
            return Err(ControlRejection::InvalidTarget);
        }
        let approach = match previous {
            Some(previous)
                if previous.target == target
                    && previous.generation == actor.control.generation
                    && previous.sequence == actor.order.sequence =>
            {
                *previous
            }
            _ => TalkApproach {
                target,
                generation: actor.control.generation,
                sequence: actor.order.sequence,
                target_anchor: Position::from_metres(transform.translation)
                    .map_err(|_| ControlRejection::InvalidTarget)?,
            },
        };
        // The physics adapter compares scene-metre body positions. Bound how
        // far a moving target may lead the actor from its selected location.
        if transform
            .translation
            .distance(approach.target_anchor.metres())
            > self.config.ai.control.pursuit_radius_metres.metres()
        {
            return Err(ControlRejection::InvalidTarget);
        }
        let reached = actor.position.0.distance(transform.translation)
            <= self.config.ai.control.talk_distance_metres.metres()
            && self.has_access(&actor, &transform, dimensions);
        let progress = if reached {
            TalkProgress::Reached
        } else {
            if actor.order.status == OrderStatus::Completed {
                return Err(ControlRejection::InvalidTarget);
            }
            TalkProgress::Travel(NavigationDestination {
                position: navigation::target_feet(&transform, dimensions)
                    .ok_or(ControlRejection::InvalidDestination)?,
            })
        };
        Ok(TalkPlan { approach, progress })
    }

    fn has_access(
        &self,
        actor: &TalkActor<'_>,
        target: &Transform,
        dimensions: &CharacterDimensions,
    ) -> bool {
        // Physical body-top points provide head-height access without treating
        // a merchant's waist-high counter as a full conversation barrier.
        let origin = actor.position.0 + Vec3::Y * actor.dimensions.body_height_metres * 0.5;
        let end = target.translation + Vec3::Y * dimensions.body_height_metres * 0.5;
        let delta = end - origin;
        let Ok(direction) = Dir3::new(delta) else {
            return delta == Vec3::ZERO;
        };
        self.spatial
            .cast_ray(
                origin,
                direction,
                delta.length(),
                true,
                &control_filter(&self.actors, &self.items),
            )
            .is_none()
    }
}

pub(super) fn prepare_approaches(
    mut commands: Commands,
    mut actors: TalkActors,
    world: TalkWorld,
    mut reached: MessageWriter<ReachedTalkTarget>,
) {
    for (
        entity,
        control,
        mut order,
        position,
        dimensions,
        side,
        state,
        mut agent,
        mut movement,
        previous,
    ) in &mut actors
    {
        let CharacterGoal::Talk { target } = order.goal else {
            if previous.is_some() {
                commands.entity(entity).remove::<TalkApproach>();
            }
            continue;
        };
        if control.mode != ControlMode::Indirect {
            commands.entity(entity).remove::<TalkApproach>();
            continue;
        }
        if order.status == OrderStatus::Defending {
            continue;
        }
        let actor = TalkActor {
            entity,
            control: *control,
            order: *order,
            position,
            dimensions,
            side: *side,
            state,
        };
        let plan = match world.plan(actor, target, previous) {
            Ok(plan) => plan,
            Err(error) => {
                order.goal = CharacterGoal::Idle;
                order.status = OrderStatus::Failed(error);
                agent.decision = BehaviorDecision::Idle;
                movement.0 = None;
                commands.entity(entity).remove::<TalkApproach>();
                continue;
            }
        };
        commands.entity(entity).insert(plan.approach);
        match plan.progress {
            TalkProgress::Travel(destination) => {
                order.status = OrderStatus::Executing;
                agent.decision = BehaviorDecision::Travel(destination);
            }
            TalkProgress::Reached => {
                agent.decision = BehaviorDecision::Idle;
                movement.0 = None;
                if order.status != OrderStatus::Completed {
                    reached.write(ReachedTalkTarget {
                        actor: entity,
                        target,
                        generation: control.generation,
                        sequence: order.sequence,
                    });
                }
                order.status = OrderStatus::Completed;
            }
        }
    }
}

#[cfg(test)]
mod tests;
