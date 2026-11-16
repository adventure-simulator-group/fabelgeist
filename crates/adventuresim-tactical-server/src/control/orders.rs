//! Authenticate, admit, acknowledge, then enact a control change atomically.

use super::*;

use crate::bot::{
    AimAtNearestOpponentAi, OffensiveCombatAi, RaisedGuardAi, ReactiveDefenseAi, RecoverToUprightAi,
};

type ControlActors<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut CharacterControl,
        &'static mut CharacterOrder,
        &'static mut WaypointQueue,
        &'static mut BehaviorPolicy,
        &'static mut AgentState,
        &'static mut ThreatMemory,
        &'static Transform,
        &'static CharacterDimensions,
        &'static mut AuthoritativeMovementIntent,
        &'static mut input::AccumulatedInput,
        &'static mut AuthoritativePostureIntent,
    ),
>;

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct OrderContext<'w, 's> {
    actors: Query<'w, 's, Entity, With<Player>>,
    targets: Query<
        'w,
        's,
        (
            &'static TacticalCombatSide,
            &'static TacticalCombatState,
            &'static Transform,
        ),
    >,
    items: Query<'w, 's, Entity, With<TacticalSceneItem>>,
    spatial: SpatialQuery<'w, 's>,
    config: Res<'w, TacticalCombatConfig>,
}

pub(super) fn on_control_request(
    event: On<FromClient<ControlRequest>>,
    mut commands: Commands,
    mut controlled: ControlActors<'_, '_>,
    context: OrderContext,
) {
    let Some(actor) = event.client_id.entity() else {
        return;
    };
    let Ok((
        mut control,
        mut order,
        mut waypoints,
        mut policy,
        mut agent,
        mut threats,
        transform,
        dimensions,
        mut movement,
        mut accumulated,
        mut posture,
    )) = controlled.get_mut(actor)
    else {
        return;
    };
    let mut result = validate_change(
        &event,
        actor,
        *control,
        *order,
        &waypoints,
        transform,
        *dimensions,
        &context,
    );
    if result.is_ok() {
        match event.change {
            ControlChange::SetMode(mode) => {
                waypoints.clear();
                control.switch(mode);
                order.goal = CharacterGoal::Idle;
                order.status = OrderStatus::Idle;
                *agent = AgentState::default();
                *threats = ThreatMemory::default();
                movement.0 = None;
                accumulated.last_movement = None;
                accumulated.jumped = None;
                posture.release_control();
                install_control_packages(&mut commands, actor, mode, &context.config);
            }
            ControlChange::Order(goal) => {
                waypoints.clear();
                order.goal = goal;
                order.status = OrderStatus::Executing;
                *agent = AgentState::default();
                movement.0 = None;
                commands.entity(actor).insert(OffensiveCombatAi::default());
            }
            ControlChange::AppendWaypoint(destination) => {
                if matches!(order.goal, CharacterGoal::MoveTo(_)) {
                    result = waypoints.append(destination);
                } else {
                    waypoints.clear();
                    order.goal = CharacterGoal::MoveTo(destination);
                    order.status = OrderStatus::Executing;
                    *agent = AgentState::default();
                    movement.0 = None;
                }
            }
            ControlChange::Stop => {
                waypoints.clear();
                order.goal = CharacterGoal::Idle;
                order.status = OrderStatus::Idle;
                *agent = AgentState::default();
                *threats = ThreatMemory::default();
                movement.0 = None;
                commands.entity(actor).insert(OffensiveCombatAi::default());
                commands
                    .entity(actor)
                    .remove::<crate::bot::defense::PendingBotReaction>();
            }
            ControlChange::Policy(value) => {
                *policy = value;
            }
        }
        if result.is_ok() {
            order.sequence = event.sequence;
        }
    }
    commands.server_trigger(ToClients {
        targets: SendTargets::Single(event.client_id),
        message: ControlResponse {
            sequence: event.sequence,
            control: *control,
            result,
        },
    });
}

