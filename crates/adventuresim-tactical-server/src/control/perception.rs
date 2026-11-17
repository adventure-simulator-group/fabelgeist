//! Only accepted actions create perceived threats. Damage establishes awareness.

use super::*;
use adventuresim_tactical_core::animation::AnimationElapsed;

#[derive(Event, Clone, Copy)]
pub(crate) struct AcceptedAttack {
    pub(crate) attacker: Entity,
    pub(crate) target: Option<Entity>,
    pub(crate) windup: AnimationElapsed,
}

#[derive(Event, Clone, Copy)]
pub(crate) struct PerceivedThreat {
    pub(crate) attacker: Entity,
    pub(crate) defender: Entity,
    pub(crate) windup: AnimationElapsed,
}

#[derive(Event, Clone, Copy)]
pub(crate) struct ReceivedDamage {
    pub(crate) attacker: Entity,
    pub(crate) defender: Entity,
}

#[derive(Component, Default)]
pub(crate) struct ThreatMemory {
    pub(crate) threat: Option<RememberedThreat>,
}

pub(crate) struct RememberedThreat {
    pub(crate) attacker: Entity,
    pub(crate) anchor: Position<Scene>,
    pub(crate) at: CombatInstant,
}

pub(crate) fn perceive(
    spatial: &SpatialQuery,
    filter: &SpatialQueryFilter,
    observer: &Transform,
    target: &Transform,
    config: &ControlConfig,
) -> bool {
    let delta = target.translation - observer.translation;
    if delta.length() > config.perception_radius_metres.metres() {
        return false;
    }
    let Ok(direction) = Dir3::new(delta) else {
        return true;
    };
    spatial
        .cast_ray(
            observer.translation,
            direction,
            delta.length(),
            true,
            filter,
        )
        .is_none()
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy supplies perception, character state, and threat publication independently"
)]
pub(crate) fn observe_attack(
    event: On<AcceptedAttack>,
    mut commands: Commands,
    time: Res<Time>,
    spatial: SpatialQuery,
    config: Res<TacticalCombatConfig>,
    actors: Query<Entity, With<Player>>,
    items: Query<Entity, With<TacticalSceneItem>>,
    characters: Query<(&Transform, &CharacterLook, &TacticalCombatSide)>,
    mut observers: Query<(
        Entity,
        &CharacterControl,
        &BehaviorPolicy,
        &mut ThreatMemory,
    )>,
) {
    let Ok((attacker, _, side)) = characters.get(event.attacker) else {
        return;
    };
    let filter = control_filter(&actors, &items);
    for (entity, control, policy, mut memory) in &mut observers {
        if control.mode != ControlMode::Indirect
            || policy.retaliation == RetaliationPolicy::Disabled
        {
            continue;
        }
        let Ok((observer, look, observer_side)) = characters.get(entity) else {
            continue;
        };
        if !observer_side.is_hostile_to(*side) {
            continue;
        }
        if event.target != Some(entity)
            && observer.translation.distance(attacker.translation)
                > config.ai.control.counterattack_radius_metres.metres()
        {
            continue;
        }
        let offset = attacker.translation - observer.translation;
        let forward = Quat::from_rotation_y(look.yaw) * Vec3::NEG_Z;
        if forward.dot(offset) < 0.0
            || !perceive(&spatial, &filter, observer, attacker, &config.ai.control)
        {
            continue;
        }
        if let Ok(anchor) = Position::from_metres(observer.translation) {
            memory.threat = Some(RememberedThreat {
                attacker: event.attacker,
                anchor,
                at: CombatInstant::from_elapsed(&time),
            });
            commands.trigger(PerceivedThreat {
                attacker: event.attacker,
                defender: entity,
                windup: event.windup,
            });
        }
    }
}

pub(crate) fn observe_damage(
    event: On<ReceivedDamage>,
    time: Res<Time>,
    mut victims: Query<(
        &CharacterControl,
        &BehaviorPolicy,
        &Transform,
        &mut ThreatMemory,
    )>,
) {
    let Ok((control, policy, transform, mut memory)) = victims.get_mut(event.defender) else {
        return;
    };
    if control.mode != ControlMode::Indirect || policy.retaliation == RetaliationPolicy::Disabled {
        return;
    }
    if let Ok(anchor) = Position::from_metres(transform.translation) {
        memory.threat = Some(RememberedThreat {
            attacker: event.attacker,
            anchor,
            at: CombatInstant::from_elapsed(&time),
        });
    }
}
