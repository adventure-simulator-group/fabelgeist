use adventuresim_tactical_core::animation::{
    AnimationElapsed, AnimationSampleSpan, AnimationSampleTick, AnimationValueResult,
};
use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_netcode::{
    bevy_replicon::prelude::ClientTriggerExt,
    client::{DirectControlState, WeaponGuardInputState},
    message::{DefendRequest, MeleeActionRequest, RangedActionRequest},
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::{
    animation::fallback::spawn_fallback_t_pose,
    presentation::{GrassInteractor, TacticalGameplayCamera},
    targeting::auto_aim_candidate,
};

use quickstep_trace::{LocalQuickstepTrace, trace_local_quickstep_state};

use action_clock::{
    MeleeAcquisitionTiming, MeleePlaybackTiming, animation_ticks, on_attack_fired_hook,
};

use attack_start::{flush_buffered_melee_attacks, try_start_attack};

mod action_clock;
mod attack_start;
mod quickstep_trace;

const HITBOX_LAYER: LayerMask = LayerMask(1 << 1);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // Replication supplies Transform but not a render hierarchy root.
        // Require visibility before Add<Player> observers attach mesh children
        // so authored rigs cannot inherit from a component-less parent.
        app.init_resource::<DirectControlState>()
            .init_resource::<TacticalCombatConfig>()
            .register_required_components_with::<Player, _>(|| Visibility::Inherited)
            .add_observer(on_new_player_added_hook)
            .add_observer(on_attack_fired_hook)
            .add_systems(
                Update,
                ((
                    apply_direct_combat_controls,
                    update_attack_state_system.run_if(any_with_component::<AttackState>),
                    flush_buffered_melee_attacks,
                )
                    .chain(),),
            )
            .add_systems(Update, trace_local_quickstep_state)
            .add_systems(
                PostUpdate,
                predict_local_body_facing.before(bevy::transform::TransformSystems::Propagate),
            );
    }
}

/// Identifies which replicated character receives local controls and the
/// gameplay camera. Kept separate from transport CLI arguments so local
/// presentation fixtures use the exact same player-spawn observer.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Resource)]
pub struct LocalCharacterId(pub u64);

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct ClientPlayer;

/// Render-frame facing state for the locally controlled character. Replicated
/// transforms remain authoritative; this state hides the gaps between their
/// rotation samples for both camera-facing guard and velocity-facing travel.
#[derive(Component, Debug, Clone, Copy, Default)]
struct LocalBodyFacing {
    rotation: Option<Quat>,
}

#[derive(EntityEvent)]
pub struct HitPerformed {
    pub entity: Entity,
    pub direction: Dir3,
    pub origin: Vec3,
    pub length: f32,
}

#[derive(Component, Clone, Copy)]
pub struct LimbHitbox(pub BodyPart);

#[derive(Component)]
pub struct AttackState {
    pub pre_hit_timer: Timer,
    facing_timer: Timer,
    pub reach: f32,
    pub ranged: bool,
    target: Option<AttackTarget>,
    target_position: Option<Vec3>,
    aim_direction: Dir3,
}

#[derive(Debug, Clone, Copy)]
struct AttackTarget {
    body: Entity,
    hitbox: Entity,
    body_part: BodyPart,
    local_strike_point: Vec3,
    arrival: MeleeLungeArrival,
}

#[derive(Clone, Copy)]
struct BodyStableChoice<T> {
    body: Entity,
    resolved: Option<(T, Vec3)>,
    ideal_position: Vec3,
    stable_key: u64,
}

fn body_stable_auto_aim<T: Copy>(
    camera_origin: Vec3,
    camera_forward: Vec3,
    actor_origin: Vec3,
    maximum_distance: f32,
    choices: &[BodyStableChoice<T>],
) -> Option<(T, Vec3)> {
    let mut exhausted = Vec::new();
    loop {
        let body = auto_aim_candidate(
            camera_origin,
            camera_forward,
            actor_origin,
            maximum_distance,
            choices
                .iter()
                .filter(|choice| !exhausted.contains(&choice.body))
                .map(|choice| (choice.body, choice.ideal_position, choice.stable_key)),
        )?;
        if let Some(resolved) = auto_aim_candidate(
            camera_origin,
            camera_forward,
            actor_origin,
            maximum_distance,
            choices
                .iter()
                .filter(|choice| choice.body == body)
                .filter_map(|choice| {
                    choice
                        .resolved
                        .map(|resolved| (resolved, resolved.1, choice.stable_key))
                }),
        ) {
            return Some(resolved);
        }
        exhausted.push(body);
    }
}

