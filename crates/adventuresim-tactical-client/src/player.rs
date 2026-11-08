use adventuresim_tactical_core::animation::{
    AnimationElapsed, AnimationSampleSpan, AnimationSampleTick, AnimationValueResult,
};
use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_netcode::{
    bevy_replicon::prelude::ClientTriggerExt,
    client::{DirectControlState, WeaponGuardInputState},
    message::{DefendRequest, MeleeActionRequest, RangedActionRequest},
};
use bevy::prelude::*;
use fabelgeist_geometry::SignedLength;
use adventuresim_building_generator::spatial_geometry::Position;
use adventuresim_tactical_core::scene_coordinates::Scene;

use crate::{
    animation::fallback::spawn_fallback_t_pose,
    presentation::{GrassInteractor, TacticalGameplayCamera},
};

use quickstep_trace::{LocalQuickstepTrace, trace_local_quickstep_state};

use action_clock::{
    MeleeAcquisitionTiming, MeleePlaybackTiming, animation_ticks, on_attack_fired_hook,
};

use acquisition::{AttackAcquisitionRange, AttackTarget, CombatTargeting};
use attack_start::{flush_buffered_melee_attacks, try_start_attack};
pub use contact::HitPerformed;
use contact::{PredictedAttackContact, PredictedContact};

mod acquisition;
mod action_clock;
mod attack_start;
mod contact;
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

#[derive(Component, Clone, Copy)]
pub struct LimbHitbox(pub BodyPart);

#[derive(Component)]
pub struct AttackState {
    pub pre_hit_timer: Timer,
    facing_timer: Timer,
    pub reach: SignedLength,
    pub ranged: bool,
    target: Option<AttackTarget>,
    target_position: Option<Position<Scene>>,
    aim_direction: Dir3,
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
        reach: SignedLength,
        ranged: bool,
        target: Option<AttackTarget>,
        target_position: Option<Position<Scene>>,
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

fn attack_origin(
    transform: &Transform,
    collider: &Collider,
    dimensions: CharacterDimensions,
) -> MeleeGeometryResult<Position<Scene>> {
    let body_base = collider
        .aabb(transform.translation, Rotation(transform.rotation))
        .min
        .y;
    Ok(Position::from_metres(transform
        .translation
        .with_y(body_base + dimensions.body_height_metres.metres() * (2.0 / 3.0)))?)
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
            let desired_forward = target_position.metres() - transform.translation;
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
    for (attacker, attacker_transform, attacker_collider, dimensions, mut state) in &mut q_attacker {
        if let Some(target) = state.target
            && let Ok((transform, _, _)) = q_collider.get(target.hitbox)
        {
            match Position::from_metres(transform.transform_point(target.local_strike_point.metres())) {
                Ok(position) => {
                    state.target_position = Some(position);
                }
                Err(error) => {
                    warn!(%error, "cannot observe predicted attack target point");
                    cmd.entity(attacker).remove::<AttackState>();
                    continue;
                }
            }
        }
        state.pre_hit_timer.tick(time.delta());
        state.facing_timer.tick(time.delta());
        if !state.pre_hit_timer.is_finished() {
            continue;
        }
        cmd.entity(attacker).remove::<AttackState>();
        let reach = if state.ranged {
            Ok(state.reach)
        } else {
            melee_interaction_range(dimensions.arm_reach_metres, state.reach)
        };
        let reach = match reach {
            Ok(reach) => reach,
            Err(error) => {
                warn!(%error, "cannot predict attack interaction range");
                continue;
            }
        };
        let contact = attack_origin(attacker_transform, attacker_collider, *dimensions)
            .and_then(|origin| PredictedAttackContact::observe(
                attacker, origin, reach, &state, &spatial, &q_collider, &q_scene_items,
            ));
        let contact = match contact {
            Ok(contact) => contact,
            Err(error) => {
                warn!(%error, "cannot observe predicted attack contact");
                continue;
            }
        };
        if state.ranged {
            cmd.client_trigger(match contact.result {
                PredictedContact::Hit { body, body_part } => RangedActionRequest::CompleteHit {
                    target: body,
                    body_part,
                    reported_precision: combat_config.targeting.reported_hit_precision,
                },
                PredictedContact::Miss => RangedActionRequest::CompleteMiss,
            });
        }
        cmd.trigger(contact.ray);
    }
}

#[derive(Debug, Clone, Copy)]
struct AttackStartRequest {
    entity: Entity,
    alternate_attack: bool,
    hand: AttackHand,
}

fn attack_reach_is_usable(ranged: bool, melee: bool, weapon_reach: SignedLength, arm_reach: SignedLength) -> bool {
    if ranged {
        weapon_reach.metres() > 0.0
    } else {
        melee && arm_reach.metres() > 0.0
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
        if controls.attack_just_pressed
            && let Err(error) = try_start_attack(
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
            )
        {
            warn!(?entity, %error, "cannot schedule attack");
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
            SignedLength::from_metres(0.5).unwrap(),
            false,
            None,
            Some(Position::from_metres(Vec3::NEG_Z).unwrap()),
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
    fn fists_are_usable_from_anatomy_while_ranged_attacks_require_weapon_reach() {
        assert!(attack_reach_is_usable(false, true, SignedLength::from_metres(0.0).unwrap(), SignedLength::from_metres(0.526_801).unwrap()));
        assert!(attack_reach_is_usable(false, true, SignedLength::from_metres(0.8).unwrap(), SignedLength::from_metres(0.526_801).unwrap()));
        assert!(!attack_reach_is_usable(false, true, SignedLength::from_metres(0.0).unwrap(), SignedLength::from_metres(0.0).unwrap()));
        assert!(!attack_reach_is_usable(true, false, SignedLength::from_metres(0.0).unwrap(), SignedLength::from_metres(0.526_801).unwrap()));
        assert!(attack_reach_is_usable(true, false, SignedLength::from_metres(20.0).unwrap(), SignedLength::from_metres(0.526_801).unwrap()));
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
            body_height_metres: SignedLength::from_metres(1.8).unwrap(),
            ..default()
        };

        assert_eq!(
            attack_origin(&transform, &collider, dimensions).unwrap().metres(),
            Vec3::new(2.0, 6.2, -3.0)
        );
    }

    #[test]
    fn close_fist_ray_hits_collider_surface_before_out_of_range_centerline() {
        let dimensions = CharacterDimensions::default();
        let attacker_collider = Collider::cylinder(0.4, dimensions.body_height_metres.metres());
        let attacker = Transform::from_xyz(0.0, dimensions.body_height_metres.metres() * 0.5, 0.0);
        let origin = attack_origin(&attacker, &attacker_collider, dimensions).unwrap().metres();
        let target_collider = Collider::cylinder(0.4, dimensions.body_height_metres.metres());
        let target_position = Vec3::new(0.9, dimensions.body_height_metres.metres() * 0.5, 0.0);

        assert!(origin.distance(target_position) > dimensions.arm_reach_metres.metres());
        let hit = target_collider.cast_ray(
            target_position,
            Rotation::default(),
            origin,
            Vec3::X,
            dimensions.arm_reach_metres.metres(),
            true,
        );
        let (surface_distance, _) = hit.expect("50 cm collider surface is within fist reach");
        assert!((surface_distance - 0.5).abs() < 1.0e-5);
    }


}
