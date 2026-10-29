use super::fallback::spawn_fallback_t_pose;
use super::*;

#[cfg(test)]
mod legacy_tests {
    use super::*;

    const LEFT_PASSING_PHASE: f32 = 0.25;
    const RIGHT_CONTACT_PHASE: f32 = 0.5;
    const RIGHT_PASSING_PHASE: f32 = 0.75;
    use adventuresim_tactical_core::animation::AnimationSampleTick;
    use semantic_route::{SemanticRoutePath, SemanticRouteTrace};

    static TEST_PACK: std::sync::LazyLock<AnimationPackId> =
        std::sync::LazyLock::new(|| AnimationPackId::from("test"));
    static ARMORED_PACK: std::sync::LazyLock<AnimationPackId> =
        std::sync::LazyLock::new(|| AnimationPackId::from("armored"));
    static FIRST_MOTION: std::sync::LazyLock<MotionId> =
        std::sync::LazyLock::new(|| MotionId::try_from("one").unwrap());
    static SECOND_MOTION: std::sync::LazyLock<MotionId> =
        std::sync::LazyLock::new(|| MotionId::try_from("two").unwrap());
    static IDLE_MOTION: std::sync::LazyLock<MotionId> =
        std::sync::LazyLock::new(|| MotionId::try_from("idle").unwrap());
    const TEST_ASSET_DIRECTORY: &str = "animations/test";
    const ARMORED_ASSET_DIRECTORY: &str = "animations/armored";
    const EXPECTED_WALK_GLB: &str = "animations/biped/unarmed/walk.glb";
    const EXPECTED_TWO_HANDED_SWING_GLB: &str = "animations/biped/2h_close/swing.glb";
    const EXPECTED_ARMORED_IDLE_GLB: &str = "animations/armored/idle.glb";

    fn route(skeleton: SkeletonState) -> AnimationValueResult<SemanticRouteTrace> {
        semantic_route::route_semantic_pose(&PresentedSkeleton::new(skeleton, None))
    }

    #[test]
    fn semantic_router_preserves_ordinary_locomotion_evaluation() {
        let skeleton = SkeletonState::default()
            .with_local_velocity(Vec3::NEG_Z * 2.0)
            .with_world_velocity(Vec3::NEG_Z * 2.0);
        let before = AnimationEvaluation::from_skeleton(&skeleton).unwrap();
        let after = route(skeleton);
        assert_eq!(
            after.as_ref().unwrap().path,
            SemanticRoutePath::OrdinaryLocomotion
        );
        assert!(after.is_ok());
        assert_eq!(before, after.as_ref().unwrap().evaluation);
    }

