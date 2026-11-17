//! Select goals before combat, then arbitrate pursuit through navigation.

use super::perception::perceive;
use super::*;

// Wait until the body is within 30 degrees of the next path segment before
// advancing in guard, so pursuit does not begin as a sideways run.
const PURSUIT_FORWARD_ALIGNMENT: f32 = 0.866_025_4;

type BehaviorActors<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static CharacterControl,
        &'static mut CharacterOrder,
        &'static BehaviorPolicy,
        &'static EngagementPolicy,
        &'static mut AgentState,
        &'static mut ThreatMemory,
    ),
>;

type BehaviorCandidates<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static TacticalCombatSide,
        &'static TacticalCombatState,
        Option<&'static crate::bot::CombatantYielded>,
        &'static CharacterDimensions,
    ),
>;

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct BehaviorContext<'w, 's> {
    spatial: SpatialQuery<'w, 's>,
    actors: Query<'w, 's, Entity, With<Player>>,
    items: Query<'w, 's, Entity, With<TacticalSceneItem>>,
    candidates: BehaviorCandidates<'w, 's>,
    config: Res<'w, TacticalCombatConfig>,
    time: Res<'w, Time>,
}

enum RouteProgress {
    Travelling,
    Arrived,
}

pub(super) fn prepare_behavior(mut agents: BehaviorActors, context: BehaviorContext) {
    let filter = control_filter(&context.actors, &context.items);
    let config = &context.config.ai.control;
    for (entity, control, mut order, policy, engagement, mut agent, mut memory) in &mut agents {
        agent.decision = BehaviorDecision::Idle;
        if control.mode != ControlMode::Indirect {
            continue;
        }
        let Ok((_, origin, side, state, yielded, dimensions)) = context.candidates.get(entity)
        else {
            continue;
        };
        if state.is_incapacitated() || yielded.is_some() {
            continue;
        }
        let valid_target = |target| {
            context.candidates.get(target).ok().filter(
                |(_, transform, target_side, target_state, yielded, _)| {
                    side.is_hostile_to(**target_side)
                        && !target_state.is_incapacitated()
                        && yielded.is_none()
                        && perceive(&context.spatial, &filter, origin, transform, config)
                },
            )
        };
        if memory.threat.as_ref().is_some_and(|threat| {
            CombatInstant::from_elapsed(&context.time)
                .elapsed_since(threat.at)
                .as_secs_f32()
                > config.threat_memory_seconds.seconds()
                || threat.anchor.metres().distance(origin.translation)
                    > config.pursuit_radius_metres.metres()
                || valid_target(threat.attacker).is_none()
        }) {
            memory.threat = None;
        }
        let retaliation = if policy.retaliation == RetaliationPolicy::SelfDefense {
            memory.threat.as_ref().map(|threat| threat.attacker)
        } else {
            None
        };
        let counterattacker = retaliation.filter(|target| {
            valid_target(*target).is_some_and(|(_, transform, _, _, _, _)| {
                transform.translation.distance(origin.translation)
                    <= config.counterattack_radius_metres.metres()
            })
        });
        match order.goal {
            CharacterGoal::Talk { .. } => {
                if order.status == OrderStatus::Defending {
                    order.status = OrderStatus::Executing;
                }
                select_counterattack(counterattacker, &mut agent, &mut order);
            }
            CharacterGoal::Attack { target } => {
                select_attack(
                    &context,
                    target,
                    valid_target(target).is_some(),
                    &mut agent,
                    &mut order,
                );
            }
            CharacterGoal::MoveTo(destination) => {
                order.status = OrderStatus::Executing;
                agent.decision = BehaviorDecision::Travel(destination);
                select_counterattack(counterattacker, &mut agent, &mut order);
            }
            CharacterGoal::Hold { anchor } => {
                order.status = OrderStatus::Executing;
                if anchor.metres().xz().distance(origin.translation.xz())
                    > config.hold_radius_metres.metres()
                {
                    let dimensions = *dimensions;
                    let world = NavigationWorld {
                        spatial: &context.spatial,
                        filter: filter.clone(),
                        config: &context.config,
                        dimensions,
                    };
                    if let Some(destination) = world.support_at(anchor) {
                        agent.decision = BehaviorDecision::Travel(NavigationDestination {
                            position: destination.position,
                        });
                    }
                } else {
                    select_counterattack(counterattacker, &mut agent, &mut order);
                }
            }
            CharacterGoal::Idle => {
                let target = retaliation.or_else(|| {
                    matches!(engagement, EngagementPolicy::SeekOpponents)
                        .then(|| nearest_opponent(&context, origin, side, &filter))
                        .flatten()
                });
                select_idle_behavior(target, &mut agent, &mut order);
            }
        }
    }
}

fn select_counterattack(
    target: Option<Entity>,
    agent: &mut AgentState,
    order: &mut CharacterOrder,
) {
    if let Some(target) = target {
        agent.decision = BehaviorDecision::Engage {
            target,
            pursuit: PursuitPolicy::Forbidden,
        };
        order.status = OrderStatus::Defending;
    }
}