#[derive(SystemParam)]
struct CombatTargeting<'w, 's> {
    spatial: SpatialQuery<'w, 's>,
    attackers: Query<
        'w,
        's,
        (
            &'static Transform,
            &'static Collider,
            &'static CharacterDimensions,
            &'static TacticalCombatSide,
            &'static CharacterControllerCamera,
        ),
    >,
    cameras: Query<'w, 's, &'static Transform, With<TacticalGameplayCamera>>,
    hitboxes: Query<
        'w,
        's,
        (
            Entity,
            &'static GlobalTransform,
            &'static Collider,
            &'static ColliderOf,
            &'static LimbHitbox,
        ),
    >,
    combatants: Query<
        'w,
        's,
        (
            &'static TacticalCombatSide,
            &'static TacticalCombatState,
            &'static Transform,
            &'static Collider,
        ),
    >,
    scene_items: Query<'w, 's, Entity, With<TacticalSceneItem>>,
}

#[derive(Component, Debug, Clone, Copy)]
struct BufferedMeleeAttack {
    family: StrikeFamily,
    hand: AttackHand,
}

impl AttackState {
    fn new(
        pre_hit_delay: AnimationElapsed,
        facing_delay: AnimationElapsed,
        reach: f32,
        ranged: bool,
        target: Option<AttackTarget>,
        target_position: Option<Vec3>,
        aim_direction: Dir3,
    ) -> AnimationValueResult<Self> {
        let pre_hit_timer = Timer::new(pre_hit_delay.native_duration()?, TimerMode::Once);
        let facing_timer = Timer::new(facing_delay.native_duration()?, TimerMode::Once);
        Ok(Self {
            pre_hit_timer,
            facing_timer,
            reach,
            ranged,
            target,
            target_position,
            aim_direction,
        })
    }

    pub fn is_attacking(&self) -> bool {
        !self.pre_hit_timer.is_paused() && !self.pre_hit_timer.is_finished()
    }
}

impl CombatTargeting<'_, '_> {
    fn arm_reach(&self, attacker: Entity) -> f32 {
        self.attackers
            .get(attacker)
            .map_or(0.0, |(_, _, dimensions, _, _)| dimensions.arm_reach_metres)
    }

    fn melee_acquisition_range(
        &self,
        attacker: Entity,
        weapon_reach: f32,
        config: &TacticalCombatConfig,
    ) -> (f32, f32) {
        let leg_length = self.attackers.get(attacker).map_or(
            config.movement.motor.reference_quickstep_leg_length_metres,
            |(_, _, dimensions, _, _)| dimensions.leg_length_metres,
        );
        (
            melee_interaction_range(self.arm_reach(attacker), weapon_reach),
            quickstep_target_displacement_metres(leg_length, &config.movement.motor),
        )
    }