    #[test]
    fn semantic_router_inputs_are_read_only_and_keep_live_attack_movement() {
        let mut skeleton = SkeletonState::default()
            .with_weapon_guard(WeaponGuardState::Raised)
            .with_local_velocity(Vec3::NEG_Z * 3.0)
            .with_world_velocity(Vec3::NEG_Z * 3.0);
        skeleton
            .begin_attack(
                AttackSpec::new(AttackAnimation::Swing),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(10u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(20u64),
            )
            .unwrap();
        skeleton
            .advance_action(adventuresim_tactical_core::animation::AnimationSampleTick::from(15u64))
            .unwrap();
        let before = serde_json::to_vec(&skeleton).unwrap();
        let presented = PresentedSkeleton::new(skeleton, None);
        let evaluation = AnimationEvaluation::from_skeleton(&presented).unwrap();
        let inputs = semantic_route::SemanticRouteInputs::from_presented(&presented, &evaluation);

        assert_eq!(inputs.action, SkeletonAction::Attack);
        assert_eq!(inputs.direction, Vec2::NEG_Y);
        assert_eq!(inputs.speed.metres_per_second(), 3.0);
        assert_eq!(presented.attack_animation(), Some(AttackAnimation::Swing));
        assert_eq!(before, serde_json::to_vec(&presented.state).unwrap());
    }

    #[test]
    fn semantic_router_preserves_raised_attack_contact_timing() {
        for (tick, expected_phase) in [(10, 0.0), (15, 0.25), (20, 0.5), (25, 0.75), (30, 1.0)] {
            let mut skeleton = SkeletonState::default()
                .with_weapon_guard(WeaponGuardState::Raised)
                .with_lead_foot(LeadFoot::Left);
            skeleton
                .begin_attack(
                    AttackSpec::default(),
                    adventuresim_tactical_core::animation::AnimationSampleTick::from(10u64),
                    adventuresim_tactical_core::animation::AnimationSampleTick::from(20u64),
                )
                .unwrap();
            skeleton
                .advance_action(
                    adventuresim_tactical_core::animation::AnimationSampleTick::from(tick as u64),
                )
                .unwrap();
            let presented = PresentedSkeleton::new(skeleton.clone(), None);
            let direct = AnimationEvaluation::from_skeleton(&skeleton).unwrap();
            let routed = route(skeleton);

            assert_eq!(
                routed.as_ref().unwrap().path,
                SemanticRoutePath::RaisedGuardAttack
            );
            assert!(routed.is_ok());
            assert_eq!(routed.as_ref().unwrap().evaluation, direct);
            assert_eq!(
                routed.as_ref().unwrap().inputs.gait_phase.cycles(),
                presented.gait_phase.cycles()
            );
            assert_eq!(
                routed.as_ref().unwrap().evaluation.action_phase.fraction(),
                expected_phase
            );
        }
    }

    #[test]
    fn asset_validation_resolves_deterministic_routes() {
        let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("assets");
        let report = catalog::validate_editor_asset_root(&asset_root).unwrap();
        assert!(report.motion_count > 0);
        assert!(report.missing_motion_count > 0);
        assert_eq!(report.missing_motion_count, report.warnings.len());
        assert!(
            report
                .route_resolutions
                .iter()
                .any(|resolution| resolution.route == "ordinary_locomotion")
        );
        assert!(
            report
                .route_resolutions
                .iter()
                .any(|resolution| resolution.route == "raised_guard_attack")
        );
    }
    #[test]
    fn runtime_catalog_registers_both_downed_gait_mirror_endpoints() {
        let catalog = catalog::AnimationPackCatalog::default();
        let pack = catalog
            .packs
            .get(&AnimationPackId::HUMANOID_UNARMED)
            .unwrap();
        assert!(pack.motions.contains_key(&MotionId::PRONE_CRAWL_MIRRORED));
        assert!(
            pack.motions
                .contains_key(&MotionId::SUPINE_SCAMPER_MIRRORED)
        );
    }

    #[test]
    fn terrain_ik_defaults_on() {
        assert_eq!(TerrainIkMode::default(), TerrainIkMode::Enabled);
    }

    #[test]
    fn presentation_phase_advances_between_sparse_authoritative_samples() {
        let mut authoritative = SkeletonState::default()
            .with_local_velocity(Vec3::NEG_Z * 5.5)
            .with_world_velocity(Vec3::NEG_Z * 5.5);
        let mut presented = PresentedSkeleton::new(authoritative.clone(), None);
        let mut replicated = authoritative.clone();
        let mut previous_phase = presented.gait_phase;
        let mut largest_step = 0.0_f32;
        let mut advanced_without_sample = false;

        for tick in 1..=64 {
            project_skeleton_locomotion(
                &mut authoritative,
                SkeletonLocomotionInput::from_controller(
                    Quat::IDENTITY,
                    Vec3::NEG_Z * 5.5,
                    GroundContact::Grounded,
                    locomotion_sample_rate().interval().elapsed(),
                    AnimationSampleTick::from(tick),
                )
                .unwrap(),
            )
            .unwrap();
            if tick % 4 == 0 {
                replicated = authoritative.clone();
            }
            advance_presented_skeleton(
                &mut presented,
                &replicated,
                locomotion_sample_rate().interval().elapsed(),
            )
            .unwrap();
            let step = circular_phase_delta(previous_phase, presented.gait_phase)
                .unwrap()
                .cycles()
                .abs();
            largest_step = largest_step.max(step);
            if tick % 4 != 0 && step > 0.001 {
                advanced_without_sample = true;
            }
            previous_phase = presented.gait_phase;
        }

        assert!(advanced_without_sample);
        assert!(largest_step < 0.06, "largest phase step was {largest_step}");
        assert!(
            circular_phase_delta(presented.gait_phase, authoritative.gait_phase)
                .unwrap()
                .cycles()
                .abs()
                < 0.08
        );
    }

    #[test]
    fn prone_presentation_does_not_predict_locomotion_from_residual_velocity() {
        let velocity = Vec3::NEG_Z * 2.0;
        let mut authoritative = SkeletonState::default()
            .with_body_state(BodyState::Prone)
            .with_local_velocity(velocity)
            .with_world_velocity(velocity);
        let mut presented = PresentedSkeleton::new(authoritative.clone(), None);

        project_skeleton_locomotion(
            &mut authoritative,
            SkeletonLocomotionInput::from_controller(
                Quat::IDENTITY,
                velocity,
                GroundContact::Grounded,
                locomotion_sample_rate().interval().elapsed(),
                AnimationSampleTick::from(1u64),
            )
            .unwrap(),
        )
        .unwrap();
        advance_presented_skeleton(
            &mut presented,
            &authoritative,
            locomotion_sample_rate().interval().elapsed(),
        )
        .unwrap();

        assert!(
            circular_phase_delta(presented.gait_phase, authoritative.gait_phase)
                .unwrap()
                .cycles()
                .abs()
                < 0.000_01
        );
        assert_eq!(presented.last_phase_correction_delta.cycles(), 0.0);
        assert_eq!(presented.phase_error_remaining.cycles(), 0.0);
    }

    #[test]
    fn presentation_phase_correction_is_rate_limited_across_packet_jitter() {
        let velocity = Vec3::NEG_Z * 5.5;
        let authoritative = SkeletonState::default()
            .with_local_velocity(velocity)
            .with_world_velocity(velocity)
            .with_gait_phase(CyclePhase::from_cycles(0.12).unwrap())
            .with_locomotion_sample_tick(
                adventuresim_tactical_core::animation::AnimationSampleTick::from(1u64),
            );
        let mut presented = PresentedSkeleton::new(
            SkeletonState::default()
                .with_local_velocity(velocity)
                .with_world_velocity(velocity),
            None,
        );
        let render_deltas = [1.0 / 60.0, 1.0 / 90.0, 1.0 / 45.0, 1.0 / 72.0];
        let mut previous_phase = presented.gait_phase;

        for delta_seconds in render_deltas.into_iter().cycle().take(16) {
            advance_presented_skeleton(
                &mut presented,
                &authoritative,
                AnimationElapsed::from_seconds(delta_seconds).unwrap(),
            )
            .unwrap();
            let actual = circular_phase_delta(previous_phase, presented.gait_phase)
                .unwrap()
                .cycles();
            let predicted = gait_cycle_phase_delta(
                locomotion_profile(&presented.state, presented.animation_speed().unwrap()).unwrap(),
                presented.animation_speed().unwrap(),
                AnimationElapsed::from_seconds(delta_seconds).unwrap(),
            )
            .unwrap()
            .cycles();
            let maximum = predicted
                + playback_tuning()
                    .phase_correction_rate_per_second
                    .phase_units_per_second()
                    * delta_seconds
                + 0.000_01;
            assert!(actual >= 0.0, "phase moved backwards by {actual}");
            assert!(actual <= maximum, "phase step {actual} exceeded {maximum}");
            previous_phase = presented.gait_phase;
        }
    }

    #[test]
    fn presentation_phase_ignores_minor_authoritative_packet_jitter() {
        const JITTER_OBSERVATION_COUNT: u64 = 32;
        let velocity = Vec3::NEG_Z * 5.5;
        let mut authoritative = SkeletonState::default()
            .with_local_velocity(velocity)
            .with_world_velocity(velocity);
        let mut presented = PresentedSkeleton::new(authoritative.clone(), None);
        let delta_seconds = 1.0 / 60.0;

        for tick in 1u64..=JITTER_OBSERVATION_COUNT {
            let previous_phase = presented.gait_phase;
            let predicted_delta = gait_cycle_phase_delta(
                locomotion_profile(&presented.state, presented.animation_speed().unwrap()).unwrap(),
                presented.animation_speed().unwrap(),
                AnimationElapsed::from_seconds(delta_seconds).unwrap(),
            )
            .unwrap()
            .cycles();
            let jitter = if tick % 2 == 0 { 0.03 } else { -0.03 };
            authoritative.gait_phase = CyclePhase::from_cycles(
                (previous_phase.cycles() + predicted_delta + jitter).rem_euclid(1.0),
            )
            .unwrap();
            authoritative.locomotion_sample_tick = AnimationSampleTick::from(tick);

            advance_presented_skeleton(
                &mut presented,
                &authoritative,
                AnimationElapsed::from_seconds(delta_seconds).unwrap(),
            )
            .unwrap();

            let actual_delta = circular_phase_delta(previous_phase, presented.gait_phase)
                .unwrap()
                .cycles();
            assert!((actual_delta - predicted_delta).abs() < 0.000_01);
            assert_eq!(presented.last_phase_correction_delta.cycles(), 0.0);
            assert_eq!(presented.phase_error_remaining.cycles(), 0.0);
            assert!(presented.last_phase_source_changed);
            assert!(presented.last_phase_measurement_error.is_some());
        }
    }

    #[test]
    fn presentation_phase_filters_persistent_drift_before_bounded_correction() {
        let velocity = Vec3::NEG_Z * 5.5;
        let mut authoritative = SkeletonState::default()
            .with_local_velocity(velocity)
            .with_world_velocity(velocity);
        let mut presented = PresentedSkeleton::new(authoritative.clone(), None);
        let delta_seconds = 1.0 / 60.0;
        let maximum_correction = playback_tuning()
            .phase_correction_rate_per_second
            .phase_units_per_second()
            * delta_seconds;

        for tick in 1u64..=8 {
            let predicted_delta = gait_cycle_phase_delta(
                locomotion_profile(&presented.state, presented.animation_speed().unwrap()).unwrap(),
                presented.animation_speed().unwrap(),
                AnimationElapsed::from_seconds(delta_seconds).unwrap(),
            )
            .unwrap()
            .cycles();
            authoritative.gait_phase = CyclePhase::from_cycles(
                (presented.gait_phase.cycles() + predicted_delta + 0.10).rem_euclid(1.0),
            )
            .unwrap();
            authoritative.locomotion_sample_tick = AnimationSampleTick::from(tick);

            advance_presented_skeleton(
                &mut presented,
                &authoritative,
                AnimationElapsed::from_seconds(delta_seconds).unwrap(),
            )
            .unwrap();

            assert!(presented.last_phase_correction_delta.cycles() > 0.0);
            assert!(
                presented.last_phase_correction_delta.cycles() <= maximum_correction + 0.000_01
            );
            let measured = presented
                .last_phase_measurement_error
                .expect("new authoritative sample should be measured");
            assert!((measured.cycles() - 0.10).abs() < 0.000_01);
            assert!(
                presented.phase_error_remaining.cycles()
                    < 0.10 - playback_tuning().phase_drift_deadband.phase_units()
            );
        }
    }

    #[test]
    fn presentation_velocity_smooths_a_sparse_turn_without_changing_authority() {
        let authoritative = SkeletonState::default()
            .with_local_velocity(Vec3::NEG_Z * 5.5)
            .with_world_velocity(Vec3::NEG_Z * 5.5);
        let mut presented = PresentedSkeleton::new(authoritative.clone(), None);
        let turned = authoritative
            .clone()
            .with_local_velocity(Vec3::X * 5.5)
            .with_world_velocity(Vec3::X * 5.5)
            .with_locomotion_sample_tick(
                adventuresim_tactical_core::animation::AnimationSampleTick::from(4u64),
            )
            .with_gait_phase(CyclePhase::from_cycles(0.1).unwrap());

        advance_presented_skeleton(
            &mut presented,
            &turned,
            locomotion_sample_rate().interval().elapsed(),
        )
        .unwrap();

        assert_eq!(turned.local_velocity, Vec3::X * 5.5);
        assert!(presented.local_velocity.x > 0.0);
        assert!(presented.local_velocity.x < 5.5);
        assert!(presented.local_velocity.z < 0.0);
    }

    fn spawn_test_t_pose(
        In(owner): In<Entity>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        commands.entity(owner).with_children(|parent| {
            spawn_fallback_t_pose(parent, owner, Color::WHITE, &mut meshes, &mut materials);
        });
    }

    #[test]
    fn default_catalog_owns_all_required_poses_once() {
        let catalog = AnimationPackCatalog::default();
        let root = &catalog.packs[&AnimationPackId::HUMANOID_UNARMED];
        let mut library = AnimationPackLibrary::default();
        library
            .insert(AnimationPack {
                id: AnimationPackId::HUMANOID_UNARMED.to_owned(),
                skeleton_family: root.skeleton_family.clone(),
                fallback: root.fallback.clone(),
                clips: root.poses.keys().copied().collect(),
            })
            .unwrap();
        for required in SemanticPose::HUMANOID_REQUIRED {
            assert!(
                matches!(
                    library.resolve(&AnimationPackId::HUMANOID_UNARMED, required),
                    ResolvedPose::Clip { semantic, .. } if semantic == required
                ),
                "required pose {required:?} did not resolve"
            );
        }
        // The 30 required semantics collapse to 29 independently resolvable
        // variants when the supported whole-body mirror pair appears once.
        let authored_variants = SemanticPose::HUMANOID_REQUIRED
            .into_iter()
            .filter(|pose| {
                pose.mirrored_counterpart()
                    .is_none_or(|counterpart| pose.as_str() < counterpart.as_str())
            })
            .count();
        assert_eq!(authored_variants, 29);
        assert_eq!(
            root.motions[&MotionId::WALK].path.as_ref(),
            EXPECTED_WALK_GLB
        );
        assert_eq!(
            root.poses[&SemanticPose::WalkPassing],
            PoseAnchor {
                motion: MotionId::WALK,
                frame: AuthoredAnimationFrame::from(16u16),
            }
        );
        assert_eq!(
            root.poses[&SemanticPose::AttackThrust],
            PoseAnchor {
                motion: MotionId::THRUST,
                frame: AuthoredAnimationFrame::from(4u16),
            }
        );
        assert_eq!(
            root.poses[&SemanticPose::DiveForward],
            PoseAnchor {
                motion: MotionId::DIVE,
                frame: AuthoredAnimationFrame::from(0u16),
            }
        );
        for pose in [
            SemanticPose::DiveBackward,
            SemanticPose::DiveLeft,
            SemanticPose::DiveRight,
        ] {
            assert_eq!(root.poses[&pose].motion, MotionId::DIVE);
        }
        assert_eq!(SemanticPose::DiveRight.mirrored_counterpart(), None);
        assert_eq!(
            root.poses[&SemanticPose::GuardSwing].frame,
            AuthoredAnimationFrame::from(0u16)
        );
        assert_eq!(
            root.poses[&SemanticPose::GuardThrust].frame,
            AuthoredAnimationFrame::from(0u16)
        );
        assert_eq!(
            root.poses[&SemanticPose::AttackOffhand].frame,
            AuthoredAnimationFrame::from(0u16)
        );
        assert_eq!(
            root.motions[&MotionId::SWING].last_frame,
            AuthoredAnimationFrame::from(12u16)
        );
        assert_eq!(
            root.motions[&MotionId::SWING].required_last_frame,
            AuthoredAnimationFrame::from(4u16)
        );
        assert_eq!(
            root.motions[&MotionId::THRUST].last_frame,
            AuthoredAnimationFrame::from(12u16)
        );
        assert_eq!(
            root.motions[&MotionId::THRUST].required_last_frame,
            AuthoredAnimationFrame::from(4u16)
        );
        assert_eq!(
            root.motions[&MotionId::OFFHAND].last_frame,
            AuthoredAnimationFrame::from(4u16)
        );
        assert_eq!(
            root.motions[&MotionId::OFFHAND].required_last_frame,
            AuthoredAnimationFrame::from(0u16)
        );
        assert_eq!(
            root.motions[&MotionId::STRAFE].last_frame,
            AuthoredAnimationFrame::from(24u16)
        );
        assert_eq!(
            root.motions[&MotionId::SKIP].last_frame,
            AuthoredAnimationFrame::from(24u16)
        );
        assert_eq!(
            root.poses[&SemanticPose::StrafeCycle].frame,
            AuthoredAnimationFrame::from(0u16)
        );
        assert_eq!(
            root.poses[&SemanticPose::SkipCycle].frame,
            AuthoredAnimationFrame::from(0u16)
        );
        for motion in [
            MotionId::QUICKSTEP_FORWARD,
            MotionId::QUICKSTEP_RIGHT,
            MotionId::QUICKSTEP_LEFT,
            MotionId::QUICKSTEP_BACK,
        ] {
            assert_eq!(
                root.motions[&motion].last_frame,
                AuthoredAnimationFrame::from(12u16)
            );
        }
    }

    #[test]
    fn two_handed_close_pack_overrides_attacks_and_inherits_other_poses() {
        let catalog = AnimationPackCatalog::default();
        let pack = &catalog.packs[&AnimationPackId::HUMANOID_2H_CLOSE];
        assert_eq!(
            pack.fallback.as_ref(),
            Some(&AnimationPackId::HUMANOID_UNARMED)
        );
        assert_eq!(
            pack.motions[&MotionId::SWING].path.as_ref(),
            EXPECTED_TWO_HANDED_SWING_GLB
        );
        assert!(pack.poses.contains_key(&SemanticPose::AttackSwing));
        assert!(pack.poses.contains_key(&SemanticPose::AttackThrust));
        assert!(!pack.poses.contains_key(&SemanticPose::WalkContact));
    }

    #[test]
    fn duplicate_authoritative_pose_is_rejected() {
        let mut builder = PackBuilder::new(
            &TEST_PACK.clone(),
            &SkeletonFamily::HUMANOID,
            None,
            AnimationAssetDirectory::try_from(TEST_ASSET_DIRECTORY).unwrap(),
        );
        builder.motion(&FIRST_MOTION.clone(), AuthoredAnimationFrame::from(0u16));
        builder.motion(&SECOND_MOTION.clone(), AuthoredAnimationFrame::from(0u16));
        builder
            .pose(
                &FIRST_MOTION.clone(),
                AuthoredAnimationFrame::from(0u16),
                SemanticPose::IdleRelaxed,
            )
            .unwrap();
        assert_eq!(
            builder.pose(
                &SECOND_MOTION.clone(),
                AuthoredAnimationFrame::from(0u16),
                SemanticPose::IdleRelaxed
            ),
            Err(CatalogError::DuplicatePose(SemanticPose::IdleRelaxed))
        );
    }

    #[test]
    fn pack_builder_supports_specialized_pack_paths_and_fallbacks() {
        let mut builder = PackBuilder::new(
            &ARMORED_PACK.clone(),
            &SkeletonFamily::HUMANOID,
            Some(&AnimationPackId::HUMANOID_UNARMED),
            AnimationAssetDirectory::try_from(ARMORED_ASSET_DIRECTORY).unwrap(),
        );
        builder.motion(&IDLE_MOTION.clone(), AuthoredAnimationFrame::from(0u16));
        builder
            .pose(
                &IDLE_MOTION.clone(),
                AuthoredAnimationFrame::from(0u16),
                SemanticPose::IdleRelaxed,
            )
            .unwrap();
        let (id, pack) = builder.finish();
        assert_eq!(id, ARMORED_PACK.clone());
        assert_eq!(
            pack.fallback.as_ref(),
            Some(&AnimationPackId::HUMANOID_UNARMED)
        );
        assert_eq!(
            pack.motions[&IDLE_MOTION.clone()].path.as_ref(),
            EXPECTED_ARMORED_IDLE_GLB
        );
    }

    #[test]
    fn frame_eight_is_sampled_at_thirty_fps() {
        assert_eq!(
            frame_seconds(AuthoredAnimationFrame::from(8u16))
                .unwrap()
                .seconds(),
            8.0 / 30.0
        );
    }

    #[test]
    fn sole_clip_accepts_named_or_unnamed_animation_equally() {
        let one = [Handle::<AnimationClip>::default()];
        let two = [
            Handle::<AnimationClip>::default(),
            Handle::<AnimationClip>::default(),
        ];
        assert!(sole_animation(&[]).is_none());
        assert!(sole_animation(&one).is_some());
        assert!(sole_animation(&two).is_none());
    }

    fn runtime_with_available(poses: impl IntoIterator<Item = SemanticPose>) -> AnimationRuntime {
        let catalog = AnimationPackCatalog::default();
        let mut assets = Assets::<AnimationClip>::default();
        let poses = poses.into_iter().collect::<BTreeSet<_>>();
        let mut library = AnimationPackLibrary::default();
        library
            .insert(AnimationPack {
                id: AnimationPackId::HUMANOID_UNARMED.to_owned(),
                skeleton_family: SkeletonFamily::HUMANOID,
                fallback: None,
                clips: poses.clone(),
            })
            .unwrap();
        let mut runtime = AnimationRuntime {
            library,
            ..default()
        };
        for pose in poses {
            let anchor = &catalog.packs[&AnimationPackId::HUMANOID_UNARMED].poses[&pose];
            let key = MotionKey::new(&AnimationPackId::HUMANOID_UNARMED, &anchor.motion);
            if runtime.clips.contains_key(&key) {
                continue;
            }
            runtime.clips.insert(
                key,
                LoadedClip {
                    handle: assets.add(AnimationClip::default()),
                    duration_seconds: ClipDuration::from_seconds(
                        f32::from(u16::from(
                            catalog.packs[&AnimationPackId::HUMANOID_UNARMED].motions
                                [&anchor.motion]
                                .last_frame,
                        )) / animation_frames_per_second().hertz(),
                    )
                    .unwrap(),
                    layer: ClipLayer::Whole,
                },
            );
        }
        runtime
    }

    #[test]
    fn span_endpoints_remain_declared_at_zero_and_complete_progress() {
        let catalog = AnimationPackCatalog::default();
        for (start, end) in [
            (SemanticPose::WalkContact, SemanticPose::WalkPassing),
            (SemanticPose::GuardThrust, SemanticPose::ProneIdle),
        ] {
            let runtime = runtime_with_available([start, end]);
            let mut identity = None;
            for progress in [0.0, 1.0e-10, 0.5, 1.0] {
                let mut weighted = Vec::new();
                append_resolved_sample(
                    &mut weighted,
                    &runtime,
                    &catalog,
                    &AnimationPackId::HUMANOID_UNARMED,
                    PoseSample {
                        pose: start,
                        sampling: PoseSampling::Span {
                            end,
                            progress: UnitInterval::from_fraction(progress).unwrap(),
                        },
                        weight: BlendWeight::from_factor(1.0).unwrap(),
                        mirror_lower_body: ClipMirror::Original,
                    },
                )
                .unwrap();
                assert_eq!(weighted.len(), 2);
                let current = weighted
                    .iter()
                    .map(|clip| (clip.clip.handle.id(), clip.clip.layer))
                    .collect::<Vec<_>>();
                assert_eq!(identity.get_or_insert_with(|| current.clone()), &current);
                assert_eq!(
                    weighted
                        .iter()
                        .map(|clip| clip.weight.factor())
                        .sum::<f32>(),
                    1.0
                );
                if progress == 0.0 || progress == 1.0 {
                    assert_eq!(
                        weighted
                            .iter()
                            .filter(|clip| clip.weight.factor() == 0.0)
                            .count(),
                        1
                    );
                }
            }
        }
    }

    #[test]
    fn same_motion_span_blends_only_the_two_exact_anchor_frames() {
        let catalog = AnimationPackCatalog::default();
        let runtime =
            runtime_with_available([SemanticPose::WalkContact, SemanticPose::WalkPassing]);
        let mut weighted = Vec::new();
        append_resolved_sample(
            &mut weighted,
            &runtime,
            &catalog,
            &AnimationPackId::HUMANOID_UNARMED,
            PoseSample {
                pose: SemanticPose::WalkContact,
                sampling: PoseSampling::Span {
                    end: SemanticPose::WalkPassing,
                    progress: UnitInterval::from_fraction(0.5).unwrap(),
                },
                weight: BlendWeight::from_factor(1.0).unwrap(),
                mirror_lower_body: ClipMirror::Original,
            },
        )
        .unwrap();
        assert_eq!(weighted.len(), 2);
        assert!(weighted.iter().any(|sample| {
            sample.time_seconds.seconds() == 0.0 && (sample.weight.factor() - 0.5).abs() < 0.0001
        }));
        assert!(weighted.iter().any(|sample| {
            (sample.time_seconds.seconds() - 16.0 / animation_frames_per_second().hertz()).abs()
                < 0.0001
                && (sample.weight.factor() - 0.5).abs() < 0.0001
        }));
    }

    #[test]
    fn attack_entry_blends_frame_zero_guard_and_contact_in_one_motion() {
        let catalog = AnimationPackCatalog::default();
        let runtime =
            runtime_with_available([SemanticPose::GuardThrust, SemanticPose::AttackThrust]);
        let mut weighted = Vec::new();
        append_resolved_sample(
            &mut weighted,
            &runtime,
            &catalog,
            &AnimationPackId::HUMANOID_UNARMED,
            PoseSample {
                pose: SemanticPose::GuardThrust,
                sampling: PoseSampling::Span {
                    end: SemanticPose::AttackThrust,
                    progress: UnitInterval::from_fraction(0.5).unwrap(),
                },
                weight: BlendWeight::from_factor(1.0).unwrap(),
                mirror_lower_body: ClipMirror::Original,
            },
        )
        .unwrap();
        assert_eq!(weighted.len(), 2);
        let thrust = runtime.clips
            [&MotionKey::new(&AnimationPackId::HUMANOID_UNARMED, &MotionId::THRUST)]
            .handle
            .id();
        assert!(
            weighted
                .iter()
                .all(|sample| sample.clip.handle.id() == thrust)
        );
        assert!(
            weighted
                .iter()
                .any(|sample| sample.time_seconds.seconds() == 0.0
                    && (sample.weight.factor() - 0.5).abs() < 0.0001)
        );
        assert!(weighted.iter().any(|sample| {
            (sample.time_seconds.seconds() - 4.0 / animation_frames_per_second().hertz()).abs()
                < 0.0001
                && (sample.weight.factor() - 0.5).abs() < 0.0001
        }));
    }

    #[test]
    fn mirrored_gait_endpoint_uses_a_distinct_pre_fk_clip_node() {
        let catalog = AnimationPackCatalog::default();
        let mut runtime =
            runtime_with_available([SemanticPose::RunContact, SemanticPose::RunFlight]);
        let mut assets = Assets::<AnimationClip>::default();
        let mirrored_handle = assets.add(AnimationClip::default());
        runtime.clips.insert(
            MotionKey::new(&AnimationPackId::HUMANOID_UNARMED, &MotionId::RUN_MIRRORED),
            LoadedClip {
                handle: mirrored_handle.clone(),
                duration_seconds: ClipDuration::from_seconds(
                    64.0 / animation_frames_per_second().hertz(),
                )
                .unwrap(),
                layer: ClipLayer::Whole,
            },
        );
        let mut weighted = Vec::new();
        append_resolved_sample(
            &mut weighted,
            &runtime,
            &catalog,
            &AnimationPackId::HUMANOID_UNARMED,
            PoseSample {
                pose: SemanticPose::RunContact,
                sampling: PoseSampling::Anchor,
                weight: BlendWeight::from_factor(0.5).unwrap(),
                mirror_lower_body: ClipMirror::Mirrored,
            },
        )
        .unwrap();

        assert_eq!(weighted.len(), 1);
        assert_eq!(weighted[0].clip.handle.id(), mirrored_handle.id());
        assert!((weighted[0].weight.factor() - 0.5).abs() < 0.0001);
    }

    #[test]
    fn zero_clip_base_gets_stable_animation_targets_without_a_player() {
        let mut world = World::new();
        world.init_resource::<AnimationRuntime>();
        let owner = world.spawn_empty().id();
        let rig = world.spawn(AnimationRigScene(owner)).id();
        let skeleton = world.spawn(Name::new("Skeleton")).id();
        let root = world.spawn(Name::new("body_world")).id();
        let pelvis = world.spawn(Name::new("root")).id();
        world.entity_mut(rig).add_child(skeleton);
        world.entity_mut(skeleton).add_child(root);
        world.entity_mut(root).add_child(pelvis);

        world
            .run_system_cached(establish_animation_targets)
            .unwrap();
        world.flush();
        world
            .run_system_cached(establish_animation_targets)
            .unwrap();
        world.flush();

        assert_eq!(
            world.get::<AnimationTargetId>(pelvis),
            Some(&AnimationTargetId::from_names(
                [
                    Name::new("Skeleton"),
                    Name::new("body_world"),
                    Name::new("root")
                ]
                .iter()
            ))
        );
        assert_eq!(
            world.resource::<AnimationRuntime>().canonical_targets.len(),
            3
        );
    }

    #[test]
    fn composite_mask_keeps_mhr_world_pelvis_and_legs_out_of_the_upper_body() {
        for lower in [
            "Skeleton",
            "body_world",
            "root",
            "l_upleg",
            "r_upleg_twist3_proc",
            "l_lowleg",
            "r_foot",
            "l_talocrural",
            "r_subtalar",
            "l_transversetarsal",
            "l_ball",
        ] {
            assert_eq!(
                AnimationBodyRegion::from(&fabelgeist_rig::RigJointName::from(lower)),
                AnimationBodyRegion::Lower,
                "{lower}"
            );
        }
        for upper in [
            "c_spine0",
            "c_spine2",
            "c_spine3",
            "l_clavicle",
            "r_uparm",
            "c_head",
        ] {
            assert_eq!(
                AnimationBodyRegion::from(&fabelgeist_rig::RigJointName::from(upper)),
                AnimationBodyRegion::Upper,
                "{upper}"
            );
        }
    }

    #[test]
    fn database_weapon_skill_weights_select_the_grip_without_item_names() {
        let mut hilt = [0.0; 9];
        hilt[3] = 1.0;
        assert_eq!(weapon_grip(&hilt), WeaponGrip::Hilt);

        let mut hybrid_polearm = [0.0; 9];
        hybrid_polearm[0] = 1.0 / 3.0;
        hybrid_polearm[1] = 1.0 / 3.0;
        hybrid_polearm[2] = 1.0 / 3.0;
        assert_eq!(weapon_grip(&hybrid_polearm), WeaponGrip::Polearm);
    }

    #[test]
    fn weapon_catalog_selects_specific_and_two_handed_fallback_packs() {
        assert_eq!(
            animation_pack_for_weapon("longsword", true),
            AnimationPackId::HUMANOID_2H_CLOSE
        );
        assert_eq!(
            animation_pack_for_weapon("zweihander", true),
            AnimationPackId::HUMANOID_2H_CLOSE
        );
        assert_eq!(
            animation_pack_for_weapon("halberd", true),
            AnimationPackId::HUMANOID_2H_CLOSE
        );
        assert_eq!(
            animation_pack_for_weapon("arming_sword", false),
            AnimationPackId::HUMANOID_UNARMED
        );
    }

    #[test]
    fn two_handed_weapon_only_recruits_an_empty_offhand() {
        assert!(weapon_uses_offhand("longsword", true));
        assert!(!weapon_uses_offhand("longsword", false));
        assert!(!weapon_uses_offhand("arming_sword", true));
        assert_eq!(
            animation_pack_for_weapon("longsword", false),
            AnimationPackId::HUMANOID_UNARMED
        );
        assert_eq!(
            weapon_grip_layers(true, true),
            [None, None],
            "an active two-handed clip owns its authored wrist and finger tracks"
        );
        assert_eq!(
            weapon_grip_layers(false, true),
            [Some(ClipLayer::MainHand), Some(ClipLayer::Offhand)]
        );
        assert_eq!(
            weapon_grip_layers(true, false),
            [Some(ClipLayer::MainHand), None]
        );
        assert!(SemanticPose::GuardSwing.is_main_hand_attack());
        assert!(SemanticPose::ContinueThrust.is_main_hand_attack());
        assert!(!SemanticPose::DiveForward.is_main_hand_attack());
    }

    #[test]
    fn raised_guard_uses_authored_hand_tracks_before_the_attack_starts() {
        let skeleton = SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised);
        let evaluation = AnimationEvaluation::from_skeleton(&skeleton).unwrap();

        assert!(evaluation.action.is_empty());
        assert!(authored_pose_owns_hands(&evaluation.base));
        assert_eq!(weapon_grip_layers(true, true), [None, None]);
    }