fn select_idle_behavior(
    target: Option<Entity>,
    agent: &mut AgentState,
    order: &mut CharacterOrder,
) {
    if let Some(target) = target {
        agent.decision = BehaviorDecision::Engage {
            target,
            pursuit: PursuitPolicy::Allowed,
        };
        order.status = OrderStatus::Defending;
    } else if order.status == OrderStatus::Defending {
        order.status = OrderStatus::Idle;
    }
}

fn select_attack(
    context: &BehaviorContext,
    target: Entity,
    visible: bool,
    agent: &mut AgentState,
    order: &mut CharacterOrder,
) {
    if visible {
        agent.decision = BehaviorDecision::Engage {
            target,
            pursuit: PursuitPolicy::Allowed,
        };
        order.status = OrderStatus::Executing;
        return;
    }
    order.goal = CharacterGoal::Idle;
    order.status = if context
        .candidates
        .get(target)
        .is_ok_and(|(_, _, _, state, yielded, _)| state.is_incapacitated() || yielded.is_some())
    {
        OrderStatus::Completed
    } else {
        OrderStatus::Failed(ControlRejection::InvalidTarget)
    };
}

fn nearest_opponent(
    context: &BehaviorContext,
    origin: &Transform,
    side: &TacticalCombatSide,
    filter: &SpatialQueryFilter,
) -> Option<Entity> {
    context
        .candidates
        .iter()
        .filter(|(_, transform, target_side, state, yielded, _)| {
            side.is_hostile_to(**target_side)
                && !state.is_incapacitated()
                && yielded.is_none()
                && perceive(
                    &context.spatial,
                    filter,
                    origin,
                    transform,
                    &context.config.ai.control,
                )
        })
        .min_by(|left, right| {
            origin
                .translation
                .distance_squared(left.1.translation)
                .total_cmp(&origin.translation.distance_squared(right.1.translation))
                .then_with(|| left.0.cmp(&right.0))
        })
        .map(|candidate| candidate.0)
}

type NavigatingActors<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static CharacterControl,
        &'static Transform,
        &'static CharacterDimensions,
        &'static TacticalCombatState,
        &'static SkeletonState,
        &'static mut AgentState,
        &'static mut CharacterOrder,
        &'static mut WaypointQueue,
        &'static mut AuthoritativeMovementIntent,
        &'static mut CharacterLook,
        Option<&'static super::navigation::NavigationAgent>,
    ),
>;

#[expect(
    clippy::too_many_arguments,
    reason = "navigation combines authoritative actors, scene queries, and Landmass output"
)]
pub(super) fn follow_routes(
    mut agents: NavigatingActors,
    spatial: SpatialQuery,
    actors: Query<Entity, With<Player>>,
    items: Query<Entity, With<TacticalSceneItem>>,
    targets: Query<(&Transform, &CharacterDimensions)>,
    navigation: Res<crate::navigation::TacticalNavigation>,
    navigation_agents: Query<(
        &bevy_landmass::AgentState,
        &bevy_landmass::AgentDesiredVelocity3d,
    )>,
    time: Res<Time>,
    config: Res<TacticalCombatConfig>,
) {
    let filter = control_filter(&actors, &items);
    for (
        entity,
        control,
        transform,
        dimensions,
        state,
        skeleton,
        mut agent,
        mut order,
        mut waypoints,
        mut movement,
        mut look,
        navigation_agent,
    ) in &mut agents
    {
        if control.mode != ControlMode::Indirect {
            continue;
        }
        if state.is_incapacitated()
            || skeleton.is_posture_transitioning()
            || matches!(
                skeleton.action_kind(),
                SkeletonAction::Attack | SkeletonAction::Dodge
            )
        {
            movement.0 = None;
            continue;
        }
        let world = NavigationWorld {
            spatial: &spatial,
            filter: filter.clone(),
            config: &config,
            dimensions: *dimensions,
        };
        let destination =
            match navigation_destination(agent.decision, &world, &targets, &mut movement) {
                Ok(Some(destination)) => destination,
                Ok(None) => {
                    agent.last_progress = None;
                    continue;
                }
                Err(error) => {
                    apply_route_result(
                        entity,
                        Err(error),
                        &mut agent,
                        &mut order,
                        &mut waypoints,
                        &mut movement,
                    );
                    continue;
                }
            };
        let result = drive_navigation(
            &navigation,
            navigation_agent.and_then(|agent| navigation_agents.get(agent.0).ok()),
            transform,
            &world,
            destination,
            &mut agent,
            &mut movement,
            &mut look,
            CombatInstant::from_elapsed(&time),
        );
        apply_route_result(
            entity,
            result,
            &mut agent,
            &mut order,
            &mut waypoints,
            &mut movement,
        );
    }
}