    fn acquire(
        &self,
        attacker: Entity,
        reach: (f32, f32),
        lunge_timing: Option<MeleeAcquisitionTiming>,
    ) -> AnimationValueResult<(Option<AttackTarget>, Option<Vec3>, Dir3)> {
        let Ok((transform, collider, dimensions, attacker_side, camera)) =
            self.attackers.get(attacker)
        else {
            return Ok((None, None, Dir3::NEG_Z));
        };
        let Ok(camera_transform) = self.cameras.get(camera.get()) else {
            return Ok((None, None, Dir3::NEG_Z));
        };
        let origin = attack_origin(transform, collider, *dimensions);
        let excluded: Vec<_> = self.scene_items.iter().chain([attacker]).collect();
        let obstruction_filter = SpatialQueryFilter::from_excluded_entities(excluded);
        let choices: Vec<_> = self
            .hitboxes
            .iter()
            .map(
                |(hitbox, target_transform, _hitbox_collider, collider_of, limb)| {
                    let target = collider_of.body;
                    let Ok((target_side, target_state, body_transform, body_collider)) =
                        self.combatants.get(target)
                    else {
                        return Ok(None);
                    };
                    if target_side == attacker_side || target_state.is_incapacitated() {
                        return Ok(None);
                    }
                    let position = target_transform.translation();
                    let sight = position - origin;
                    let distance = sight.length();
                    let Ok(direction) = Dir3::new(sight) else { return Ok(None); };
                    let Some(impact) = (distance > 0.0)
                        .then(|| {
                            self.spatial.cast_ray(
                                origin,
                                direction,
                                distance,
                                true,
                                &obstruction_filter,
                            )
                        })
                        .flatten() else { return Ok(None); };
                    let visible = impact.entity == target
                        || impact.entity == hitbox
                        || self.hitboxes.get(impact.entity).is_ok_and(
                            |(_, _, _, impact_collider, _)| impact_collider.body == target,
                        );
                    let surface_distance = impact.distance;
                    let resolved_strike = if let Some(lunge_timing) = lunge_timing {
                        let maximum_travel = reach.1.min(horizontal_collider_clearance(
                            transform.translation,
                            collider,
                            body_transform,
                            body_collider,
                        ));
                        let surface_measure = (transform
                            .translation
                            .xz()
                            .distance(body_transform.translation.xz())
                            - adventuresim_core::combat::HUMANOID_MELEE_MINIMUM_CENTER_SEPARATION_METRES)
                            .max(0.0);
                        lunge_timing.contact_arrival(surface_measure, reach.0, maximum_travel)?
                            .map(|arrival| (position, arrival))
                    } else {
                        Some((position, MeleeLungeArrival::NotNeeded))
                    };
                    if !attack_target_within_angular_threshold(
                        camera_transform.forward().as_vec3(),
                        position - camera_transform.translation,
                        surface_distance,
                    ) {
                        return Ok(None);
                    }
                    Ok(visible.then_some(BodyStableChoice {
                        body: target,
                        resolved: resolved_strike.map(|(strike_point, arrival)| {
                            (
                                AttackTarget {
                                    body: target,
                                    hitbox,
                                    body_part: limb.0,
                                    local_strike_point: target_transform
                                        .affine()
                                        .inverse()
                                        .transform_point3(strike_point),
                                    arrival,
                                },
                                strike_point,
                            )
                        }),
                        ideal_position: position,
                        stable_key: hitbox.to_bits(),
                    }))
                },
            )
            .collect::<AnimationValueResult<Vec<_>>>()?
            .into_iter().flatten().collect();
        let selected = body_stable_auto_aim(
            camera_transform.translation,
            camera_transform.forward().as_vec3(),
            origin,
            if lunge_timing.is_some() {
                f32::MAX
            } else {
                reach.0
            },
            &choices,
        );
        Ok(match selected {
            Some((target, position)) => (Some(target), Some(position), camera_transform.forward()),
            None => (None, None, camera_transform.forward()),
        })
    }
}

fn attack_origin(
    transform: &Transform,
    collider: &Collider,
    dimensions: CharacterDimensions,
) -> Vec3 {
    let body_base = collider
        .aabb(transform.translation, Rotation(transform.rotation))
        .min
        .y;
    transform
        .translation
        .with_y(body_base + dimensions.body_height_metres * (2.0 / 3.0))
}

fn horizontal_collider_clearance(
    attacker: Vec3,
    attacker_collider: &Collider,
    target_transform: &Transform,
    target_collider: &Collider,
) -> f32 {
    let attacker_bounds = attacker_collider.aabb(attacker, Rotation::default());
    let target_bounds = target_collider.aabb(
        target_transform.translation,
        Rotation(target_transform.rotation),
    );
    let attacker_radius = ((attacker_bounds.max - attacker_bounds.min) * 0.5)
        .xz()
        .max_element();
    let target_radius = ((target_bounds.max - target_bounds.min) * 0.5)
        .xz()
        .max_element();
    (attacker.xz().distance(target_transform.translation.xz()) - attacker_radius - target_radius)
        .max(0.0)
}

fn attack_target_angular_threshold_degrees(surface_distance_metres: f32) -> f32 {
    90.0 / surface_distance_metres.max(0.5)
}