    #[test]
    fn analytical_attack_curves_bypass_the_fixed_rate_clip_sampler() {
        let mut playback = AnimationPlayback::default();
        assert_eq!(playback.sampling_cadence(), PoseSamplingCadence::Buffered);

        let clip = LoadedClip {
            handle: Handle::default(),
            duration_seconds: ClipDuration::from_seconds(0.4).unwrap(),
            layer: ClipLayer::Whole,
        };
        playback.plan = PlaybackContributions::admit(
            Vec::new(),
            vec![ExtrapolatedSpan {
                start: clip.clone(),
                start_time_seconds: ClipTime::ZERO,
                end: clip,
                end_time_seconds: ClipTime::from_seconds(
                    4.0 / animation_frames_per_second().hertz(),
                )
                .unwrap(),
                coordinate: AttackPoseCoordinate::from_coordinate(1.0).unwrap(),
                weight: BlendWeight::ONE,
                mirrored_weight: BlendWeight::ZERO,
            }],
            Vec::new(),
        )
        .unwrap();

        assert_eq!(playback.sampling_cadence(), PoseSamplingCadence::Direct);
    }

    #[test]
    fn authored_rig_attaches_to_a_player_with_skeleton_state() {
        let mut world = World::new();
        let runtime = AnimationRuntime {
            base_rig: BaseRigLoad::Ready {
                source: Handle::default(),
                scene: Handle::default(),
            },
            ..default()
        };
        world.insert_resource(runtime);
        let owner = world
            .spawn((Player::default(), SkeletonState::default()))
            .id();

        world.run_system_cached(attach_loaded_rig_scenes).unwrap();
        world.flush();

        assert!(world.get::<AnimationRigAttached>(owner).is_some());
        let rig = world
            .query::<(Entity, &AnimationRigScene)>()
            .iter(&world)
            .find_map(|(entity, scene)| (scene.0 == owner).then_some(entity))
            .expect("client authored rig");
        assert_eq!(
            world.get::<Transform>(rig).unwrap().translation.y,
            player_visual_y_offset().metres()
        );
    }