fn navigation_destination(
    decision: BehaviorDecision,
    world: &NavigationWorld,
    targets: &Query<(&Transform, &CharacterDimensions)>,
    movement: &mut AuthoritativeMovementIntent,
) -> Result<Option<crate::navigation::SupportedPoint>, ControlRejection> {
    match decision {
        BehaviorDecision::Travel(destination) => world.admit(destination).map(Some),
        BehaviorDecision::Engage {
            pursuit: PursuitPolicy::Forbidden,
            ..
        }
        | BehaviorDecision::Idle => {
            movement.0 = None;
            Ok(None)
        }
        BehaviorDecision::Engage {
            target,
            pursuit: PursuitPolicy::Allowed,
        } if movement.0.is_some_and(|input| input.y > 0.0) => Ok(targets
            .get(target)
            .ok()
            .and_then(|(transform, dimensions)| {
                super::navigation::target_feet(transform, dimensions)
            })
            .and_then(|position| world.support_at(position))),
        _ => Ok(None),
    }
}

fn apply_route_result(
    entity: Entity,
    result: Result<RouteProgress, ControlRejection>,
    agent: &mut AgentState,
    order: &mut CharacterOrder,
    waypoints: &mut WaypointQueue,
    movement: &mut AuthoritativeMovementIntent,
) {
    match result {
        Ok(RouteProgress::Arrived)
            if matches!(agent.decision, BehaviorDecision::Travel(_))
                && matches!(order.goal, CharacterGoal::MoveTo(_)) =>
        {
            if let Some(destination) = waypoints.pop_front() {
                order.goal = CharacterGoal::MoveTo(destination);
                order.status = OrderStatus::Executing;
                *agent = AgentState::default();
            } else {
                order.goal = CharacterGoal::Idle;
                order.status = OrderStatus::Completed;
            }
        }
        Err(error) => {
            movement.0 = None;
            order.status = OrderStatus::Failed(error);
            order.goal = CharacterGoal::Idle;
            waypoints.clear();
            agent.decision = BehaviorDecision::Idle;
            trace!(?entity, ?error, "navigation stopped");
        }
        _ => {}
    }
}

fn feet(transform: &Transform, dimensions: CharacterDimensions) -> Option<Position<Scene>> {
    Position::from_metres(transform.translation - Vec3::Y * dimensions.body_height_metres * 0.5)
        .ok()
}

#[expect(
    clippy::too_many_arguments,
    reason = "navigation combines server authority, physical state, and Landmass output"
)]
fn drive_navigation(
    navigation: &crate::navigation::TacticalNavigation,
    navigation_agent: Option<(
        &bevy_landmass::AgentState,
        &bevy_landmass::AgentDesiredVelocity3d,
    )>,
    transform: &Transform,
    world: &NavigationWorld,
    destination: crate::navigation::SupportedPoint,
    agent: &mut AgentState,
    movement: &mut AuthoritativeMovementIntent,
    look: &mut CharacterLook,
    now: CombatInstant,
) -> Result<RouteProgress, ControlRejection> {
    let position = feet(transform, world.dimensions).ok_or(ControlRejection::InvalidDestination)?;
    let config = &world.config.ai.control;
    let offset = destination.position.metres() - position.metres();
    if offset.xz().length() <= config.arrival_radius_metres.metres()
        && offset.y.abs() <= config.support_tolerance_metres.metres()
    {
        movement.0 = None;
        return Ok(RouteProgress::Arrived);
    }
    if navigation.failed {
        return Err(ControlRejection::NavigationUnavailable);
    }
    let Some((state, desired)) = navigation_agent.filter(|_| navigation.ready) else {
        movement.0 = None;
        agent.last_progress = None;
        return Ok(RouteProgress::Travelling);
    };
    if matches!(
        state,
        bevy_landmass::AgentState::NoPath
            | bevy_landmass::AgentState::TargetNotOnNavMesh
            | bevy_landmass::AgentState::AgentNotOnNavMesh
    ) {
        movement.0 = None;
        return Err(ControlRejection::Unreachable);
    }
    if agent.last_progress.as_ref().is_none_or(|previous| {
        previous.position.metres().distance(position.metres())
            >= config.arrival_radius_metres.metres()
    }) {
        agent.last_progress = Some(ProgressObservation { position, at: now });
    } else if agent.last_progress.as_ref().is_some_and(|previous| {
        now.elapsed_since(previous.at).as_secs_f32() >= config.no_progress_seconds.seconds()
    }) {
        return Err(ControlRejection::NoProgress);
    }
    let direction = desired.velocity().xz();
    if direction.length_squared() == 0.0 {
        movement.0 = None;
        return Ok(RouteProgress::Travelling);
    }
    look.yaw = (-direction.x).atan2(-direction.y);
    // Skeleton roots face +Z, independently of the controller's -Z look frame.
    let forward = transform.rotation * Vec3::Z;
    let aligned = forward
        .xz()
        .normalize_or_zero()
        .dot(direction.normalize_or_zero())
        >= PURSUIT_FORWARD_ALIGNMENT;
    movement.0 = if matches!(agent.decision, BehaviorDecision::Engage { .. }) && !aligned {
        None
    } else {
        Some(Vec2::Y)
    };
    Ok(RouteProgress::Travelling)
}