fn attack_target_within_angular_threshold(
    camera_forward: Vec3,
    direction_to_candidate: Vec3,
    surface_distance_metres: f32,
) -> bool {
    let Some(camera_forward) = camera_forward.try_normalize() else {
        return false;
    };
    let Some(direction) = direction_to_candidate.try_normalize() else {
        return false;
    };
    let threshold = attack_target_angular_threshold_degrees(surface_distance_metres).to_radians();
    camera_forward.dot(direction).clamp(-1.0, 1.0) >= threshold.cos()
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects player ownership, camera, mesh, material, and combat configuration state independently"
)]
fn on_new_player_added_hook(
    event: On<Add, Player>,
    mut commands: Commands,
    camera: Single<Entity, With<TacticalGameplayCamera>>,
    query: Query<(&Player, &CharacterId)>,
    local_character: Res<LocalCharacterId>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    combat_config: Res<TacticalCombatConfig>,
) -> Result {
    let (Player { name }, id) = query.get(event.entity)?;
    info!(entity = ?event.entity, id = id.0, "Added new player {name}");

    let is_client_player = local_character.0 == id.0;
    if is_client_player {
        info!(
            entity = ?event.entity,
            "New player is assigned to this client. Assuming control...",
        );

        commands.entity(event.entity).insert((
            tactical_character_controller(),
            ClientPlayer,
            LocalBodyFacing::default(),
            LocalQuickstepTrace::default(),
            GrassInteractor,
            actions!(Player[
                (
                    Action::<input::Movement>::new(),
                    DeadZone::default(),
                    Bindings::spawn((
                        Cardinal::wasd_keys(),
                        Axial::left_stick()
                    ))
                ),
                (
                    Action::<input::RotateCamera>::new(),
                    Bindings::spawn((
                        Spawn((Binding::mouse_motion(), Scale::splat(0.15))),
                        Axial::right_stick().with((
                            Scale::new(Vec3::from_array(
                                combat_config.client_input.gamepad_look_scale,
                            )),
                            DeadZone::default(),
                        )),
                    ))
                ),
            ]),
        ));

        #[cfg(feature = "debug")]
        commands.entity(event.entity).insert(DebugRender::none());

        commands
            .entity(camera.into_inner())
            .insert(CharacterControllerCameraOf::new(event.entity));
    }

    commands.entity(event.entity).with_children(|parent| {
        spawn_fallback_t_pose(
            parent,
            event.entity,
            id.color(),
            &mut meshes,
            &mut materials,
        );

        if !is_client_player {
            for hitbox in &combat_config.targeting.body_part_hitboxes {
                let half_extents = Vec3::from_array(hitbox.half_extents_metres);
                parent.spawn((
                    LimbHitbox(hitbox.body_part),
                    Collider::cuboid(
                        half_extents.x * 2.0,
                        half_extents.y * 2.0,
                        half_extents.z * 2.0,
                    ),
                    CollisionLayers::new(HITBOX_LAYER, LayerMask::ALL),
                    Transform::from_translation(Vec3::from_array(hitbox.center_metres)),
                ));
            }
        }
    });

    Ok(())
}

#[expect(
    clippy::type_complexity,
    reason = "the Bevy query selects the complete local-facing state while excluding the gameplay camera from player transforms"
)]
fn predict_local_body_facing(
    time: Res<Time>,
    guard: Res<WeaponGuardInputState>,
    combat_config: Res<TacticalCombatConfig>,
    mut players: Query<
        (
            &CharacterControllerCamera,
            &SkeletonState,
            Option<&AttackState>,
            &mut Transform,
            &mut LocalBodyFacing,
        ),
        With<ClientPlayer>,
    >,
    cameras: Query<&Transform, (With<TacticalGameplayCamera>, Without<ClientPlayer>)>,
) {
    let elapsed = AnimationElapsed::from(time.delta());
    for (camera, skeleton, attack, mut transform, mut facing) in &mut players {
        if skeleton.body().is_downed() || skeleton.is_posture_transitioning() {
            facing.rotation = None;
            continue;
        }

        let Ok(camera_transform) = cameras.get(camera.get()) else {
            continue;
        };
        let current = facing.rotation.unwrap_or(transform.rotation);
        let next = if let Some((target_position, remaining)) = attack.and_then(|state| {
            state.target_position.map(|target| {
                (
                    target,
                    AnimationElapsed::from(state.facing_timer.remaining()),
                )
            })
        }) {
            let desired_forward = target_position - transform.translation;
            BodyFacingTurn::for_deadline(current, desired_forward, remaining, elapsed)
                .and_then(|turn| turn.advance(current, desired_forward, elapsed))
        } else {
            advance_body_facing_with_turn(
                current,
                camera_transform.rotation,
                skeleton.world_velocity,
                skeleton.action_kind(),
                guard.desired,
                elapsed,
                BodyFacingTurn::configured_half_turn(
                    combat_config.presentation.body_turn_seconds_per_half_turn,
                ),
            )
        };
        match next {
            Ok(rotation) => {
                facing.rotation = Some(rotation);
                transform.rotation = rotation;
            }
            Err(error) => warn!(%error, "cannot predict body facing"),
        }
    }
}