    #[test]
    fn incompatible_motion_target_set_is_rejected_independently() {
        let root = AnimationTargetId::from_names([Name::new("Skeleton")].iter());
        let pelvis = AnimationTargetId::from_names(
            [
                Name::new("Skeleton"),
                Name::new("body_world"),
                Name::new("root"),
            ]
            .iter(),
        );
        let foreign = AnimationTargetId::from_names([Name::new("OtherRig")].iter());
        let base = HashSet::from([root, pelvis]);
        assert!(targets_match_base([&root, &pelvis].into_iter(), &base));
        assert!(!targets_match_base([&root, &foreign].into_iter(), &base));
        assert!(!targets_match_base([].iter(), &base));
    }

    #[test]
    fn unavailable_motion_uses_similar_pose_fallback() {
        let catalog = AnimationPackCatalog::default();
        let runtime = runtime_with_available([SemanticPose::WalkContact]);
        let mut weighted = Vec::new();
        append_resolved_sample(
            &mut weighted,
            &runtime,
            &catalog,
            &AnimationPackId::HUMANOID_UNARMED,
            PoseSample {
                pose: SemanticPose::RunContact,
                sampling: PoseSampling::Anchor,
                weight: BlendWeight::from_factor(1.0).unwrap(),
                mirror_lower_body: ClipMirror::Original,
            },
        )
        .unwrap();
        assert_eq!(weighted.len(), 1);
        assert!(weighted[0].time_seconds.seconds().abs() < 0.0001);
    }