fn install_control_packages(
    commands: &mut Commands,
    actor: Entity,
    mode: ControlMode,
    config: &TacticalCombatConfig,
) {
    commands
        .entity(actor)
        .remove::<crate::bot::defense::PendingBotReaction>();
    commands.entity(actor).remove::<OffensiveCombatAi>();
    commands.entity(actor).remove::<(
        ReactiveDefenseAi,
        RaisedGuardAi,
        AimAtNearestOpponentAi,
        RecoverToUprightAi,
        CombatantBehaviorPackages,
    )>();
    if mode == ControlMode::Indirect {
        commands.entity(actor).insert(indirect_packages(config));
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "admission checks the actor, authority, queued goals, and current scene"
)]
fn validate_change(
    request: &ControlRequest,
    actor: Entity,
    control: CharacterControl,
    order: CharacterOrder,
    waypoints: &WaypointQueue,
    transform: &Transform,
    dimensions: CharacterDimensions,
    context: &OrderContext,
) -> Result<(), ControlRejection> {
    if request.generation != control.generation {
        return Err(ControlRejection::StaleGeneration);
    }
    if request.sequence <= order.sequence {
        return Err(ControlRejection::StaleOrder);
    }
    if matches!(request.change, ControlChange::SetMode(_)) {
        return Ok(());
    }
    if control.mode != ControlMode::Indirect {
        return Err(ControlRejection::WrongMode);
    }
    match request.change {
        ControlChange::Order(CharacterGoal::Attack { target }) => {
            let Ok((side, _, _)) = context.targets.get(actor) else {
                return Err(ControlRejection::Unauthorized);
            };
            let Ok((target_side, target_state, target_transform)) = context.targets.get(target)
            else {
                return Err(ControlRejection::InvalidTarget);
            };
            if side == target_side
                || target_state.is_incapacitated()
                || actor == target
                || transform.translation.distance(target_transform.translation)
                    > context.config.ai.control.perception_radius_metres.metres()
            {
                return Err(ControlRejection::InvalidTarget);
            }
            if !super::perception::perceive(
                &context.spatial,
                &control_filter(&context.actors, &context.items),
                transform,
                target_transform,
                &context.config.ai.control,
            ) {
                return Err(ControlRejection::InvalidTarget);
            }
        }
        ControlChange::Order(CharacterGoal::MoveTo(destination))
        | ControlChange::AppendWaypoint(destination) => {
            if matches!(request.change, ControlChange::AppendWaypoint(_))
                && matches!(order.goal, CharacterGoal::MoveTo(_))
                && waypoints.destinations().len() >= WaypointQueue::MAX_DESTINATIONS
            {
                return Err(ControlRejection::WaypointQueueFull);
            }
            let world = NavigationWorld {
                spatial: &context.spatial,
                filter: control_filter(&context.actors, &context.items),
                config: &context.config,
                dimensions,
            };
            world.admit(destination)?;
        }
        ControlChange::Order(CharacterGoal::Hold { anchor })
            if anchor.metres().distance(transform.translation)
                > context.config.ai.control.hold_radius_metres.metres() =>
        {
            return Err(ControlRejection::InvalidDestination);
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn on_direct_action(
    event: On<FromClient<DirectActionRequest>>,
    mut commands: Commands,
    controls: Query<&CharacterControl>,
) {
    let Some(actor) = event.client_id.entity() else {
        return;
    };
    if !controls
        .get(actor)
        .is_ok_and(|control| control.accepts_direct(event.generation))
    {
        return;
    }
    // These are admitted payload adapters, not registered wire endpoints.
    match event.action {
        DirectAction::Melee(message) => commands.trigger(FromClient {
            client_id: event.client_id,
            message,
        }),
        DirectAction::Ranged(message) => commands.trigger(FromClient {
            client_id: event.client_id,
            message,
        }),
        DirectAction::Defend(message) => commands.trigger(FromClient {
            client_id: event.client_id,
            message,
        }),
    }
}
