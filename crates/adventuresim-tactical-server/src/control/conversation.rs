//! Approach a resident through ordinary navigation; only physical arrival
//! produces a request to establish durable dialogue authority.

use super::*;
use crate::residents::ResidentActor;
use bevy::ecs::system::SystemParam;

type TalkActors<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static CharacterControl,
        &'static mut CharacterOrder,
        &'static Transform,
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
        &'static Transform,
        &'static CharacterDimensions,
        &'static TacticalCombatSide,
        &'static TacticalCombatState,
    ),
    With<ResidentActor>,
>;

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
    transform: &'a Transform,
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

enum TalkProgress {
    Travel(NavigationDestination),
    Reached,
}

impl TalkWorld<'_, '_> {
    fn plan(
        &self,
        actor: TalkActor<'_>,
        target: Entity,
        previous: Option<&TalkApproach>,
    ) -> Result<TalkPlan, ControlRejection> {
        let (transform, dimensions, side, state) = self
            .targets
            .get(target)
            .map_err(|_| ControlRejection::InvalidTarget)?;
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
        let reached = actor.transform.translation.distance(transform.translation)
            <= self.config.ai.control.talk_distance_metres.metres()
            && self.has_access(&actor, transform, dimensions);
        let progress = if reached {
            TalkProgress::Reached
        } else {
            if actor.order.status == OrderStatus::Completed {
                return Err(ControlRejection::InvalidTarget);
            }
            TalkProgress::Travel(NavigationDestination {
                position: navigation::target_feet(transform, dimensions)
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
        let origin =
            actor.transform.translation + Vec3::Y * actor.dimensions.body_height_metres * 0.5;
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
        transform,
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
        let actor = TalkActor {
            entity,
            control: *control,
            order: *order,
            transform,
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