fn update_attack_state_system(
    mut cmd: Commands,
    spatial: SpatialQuery,
    time: Res<Time>,
    mut q_attacker: Query<(
        Entity,
        &Transform,
        &Collider,
        &CharacterDimensions,
        &mut AttackState,
    )>,
    q_collider: Query<(&GlobalTransform, &ColliderOf, &LimbHitbox)>,
    q_scene_items: Query<Entity, With<TacticalSceneItem>>,
    combat_config: Res<TacticalCombatConfig>,
) {
    for (attacker, attacker_transform, attacker_collider, dimensions, mut state) in &mut q_attacker
    {
        if let Some(target) = state.target
            && let Ok((transform, _, _)) = q_collider.get(target.hitbox)
        {
            state.target_position = Some(transform.transform_point(target.local_strike_point));
        }
        state.pre_hit_timer.tick(time.delta());
        state.facing_timer.tick(time.delta());
        if !state.pre_hit_timer.is_finished() {
            continue;
        }

        cmd.entity(attacker).remove::<AttackState>();
        let reach = if state.ranged {
            state.reach
        } else {
            melee_interaction_range(dimensions.arm_reach_metres, state.reach)
        };
        let origin = attack_origin(attacker_transform, attacker_collider, *dimensions);
        let Some(target) = state.target else {
            if state.ranged {
                cmd.client_trigger(RangedActionRequest::CompleteMiss);
            }
            cmd.trigger(HitPerformed {
                entity: attacker,
                direction: state.aim_direction,
                origin,
                length: reach,
            });
            continue;
        };
        let intended_point = state
            .target_position
            .unwrap_or(origin + *state.aim_direction * reach);
        let delta = intended_point - origin;
        let direction = Dir3::new(delta).unwrap_or(state.aim_direction);
        let excluded: Vec<_> = q_scene_items.iter().chain([attacker]).collect();
        let obstruction_filter = SpatialQueryFilter::from_excluded_entities(excluded);
        let obstruction = spatial.cast_ray(
            origin,
            direction,
            delta.length().min(reach),
            true,
            &obstruction_filter,
        );
        let unobstructed = obstruction.is_some_and(|hit| {
            hit.entity == target.body
                || q_collider
                    .get(hit.entity)
                    .is_ok_and(|(_, collider, _)| collider.body == target.body)
        });

        if unobstructed {
            if state.ranged {
                cmd.client_trigger(RangedActionRequest::CompleteHit {
                    target: target.body,
                    body_part: target.body_part,
                    reported_precision: combat_config.targeting.reported_hit_precision,
                });
            }
            cmd.trigger(HitPerformed {
                entity: attacker,
                direction,
                origin,
                length: obstruction.map_or(delta.length(), |hit| hit.distance),
            });
        } else {
            if state.ranged {
                cmd.client_trigger(RangedActionRequest::CompleteMiss);
            }
            cmd.trigger(HitPerformed {
                entity: attacker,
                direction,
                origin,
                length: obstruction.map_or(delta.length().min(reach), |hit| hit.distance),
            });
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct AttackStartRequest {
    entity: Entity,
    alternate_attack: bool,
    hand: AttackHand,
}

fn attack_reach_is_usable(ranged: bool, melee: bool, weapon_reach: f32, arm_reach: f32) -> bool {
    if ranged {
        weapon_reach.is_finite() && weapon_reach > 0.0
    } else {
        melee && arm_reach.is_finite() && arm_reach > 0.0
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects direct controls, player queries, combat resources, and targeting state independently"
)]
fn apply_direct_combat_controls(
    controls: Res<DirectControlState>,
    mut cmd: Commands,
    players: Query<Entity, With<ControlledPlayer>>,
    mut q_character: Query<(Has<AttackState>, &mut SkeletonState)>,
    viewer: TacticalPlayerViewer,
    time: Res<Time>,
    combat_config: Res<TacticalCombatConfig>,
    targeting: CombatTargeting,
) {
    for entity in &players {
        if let Ok((_, mut skeleton)) = q_character.get_mut(entity)
            && let Ok(view) = viewer.get_for_attack(entity, AttackHand::Main)
        {
            let preferred = StrikeFamily::from_melee_style(view.weapon_preferred_melee_style());
            let resolve = |input: MeleePreparationInput| match input {
                MeleePreparationInput::Preferred => AttackAnimation::initial(
                    skeleton
                        .available_strike_family(preferred)
                        .unwrap_or(preferred),
                ),
                MeleePreparationInput::Alternate => AttackAnimation::initial(
                    skeleton
                        .available_strike_family(preferred.alternate())
                        .unwrap_or(preferred),
                ),
                MeleePreparationInput::Offhand
                    if skeleton.attack_animations.offhand_preparation =>
                {
                    AttackAnimation::Offhand
                }
                MeleePreparationInput::Offhand => AttackAnimation::initial(
                    skeleton
                        .available_strike_family(preferred)
                        .unwrap_or(preferred),
                ),
            };
            let from = resolve(controls.local_preparation_from);
            let to = resolve(controls.local_preparation_to);
            skeleton.set_attack_preparation(AttackPreparation {
                from,
                to,
                progress: controls.local_preparation_progress,
            });
        }
        if controls.attack_just_pressed {
            if let Err(error) = try_start_attack(
                AttackStartRequest {
                    entity,
                    alternate_attack: controls.alternate_attack,
                    hand: controls.attack_hand,
                },
                &mut cmd,
                &mut q_character,
                &viewer,
                &time,
                &combat_config,
                &targeting,
            ) {
                warn!(?entity, %error, "cannot schedule attack");
            }
        }
        // Quicksteps travel on the sequenced PlayerInputRequest stream. Do not
        // predict only the SkeletonState here: the client does not simulate the
        // matching QuickstepPush, so an authoritative rejection would otherwise
        // animate a dodge over a stationary replicated transform.
        if controls.roll_just_pressed {
            cmd.client_trigger(DefendRequest::Roll);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn melee_animation_overlaps_lunge_and_contact_waits_for_arrival() {
        assert_eq!(
            MeleeLungeArrival::ArrivesIn(
                adventuresim_core::elapsed::AnimationElapsed::from_seconds(0.1).unwrap()
            )
            .contact_timing(
                adventuresim_core::elapsed::AnimationElapsed::from_seconds(0.3).unwrap()
            )
            .unwrap(),
            MeleeContactTiming {
                delay: adventuresim_core::elapsed::AnimationElapsed::from_seconds(0.0).unwrap(),
                contact: adventuresim_core::elapsed::AnimationElapsed::from_seconds(0.3).unwrap()
            }
        );
        let MeleeContactTiming {
            delay: animation_delay,
            contact,
        } = MeleeLungeArrival::ArrivesIn(
            adventuresim_core::elapsed::AnimationElapsed::from_seconds(0.5).unwrap(),
        )
        .contact_timing(adventuresim_core::elapsed::AnimationElapsed::from_seconds(0.3).unwrap())
        .unwrap();
        assert!((animation_delay.seconds() - 0.2).abs() < 1.0e-6);
        assert!((contact.seconds() - 0.5).abs() < 1.0e-6);
        assert!((contact.seconds() - animation_delay.seconds() - 0.3).abs() < 1.0e-6);

        let state = AttackState::new(
            contact,
            contact,
            0.5,
            false,
            None,
            Some(Vec3::NEG_Z),
            Dir3::NEG_Z,
        )
        .unwrap();
        assert_eq!(
            state.facing_timer.remaining(),
            state.pre_hit_timer.remaining(),
            "facing remains active from input through delayed contact"
        );
    }

    #[test]
    fn unreachable_preferred_head_falls_back_within_the_same_enemy() {
        let preferred = Entity::from_bits(1);
        let other = Entity::from_bits(2);
        let choices = [
            BodyStableChoice {
                body: preferred,
                resolved: None,
                ideal_position: Vec3::new(0.0, 1.8, -2.0),
                stable_key: 1,
            },
            BodyStableChoice {
                body: preferred,
                resolved: Some((BodyPart::Chest, Vec3::new(0.0, 1.2, -2.0))),
                ideal_position: Vec3::new(0.0, 1.2, -2.0),
                stable_key: 2,
            },
            BodyStableChoice {
                body: other,
                resolved: Some((BodyPart::Head, Vec3::new(0.05, 1.8, -1.8))),
                ideal_position: Vec3::new(0.05, 1.8, -1.8),
                stable_key: 3,
            },
        ];
        assert_eq!(
            body_stable_auto_aim(
                Vec3::new(0.0, 1.8, 0.0),
                Vec3::NEG_Z,
                Vec3::ZERO,
                10.0,
                &choices
            ),
            Some((BodyPart::Chest, Vec3::new(0.0, 1.2, -2.0)))
        );
    }

    #[test]
    fn fists_are_usable_from_anatomy_while_ranged_attacks_require_weapon_reach() {
        assert!(attack_reach_is_usable(false, true, 0.0, 0.526_801));
        assert!(attack_reach_is_usable(false, true, 0.8, 0.526_801));
        assert!(!attack_reach_is_usable(false, true, 0.0, 0.0));
        assert!(!attack_reach_is_usable(true, false, 0.0, 0.526_801));
        assert!(attack_reach_is_usable(true, false, 20.0, 0.526_801));
    }

    #[test]
    fn gamepad_look_keeps_horizontal_and_reverses_vertical_input() {
        let scale = Vec3::from_array(
            TacticalCombatConfig::default()
                .client_input
                .gamepad_look_scale,
        );
        assert!(scale.x.is_sign_positive());
        assert!(scale.y.is_sign_negative());
    }

    #[test]
    fn attack_origin_is_two_thirds_of_body_height_above_collider_base() {
        let collider = Collider::cylinder(0.4, 1.8);
        let transform = Transform::from_xyz(2.0, 5.9, -3.0);
        let dimensions = CharacterDimensions {
            body_height_metres: 1.8,
            ..default()
        };

        assert_eq!(
            attack_origin(&transform, &collider, dimensions),
            Vec3::new(2.0, 6.2, -3.0)
        );
    }

    #[test]
    fn close_fist_ray_hits_collider_surface_before_out_of_range_centerline() {
        let dimensions = CharacterDimensions::default();
        let attacker_collider = Collider::cylinder(0.4, dimensions.body_height_metres);
        let attacker = Transform::from_xyz(0.0, dimensions.body_height_metres * 0.5, 0.0);
        let origin = attack_origin(&attacker, &attacker_collider, dimensions);
        let target_collider = Collider::cylinder(0.4, dimensions.body_height_metres);
        let target_position = Vec3::new(0.9, dimensions.body_height_metres * 0.5, 0.0);

        assert!(origin.distance(target_position) > dimensions.arm_reach_metres);
        let hit = target_collider.cast_ray(
            target_position,
            Rotation::default(),
            origin,
            Vec3::X,
            dimensions.arm_reach_metres,
            true,
        );
        let (surface_distance, _) = hit.expect("50 cm collider surface is within fist reach");
        assert!((surface_distance - 0.5).abs() < 1.0e-5);
    }

    #[test]
    fn attack_target_threshold_scales_with_surface_distance() {
        assert_eq!(attack_target_angular_threshold_degrees(0.0), 180.0);
        assert_eq!(attack_target_angular_threshold_degrees(0.5), 180.0);
        assert_eq!(attack_target_angular_threshold_degrees(1.0), 90.0);
        assert_eq!(attack_target_angular_threshold_degrees(2.0), 45.0);
        assert!((attack_target_angular_threshold_degrees(1_000.0) - 0.09).abs() < 1.0e-6);
    }

    #[test]
    fn attack_target_threshold_accepts_wide_nearby_and_narrow_distant_aim() {
        assert!(attack_target_within_angular_threshold(
            Vec3::NEG_Z,
            Vec3::Z,
            0.0,
        ));
        assert!(attack_target_within_angular_threshold(
            Vec3::NEG_Z,
            Vec3::X,
            1.0,
        ));
        assert!(attack_target_within_angular_threshold(
            Vec3::NEG_Z,
            Vec3::new(1.0, 0.0, -1.0),
            2.0,
        ));
        assert!(!attack_target_within_angular_threshold(
            Vec3::NEG_Z,
            Quat::from_rotation_y(0.1_f32.to_radians()) * Vec3::NEG_Z,
            1_000.0,
        ));
    }
}
