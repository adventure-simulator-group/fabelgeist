//! Translate tactical goals and physical motion to Landmass agent state.

use super::*;
use bevy_landmass::{
    Agent, Agent3dBundle, AgentSettings, AgentTarget3d, ArchipelagoRef3d, TargetReachedCondition,
    Velocity3d,
};

#[derive(Component)]
pub(super) struct NavigationAgent(pub(super) Entity);

#[expect(
    clippy::type_complexity,
    reason = "the Bevy query joins each actor's tactical goal and physical velocity"
)]
pub(super) fn sync_navigation_agents(
    mut commands: Commands,
    navigation: Res<crate::navigation::TacticalNavigation>,
    config: Res<TacticalCombatConfig>,
    actors: Query<(
        Entity,
        &CharacterControl,
        &CharacterDimensions,
        &AgentState,
        &AuthoritativeMovementIntent,
        Option<&LinearVelocity>,
        Option<&NavigationAgent>,
    )>,
    targets: Query<(&Transform, &CharacterDimensions)>,
    mut agents: Query<(&mut AgentTarget3d, &mut Velocity3d)>,
) {
    let Some(archipelago) = navigation.archipelago else {
        return;
    };
    for (actor, control, dimensions, state, movement, velocity, nav_agent) in &actors {
        let target = if control.mode != ControlMode::Indirect {
            None
        } else {
            match state.decision {
                BehaviorDecision::Travel(destination) => Some(destination.position.metres()),
                BehaviorDecision::Engage {
                    target,
                    pursuit: PursuitPolicy::Allowed,
                } if movement.0.is_some_and(|input| input.y > 0.0) => {
                    targets.get(target).ok().map(|(transform, dimensions)| {
                        transform.translation - Vec3::Y * dimensions.body_height_metres * 0.5
                    })
                }
                _ => None,
            }
        };
        let target = target.map_or(AgentTarget3d::None, AgentTarget3d::Point);
        if let Some(nav_agent) = nav_agent {
            if let Ok((mut current, mut nav_velocity)) = agents.get_mut(nav_agent.0) {
                if *current != target {
                    *current = target;
                }
                nav_velocity.velocity = velocity.map_or(Vec3::ZERO, |velocity| velocity.0);
            }
            continue;
        }
        let radius = adventuresim_core::combat::HUMANOID_COLLISION_RADIUS_METRES;
        let speed = TACTICAL_WALK_SPEED_METRES_PER_SECOND.metres_per_second();
        let agent = commands
            .spawn((
                Agent3dBundle {
                    agent: Agent::default(),
                    archipelago_ref: ArchipelagoRef3d::new(archipelago),
                    settings: AgentSettings {
                        radius,
                        desired_speed: speed,
                        max_speed: speed,
                    },
                },
                Transform::from_xyz(0.0, -dimensions.body_height_metres * 0.5, 0.0),
                ChildOf(actor),
                target,
                TargetReachedCondition::Distance(Some(
                    config.ai.control.arrival_radius_metres.metres(),
                )),
            ))
            .id();
        commands.entity(actor).insert(NavigationAgent(agent));
    }
}