    #[test]
    fn missing_forward_quickstep_plays_the_authored_back_clip_in_reverse() {
        let catalog = AnimationPackCatalog::default();
        let runtime = runtime_with_available([
            SemanticPose::QuickstepBackTakeoff,
            SemanticPose::QuickstepBackContact,
        ]);
        let mut weighted = Vec::new();
        append_resolved_sample(
            &mut weighted,
            &runtime,
            &catalog,
            &AnimationPackId::HUMANOID_UNARMED,
            PoseSample {
                pose: SemanticPose::QuickstepForwardTakeoff,
                sampling: PoseSampling::Timeline {
                    progress: UnitInterval::from_fraction(0.25).unwrap(),
                },
                weight: BlendWeight::from_factor(1.0).unwrap(),
                mirror_lower_body: ClipMirror::Original,
            },
        )
        .unwrap();

        let back = runtime.clips[&MotionKey::new(
            &AnimationPackId::HUMANOID_UNARMED,
            &MotionId::QUICKSTEP_BACK,
        )]
            .handle
            .id();
        assert_eq!(weighted.len(), 1);
        assert_eq!(weighted[0].clip.handle.id(), back);
        assert!(
            (weighted[0].time_seconds.seconds() - 9.0 / animation_frames_per_second().hertz())
                .abs()
                < 0.0001
        );
        assert!((weighted[0].weight.factor() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn authored_quickstep_direction_wins_over_its_reverse_fallback() {
        let catalog = AnimationPackCatalog::default();
        let runtime = runtime_with_available([
            SemanticPose::QuickstepRightTakeoff,
            SemanticPose::QuickstepRightContact,
            SemanticPose::QuickstepLeftTakeoff,
            SemanticPose::QuickstepLeftContact,
        ]);
        let mut weighted = Vec::new();
        append_resolved_sample(
            &mut weighted,
            &runtime,
            &catalog,
            &AnimationPackId::HUMANOID_UNARMED,
            PoseSample {
                pose: SemanticPose::QuickstepRightTakeoff,
                sampling: PoseSampling::Timeline {
                    progress: UnitInterval::from_fraction(0.5).unwrap(),
                },
                weight: BlendWeight::from_factor(1.0).unwrap(),
                mirror_lower_body: ClipMirror::Original,
            },
        )
        .unwrap();

        let right = runtime.clips[&MotionKey::new(
            &AnimationPackId::HUMANOID_UNARMED,
            &MotionId::QUICKSTEP_RIGHT,
        )]
            .handle
            .id();
        assert_eq!(weighted.len(), 1);
        assert_eq!(weighted[0].clip.handle.id(), right);
        assert!(
            (weighted[0].time_seconds.seconds() - 6.0 / animation_frames_per_second().hertz())
                .abs()
                < 0.0001
        );
    }

    #[test]
    fn timeline_and_cycle_samples_preserve_the_requested_bone_layer() {
        let catalog = AnimationPackCatalog::default();
        let runtime = runtime_with_available([
            SemanticPose::StrafeCycle,
            SemanticPose::QuickstepRightTakeoff,
            SemanticPose::QuickstepRightContact,
        ]);
        let strides = AuthoredLocomotionStrides::default();
        let sample_resolver = PoseSampleResolver {
            runtime: &runtime,
            catalog: &catalog,
            pack: &AnimationPackId::HUMANOID_UNARMED,
            locomotion_strides: &strides,
        };
        for (sample, expected_layer) in [
            (
                PoseSample {
                    pose: SemanticPose::StrafeCycle,
                    sampling: PoseSampling::Cycle {
                        phase: CyclePhase::from_cycles(RIGHT_CONTACT_PHASE).unwrap(),
                    },
                    weight: BlendWeight::from_factor(1.0).unwrap(),
                    mirror_lower_body: ClipMirror::Original,
                },
                ClipLayer::Lower,
            ),
            (
                PoseSample {
                    pose: SemanticPose::QuickstepRightTakeoff,
                    sampling: PoseSampling::Timeline {
                        progress: UnitInterval::from_fraction(0.5).unwrap(),
                    },
                    weight: BlendWeight::from_factor(1.0).unwrap(),
                    mirror_lower_body: ClipMirror::Original,
                },
                ClipLayer::Lower,
            ),
        ] {
            let mut weighted = Vec::new();
            let mut spans = Vec::new();
            let mut continuations = Vec::new();
            sample_resolver
                .append_layer(
                    &mut weighted,
                    &mut spans,
                    &mut continuations,
                    sample,
                    expected_layer,
                )
                .unwrap();
            assert_eq!(weighted.len(), 1);
            assert_eq!(weighted[0].clip.layer, expected_layer);
        }
    }

    #[test]
    fn locomotion_phase_curve_changes_only_authored_cycle_sampling() {
        let catalog = AnimationPackCatalog::default();
        let runtime =
            runtime_with_available([SemanticPose::WalkContact, SemanticPose::StrafeCycle]);
        let mut strides = AuthoredLocomotionStrides::default();
        strides.phase_curves.insert(
            CalibrationMotion::Walk,
            AuthoredPhaseCurve::from_samples(
                vec![0.0, 0.75, 1.0]
                    .into_iter()
                    .map(|phase| CyclePhase::from_cycles(phase).unwrap())
                    .collect(),
            )
            .unwrap(),
        );
        let physical_phase = 0.5;
        let mut weighted = Vec::new();
        let mut spans = Vec::new();
        let mut continuations = Vec::new();
        let sample_resolver = PoseSampleResolver {
            runtime: &runtime,
            catalog: &catalog,
            pack: &AnimationPackId::HUMANOID_UNARMED,
            locomotion_strides: &strides,
        };
        sample_resolver
            .append_layer(
                &mut weighted,
                &mut spans,
                &mut continuations,
                PoseSample {
                    pose: SemanticPose::WalkContact,
                    sampling: PoseSampling::Cycle {
                        phase: CyclePhase::from_cycles(physical_phase).unwrap(),
                    },
                    weight: BlendWeight::from_factor(1.0).unwrap(),
                    mirror_lower_body: ClipMirror::Original,
                },
                ClipLayer::Lower,
            )
            .unwrap();
        let walk_duration = runtime.clips
            [&MotionKey::new(&AnimationPackId::HUMANOID_UNARMED, &MotionId::WALK)]
            .duration_seconds
            .seconds();
        assert!((weighted[0].time_seconds.seconds() - walk_duration * 0.75).abs() < 0.0001);
        assert_eq!(
            weighted[0].locomotion_phase,
            Some(CyclePhase::from_cycles(physical_phase).unwrap())
        );

        weighted.clear();
        sample_resolver
            .append_layer(
                &mut weighted,
                &mut spans,
                &mut continuations,
                PoseSample {
                    pose: SemanticPose::StrafeCycle,
                    sampling: PoseSampling::Cycle {
                        phase: CyclePhase::from_cycles(physical_phase).unwrap(),
                    },
                    weight: BlendWeight::from_factor(1.0).unwrap(),
                    mirror_lower_body: ClipMirror::Original,
                },
                ClipLayer::Lower,
            )
            .unwrap();
        let strafe_duration = runtime.clips
            [&MotionKey::new(&AnimationPackId::HUMANOID_UNARMED, &MotionId::STRAFE)]
            .duration_seconds
            .seconds();
        assert!(
            (weighted[0].time_seconds.seconds() - strafe_duration * physical_phase).abs() < 0.0001
        );
        assert_eq!(weighted[0].locomotion_phase, None);
    }

    #[test]
    fn out_of_range_catalog_frame_is_unavailable() {
        assert!(frame_fits_clip(
            AuthoredAnimationFrame::from(8u16),
            ClipDuration::from_seconds(8.0 / 30.0).unwrap()
        ));
        assert!(!frame_fits_clip(
            AuthoredAnimationFrame::from(20u16),
            ClipDuration::from_seconds(0.1).unwrap()
        ));
    }

    #[test]
    fn missing_base_keeps_generated_mannequin_visible() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        let owner = world.spawn_empty().id();
        world
            .run_system_cached_with(spawn_test_t_pose, owner)
            .unwrap();
        world.flush();
        world.run_system_cached(update_rig_visibility).unwrap();
        let (_, visibility) = world
            .query::<(&FallbackAnimationRig, &Visibility)>()
            .single(&world)
            .unwrap();
        assert_eq!(*visibility, Visibility::Inherited);
    }

    #[test]
    fn authored_zero_motion_rig_hides_mannequin_and_shows_bind_pose() {
        let mut world = World::new();
        let owner = world.spawn(AnimationPlayback::default()).id();
        let fallback = world
            .spawn((FallbackAnimationRig(owner), Visibility::Inherited))
            .id();
        let authored = world
            .spawn((AnimationRigScene(owner), Visibility::Hidden))
            .id();
        world.run_system_cached(update_rig_visibility).unwrap();
        assert_eq!(
            *world.get::<Visibility>(fallback).unwrap(),
            Visibility::Hidden
        );
        assert_eq!(
            *world.get::<Visibility>(authored).unwrap(),
            Visibility::Inherited
        );
    }

    #[test]
    fn unresolved_motion_restores_authored_bind_transform() {
        let mut world = World::new();
        let owner = world.spawn(AnimationPlayback::default()).id();
        let bind = Transform::from_rotation(Quat::from_rotation_x(0.4));
        let node = world
            .spawn((
                AuthoredBindTransform { owner, local: bind },
                Transform::from_rotation(Quat::from_rotation_y(1.2)),
            ))
            .id();
        world.run_system_cached(restore_authored_bind_pose).unwrap();
        assert_eq!(*world.get::<Transform>(node).unwrap(), bind);
    }

    #[test]
    fn client_constraint_api_is_reexported() {
        let _: Option<HandIkTarget> = None;
        let _: HumanoidIkTargets = default();
        let _ = [HandSide::Left, HandSide::Right];
        let constraint = HeldWeaponConstraint {
            owner: Entity::PLACEHOLDER,
            primary_hand: HandSide::Right,
            secondary_grip_local: None,
        };
        assert_eq!(constraint.primary_hand, HandSide::Right);
    }

    #[test]
    fn presentation_sequence_gaps_are_bounded_ordered_and_reset_safe() {
        assert_eq!(
            coalesced_contacts(
                ContactSequence::from(10u64),
                ContactSequence::from(13u64),
                LeadFoot::Right
            ),
            Some(vec![
                CoalescedContact {
                    sequence: ContactSequence::from(11u64),
                    foot: LeadFoot::Right
                },
                CoalescedContact {
                    sequence: ContactSequence::from(12u64),
                    foot: LeadFoot::Left
                },
                CoalescedContact {
                    sequence: ContactSequence::from(13u64),
                    foot: LeadFoot::Right
                },
            ])
        );
        assert_eq!(
            coalesced_contacts(
                ContactSequence::from(13u64),
                ContactSequence::from(13u64),
                LeadFoot::Right
            ),
            Some(vec![])
        );
        assert_eq!(
            coalesced_contacts(
                ContactSequence::from(13u64),
                ContactSequence::from(2u64),
                LeadFoot::Left
            ),
            None
        );
        assert_eq!(
            coalesced_contacts(
                ContactSequence::from(2u64),
                ContactSequence::from(20u64),
                LeadFoot::Left
            ),
            None
        );
        assert_eq!(
            ContactSequence::from(9u64).events_since(ContactSequence::from(7u64)),
            Some(adventuresim_tactical_core::animation::AnimationEventSpan::from(2u64))
        );
        assert_eq!(
            ContactSequence::from(7u64).events_since(ContactSequence::from(9u64)),
            None
        );
        assert_eq!(
            latest_coalesced_landing(
                adventuresim_tactical_core::animation::LandingSequence::from(7u64),
                adventuresim_tactical_core::animation::LandingSequence::from(9u64)
            ),
            Some(adventuresim_tactical_core::animation::LandingSequence::from(9u64))
        );
        assert_eq!(
            latest_coalesced_landing(
                adventuresim_tactical_core::animation::LandingSequence::from(9u64),
                adventuresim_tactical_core::animation::LandingSequence::from(7u64)
            ),
            None
        );
        assert_eq!(
            latest_coalesced_landing(
                adventuresim_tactical_core::animation::LandingSequence::from(2u64),
                adventuresim_tactical_core::animation::LandingSequence::from(20u64)
            ),
            None
        );
    }

    #[test]
    fn downed_camera_alignment_enters_the_idle_locomotion_crossfade() {
        let idle = SkeletonState::default().with_body_state(BodyState::Supine);
        assert!(!ordinary_locomotion_candidate(&idle));

        let mut turning = idle;
        turning.set_downed_motion(DownedMotion::Turning);
        assert!(ordinary_locomotion_candidate(&turning));
    }

    #[test]
    fn combat_cycle_support_is_exclusive_and_switches_at_authored_contacts() {
        for sample in 0..240 {
            let phase = sample as f32 / 240.0;
            let weights = FootIkInfluence::combat_cycle(CyclePhase::from_cycles(phase).unwrap());
            assert!(
                (weights.left() > UnitInterval::ZERO) ^ (weights.right() > UnitInterval::ZERO),
                "phase {phase:.4} reported overlapping or absent support: {weights:?}"
            );
        }

        assert_eq!(
            FootIkInfluence::combat_cycle(CyclePhase::from_cycles(0.2499).unwrap()),
            FootIkInfluence::RIGHT
        );
        assert_eq!(
            FootIkInfluence::combat_cycle(CyclePhase::from_cycles(LEFT_PASSING_PHASE).unwrap()),
            FootIkInfluence::LEFT
        );
        assert_eq!(
            FootIkInfluence::combat_cycle(CyclePhase::from_cycles(0.7499).unwrap()),
            FootIkInfluence::LEFT
        );
        assert_eq!(
            FootIkInfluence::combat_cycle(CyclePhase::from_cycles(RIGHT_PASSING_PHASE).unwrap()),
            FootIkInfluence::RIGHT
        );
    }
}
