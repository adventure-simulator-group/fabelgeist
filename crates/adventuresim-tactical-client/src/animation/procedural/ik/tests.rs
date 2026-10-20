use super::terminal::TerminalReach;
use super::*;
use adventuresim_core::linear_speed::NonnegativeSpeed;

mod coordination_tests {
    // These contracts deliberately stay beside the coordinator: they exercise
    // handoffs between authored motion, retained contact state, terrain
    // planning, settlement, and orientation rather than one policy owner in
    // isolation. Owner-local tests remain in hands, locomotion, and solver.
    use super::*;
    use adventuresim_tactical_core::animation::AnimationSampleTick;

    #[test]
    fn authored_to_idle_handoff_does_not_reset_contact_sequence() {
        assert_eq!(
            retain_monotonic_contact_sequence(
                ContactSequence::from(1u64),
                ContactSequence::from(0u64)
            ),
            ContactSequence::from(1u64)
        );
        assert_eq!(
            retain_monotonic_contact_sequence(
                ContactSequence::from(1u64),
                ContactSequence::from(1u64)
            ),
            ContactSequence::from(1u64)
        );
        assert_eq!(
            retain_monotonic_contact_sequence(
                ContactSequence::from(1u64),
                ContactSequence::from(2u64)
            ),
            ContactSequence::from(2u64)
        );
        assert_eq!(
            retain_monotonic_contact_sequence(
                ContactSequence::from(u64::MAX),
                ContactSequence::from(0u64)
            ),
            ContactSequence::from(0u64)
        );
    }

    #[test]
    fn guard_step_state_cannot_represent_two_swing_feet() {
        let planted = GuardStepState::Stationary {
            left: Vec3::NEG_X,
            right: Vec3::X,
            next: LeadFoot::Right,
        };
        assert_eq!(planted.swing_foot(), None);
        assert_eq!(planted.progress(), None);

        let swinging = GuardStepState::RightSwing {
            left_support: Vec3::NEG_X,
            right: GuardSwing {
                start: Vec3::X,
                end: Vec3::X + Vec3::Z,
                progress: UnitInterval::from_fraction(0.4).unwrap(),
            },
        };
        assert_eq!(swinging.swing_foot(), Some(LeadFoot::Right));
        assert_eq!(
            swinging.progress(),
            Some(UnitInterval::from_fraction(0.4).unwrap())
        );
    }

    fn guard_test_geometry() -> ([GuardLegGeometry; 2], GuardTargetRequest) {
        (
            [
                GuardLegGeometry {
                    hip: Vec3::new(-0.12, 0.0, 0.0),
                    maximum_reach: ReachLimit::from(NonnegativeLength::from_metres(1.0).unwrap()),
                },
                GuardLegGeometry {
                    hip: Vec3::new(0.12, 0.0, 0.0),
                    maximum_reach: ReachLimit::from(NonnegativeLength::from_metres(1.0).unwrap()),
                },
            ],
            GuardTargetRequest {
                left: Vec3::new(-0.15, -0.8, 0.0),
                right: Vec3::new(0.15, -0.8, 0.0),
            },
        )
    }

    #[test]
    fn guard_frame_validation_allows_airborne_foot_to_pass_support() {
        let (geometry, authored) = guard_test_geometry();
        let crossed = GuardTargetRequest {
            left: Vec3::new(0.3, -0.8, -0.4),
            right: authored.right,
        };

        assert!(validate_guard_frame_targets(crossed, geometry, Some(LeadFoot::Left)).is_some());
    }

    #[test]
    fn guard_target_validation_never_moves_support_and_rejects_non_finite_input() {
        let (geometry, authored) = guard_test_geometry();
        let unreachable_support = GuardTargetRequest {
            left: Vec3::new(-0.15, -4.0, 0.0),
            right: authored.right,
        };
        let validated =
            validate_guard_frame_targets(unreachable_support, geometry, Some(LeadFoot::Right))
                .expect("one leg cannot suppress the other leg's solve");
        assert_eq!(validated.left(), unreachable_support.left);
        let non_finite_swing = GuardTargetRequest {
            left: Vec3::new(f32::NAN, -0.8, 0.0),
            right: authored.right,
        };
        assert!(
            validate_guard_frame_targets(non_finite_swing, geometry, Some(LeadFoot::Left))
                .is_none()
        );
    }

    #[test]
    fn guard_target_validation_preserves_horizontal_swing_while_lifting_for_reach() {
        let (geometry, authored) = guard_test_geometry();
        let requested = GuardTargetRequest {
            left: Vec3::new(-0.15, -0.8, 0.8),
            right: authored.right,
        };

        let validated = validate_guard_frame_targets(requested, geometry, Some(LeadFoot::Left))
            .expect("a finite overextension can be shortened in place");

        assert!(validated.adjusted_for_reach());
        assert_eq!(validated.left().xz(), requested.left.xz());
        assert!(validated.left().y > requested.left.y);
        assert_eq!(validated.right(), authored.right);
        assert!(validated.left().distance(geometry[0].hip) <= 1.0001);
    }

    #[test]
    fn guard_swing_has_zero_endpoint_velocity_and_acceleration() {
        let epsilon = 0.001;
        assert_eq!(smootherstep01(0.0), 0.0);
        assert_eq!(smootherstep01(1.0), 1.0);
        assert!(smootherstep01(epsilon) < epsilon * epsilon);
        assert!(1.0 - smootherstep01(1.0 - epsilon) < epsilon * epsilon);
    }

    #[test]
    fn stationary_turn_retains_plants_until_limit_then_lifts_one_foot() {
        let left = Vec3::new(-0.18, 0.1, 0.2);
        let right = Vec3::new(0.18, 0.1, -0.2);
        let planted = GuardStepState::Stationary {
            left,
            right,
            next: LeadFoot::Left,
        };
        let nearby = GuardStepState::Stationary {
            left: left + Vec3::X * 0.02,
            right: right + Vec3::X * 0.02,
            next: LeadFoot::Left,
        };
        assert_eq!(
            advance_stationary_turn_step(
                planted,
                nearby,
                AnimationElapsed::from_seconds(1.0 / 64.0).unwrap()
            )
            .unwrap(),
            planted
        );

        let turned = GuardStepState::Stationary {
            left: left + Vec3::Z * 0.2,
            right: right + Vec3::Z * 0.2,
            next: LeadFoot::Left,
        };
        let first = advance_stationary_turn_step(
            planted,
            turned,
            AnimationElapsed::from_seconds(1.0 / 64.0).unwrap(),
        )
        .unwrap();
        assert!(matches!(first, GuardStepState::LeftSwing { .. }));
        let second = advance_stationary_turn_step(
            first,
            turned,
            AnimationElapsed::from_seconds(1.0 / 64.0).unwrap(),
        )
        .unwrap();
        let GuardStepState::LeftSwing {
            right_support,
            left: swing,
        } = second
        else {
            panic!("turn adjustment should remain a one-foot swing");
        };
        assert_eq!(right_support, right);
        assert!(swing.progress.fraction() > 0.0);
        assert!(guard_swing_target(swing).y > swing.start.y);

        let vertical_settle = GuardStepState::Stationary {
            left: left + Vec3::Y * 0.2,
            right: right - Vec3::Y * 0.2,
            next: LeadFoot::Left,
        };
        assert_eq!(
            advance_stationary_turn_step(
                planted,
                vertical_settle,
                AnimationElapsed::from_seconds(1.0 / 64.0).unwrap()
            )
            .unwrap(),
            planted
        );
    }

    #[test]
    fn raised_footwork_reports_the_visual_support_contact() {
        let mut state = RaisedFootworkState {
            step: GuardStepState::Stationary {
                left: Vec3::NEG_X,
                right: Vec3::X,
                next: LeadFoot::Left,
            },
            ..default()
        };
        assert_eq!(state.contact_foot(), Some(LeadFoot::Right));
        state.step = GuardStepState::LeftSwing {
            right_support: Vec3::X,
            left: GuardSwing {
                start: Vec3::NEG_X,
                end: Vec3::NEG_Z,
                progress: UnitInterval::from_fraction(0.5).unwrap(),
            },
        };
        assert_eq!(state.contact_foot(), Some(LeadFoot::Right));
        state.step = GuardStepState::RightSwing {
            left_support: Vec3::NEG_X,
            right: GuardSwing {
                start: Vec3::X,
                end: Vec3::Z,
                progress: UnitInterval::from_fraction(0.5).unwrap(),
            },
        };
        assert_eq!(state.contact_foot(), Some(LeadFoot::Left));
    }

    #[test]
    fn guard_reacquire_moves_the_foot_whose_path_does_not_cross_support() {
        let current_left = Vec3::new(-0.02, 0.0, 0.02);
        let current_right = Vec3::new(0.18, 0.0, 0.20);
        let desired_left = Vec3::new(-0.17, 0.0, 0.0);
        let desired_right = Vec3::new(-0.06, 0.0, 0.0);

        assert_eq!(
            safer_guard_reacquire_foot(
                current_left,
                current_right,
                desired_left,
                desired_right,
                LeadFoot::Right,
            ),
            LeadFoot::Left
        );
    }

    #[test]
    fn quickstep_contact_handoff_can_be_held_without_waiting_for_a_full_stop() {
        let mut handoff = QuickstepContactHandoff::Converging {
            left: Vec3::X,
            right: Vec3::NEG_X,
        };
        handoff.hold();
        assert!(handoff.is_held());
        assert_eq!(handoff.targets(), Some((Vec3::X, Vec3::NEG_X)));
    }

    #[test]
    fn authored_locomotion_releases_dormant_raised_pelvis_correction() {
        let mut memory = LegIkMemory {
            quickstep_handoff: QuickstepContactHandoff::Converging {
                left: Vec3::NEG_X,
                right: Vec3::X,
            },
            raised_pelvis_shift: SignedLength::from_metres(
                -ik_tuning().guard_reach_pelvis_drop_metres.metres(),
            )
            .unwrap(),
            ..default()
        };
        release_raised_state_for_authored_locomotion(&mut memory);
        assert_eq!(memory.quickstep_handoff, QuickstepContactHandoff::None);
        assert_eq!(memory.raised_pelvis_shift.metres(), 0.0);
        assert_eq!(memory.left_support_weight, None);
        assert_eq!(memory.right_support_weight, None);
    }

    #[test]
    fn quickstep_contact_handoff_is_seeded_from_the_last_rendered_pose() {
        let mut memory = LegIkMemory::default();
        let origin = Vec3::new(3.0, 0.5, -2.0);
        let rotation = Quat::from_rotation_y(0.6);
        let left_local = Vec3::new(-0.18, 0.12, 0.25);
        let right_local = Vec3::new(0.16, 0.14, -0.2);

        assert!(seed_quickstep_contact_handoff(
            &mut memory,
            ObservedRigFrame::from_native(origin, rotation).unwrap(),
            origin + rotation * left_local,
            origin + rotation * right_local,
        ));
        let (left, right) = memory.quickstep_handoff.targets().unwrap();
        assert!(left.distance(left_local) <= 1.0e-6);
        assert!(right.distance(right_local) <= 1.0e-6);
        assert_eq!(
            memory.owner_frame.map(ObservedRigFrame::origin),
            Some(origin)
        );
        assert_eq!(
            memory.owner_frame.map(ObservedRigFrame::rotation),
            Some(rotation)
        );
    }

    #[test]
    fn slope_rotation_cache_is_preserved_within_tick_and_cleared_between_modes() {
        let cached = LegRotationChain {
            upper: Quat::from_rotation_x(0.2),
            lower: Quat::from_rotation_z(-0.3),
            foot: Quat::from_rotation_y(0.4),
        };
        let mut memory = LegIkMemory {
            left_rotation_chain: Some(cached),
            slope_alignment_mode: Some(SlopeAlignmentMode::Raised),
            ..default()
        };

        prepare_slope_rotation_cache(&mut memory, SlopeAlignmentMode::Raised);
        assert_eq!(memory.left_rotation_chain, Some(cached));

        prepare_slope_rotation_cache(&mut memory, SlopeAlignmentMode::Ordinary);
        assert_eq!(memory.left_rotation_chain, None);
        assert_eq!(memory.right_rotation_chain, None);
        assert_eq!(
            memory.slope_alignment_mode,
            Some(SlopeAlignmentMode::Ordinary)
        );

        memory.right_rotation_chain = Some(cached);
        clear_slope_rotation_cache(&mut memory);
        assert_eq!(memory.right_rotation_chain, None);
        assert_eq!(memory.slope_alignment_mode, None);
    }

    #[test]
    fn repeated_evaluation_restores_the_exact_cached_leg_chain() {
        let cached = LegRotationChain {
            upper: Quat::from_rotation_x(0.2),
            lower: Quat::from_rotation_z(-0.3),
            foot: Quat::from_rotation_y(0.4),
        };
        let perturbed_by_second_solve = LegRotationChain {
            upper: Quat::from_rotation_x(-0.5),
            lower: Quat::from_rotation_z(0.6),
            foot: Quat::from_rotation_y(-0.7),
        };

        assert_eq!(
            final_leg_rotation_chain(Some(cached), perturbed_by_second_solve, false),
            cached
        );
        assert_eq!(
            final_leg_rotation_chain(Some(cached), perturbed_by_second_solve, true),
            perturbed_by_second_solve
        );
        assert_eq!(
            final_leg_rotation_chain(None, perturbed_by_second_solve, false),
            perturbed_by_second_solve
        );
    }

    #[test]
    fn airborne_foot_orientation_releases_at_a_bounded_angular_speed() {
        let previous = Quat::IDENTITY;
        let desired = Quat::from_rotation_x(90.0_f32.to_radians());
        let advanced = advance_airborne_foot_rotation(
            Some(previous),
            desired,
            AnimationElapsed::from_seconds(1.0 / 64.0).unwrap(),
            ik_tuning().airborne_foot_rotation_speed_degrees_per_second,
        );

        assert!((previous.angle_between(advanced).to_degrees() - 9.0).abs() < 0.0001);
        assert!(advanced.angle_between(desired) < previous.angle_between(desired));
        assert_eq!(
            advance_airborne_foot_rotation(
                Some(advanced),
                desired,
                AnimationElapsed::from_seconds(0.0).unwrap(),
                ik_tuning().airborne_foot_rotation_speed_degrees_per_second,
            ),
            advanced
        );
        assert_eq!(
            advance_airborne_foot_rotation(
                None,
                desired,
                AnimationElapsed::from_seconds(1.0 / 64.0).unwrap(),
                ik_tuning().airborne_foot_rotation_speed_degrees_per_second,
            ),
            desired
        );
    }

    #[test]
    fn run_contact_approach_reaches_the_plant_at_support_entry() {
        let radius = run_locomotion_profile().support_phase_radius.cycles();
        let ready = radius + ik_tuning().run_contact_chain_settle_phase.fraction();
        assert_eq!(
            run_contact_approach_progress(
                CycleDelta::from_cycles(ik_tuning().run_contact_approach_phase.fraction()).unwrap(),
                CycleDelta::from_unit_cycles(ik_tuning().run_contact_approach_phase),
                CycleDelta::from_cycles(ready).unwrap(),
            )
            .unwrap()
            .fraction(),
            0.0
        );
        assert_eq!(
            run_contact_approach_progress(
                CycleDelta::from_cycles(ready).unwrap(),
                CycleDelta::from_unit_cycles(ik_tuning().run_contact_approach_phase),
                CycleDelta::from_cycles(ready).unwrap()
            )
            .unwrap()
            .fraction(),
            1.0
        );
        assert_eq!(
            run_contact_approach_progress(
                CycleDelta::from_cycles(radius).unwrap(),
                CycleDelta::from_unit_cycles(ik_tuning().run_contact_approach_phase),
                CycleDelta::from_cycles(ready).unwrap()
            )
            .unwrap()
            .fraction(),
            1.0
        );
        let middle = run_contact_approach_progress(
            CycleDelta::from_cycles(
                (ik_tuning().run_contact_approach_phase.fraction() + ready) * 0.5,
            )
            .unwrap(),
            CycleDelta::from_unit_cycles(ik_tuning().run_contact_approach_phase),
            CycleDelta::from_cycles(ready).unwrap(),
        )
        .unwrap()
        .fraction();
        assert!((middle - 0.5).abs() < 0.0001);
        let release_finished_phase = 0.81;
        assert_eq!(
            run_contact_approach_progress(
                CycleDelta::from_cycles(release_finished_phase).unwrap(),
                CycleDelta::from_cycles(release_finished_phase).unwrap(),
                CycleDelta::from_cycles(ready).unwrap(),
            )
            .unwrap()
            .fraction(),
            0.0
        );
        assert!(
            run_swing_clearance(
                CycleDelta::from_cycles(radius).unwrap(),
                Some(UnitInterval::from_fraction(1.0).unwrap())
            )
            .unwrap()
            .metres()
                <= 0.0
        );
        assert!(
            run_swing_clearance(
                CycleDelta::from_cycles(0.3375).unwrap(),
                Some(UnitInterval::from_fraction(0.5).unwrap())
            )
            .unwrap()
            .metres()
                > 0.08
        );

        let phase_step = gait_cycle_phase_delta(
            run_locomotion_profile(),
            NonnegativeSpeed::from_metres_per_second(5.5).unwrap(),
            AnimationElapsed::from_seconds(1.0 / 64.0).unwrap(),
        )
        .unwrap()
        .cycles();
        let mut phase_to_contact = ik_tuning().run_contact_approach_phase.fraction();
        let mut previous_progress = run_contact_approach_progress(
            CycleDelta::from_cycles(phase_to_contact).unwrap(),
            CycleDelta::from_unit_cycles(ik_tuning().run_contact_approach_phase),
            CycleDelta::from_cycles(ready).unwrap(),
        )
        .unwrap()
        .fraction();
        while phase_to_contact > ready {
            phase_to_contact = (phase_to_contact - phase_step).max(ready);
            let progress = run_contact_approach_progress(
                CycleDelta::from_cycles(phase_to_contact).unwrap(),
                CycleDelta::from_unit_cycles(ik_tuning().run_contact_approach_phase),
                CycleDelta::from_cycles(ready).unwrap(),
            )
            .unwrap()
            .fraction();
            let three_metre_world_step = 3.0 * (progress - previous_progress);
            let root_step = 5.5 / 64.0;
            assert!((three_metre_world_step - root_step).abs() <= 0.095);
            previous_progress = progress;
        }
    }

    #[test]
    fn planned_run_contact_anticipates_a_bounded_pelvis_reach_drop() {
        let radius = run_locomotion_profile().support_phase_radius.cycles();
        let ready = radius + ik_tuning().run_contact_chain_settle_phase.fraction();
        let early = run_contact_approach_progress(
            CycleDelta::from_cycles(ik_tuning().run_contact_approach_phase.fraction()).unwrap(),
            CycleDelta::from_unit_cycles(ik_tuning().run_contact_approach_phase),
            CycleDelta::from_cycles(ready).unwrap(),
        )
        .unwrap()
        .fraction();
        let late = run_contact_approach_progress(
            CycleDelta::from_cycles(ready).unwrap(),
            CycleDelta::from_unit_cycles(ik_tuning().run_contact_approach_phase),
            CycleDelta::from_cycles(ready).unwrap(),
        )
        .unwrap()
        .fraction();
        assert_eq!(early, 0.0);
        assert_eq!(late, 1.0);

        let required_reach_shift = -0.11;
        let early_target = (required_reach_shift * early).clamp(
            -ik_tuning()
                .run_maximum_planned_reach_pelvis_drop_metres
                .metres(),
            0.0,
        );
        let late_target = (required_reach_shift * late).clamp(
            -ik_tuning()
                .run_maximum_planned_reach_pelvis_drop_metres
                .metres(),
            0.0,
        );
        assert_eq!(early_target, 0.0);
        assert_eq!(late_target, required_reach_shift);
        assert!(
            advance_scalar_at_speed(
                SignedLength::from_metres(0.0).unwrap(),
                SignedLength::from_metres(late_target).unwrap(),
                AnimationElapsed::from_seconds(1.0 / 64.0).unwrap(),
                ik_tuning().run_pelvis_correction_speed_metres_per_second
            )
            .unwrap()
            .metres()
            .abs()
                <= 0.01
        );
    }

    #[test]
    fn frozen_run_contact_is_reachable_through_predicted_downhill_stance() {
        // Production-sized 0.523 m + 0.430 m leg and the captured downhill
        // plan geometry that previously froze an unreachable -6.117 m plant.
        let upper = Vec3::new(0.1, 3.109, -2.847);
        let velocity = Vec3::NEG_Z * 5.5;
        let ready = run_locomotion_profile().support_phase_radius.cycles()
            + ik_tuning().run_contact_chain_settle_phase.fraction();
        let reach = ReachLimit::Fixed(NonnegativeLength::from_metres(0.953).unwrap());
        let phase_to_contact = 0.744;
        let travel_per_phase = ordinary_step_distance(
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(5.5).unwrap(),
        )
        .unwrap()
        .metres()
            * 2.0;
        let downhill = |point: ScenePlanPoint| {
            Some(SupportElevation::from_metres(2.38 + point.metres().y * 0.08).unwrap())
        };
        let current_height = downhill(ScenePlanPoint::from_metres(upper.xz()).unwrap()).unwrap();
        let predicted_roots = [
            phase_to_contact - run_locomotion_profile().support_phase_radius.cycles(),
            phase_to_contact,
            phase_to_contact + run_locomotion_profile().support_phase_radius.cycles(),
        ]
        .map(|remaining_phase| {
            let mut root = upper + Vec3::NEG_Z * (remaining_phase * travel_per_phase);
            root.y += downhill(ScenePlanPoint::from_metres(root.xz()).unwrap()).unwrap().metres() - current_height.metres();
            root - Vec3::Y
                * ik_tuning()
                    .run_maximum_planned_reach_pelvis_drop_metres
                    .metres()
        });
        let candidate = Vec3::new(0.1, 0.0, -6.117);
        let frozen = reachable_run_contact_target(
            candidate,
            upper,
            velocity,
            ordinary_step_distance(NonnegativeSpeed::from_metres_per_second(5.5).unwrap()).unwrap(),
            CycleDelta::from_cycles(phase_to_contact).unwrap(),
            CycleDelta::from_cycles(ready).unwrap(),
            reach,
            downhill,
        )
        .unwrap();
        assert!(frozen.is_finite());
        for predicted_root in predicted_roots {
            assert!(frozen.distance(predicted_root) <= reach.native_metres().unwrap() + 0.001);
        }
        assert_eq!(
            frozen,
            reachable_run_contact_target(
                candidate,
                upper,
                velocity,
                ordinary_step_distance(NonnegativeSpeed::from_metres_per_second(5.5).unwrap())
                    .unwrap(),
                CycleDelta::from_cycles(phase_to_contact).unwrap(),
                CycleDelta::from_cycles(ready).unwrap(),
                reach,
                downhill,
            )
            .unwrap()
        );

        let flat_predicted_center = upper + Vec3::NEG_Z * (phase_to_contact * travel_per_phase)
            - Vec3::Y
                * ik_tuning()
                    .run_maximum_planned_reach_pelvis_drop_metres
                    .metres();
        let flat_candidate = flat_predicted_center + Vec3::new(0.1, -0.5, 0.0);
        let flat_height = SupportElevation::from_metres(
            flat_candidate.y - measured_ankle_sole_offset_metres().metres(),
        ).unwrap();
        let flat = reachable_run_contact_target(
            flat_candidate,
            upper,
            velocity,
            ordinary_step_distance(NonnegativeSpeed::from_metres_per_second(5.5).unwrap()).unwrap(),
            CycleDelta::from_cycles(phase_to_contact).unwrap(),
            CycleDelta::from_cycles(ready).unwrap(),
            reach,
            |_| Some(flat_height),
        )
        .unwrap();
        assert!(flat.distance(flat_candidate) <= 0.0001);
    }

    #[test]
    fn run_swing_end_and_first_support_sample_share_target_and_pole() {
        let planted = Vec3::new(0.1, 1.97, -7.477);
        let authored_upper = Vec3::new(0.1, 3.04, -6.25);
        let pelvis_shift = (0..20).fold(0.0, |shift, _| {
            advance_scalar_at_speed(
                SignedLength::from_metres(shift).unwrap(),
                SignedLength::from_metres(
                    -ik_tuning()
                        .run_maximum_planned_reach_pelvis_drop_metres
                        .metres(),
                )
                .unwrap(),
                AnimationElapsed::from_seconds(1.0 / 64.0).unwrap(),
                ik_tuning().run_pelvis_correction_speed_metres_per_second,
            )
            .unwrap()
            .metres()
        });
        let upper = authored_upper + Vec3::Y * pelvis_shift;
        let reach = ReachLimit::Fixed(NonnegativeLength::from_metres(0.953).unwrap());
        let swing_end =
            acquisition_planted_target(planted, upper, reach, LocomotionGait::Run, false).unwrap();
        let first_acquired =
            acquisition_planted_target(planted, upper, reach, LocomotionGait::Run, true).unwrap();
        assert_eq!(swing_end, planted);
        assert_eq!(first_acquired, swing_end);

        let authored_knee = upper + Vec3::new(0.0, -0.52, -0.05);
        let authored_foot = authored_knee + Vec3::new(0.0, -0.43, -0.04);
        let pole = Vec3::NEG_Z;
        let before = solve_two_bone_with_reach(
            TwoBoneChain::new(
                upper,
                authored_knee,
                authored_foot,
                NonnegativeLength::from_metres(0.523).unwrap(),
                NonnegativeLength::from_metres(0.430).unwrap(),
                pole,
            ),
            swing_end,
            reach,
        )
        .unwrap();
        let after = solve_two_bone_with_reach(
            TwoBoneChain::new(
                upper,
                authored_knee,
                authored_foot,
                NonnegativeLength::from_metres(0.523).unwrap(),
                NonnegativeLength::from_metres(0.430).unwrap(),
                pole,
            ),
            first_acquired,
            reach,
        )
        .unwrap();
        assert!(before.knee.distance(after.knee) <= 0.0);
        assert!(before.end.distance(after.end) <= 0.0);
    }

    #[test]
    fn shallow_acquisition_pole_survives_support_confidence_ramp() {
        let canonical = Vec3::new(-0.177153, 0.0, -0.984183);
        let shallow = Vec3::new(-0.999273, 0.038100, -0.001613);
        assert!(shallow.normalize().dot(canonical) < 0.2);
        let retained = retained_terrain_pole(shallow, canonical).unwrap();
        assert!(retained.dot(canonical) > 0.0);

        let first_root = Vec3::new(-0.100270, 2.863136, -10.316_13);
        let next_root = Vec3::new(-0.100349, 2.875328, -10.407523);
        let target = Vec3::new(-0.120271, 2.308135, -11.034_69);
        let authored_knee = first_root + Vec3::new(0.0, -0.52, -0.05);
        let authored_foot = authored_knee + Vec3::new(0.0, -0.43, -0.04);
        let terrain_reach = terrain_maximum_reach(
            NonnegativeLength::from_metres(0.523).unwrap(),
            NonnegativeLength::from_metres(0.430).unwrap(),
        );
        let first = solve_two_bone_with_reach(
            TwoBoneChain::new(
                first_root,
                authored_knee,
                authored_foot,
                NonnegativeLength::from_metres(0.523).unwrap(),
                NonnegativeLength::from_metres(0.430).unwrap(),
                retained,
            ),
            target,
            terrain_reach,
        )
        .unwrap();
        let next = solve_two_bone_with_reach(
            TwoBoneChain::new(
                next_root,
                authored_knee + (next_root - first_root),
                authored_foot + (next_root - first_root),
                NonnegativeLength::from_metres(0.523).unwrap(),
                NonnegativeLength::from_metres(0.430).unwrap(),
                retained,
            ),
            target,
            terrain_reach,
        )
        .unwrap();
        let root_relative_step = (next.knee - next_root).distance(first.knee - first_root);
        assert!(root_relative_step <= 0.10);

        let previous_direction = (target - first_root).normalize();
        let next_direction = (target - next_root).normalize();
        let transported = transported_terrain_pole(
            Some(retained),
            Some(previous_direction),
            next_direction,
            canonical,
        )
        .unwrap();
        assert!(
            transported.dot(next_direction).abs()
                <= retained.dot(previous_direction).abs() + 0.0001
        );
    }

    #[test]
    fn attack_knee_bend_parallel_transports_with_the_leg() {
        let previous_end = Vec3::NEG_Y;
        let remembered = Vec3::Z;
        let next_end = Vec3::X;
        let expected = Quat::from_rotation_arc(previous_end, next_end) * remembered;
        let pole = stabilized_knee_pole(
            Some(remembered),
            Some(previous_end),
            Vec3::ZERO,
            Vec3::new(0.0, -0.5, 0.1),
            next_end,
            expected,
            None,
        )
        .unwrap();

        assert!(pole.dot(expected) > 0.999);
        assert!(pole.dot(next_end).abs() < 0.0001);
    }

    #[test]
    fn attack_knee_bend_survives_a_straight_leg_singularity() {
        let previous_end = Vec3::NEG_Y;
        let remembered = Vec3::Z;
        let next_target = Vec3::new(0.02, -1.0, 0.0).normalize();
        let expected = Quat::from_rotation_arc(previous_end, next_target) * remembered;
        let pole = stabilized_knee_pole(
            Some(remembered),
            Some(previous_end),
            Vec3::ZERO,
            next_target * 0.5,
            next_target,
            Vec3::Z,
            None,
        )
        .unwrap();

        assert!(pole.dot(expected) > 0.999);
        assert!(pole.dot(Vec3::Z) > 0.0);
    }

    #[test]
    fn attack_knee_bend_rejects_an_inward_authored_pole() {
        let pole = stabilized_knee_pole(
            None,
            None,
            Vec3::ZERO,
            Vec3::new(0.0, -0.5, -0.2),
            Vec3::NEG_Y,
            Vec3::Z,
            None,
        )
        .unwrap();

        assert!(pole.dot(Vec3::Z) > 0.999);
    }

    #[test]
    fn attack_knee_bend_retains_the_pre_attack_rendered_pole() {
        let remembered = Vec3::new(0.3, 0.0, 0.95).normalize();
        let pole = stabilized_knee_pole(
            Some(remembered),
            None,
            Vec3::ZERO,
            Vec3::new(0.0, -0.5, 0.4),
            Vec3::NEG_Y,
            Vec3::Z,
            None,
        )
        .unwrap();

        assert!(pole.dot(remembered) > 0.999);
    }

    #[test]
    fn knee_pole_is_clamped_to_pi_over_eight_from_foot_facing() {
        let leg_direction = Vec3::NEG_Y;
        let foot_facing = Vec3::Z;
        let pole = constrain_knee_pole_to_foot_facing(
            Vec3::X,
            leg_direction,
            foot_facing,
            AngularMagnitudeDegrees::from_degrees(22.5).unwrap(),
        )
        .unwrap();

        assert!(pole.dot(leg_direction).abs() < 0.0001);
        assert!(pole.xz().angle_to(foot_facing.xz()).abs() <= std::f32::consts::FRAC_PI_8 + 0.0001);
    }

    #[test]
    fn diagonal_leg_cannot_rotate_clamped_pole_yaw_sideways() {
        let leg_direction = Vec3::new(0.0, -0.3, 0.954).normalize();
        let foot_facing = Vec3::Z;
        let pole = constrain_knee_pole_to_foot_facing(
            Vec3::X,
            leg_direction,
            foot_facing,
            AngularMagnitudeDegrees::from_degrees(22.5).unwrap(),
        )
        .unwrap();

        assert!(pole.dot(leg_direction).abs() < 0.0001);
        assert!(pole.xz().angle_to(foot_facing.xz()).abs() <= std::f32::consts::FRAC_PI_8 + 0.0001);
    }

    #[test]
    fn knee_pole_inside_foot_facing_cone_is_unchanged() {
        let leg_direction = Vec3::NEG_Y;
        let expected = Quat::from_rotation_y(0.2) * Vec3::Z;
        let pole = constrain_knee_pole_to_foot_facing(
            expected,
            leg_direction,
            Vec3::Z,
            AngularMagnitudeDegrees::from_degrees(22.5).unwrap(),
        )
        .unwrap();

        assert!(pole.dot(expected) > 0.9999);
    }

    #[test]
    fn release_to_planned_contact_starts_at_the_visible_solve_target() {
        let visible_release = Vec3::new(0.1, 0.25, -6.094);
        let restored_authored = Vec3::new(0.1, 0.5, -6.9);
        let start = planned_contact_start(None, Some(visible_release), restored_authored);
        assert_eq!(start, visible_release);
        assert_eq!(
            start.lerp(Vec3::new(0.1, 0.085, -8.0), 0.0),
            visible_release
        );

        let retained = Vec3::new(0.1, 0.3, -6.2);
        assert_eq!(
            planned_contact_start(Some(retained), Some(visible_release), restored_authored),
            retained
        );
    }

    #[test]
    fn new_run_plan_transports_in_progress_release_start_with_owner() {
        // Captured right release-to-plan seam f71->72. Holding the f71 ankle
        // in world space moved it 8.6 cm relative to the advancing hip while
        // Hermite progress was still zero, amplifying into a 13.7 cm knee
        // step. The seed must retain the same owner-local point instead.
        let previous_root = Vec3::new(0.0, 2.8301053, -6.1015625);
        let current_root = Vec3::new(0.0, 2.8237216, -6.1875);
        let previous_ankle = Vec3::new(0.12985985, 2.1838071, -5.3937254);
        let previous_owner = previous_ankle - previous_root;
        let stale_analytic_owner = previous_owner + Vec3::new(0.0, -0.06, -0.11);
        assert_eq!(
            run_previous_owner_target(
                LocomotionGait::Run,
                Some(previous_owner),
                Some(stale_analytic_owner),
            ),
            Some(previous_owner)
        );
        assert_eq!(
            run_previous_owner_target(
                LocomotionGait::Walk,
                Some(previous_owner),
                Some(stale_analytic_owner),
            ),
            Some(stale_analytic_owner)
        );
        let transported = run_plan_visible_start(
            LocomotionGait::Run,
            true,
            true,
            Some(previous_owner),
            current_root,
            Quat::IDENTITY,
            Some(previous_ankle),
        )
        .unwrap();
        assert!((transported - current_root - previous_owner).length() < 0.0001);
        assert!((transported - previous_ankle - (current_root - previous_root)).length() < 0.0001);
        assert!((transported - current_root).distance(previous_ankle - previous_root) < 0.0001);

        // Retained plans keep their original frozen start, and walk/stop keep
        // world-hold semantics rather than inheriting Run's owner transport.
        assert_eq!(
            run_plan_visible_start(
                LocomotionGait::Run,
                false,
                true,
                Some(previous_owner),
                current_root,
                Quat::IDENTITY,
                Some(previous_ankle),
            ),
            Some(previous_ankle)
        );
        assert_eq!(
            run_plan_visible_start(
                LocomotionGait::Walk,
                true,
                true,
                Some(previous_owner),
                current_root,
                Quat::IDENTITY,
                Some(previous_ankle),
            ),
            Some(previous_ankle)
        );
    }

    #[test]
    fn new_run_plan_prefers_last_propagated_ankle_over_stale_solve() {
        let stale_solve = Vec3::new(0.1, 2.1, -0.767);
        let rendered_ankle = Vec3::new(0.1, 2.1, -1.749);
        let visible = Some(rendered_ankle).or(Some(stale_solve));
        assert_eq!(
            planned_contact_start(None, visible, Vec3::ZERO),
            rendered_ankle
        );
    }

    #[test]
    fn cold_start_run_plan_is_bounded_over_the_remaining_approach() {
        // Captured hard-start geometry: the right plan first became airborne
        // late in the approach and previously tried to cover 1.525 m in four
        // presentation samples.
        let start = Vec3::new(0.1, 2.1, -0.304);
        let desired = Vec3::new(0.1, 2.0, -1.829);
        let phase_to_contact = 0.418;
        assert!(late_run_plan_requires_bound(
            None,
            CycleDelta::from_cycles(phase_to_contact).unwrap()
        ));
        assert!(!late_run_plan_requires_bound(
            None,
            CycleDelta::from_cycles(0.75).unwrap()
        ));
        assert!(!late_run_plan_requires_bound(
            Some(desired),
            CycleDelta::from_cycles(phase_to_contact).unwrap()
        ));
        let ready = run_locomotion_profile().support_phase_radius.cycles()
            + ik_tuning().run_contact_chain_settle_phase.fraction();
        let bounded = bound_late_run_contact(
            start,
            desired,
            NonnegativeSpeed::from_metres_per_second(5.5).unwrap(),
            ordinary_step_distance(NonnegativeSpeed::from_metres_per_second(5.5).unwrap()).unwrap(),
            CycleDelta::from_cycles(phase_to_contact).unwrap(),
            CycleDelta::from_cycles(ready).unwrap(),
        );
        assert!(bounded.xz().distance(desired.xz()) > 0.5);

        let phase_step = gait_cycle_phase_delta(
            run_locomotion_profile(),
            NonnegativeSpeed::from_metres_per_second(5.5).unwrap(),
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
        )
        .unwrap()
        .cycles();
        let first_progress = run_contact_approach_progress(
            CycleDelta::from_cycles(phase_to_contact).unwrap(),
            CycleDelta::from_cycles(phase_to_contact).unwrap(),
            CycleDelta::from_cycles(ready).unwrap(),
        )
        .unwrap()
        .fraction();
        let second_progress = run_contact_approach_progress(
            CycleDelta::from_cycles(phase_to_contact - phase_step).unwrap(),
            CycleDelta::from_cycles(phase_to_contact).unwrap(),
            CycleDelta::from_cycles(ready).unwrap(),
        )
        .unwrap()
        .fraction();
        assert_eq!(start.lerp(bounded, first_progress).xz(), start.xz());
        let first_step = start
            .lerp(bounded, second_progress)
            .xz()
            .distance(start.xz());
        let root_step = 5.5 / ik_tuning().continuity_sample_hz.hertz();
        assert!(
            first_step - root_step
                <= ik_tuning()
                    .maximum_run_swing_root_relative_step_metres
                    .metres()
                    + 0.0001
        );
    }

    #[test]
    fn reach_released_support_lobe_cannot_reenter_before_true_flight() {
        let (still_exhausted, effective_support) =
            support_after_exhausted_lobe(true, UnitInterval::from_fraction(0.4).unwrap());
        assert!(still_exhausted);
        assert_eq!(effective_support.fraction(), 0.0);
        assert!(!run_planned_contact_allowed(
            still_exhausted,
            CycleDelta::from_cycles(0.2).unwrap(),
            CycleDelta::from_cycles(0.75).unwrap()
        ));

        let visible_release = Vec3::new(0.1, 0.2, -8.757);
        let stale_same_lobe_plan = Vec3::new(0.1, 0.08, -10.203);
        let followed = advance_foot_target_at_speed(
            Some(visible_release),
            stale_same_lobe_plan,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                ik_tuning().airborne_release_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz(),
            )
            .unwrap(),
        );
        assert!(
            followed.distance(visible_release)
                <= (ik_tuning().airborne_release_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz())
                    / ik_tuning().continuity_sample_hz.hertz()
                    + 0.0001
        );

        let (cleared, flight_support) =
            support_after_exhausted_lobe(true, UnitInterval::from_fraction(0.0).unwrap());
        assert!(!cleared);
        assert_eq!(flight_support.fraction(), 0.0);
        assert!(run_planned_contact_allowed(
            cleared,
            CycleDelta::from_cycles(0.75).unwrap(),
            CycleDelta::from_cycles(0.75).unwrap()
        ));
    }

    #[test]
    fn unplanned_run_support_lobe_waits_for_true_flight() {
        assert!(unplanned_run_support_requires_flight(
            LocomotionGait::Run,
            NonnegativeSpeed::from_metres_per_second(5.5).unwrap(),
            UnitInterval::from_fraction(0.8).unwrap(),
            false,
            None,
        ));
        assert!(!unplanned_run_support_requires_flight(
            LocomotionGait::Run,
            NonnegativeSpeed::from_metres_per_second(5.5).unwrap(),
            UnitInterval::from_fraction(0.8).unwrap(),
            true,
            None,
        ));
        assert!(!unplanned_run_support_requires_flight(
            LocomotionGait::Run,
            NonnegativeSpeed::from_metres_per_second(5.5).unwrap(),
            UnitInterval::from_fraction(0.8).unwrap(),
            false,
            Some(Vec3::NEG_Z),
        ));
        assert!(!unplanned_run_support_requires_flight(
            LocomotionGait::Run,
            NonnegativeSpeed::from_metres_per_second(0.0).unwrap(),
            UnitInterval::from_fraction(0.8).unwrap(),
            false,
            None,
        ));
    }

    #[test]
    fn newly_acquired_contact_keeps_orientation_blending_until_converged() {
        assert!(update_contact_orientation_blend(
            false,
            Some(UnitInterval::from_fraction(0.0).unwrap()),
            UnitInterval::from_fraction(1.0).unwrap()
        ));
        assert!(update_contact_orientation_blend(
            true,
            Some(UnitInterval::from_fraction(1.0).unwrap()),
            UnitInterval::from_fraction(1.0).unwrap()
        ));
        assert!(!update_contact_orientation_blend(
            false,
            Some(UnitInterval::from_fraction(1.0).unwrap()),
            UnitInterval::from_fraction(1.0).unwrap()
        ));
        assert!(!update_contact_orientation_blend(
            true,
            Some(UnitInterval::from_fraction(1.0).unwrap()),
            UnitInterval::from_fraction(0.0).unwrap()
        ));

        let airborne = Quat::IDENTITY;
        let contact = Quat::from_rotation_x(63.54_f32.to_radians());
        let first_contact = advance_airborne_foot_rotation(
            Some(airborne),
            contact,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            ik_tuning().airborne_foot_rotation_speed_degrees_per_second,
        );
        assert!(
            airborne.angle_between(first_contact).to_degrees()
                <= ik_tuning()
                    .airborne_foot_rotation_speed_degrees_per_second
                    .degrees_per_second()
                    / ik_tuning().continuity_sample_hz.hertz()
                    + 0.0001
        );
        assert!(first_contact.angle_between(contact) < airborne.angle_between(contact));
    }

    #[test]
    fn release_target_cap_preserves_the_knee_continuity_budget() {
        let maximum_target_step = (ik_tuning().airborne_release_step_metres.metres()
            * ik_tuning().continuity_sample_hz.hertz())
            / ik_tuning().continuity_sample_hz.hertz();
        assert!(
            maximum_target_step * ik_tuning().maximum_knee_target_amplification.factor()
                < ik_tuning().maximum_knee_step_metres.metres()
        );
        assert!(maximum_target_step < 3.4 / ik_tuning().continuity_sample_hz.hertz());
    }

    #[test]
    fn raised_support_requires_rendered_sole_contact() {
        let terrain_height = SupportElevation::ZERO;
        assert!(raised_support_is_actual(
            true,
            measured_ankle_sole_offset_metres().metres() + sole_contact_tolerance_metres().metres()
                - 0.001,
            terrain_height,
        ));
        assert!(!raised_support_is_actual(
            true,
            measured_ankle_sole_offset_metres().metres() + 0.023,
            terrain_height,
        ));
        assert!(!raised_support_is_actual(
            false,
            measured_ankle_sole_offset_metres().metres(),
            terrain_height,
        ));
    }

    #[test]
    fn raised_stop_handoff_preserves_visible_targets_in_owner_space() {
        let rig_origin = Vec3::new(4.0, 0.0, -2.0);
        let rig_rotation = Quat::from_rotation_y(0.7);
        let left = Vec3::new(3.8, 0.1, -2.4);
        let right = Vec3::new(4.3, 0.1, -1.8);
        let raised = RaisedFootworkState {
            step: GuardStepState::Stationary {
                left,
                right,
                next: LeadFoot::Left,
            },
            left_solve_target: Some(left),
            right_solve_target: Some(right),
            ..default()
        };
        let mut memory = LegIkMemory::default();

        preserve_raised_handoff_targets(&mut memory, raised, rig_origin, rig_rotation);

        assert_eq!(memory.left_foot_world_target, Some(left));
        assert_eq!(memory.right_foot_world_target, Some(right));
        assert!(memory.left_release_active && memory.right_release_active);
        let restored_left =
            rig_origin + rig_rotation * memory.left_foot_target.expect("left owner target");
        let restored_right =
            rig_origin + rig_rotation * memory.right_foot_target.expect("right owner target");
        assert!(restored_left.distance(left) < 0.000001);
        assert!(restored_right.distance(right) < 0.000001);
    }

    #[test]
    fn raised_stop_settle_keeps_terrain_ik_alive_across_ticks() {
        let mut settle = LocomotionSettleState {
            support_foot: LeadFoot::Left,
            swing_start: Vec3::new(0.2, 0.1, 0.0),
            capture_point: Vec3::ZERO,
            landing_target: Vec3::new(-0.2, 0.1, -0.3),
            progress: SettleProgress::from_progress(0.0).unwrap(),
            elapsed_seconds: AnimationElapsed::from_seconds(0.0).unwrap(),
            raised_handoff: true,
        };

        assert!(terrain_ik_is_required(
            crate::animation::TerrainIkMode::Disabled,
            false,
            true
        ));
        for tick in 0..4 {
            settle = advance_settle_state(
                settle,
                AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz())
                    .unwrap(),
            )
            .unwrap();
            assert!(
                terrain_ik_is_required(crate::animation::TerrainIkMode::Disabled, true, false),
                "tick {tick}"
            );
            assert!(settle.progress.fraction() > 0.0 && settle.progress.fraction() < 1.0);
        }
        assert!(
            (settle.progress.fraction()
                - 4.0
                    / ik_tuning().continuity_sample_hz.hertz()
                    / ik_tuning().settle_step_seconds.seconds())
            .abs()
                < 0.0001
        );
        assert_eq!(
            FootTargetSpeeds::configured()
                .unwrap()
                .settle(settle)
                .metres_per_second(),
            (ik_tuning().raised_settle_step_metres.metres()
                * ik_tuning().continuity_sample_hz.hertz())
        );
        assert!(
            (ik_tuning().raised_settle_step_metres.metres()
                * ik_tuning().continuity_sample_hz.hertz())
                < (ik_tuning().airborne_release_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz())
        );
        assert!(
            (ik_tuning().raised_settle_step_metres.metres()
                * ik_tuning().continuity_sample_hz.hertz())
                / ik_tuning().continuity_sample_hz.hertz()
                * ik_tuning().maximum_knee_target_amplification.factor()
                + ik_tuning().raised_settle_pelvis_knee_budget_metres.metres()
                < ik_tuning().maximum_knee_step_metres.metres()
        );
        assert!(!terrain_ik_is_required(
            crate::animation::TerrainIkMode::Disabled,
            false,
            false
        ));
    }

    #[test]
    fn immediate_restart_cancels_settle_without_waiting_for_release_targets() {
        let settle = LocomotionSettleState {
            support_foot: LeadFoot::Right,
            swing_start: Vec3::ZERO,
            capture_point: Vec3::Z,
            landing_target: Vec3::NEG_Z,
            progress: SettleProgress::from_progress(0.4).unwrap(),
            elapsed_seconds: AnimationElapsed::from_seconds(0.1).unwrap(),
            raised_handoff: false,
        };
        let mut memory = LegIkMemory {
            settle: Some(settle),
            left_foot_plant: FootPlant::Acquiring(Vec3::new(-0.1, 0.085, -0.8)),
            right_foot_plant: FootPlant::Acquiring(Vec3::new(0.1, 0.085, -0.9)),
            left_last_rendered_world: Some(Vec3::new(-0.1, 0.14, -0.4)),
            right_last_rendered_world: Some(Vec3::new(0.1, 0.15, -0.5)),
            left_last_rendered_owner: Some(Vec3::new(-0.1, -0.8, -0.4)),
            right_last_rendered_owner: Some(Vec3::new(0.1, -0.79, -0.5)),
            left_release_active: true,
            right_release_active: true,
            ..default()
        };
        let restarted_velocity = Vec3::new(2.0, 4.0, -3.0);

        cancel_settle_for_restart(&mut memory, restarted_velocity);

        assert!(memory.settle.is_none());
        assert_eq!(
            memory.recent_movement_velocity,
            restarted_velocity.with_y(0.0)
        );
        assert!(memory.left_release_active && memory.right_release_active);
        assert!(
            memory.left_foot_plant.target().is_none() && memory.right_foot_plant.target().is_none()
        );
        assert_eq!(
            memory.left_foot_world_target,
            memory.left_last_rendered_world
        );
        assert_eq!(
            memory.right_foot_world_target,
            memory.right_last_rendered_world
        );
        assert_eq!(memory.left_foot_target, memory.left_last_rendered_owner);
        assert_eq!(memory.right_foot_target, memory.right_last_rendered_owner);
        assert_eq!(
            memory.left_transition_support_weight,
            Some(UnitInterval::from_fraction(0.0).unwrap())
        );
        assert_eq!(
            memory.right_transition_support_weight,
            Some(UnitInterval::from_fraction(0.0).unwrap())
        );
    }

    #[test]
    fn owner_discontinuity_clears_both_plans_and_all_frozen_trajectory_metadata() {
        let mut memory = LegIkMemory {
            left_planned_contact: Some(Vec3::new(-0.1, 0.2, -1.0)),
            right_planned_contact: Some(Vec3::new(0.1, 0.3, -2.0)),
            left_planned_contact_start: Some(Vec3::new(-0.1, 0.8, 0.0)),
            right_planned_contact_start: Some(Vec3::new(0.1, 0.7, -0.5)),
            left_planned_contact_phase_start: Some(CycleDelta::from_cycles(0.8).unwrap()),
            right_planned_contact_phase_start: Some(CycleDelta::from_cycles(0.3).unwrap()),
            ..default()
        };

        clear_all_planned_contact_metadata(&mut memory);

        assert!(memory.left_planned_contact.is_none());
        assert!(memory.right_planned_contact.is_none());
        assert!(memory.left_planned_contact_start.is_none());
        assert!(memory.right_planned_contact_start.is_none());
        assert!(memory.left_planned_contact_phase_start.is_none());
        assert!(memory.right_planned_contact_phase_start.is_none());
    }

    #[test]
    fn cancelled_settle_returns_to_run_inside_the_existing_knee_budget() {
        assert_eq!(
            FootTargetSpeeds::configured()
                .unwrap()
                .run_sample(false, true)
                .metres_per_second(),
            (ik_tuning().airborne_release_step_metres.metres()
                * ik_tuning().continuity_sample_hz.hertz())
        );
        assert_eq!(
            FootTargetSpeeds::configured()
                .unwrap()
                .run_sample(false, false)
                .metres_per_second(),
            (ik_tuning().run_airborne_owner_step_metres.metres()
                * ik_tuning().continuity_sample_hz.hertz())
        );

        // Native terrain-tap-restart-crossfade frames 39 -> 40: the settle
        // swing is cancelled as the owner resumes 5.5 m/s. The ordinary Run
        // budget moved the reachable ankle only 9.3 cm but amplified its
        // near-extension knee by 12.8 cm. The first-sample settle budget keeps
        // the transported analytic chain below the same 10 cm contract.
        let previous_root = Vec3::new(0.0, 3.0130908, -1.71875);
        let current_root = Vec3::new(0.0, 3.017059, -1.8046875);
        let previous_hip = Vec3::new(0.10195288, 3.057775, -1.7341061);
        let previous_knee = Vec3::new(0.13492808, 2.5361009, -1.7145816);
        let previous_ankle = Vec3::new(0.13445835, 2.1369128, -1.554793);
        let current_hip = Vec3::new(0.10195502, 3.0623627, -1.817662);
        let desired_ankle = Vec3::new(0.13222283, 2.1976547, -1.6857854);
        let previous_owner = previous_ankle - previous_root;
        let resolved_ankle = advance_run_airborne_world_target(
            Some(previous_owner),
            desired_ankle,
            current_root,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            FootTargetSpeeds::configured()
                .unwrap()
                .run_sample(false, true),
            |_| checked_scene_elevation(-100.0).map(|height| Some(FootClearanceFloor::Minimum(height))),
        )
        .unwrap();
        assert!(
            (resolved_ankle - current_root).distance(previous_owner)
                <= (ik_tuning().airborne_release_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz())
                    / ik_tuning().continuity_sample_hz.hertz()
                    + 0.0001
        );

        let upper_length = previous_hip.distance(previous_knee);
        let lower_length = previous_knee.distance(previous_ankle);
        let previous_end_direction = (previous_ankle - previous_hip).normalize();
        let previous_pole = (previous_knee - previous_hip)
            .reject_from_normalized(previous_end_direction)
            .normalize();
        let next_end_direction = (resolved_ankle - current_hip).normalize();
        let pole = transported_terrain_pole(
            Some(previous_pole),
            Some(previous_end_direction),
            next_end_direction,
            previous_pole,
        )
        .expect("the settle knee pole remains transportable on restart");
        let solution = solve_two_bone_with_reach(
            TwoBoneChain::new(
                current_hip,
                previous_knee,
                previous_ankle,
                NonnegativeLength::from_metres(upper_length).unwrap(),
                NonnegativeLength::from_metres(lower_length).unwrap(),
                pole,
            ),
            resolved_ankle,
            maximum_reach(
                NonnegativeLength::from_metres(upper_length).unwrap(),
                NonnegativeLength::from_metres(lower_length).unwrap(),
            ),
        )
        .expect("the bounded restart target remains reachable");
        let knee_root_relative_step =
            (solution.knee - current_root).distance(previous_knee - previous_root);
        assert!(knee_root_relative_step <= ik_tuning().maximum_knee_step_metres.metres());
    }

    #[test]
    fn toe_aware_settle_height_couples_ankle_clearance_to_the_visible_toe_lever() {
        // Native stop frame 25 had an 11.54 cm ankle clearance but a -1.72 cm
        // toe clearance. Preserve that measured 13.26 cm lever while asking
        // the next target for the strict +1.1 cm transition toe floor.
        let rendered_ankle = Vec3::new(0.14, 0.1154449, -1.5);
        let rendered_toe = Vec3::new(0.14, -0.017214656, -1.62);
        let minimum = toe_aware_clearance_floor(
            rendered_ankle,
            rendered_toe,
            ScenePlanPoint::from_metres(Vec2::new(0.14, -1.7)).unwrap(),
            ik_tuning().terrain_transition_flight_toe_clearance_metres,
            |_| Some(SupportElevation::ZERO),
        )
        .unwrap()
        .unwrap();
        assert!((minimum.minimum_height().unwrap().metres() - 0.14365956).abs() <= 0.000001);
        let rotation_safe_clearance = transition_toe_clearance_with_rotation_margin(
            rendered_ankle,
            rendered_toe,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
        );
        let rotation_safe_clearance = rotation_safe_clearance.unwrap().metres();
        assert!(rotation_safe_clearance > 0.03);
        let resolved = advance_run_airborne_world_target(
            Some(rendered_ankle),
            Vec3::new(0.14, 0.05, -1.55),
            Vec3::ZERO,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                ik_tuning().airborne_release_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz(),
            )
            .unwrap(),
            |_| Ok(Some(minimum)),
        )
        .unwrap();
        assert!(resolved.y + 0.000001 >= minimum.minimum_height().unwrap().metres());
        assert!(
            resolved.distance(rendered_ankle)
                <= (ik_tuning().airborne_release_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz())
                    / ik_tuning().continuity_sample_hz.hertz()
                    + 0.0001
        );

        let contact_minimum = toe_aware_clearance_floor(
            Vec3::new(0.21, 0.085, 0.0),
            Vec3::new(0.21, -0.015733838, -0.1),
            ScenePlanPoint::from_metres(Vec2::new(0.21, 0.0)).unwrap(),
            ik_tuning().terrain_contact_toe_clearance_metres,
            |_| Some(SupportElevation::ZERO),
        )
        .unwrap()
        .unwrap();
        assert!(contact_minimum.minimum_height().unwrap().metres() > measured_ankle_sole_offset_metres().metres());
        assert!((contact_minimum.minimum_height().unwrap().metres() - 0.09173384).abs() <= 0.000001);
    }

    #[test]
    fn airborne_settle_support_lands_atomically_once_contact_is_reachable() {
        let contact = Vec3::new(0.1, 0.09173384, -0.5);
        let previous = contact + Vec3::Y * 0.04;
        let contact_candidate = advance_run_airborne_world_target(
            Some(previous),
            contact,
            Vec3::ZERO,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                ik_tuning().airborne_release_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz(),
            )
            .unwrap(),
            |_| checked_scene_elevation(measured_ankle_sole_offset_metres().metres()).map(|height| Some(FootClearanceFloor::Minimum(height))),
        )
        .unwrap();
        assert!(contact_candidate.distance_squared(contact) <= 0.000001);

        let flight_candidate = advance_run_airborne_world_target(
            Some(previous),
            contact,
            Vec3::ZERO,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                ik_tuning().airborne_release_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz(),
            )
            .unwrap(),
            |_| checked_scene_elevation(0.14365956).map(|height| Some(FootClearanceFloor::Minimum(height))),
        )
        .unwrap();
        assert!(flight_candidate.distance_squared(contact) > 0.000001);
        // The production branch selects contact_candidate in this state, so
        // the same sample can report truthful support instead of reclamping
        // to the airborne floor forever.
        assert_eq!(contact_candidate, contact);
    }

    #[test]
    fn terminal_contact_preparation_preserves_the_visible_pelvis_shift() {
        let left = Vec3::new(-0.1, 0.085, 0.0);
        let right = Vec3::new(0.1, 0.085, -0.4);
        let mut memory = LegIkMemory {
            settle: Some(LocomotionSettleState {
                support_foot: LeadFoot::Left,
                swing_start: right,
                capture_point: Vec3::NEG_Z,
                landing_target: right,
                progress: SettleProgress::from_progress(1.0).unwrap(),
                elapsed_seconds: AnimationElapsed::from_seconds(0.5).unwrap(),
                raised_handoff: false,
            }),
            pelvis_shift: SignedLength::from_metres(-0.21).unwrap(),
            left_last_rendered_world: Some(left),
            right_last_rendered_world: Some(right),
            ..default()
        };

        assert!(prepare_terminal_settle_contacts(
            &mut memory,
            Vec3::ZERO,
            Quat::IDENTITY,
            |_| Some(SupportElevation::ZERO),
        ));
        assert_eq!(memory.terminal_contacts.shift().metres(), -0.21);
        assert!(matches!(
            memory.terminal_contacts,
            TerminalContacts::Prepared(TerminalReach {
                target_shift: None,
                ..
            })
        ));
    }

    #[test]
    fn completed_settle_promotes_both_targets_to_stable_idle_plants() {
        let mut memory = LegIkMemory {
            settle: Some(LocomotionSettleState {
                support_foot: LeadFoot::Left,
                swing_start: Vec3::ZERO,
                capture_point: Vec3::NEG_Z,
                landing_target: Vec3::new(0.2, 0.085, -0.4),
                progress: SettleProgress::from_progress(1.0).unwrap(),
                elapsed_seconds: AnimationElapsed::from_seconds(0.4).unwrap(),
                raised_handoff: false,
            }),
            recent_movement_velocity: Vec3::NEG_Z * 5.5,
            left_foot_plant: FootPlant::Acquiring(Vec3::NEG_Z),
            left_foot_world_target: Some(Vec3::new(-0.2, 0.085, 0.0)),
            right_foot_world_target: Some(Vec3::new(0.2, 0.085, -0.5)),
            left_release_active: true,
            right_release_active: true,
            left_support_exhausted_until_flight: true,
            left_terrain_pole_world: Some(Vec3::Z),
            ..default()
        };

        finish_settle_for_idle(&mut memory);

        assert!(memory.settle.is_none());
        assert_eq!(memory.recent_movement_velocity, Vec3::ZERO);
        assert_eq!(
            memory.left_foot_plant.target(),
            memory.left_foot_world_target
        );
        assert_eq!(
            memory.right_foot_plant.target(),
            memory.right_foot_world_target
        );
        assert!(memory.left_foot_plant.is_acquired() && memory.right_foot_plant.is_acquired());
        assert_eq!(
            memory.left_transition_support_weight,
            Some(UnitInterval::from_fraction(1.0).unwrap())
        );
        assert_eq!(
            memory.right_transition_support_weight,
            Some(UnitInterval::from_fraction(1.0).unwrap())
        );
        assert!(!memory.left_support_exhausted_until_flight);
        assert!(!memory.right_support_exhausted_until_flight);
        assert!(!memory.left_release_active && !memory.right_release_active);
        assert_eq!(memory.left_terrain_pole_world, Some(Vec3::Z));
    }

    #[test]
    fn terminal_settle_with_idle_followers_finishes_on_dual_terrain_contacts() {
        let settle = advance_settle_state(
            LocomotionSettleState {
                support_foot: LeadFoot::Left,
                swing_start: Vec3::ZERO,
                capture_point: Vec3::NEG_Z,
                landing_target: Vec3::new(0.2, 0.085, -0.4),
                progress: SettleProgress::from_progress(0.99).unwrap(),
                elapsed_seconds: AnimationElapsed::from_seconds(0.4).unwrap(),
                raised_handoff: false,
            },
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
        )
        .unwrap();
        let mut memory = LegIkMemory {
            settle: Some(settle),
            left_foot_world_target: Some(Vec3::new(-0.12, 0.160, -0.2)),
            right_foot_world_target: Some(Vec3::new(0.12, 0.080, -0.5)),
            left_release_active: false,
            right_release_active: false,
            ..default()
        };
        assert!(settle_is_terminal(&memory));
        assert!(prepare_terminal_settle_contacts(
            &mut memory,
            Vec3::ZERO,
            Quat::IDENTITY,
            |_| Some(SupportElevation::ZERO),
        ));
        assert!(memory.settle.is_some());
        assert!(!terminal_settle_contacts_are_rendered(&memory, |_| Some(SupportElevation::ZERO),));
        memory.left_last_rendered_world = memory.left_foot_world_target;
        memory.right_last_rendered_world = memory.right_foot_world_target;
        memory.left_last_rendered_toe_world = Some(Vec3::new(-0.12, 0.005, -0.3));
        memory.right_last_rendered_toe_world = Some(Vec3::new(0.12, 0.005, -0.6));
        assert!(terminal_settle_contacts_are_rendered(&memory, |_| Some(SupportElevation::ZERO),));
        finish_settle_for_idle(&mut memory);
        assert!(memory.settle.is_none());
        assert_eq!(
            memory.left_foot_plant.target().unwrap().y,
            measured_ankle_sole_offset_metres().metres()
        );
        assert_eq!(
            memory.right_foot_plant.target().unwrap().y,
            measured_ankle_sole_offset_metres().metres()
        );
        assert_eq!(
            memory.left_support_weight,
            Some(UnitInterval::from_fraction(1.0).unwrap())
        );
        assert_eq!(
            memory.right_support_weight,
            Some(UnitInterval::from_fraction(1.0).unwrap())
        );
    }

    #[test]
    fn terminal_settle_lowers_shared_root_until_both_contacts_are_reachable() {
        // Production-like geometry from the stop capture: the ankle target is
        // at terrain contact, but the restored idle hip leaves the chain more
        // than eight centimetres short. Terminal settle must keep requesting
        // a bounded shared-root drop instead of promoting false support.
        let upper = Vec3::new(-0.10, 3.08, -1.00);
        let target = Vec3::new(-0.12, 2.13, -1.38);
        let reach = ReachLimit::Fixed(NonnegativeLength::from_metres(0.953).unwrap());
        let required = reach
            .terrain_pelvis_shift(upper, target, UnitInterval::ONE)
            .unwrap()
            .metres();
        assert!(required < -0.05);

        let mut shift = 0.0;
        let base_root = Vec3::new(0.0, 1.0, 0.0);
        for _ in 0..16 {
            let next = advance_pelvis_shift(
                SignedLength::from_metres(shift).unwrap(),
                SignedLength::from_metres(required).unwrap(),
                AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz())
                    .unwrap(),
            )
            .unwrap()
            .metres();
            assert!(
                (next - shift).abs() <= maximum_pelvis_correction_step_metres().metres() + 0.0001
            );
            shift = next;
            // Sparse idle FK may preserve the preceding procedural local.
            // Absolute application from the frozen base must still converge,
            // rather than repeatedly adding the retained scalar.
            let applied_root = base_root + Vec3::Y * shift;
            assert!((applied_root.y - (base_root.y + shift)).abs() <= 0.0001);
        }
        assert!((shift - required).abs() <= 0.0001);
        let applied_root = base_root + Vec3::Y * shift;
        assert!((applied_root.y - (base_root.y + required)).abs() <= 0.0001);

        let lowered_upper = upper + Vec3::Y * shift;
        assert!(lowered_upper.distance(target) <= reach.native_metres().unwrap() + 0.0001);

        let memory = LegIkMemory {
            settle: Some(LocomotionSettleState {
                support_foot: LeadFoot::Left,
                swing_start: target,
                capture_point: target,
                landing_target: target,
                progress: SettleProgress::from_progress(1.0).unwrap(),
                elapsed_seconds: AnimationElapsed::from_seconds(1.0).unwrap(),
                raised_handoff: false,
            }),
            left_foot_world_target: Some(target),
            right_foot_world_target: Some(target + Vec3::X * 0.24),
            left_last_rendered_world: Some(target + Vec3::Y * 0.075),
            right_last_rendered_world: Some(target + Vec3::X * 0.24),
            left_last_rendered_toe_world: Some(target + Vec3::Y * 0.075),
            right_last_rendered_toe_world: Some(target + Vec3::X * 0.24),
            ..default()
        };
        assert!(!terminal_settle_contacts_are_rendered(&memory, |_| Some(SupportElevation::from_metres(2.045).unwrap())));
    }

    #[test]
    fn terminal_prepared_contacts_own_both_solves_despite_zero_idle_cadence() {
        let left = Vec3::new(-0.12, measured_ankle_sole_offset_metres().metres(), -0.2);
        let right = Vec3::new(0.12, measured_ankle_sole_offset_metres().metres(), -0.5);
        for plant in [left, right] {
            let (logical_weight, solve_plant) =
                terminal_contact_solve_ownership(true, UnitInterval::ZERO, Some(plant));
            assert_eq!(logical_weight, UnitInterval::ONE);
            assert_eq!(solve_plant, Some(plant));

            let restored_idle_fk = plant + Vec3::new(0.0, 0.12, 0.4);
            assert!(!ordinary_plant_requires_clear(
                logical_weight,
                true,
                solve_plant,
                restored_idle_fk,
            ));
            let (_, next_tick_plant) =
                terminal_contact_solve_ownership(true, UnitInterval::ZERO, solve_plant);
            assert_eq!(next_tick_plant, Some(plant));
            assert_eq!(next_tick_plant.unwrap().distance(plant), 0.0);
        }

        assert_eq!(
            terminal_contact_solve_ownership(false, UnitInterval::ZERO, Some(left)),
            (UnitInterval::ZERO, Some(left))
        );
    }

    #[test]
    fn terminal_contact_preparation_prefers_last_rendered_stance_over_stale_solve() {
        let stale_left = Vec3::new(-0.12, 0.4, -1.245);
        let stale_right = Vec3::new(0.12, 0.4, -0.900);
        let visible_left = Vec3::new(-0.116, 0.3, -1.342);
        let visible_right = Vec3::new(0.118, 0.3, -0.784);
        let mut memory = LegIkMemory {
            settle: Some(LocomotionSettleState {
                support_foot: LeadFoot::Left,
                swing_start: visible_right,
                capture_point: Vec3::ZERO,
                landing_target: stale_right,
                progress: SettleProgress::from_progress(1.0).unwrap(),
                elapsed_seconds: AnimationElapsed::from_seconds(1.0).unwrap(),
                raised_handoff: false,
            }),
            left_foot_world_target: Some(stale_left),
            right_foot_world_target: Some(stale_right),
            left_last_rendered_world: Some(visible_left),
            right_last_rendered_world: Some(visible_right),
            ..default()
        };

        assert!(prepare_terminal_settle_contacts(
            &mut memory,
            Vec3::ZERO,
            Quat::IDENTITY,
            |_| Some(SupportElevation::ZERO),
        ));
        let left = memory.left_foot_world_target.unwrap();
        let right = memory.right_foot_world_target.unwrap();
        assert_eq!(left.xz(), visible_left.xz());
        assert_eq!(right.xz(), visible_right.xz());
        assert_eq!(left.y, measured_ankle_sole_offset_metres().metres());
        assert_eq!(right.y, measured_ankle_sole_offset_metres().metres());
        assert_eq!(memory.left_foot_plant.target(), Some(left));
        assert_eq!(memory.right_foot_plant.target(), Some(right));

        memory.left_last_rendered_world = Some(visible_left + Vec3::Z * 0.2);
        memory.right_last_rendered_world = Some(visible_right - Vec3::Z * 0.2);
        assert!(prepare_terminal_settle_contacts(
            &mut memory,
            Vec3::ZERO,
            Quat::IDENTITY,
            |_| Some(SupportElevation::ZERO),
        ));
        assert_eq!(memory.left_foot_world_target, Some(left));
        assert_eq!(memory.right_foot_world_target, Some(right));
    }

    #[test]
    fn finished_terminal_reach_persists_through_held_idle() {
        let left = Vec3::new(-0.12, measured_ankle_sole_offset_metres().metres(), -0.2);
        let right = Vec3::new(0.12, measured_ankle_sole_offset_metres().metres(), -0.5);
        let mut memory = LegIkMemory {
            settle: Some(LocomotionSettleState {
                support_foot: LeadFoot::Left,
                swing_start: right,
                capture_point: Vec3::ZERO,
                landing_target: right,
                progress: SettleProgress::from_progress(1.0).unwrap(),
                elapsed_seconds: AnimationElapsed::from_seconds(1.0).unwrap(),
                raised_handoff: false,
            }),
            left_foot_world_target: Some(left),
            right_foot_world_target: Some(right),
            left_foot_plant: FootPlant::Acquiring(left),
            right_foot_plant: FootPlant::Acquiring(right),
            terminal_contacts: TerminalContacts::Prepared(TerminalReach {
                base_translation: None,
                shift: SignedLength::from_metres(-0.08).unwrap(),
                target_shift: Some(SignedLength::from_metres(-0.08).unwrap()),
            }),
            ..default()
        };

        finish_settle_for_idle(&mut memory);
        assert_eq!(memory.pelvis_shift.metres(), -0.08);
        for _ in 0..20 {
            memory.pelvis_shift = advance_pelvis_shift(
                memory.pelvis_shift,
                SignedLength::from_metres(-0.08).unwrap(),
                AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz())
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(memory.pelvis_shift.metres(), -0.08);
            assert!(memory.settle.is_none());
            assert_eq!(memory.left_foot_plant.target(), Some(left));
            assert_eq!(memory.right_foot_plant.target(), Some(right));
            assert_eq!(
                memory.left_support_weight,
                Some(UnitInterval::from_fraction(1.0).unwrap())
            );
            assert_eq!(
                memory.right_support_weight,
                Some(UnitInterval::from_fraction(1.0).unwrap())
            );
        }
    }

    #[test]
    fn stop_settle_seeds_from_visible_reach_limited_feet() {
        let invisible_goal = Vec3::new(-0.178, 1.934, 0.0);
        let prior_rendered = Vec3::new(-0.178, 1.934, -0.253);
        let restored_idle_fk = Vec3::new(-0.178, 1.934, -1.255);
        let landing = Vec3::new(-0.099, 2.085, -0.871);
        let mut memory = LegIkMemory {
            left_foot_world_target: Some(invisible_goal),
            left_foot_target: Some(invisible_goal),
            left_last_rendered_world: Some(prior_rendered),
            left_release_active: true,
            ..default()
        };

        let visible = settle_visible_foot(memory.left_last_rendered_world, Some(restored_idle_fk));

        seed_settle_from_rendered_feet(&mut memory, visible, None, Vec3::ZERO, Quat::IDENTITY);
        assert_eq!(visible, Some(prior_rendered));
        assert_eq!(memory.left_foot_world_target, Some(prior_rendered));
        let next = advance_foot_target_at_speed(
            memory.left_foot_world_target,
            landing,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                ik_tuning().airborne_release_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz(),
            )
            .unwrap(),
        );
        assert!(
            next.distance(prior_rendered)
                <= (ik_tuning().airborne_release_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz())
                    / ik_tuning().continuity_sample_hz.hertz()
                    + 0.0001
        );
        assert!(next.distance_squared(landing) > 0.000001);
    }

    #[test]
    fn stop_settle_retains_the_selected_rendered_support() {
        let left = Vec3::new(-0.1, 2.085, -0.262);
        let right = Vec3::new(0.1, 2.085, -0.643);
        let stale_plan = Vec3::new(-0.1, 2.085, -1.829);
        let mut memory = LegIkMemory {
            left_planned_contact: Some(stale_plan),
            right_planned_contact: Some(stale_plan),
            ..default()
        };

        seed_settle_from_rendered_feet(
            &mut memory,
            Some(left),
            Some(right),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        retain_settle_support(&mut memory, LeadFoot::Right, Some(left), Some(right), true);

        assert_eq!(memory.right_foot_plant.target(), Some(right));
        assert!(memory.right_foot_plant.is_acquired());
        assert_eq!(
            memory.right_transition_support_weight,
            Some(UnitInterval::from_fraction(1.0).unwrap())
        );
        assert!(memory.left_planned_contact.is_none());
        assert!(memory.right_planned_contact.is_none());
    }

    #[test]
    fn stop_settle_visible_airborne_support_remains_unacquired() {
        let airborne_right = Vec3::new(0.1, 2.16, -0.64);
        let mut memory = LegIkMemory {
            right_support_weight: Some(UnitInterval::from_fraction(0.0).unwrap()),

            ..default()
        };

        retain_settle_support(
            &mut memory,
            LeadFoot::Right,
            None,
            Some(airborne_right),
            false,
        );

        assert_eq!(memory.right_foot_plant.target(), Some(airborne_right));
        assert!(!memory.right_foot_plant.is_acquired());
        assert_eq!(
            memory.right_transition_support_weight,
            Some(UnitInterval::from_fraction(1.0).unwrap())
        );
    }

    #[test]
    fn stop_settle_uses_current_fk_only_without_a_rendered_snapshot() {
        let restored_idle_fk = Vec3::new(0.1, 2.085, -0.422);
        assert_eq!(
            settle_visible_foot(None, Some(restored_idle_fk)),
            Some(restored_idle_fk)
        );
    }

    #[test]
    fn truthful_reported_support_does_not_erase_solver_ownership() {
        let mut memory = LegIkMemory {
            left_support_weight: Some(UnitInterval::from_fraction(1.0).unwrap()),
            left_transition_support_weight: Some(UnitInterval::from_fraction(1.0).unwrap()),
            ..default()
        };
        memory.left_support_weight = Some(UnitInterval::from_fraction(0.0).unwrap());
        assert_eq!(
            memory.left_support_weight,
            Some(UnitInterval::from_fraction(0.0).unwrap())
        );
        assert_eq!(
            memory.left_transition_support_weight,
            Some(UnitInterval::from_fraction(1.0).unwrap())
        );
    }

    #[test]
    fn repeated_fixed_tick_leaves_advanced_ik_memory_identical() {
        let mut memory = LegIkMemory {
            left_foot_plant: FootPlant::Acquired(Vec3::new(-0.1, 0.085, -2.0)),
            left_foot_world_target: Some(Vec3::new(-0.1, 0.085, -2.0)),
            left_support_weight: Some(UnitInterval::from_fraction(0.4).unwrap()),
            left_transition_support_weight: Some(UnitInterval::from_fraction(0.4).unwrap()),
            left_release_active: false,
            evaluation_tick: Some(AnimationSampleTick::from(91u64)),
            ..default()
        };
        let advanced = memory;
        if !repeated_fixed_tick_skips_ik(true, false) {
            memory.left_foot_plant = FootPlant::Absent;
            memory.left_support_weight = Some(UnitInterval::from_fraction(0.0).unwrap());
            memory.left_transition_support_weight = Some(UnitInterval::from_fraction(0.0).unwrap());
            memory.left_release_active = true;
        }
        assert_eq!(memory, advanced);
        assert!(!repeated_fixed_tick_skips_ik(true, true));
        assert!(!repeated_fixed_tick_skips_ik(false, false));
    }

    #[test]
    fn acquired_plant_survives_authored_fk_divergence_until_support_exit() {
        let plant = Vec3::new(-0.1, 0.1, -2.0);
        let divergent_authored_swing = Vec3::new(-0.1, 0.6, 0.5);
        assert!(!ordinary_plant_requires_clear(
            UnitInterval::from_fraction(0.2).unwrap(),
            true,
            Some(plant),
            divergent_authored_swing,
        ));
        assert!(ordinary_plant_requires_clear(
            UnitInterval::from_fraction(0.0).unwrap(),
            true,
            Some(plant),
            divergent_authored_swing,
        ));
        assert!(ordinary_plant_requires_clear(
            UnitInterval::from_fraction(0.2).unwrap(),
            false,
            Some(plant),
            divergent_authored_swing,
        ));
    }

    #[test]
    fn acquired_support_waits_for_replacement_contact_not_phase_exit() {
        let plant = Vec3::new(-0.1, 0.085, -2.0);
        let authored_swing = Vec3::new(-0.1, 0.5, -1.0);

        let retained = coordinated_support_weight(
            LocomotionGait::Walk,
            UnitInterval::from_fraction(0.0).unwrap(),
            true,
            false,
        );
        assert_eq!(retained.fraction(), 1.0);
        assert!(!ordinary_plant_requires_clear(
            retained,
            true,
            Some(plant),
            authored_swing,
        ));

        let handed_off = coordinated_support_weight(
            LocomotionGait::Walk,
            UnitInterval::from_fraction(0.0).unwrap(),
            true,
            true,
        );
        assert_eq!(handed_off.fraction(), 0.0);
        assert!(ordinary_plant_requires_clear(
            handed_off,
            true,
            Some(plant),
            authored_swing,
        ));

        // Explicit reach failure clears the plant before coordination, so the
        // phase-independent owner cannot retain an unreachable footprint.
        let reach_released = coordinated_support_weight(
            LocomotionGait::Walk,
            UnitInterval::from_fraction(0.0).unwrap(),
            false,
            false,
        );
        assert_eq!(reach_released.fraction(), 0.0);
        assert!(ordinary_plant_requires_clear(
            reach_released,
            true,
            None,
            authored_swing,
        ));

        let run_flight = coordinated_support_weight(
            LocomotionGait::Run,
            UnitInterval::from_fraction(0.0).unwrap(),
            true,
            false,
        );
        assert_eq!(run_flight.fraction(), 0.0);
        assert!(ordinary_plant_requires_clear(
            run_flight,
            true,
            Some(plant),
            authored_swing,
        ));

        assert_eq!(
            run_toe_off_support_weight(
                LocomotionGait::Run,
                UnitInterval::from_fraction(0.773).unwrap(),
                true,
                false
            ),
            (false, UnitInterval::from_fraction(0.773).unwrap())
        );
        assert_eq!(
            run_toe_off_support_weight(
                LocomotionGait::Run,
                UnitInterval::from_fraction(0.0).unwrap(),
                true,
                true
            ),
            (true, UnitInterval::from_fraction(0.0).unwrap())
        );
        assert_eq!(
            run_retained_support_through_lobe_edge(
                LocomotionGait::Run,
                UnitInterval::from_fraction(0.0).unwrap(),
                true,
                false,
            )
            .fraction(),
            1.0
        );
        assert_eq!(
            run_retained_support_through_lobe_edge(
                LocomotionGait::Run,
                UnitInterval::from_fraction(0.26).unwrap(),
                true,
                false,
            )
            .fraction(),
            1.0
        );
        assert_eq!(
            run_retained_support_through_lobe_edge(
                LocomotionGait::Run,
                UnitInterval::from_fraction(0.0).unwrap(),
                true,
                true,
            )
            .fraction(),
            0.0
        );
        assert!(
            run_swing_clearance(
                CycleDelta::from_cycles(0.82).unwrap(),
                Some(UnitInterval::from_fraction(0.0).unwrap())
            )
            .unwrap()
            .metres()
                >= 0.05
        );
        let phase_step = gait_cycle_phase_delta(
            run_locomotion_profile(),
            NonnegativeSpeed::from_metres_per_second(5.5).unwrap(),
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
        )
        .unwrap()
        .cycles();
        let samples_to_opposite_acquisition = ((0.891_f32 - 0.698) / phase_step).ceil();
        let unsupported_seconds = (samples_to_opposite_acquisition - 1.0).max(0.0)
            / ik_tuning().continuity_sample_hz.hertz();
        assert!(unsupported_seconds <= 0.12);
        assert_eq!(
            run_toe_off_support_weight(
                LocomotionGait::Run,
                UnitInterval::from_fraction(0.260).unwrap(),
                false,
                true
            ),
            (false, UnitInterval::from_fraction(0.260).unwrap())
        );
        assert_eq!(
            run_toe_off_support_weight(
                LocomotionGait::Walk,
                UnitInterval::from_fraction(0.260).unwrap(),
                true,
                true
            ),
            (false, UnitInterval::from_fraction(0.260).unwrap())
        );
        for rising_phase in [0.853, 0.877, 0.901, 0.926] {
            assert!(!run_is_at_support_exit(
                CyclePhase::from_cycles(rising_phase).unwrap(),
                LeadFoot::Left,
                run_locomotion_profile().support_phase_radius,
            ));
            assert_eq!(
                run_toe_off_support_weight(
                    LocomotionGait::Run,
                    UnitInterval::from_fraction(0.21).unwrap(),
                    true,
                    false
                ),
                (false, UnitInterval::from_fraction(0.21).unwrap())
            );
        }
        for retained_phase in [0.602, 0.626, 0.650] {
            assert!(!run_is_at_support_exit(
                CyclePhase::from_cycles(retained_phase).unwrap(),
                LeadFoot::Right,
                run_locomotion_profile().support_phase_radius,
            ));
        }
        assert!(!run_is_at_support_exit(
            CyclePhase::from_cycles(0.674).unwrap(),
            LeadFoot::Right,
            run_locomotion_profile().support_phase_radius,
        ));
        assert!(run_is_at_support_exit(
            CyclePhase::from_cycles(0.698).unwrap(),
            LeadFoot::Right,
            run_locomotion_profile().support_phase_radius,
        ));
        assert!(run_release_edge(false, true));
        assert!(run_release_edge(true, false));
        assert!(!run_release_edge(false, false));
        assert_eq!(
            run_toe_off_support_weight(
                LocomotionGait::Run,
                UnitInterval::from_fraction(0.0).unwrap(),
                true,
                true
            ),
            (true, UnitInterval::from_fraction(0.0).unwrap())
        );
        let (still_exhausted, suppressed_reentry) =
            support_after_exhausted_lobe(true, UnitInterval::from_fraction(0.2).unwrap());
        assert!(still_exhausted);
        assert_eq!(suppressed_reentry.fraction(), 0.0);
        let (cleared_in_flight, flight_weight) =
            support_after_exhausted_lobe(true, UnitInterval::from_fraction(0.0).unwrap());
        assert!(!cleared_in_flight);
        assert_eq!(flight_weight.fraction(), 0.0);
    }

    #[test]
    fn cold_start_clearance_solve_reports_procedural_release_ownership() {
        let authored = Vec3::new(-0.09, 1.90, -0.20);
        let terrain_cleared = authored + Vec3::Y * 0.095;
        assert!(unplanned_terrain_solve_requires_release(
            None,
            terrain_cleared,
            authored,
        ));
        assert!(!unplanned_terrain_solve_requires_release(
            Some(terrain_cleared),
            terrain_cleared,
            authored,
        ));
        assert!(!unplanned_terrain_solve_requires_release(
            None,
            authored + Vec3::Y * 0.02,
            authored,
        ));
    }

    #[test]
    fn frozen_plan_survives_support_entry_until_actual_acquisition() {
        let plan = Some(Vec3::new(0.1, 2.062, -5.548));
        assert!(!acquired_plan_can_clear(false));
        assert!(!acquisition_lobe_exited_without_contact(
            plan,
            false,
            Some(UnitInterval::from_fraction(0.2).unwrap()),
            UnitInterval::from_fraction(0.8).unwrap(),
        ));
        assert!(acquired_plan_can_clear(true));
        assert!(!acquisition_lobe_exited_without_contact(
            plan,
            true,
            Some(UnitInterval::from_fraction(0.2).unwrap()),
            UnitInterval::from_fraction(0.0).unwrap(),
        ));
        assert!(acquisition_lobe_exited_without_contact(
            plan,
            false,
            Some(UnitInterval::from_fraction(0.2).unwrap()),
            UnitInterval::from_fraction(0.0).unwrap(),
        ));
    }

    #[test]
    fn expired_late_plan_replaces_all_frozen_swing_metadata() {
        let mut contact = Some(Vec3::new(0.1, 2.06, -0.607));
        let mut start = Some(Vec3::new(0.1, 2.1, -0.268));
        let mut phase_start = Some(CycleDelta::from_cycles(0.418).unwrap());
        clear_planned_contact_metadata(&mut contact, &mut start, &mut phase_start);
        assert!(contact.is_none() && start.is_none() && phase_start.is_none());

        let visible = Vec3::new(0.1, 2.12, -2.3);
        let replacement = Vec3::new(0.1, 2.06, -5.548);
        // The .18 readiness boundary gives this metadata-only full-cycle
        // fixture a matching .866 start, preserving its approach span while
        // isolating frozen-state replacement from cadence tuning.
        let replacement_phase = 0.866;
        contact = Some(replacement);
        start = contact.map(|_| planned_contact_start(start, Some(visible), visible));
        phase_start = contact
            .map(|_| phase_start.unwrap_or(CycleDelta::from_cycles(replacement_phase).unwrap()));
        assert_eq!(start, Some(visible));
        assert_eq!(phase_start.map(CycleDelta::cycles), Some(replacement_phase));

        let ready = run_locomotion_profile().support_phase_radius.cycles()
            + ik_tuning().run_contact_chain_settle_phase.fraction();
        let first = run_contact_approach_progress(
            CycleDelta::from_cycles(replacement_phase).unwrap(),
            phase_start.unwrap(),
            CycleDelta::from_cycles(ready).unwrap(),
        )
        .unwrap()
        .fraction();
        let phase_step = gait_cycle_phase_delta(
            run_locomotion_profile(),
            NonnegativeSpeed::from_metres_per_second(5.5).unwrap(),
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
        )
        .unwrap()
        .cycles();
        let second = run_contact_approach_progress(
            CycleDelta::from_cycles(replacement_phase - phase_step).unwrap(),
            phase_start.unwrap(),
            CycleDelta::from_cycles(ready).unwrap(),
        )
        .unwrap()
        .fraction();
        assert_eq!(visible.lerp(replacement, first), visible);
        let world_step = visible.lerp(replacement, second).distance(visible);
        let root_step = 5.5 / ik_tuning().continuity_sample_hz.hertz();
        assert!(
            world_step - root_step
                <= ik_tuning()
                    .maximum_run_swing_root_relative_step_metres
                    .metres()
                    + 0.0001
        );
    }

    #[test]
    fn full_cycle_run_plan_has_no_progress_velocity_seam() {
        let start = Vec3::new(0.1163, 2.1378, -5.5478);
        let endpoint = Vec3::new(0.1199, 2.1157, -9.2572);
        let mut phase_to_contact = 0.856;
        let phase_start = phase_to_contact;
        let ready = run_locomotion_profile().support_phase_radius.cycles()
            + ik_tuning().run_contact_chain_settle_phase.fraction();
        let phase_step = gait_cycle_phase_delta(
            run_locomotion_profile(),
            NonnegativeSpeed::from_metres_per_second(5.5).unwrap(),
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
        )
        .unwrap()
        .cycles();
        let root_step = 5.5 / ik_tuning().continuity_sample_hz.hertz();
        let mut previous = start;
        while phase_to_contact > ready {
            phase_to_contact = (phase_to_contact - phase_step).max(ready);
            let progress = run_contact_approach_progress(
                CycleDelta::from_cycles(phase_to_contact).unwrap(),
                CycleDelta::from_cycles(phase_start).unwrap(),
                CycleDelta::from_cycles(ready).unwrap(),
            )
            .unwrap()
            .fraction();
            let target = start.lerp(endpoint, progress);
            let root_relative_step = (target.distance(previous) - root_step).max(0.0);
            assert!(root_relative_step <= 0.095);
            previous = target;
        }
        assert!(previous.distance(endpoint) < 0.0001);
    }

    #[test]
    fn run_toe_off_plan_survives_same_lobe_tail_and_next_ticks() {
        let start = Vec3::new(-0.1208, 1.9523, -7.4717);
        let endpoint = Vec3::new(-0.1210, 2.3074, -11.0308);
        let phase_start = 0.8674;
        let ready = run_locomotion_profile().support_phase_radius.cycles()
            + ik_tuning().run_contact_chain_settle_phase.fraction();
        let phase_step = gait_cycle_phase_delta(
            run_locomotion_profile(),
            NonnegativeSpeed::from_metres_per_second(5.5).unwrap(),
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
        )
        .unwrap()
        .cycles();
        assert_eq!(
            run_toe_off_support_weight(
                LocomotionGait::Run,
                UnitInterval::from_fraction(0.773).unwrap(),
                true,
                false
            ),
            (false, UnitInterval::from_fraction(0.773).unwrap())
        );
        let (toe_off, first_weight) = run_toe_off_support_weight(
            LocomotionGait::Run,
            UnitInterval::from_fraction(0.0).unwrap(),
            true,
            true,
        );
        assert!(toe_off);
        assert_eq!(first_weight.fraction(), 0.0);
        assert!(
            run_swing_clearance(
                CycleDelta::from_cycles(0.86).unwrap(),
                Some(UnitInterval::from_fraction(0.0).unwrap())
            )
            .unwrap()
            .metres()
                >= 0.05
        );

        let frozen = (Some(endpoint), Some(start), Some(phase_start));
        let mut exhausted = toe_off;
        let mut previous = start;
        for (index, raw_support) in [0.0, 0.0, 0.0].into_iter().enumerate() {
            let (next_exhausted, effective) = support_after_exhausted_lobe(
                exhausted,
                UnitInterval::from_fraction(raw_support).unwrap(),
            );
            exhausted = next_exhausted;
            assert_eq!(effective.fraction(), 0.0);
            assert_eq!(frozen, (Some(endpoint), Some(start), Some(phase_start)));
            let phase = phase_start - phase_step * (index as f32 + 1.0);
            let progress = run_contact_approach_progress(
                CycleDelta::from_cycles(phase).unwrap(),
                CycleDelta::from_cycles(phase_start).unwrap(),
                CycleDelta::from_cycles(ready).unwrap(),
            )
            .unwrap()
            .fraction();
            let target = start.lerp(endpoint, progress);
            let root_relative = (target.distance(previous)
                - 5.5 / ik_tuning().continuity_sample_hz.hertz())
            .max(0.0);
            assert!(root_relative <= 0.095);
            previous = target;
        }
    }

    #[test]
    fn raw_run_cycle_clears_toe_off_latch_and_reacquires_rising_plan() {
        let profile = run_locomotion_profile();
        let radius = profile.support_phase_radius.cycles();
        let endpoint = Vec3::new(0.1, measured_ankle_sole_offset_metres().metres(), -9.256);

        // The acquired right foot owns the post-contact shoulder until its
        // signed support exit, where toe-off exhausts only this lobe.
        let exit_phase = 0.698;
        let (_, exit_raw) =
            gait_support_weights(profile, CyclePhase::from_cycles(exit_phase).unwrap()).unwrap();
        assert!(run_is_at_support_exit(
            CyclePhase::from_cycles(exit_phase).unwrap(),
            LeadFoot::Right,
            CycleDelta::from_cycles(radius).unwrap(),
        ));
        let (mut exhausted, effective) =
            run_toe_off_support_weight(LocomotionGait::Run, exit_raw, true, true);
        assert!(exhausted);
        assert_eq!(effective.fraction(), 0.0);

        // The raw cadence, not the support value suppressed by the latch,
        // proves that this foot crossed flight and begins a fresh cycle.
        let flight_phase = 0.75;
        let (_, flight_raw) =
            gait_support_weights(profile, CyclePhase::from_cycles(flight_phase).unwrap()).unwrap();
        assert!(!terrain_leg_has_support(flight_raw));
        exhausted = exhausted_latch_after_raw_cadence(exhausted, flight_raw);
        assert!(!exhausted);

        // At the next rising shoulder the frozen endpoint has caught up in XZ
        // and sits on the semantic 5 cm flight floor. Unsuppressed raw support
        // makes the final bounded descent eligible, so contact can be acquired
        // by phase .35-.40 instead of remaining pinned above terrain.
        let rising_phase = 0.36;
        let (_, rising_raw) =
            gait_support_weights(profile, CyclePhase::from_cycles(rising_phase).unwrap()).unwrap();
        assert!(terrain_leg_has_support(rising_raw));
        assert!(run_plan_is_on_rising_support(
            LocomotionGait::Run,
            CyclePhase::from_cycles(rising_phase).unwrap(),
            LeadFoot::Right,
            CycleDelta::from_cycles(radius).unwrap(),
            rising_raw,
            Some(endpoint),
            false,
        ));
        let carried = exhausted_latch_after_raw_cadence(exhausted, rising_raw);
        let (mut next_exhausted, mut effective_support) =
            support_after_exhausted_lobe(carried, rising_raw);
        if run_plan_is_on_rising_support(
            LocomotionGait::Run,
            CyclePhase::from_cycles(rising_phase).unwrap(),
            LeadFoot::Right,
            CycleDelta::from_cycles(radius).unwrap(),
            rising_raw,
            Some(endpoint),
            false,
        ) {
            next_exhausted = false;
            effective_support = rising_raw;
        }
        assert!(!next_exhausted);
        assert!(terrain_leg_has_support(effective_support));

        let prior_floor =
            endpoint + Vec3::Y * ik_tuning().run_swing_minimum_sole_clearance_metres.metres();
        let reachable = run_contact_within_follower_step(
            Some(prior_floor),
            endpoint,
            Vec3::ZERO,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
        );
        assert!(reachable);
        let descent = run_support_descent(
            LocomotionGait::Run,
            CyclePhase::from_cycles(rising_phase).unwrap(),
            LeadFoot::Right,
            CycleDelta::from_cycles(radius).unwrap(),
            rising_raw,
            reachable,
        );
        assert!(descent.permits_descent());
        assert!(
            run_airborne_clearance(
                CyclePhase::from_cycles(rising_phase)
                    .unwrap()
                    .until_contact(LeadFoot::Right),
                Some(UnitInterval::from_fraction(1.0).unwrap()),
                descent,
            )
            .unwrap()
            .metres()
                <= 0.0
        );
        let lowered_y =
            FootClearanceFloor::Minimum(checked_scene_elevation(endpoint.y).unwrap())
                .apply(prior_floor, descent)
                .unwrap()
                .y;
        assert!(lowered_y < prior_floor.y);
        assert!((lowered_y - endpoint.y) == 0.0);
        let descended = advance_run_airborne_world_target(
            Some(prior_floor),
            Vec3::new(endpoint.x, lowered_y, endpoint.z),
            Vec3::ZERO,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                ik_tuning().run_airborne_owner_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz(),
            )
            .unwrap(),
            |_| checked_scene_elevation(endpoint.y).map(|height| Some(FootClearanceFloor::Minimum(height))),
        )
        .unwrap();
        assert!(descended.y < prior_floor.y);
        assert!(descended.distance(endpoint) <= 0.0001);
        assert_eq!(
            FootClearanceFloor::Minimum(checked_scene_elevation(prior_floor.y).unwrap())
                .apply(endpoint, FootDescent::RaiseOnly)
                .unwrap()
                .y,
            prior_floor.y
        );
        let (_, post_contact_raw) =
            gait_support_weights(profile, CyclePhase::from_cycles(0.65).unwrap()).unwrap();
        assert!(!run_support_descent(
            LocomotionGait::Run,
            CyclePhase::from_cycles(0.65).unwrap(),
            LeadFoot::Right,
            CycleDelta::from_cycles(radius).unwrap(),
            post_contact_raw,
            true,
        ).permits_descent());

        // Even if a low-rate consumer skipped the explicit flight sample, the
        // signed rising shoulder is an unambiguous new-lobe boundary.
        let (mut stale_latch, mut stale_support) = support_after_exhausted_lobe(true, rising_raw);
        assert!(stale_latch);
        if run_plan_is_on_rising_support(
            LocomotionGait::Run,
            CyclePhase::from_cycles(rising_phase).unwrap(),
            LeadFoot::Right,
            CycleDelta::from_cycles(radius).unwrap(),
            rising_raw,
            Some(endpoint),
            false,
        ) {
            stale_latch = false;
            stale_support = rising_raw;
        }
        assert!(!stale_latch);
        assert!(terrain_leg_has_support(stale_support));
    }

    #[test]
    fn run_release_follows_root_once_and_lifts_only_clearance_floor() {
        let release_clearance = run_airborne_clearance_for_sample(
            true,
            CycleDelta::from_cycles(0.81).unwrap(),
            None,
            FootDescent::RaiseOnly,
        )
        .unwrap()
        .metres();
        assert_eq!(
            release_clearance,
            ik_tuning().run_swing_minimum_sole_clearance_metres.metres()
        );
        assert!(
            run_airborne_clearance_for_sample(
                false,
                CycleDelta::from_cycles(0.81).unwrap(),
                None,
                FootDescent::RaiseOnly
            )
            .unwrap()
            .metres()
                > release_clearance
        );
        let previous_root = Vec3::new(0.0, 3.10, -4.2109);
        let next_root =
            previous_root + Vec3::NEG_Z * (5.5 / ik_tuning().continuity_sample_hz.hertz());
        let planted_world = Vec3::new(-0.12, 2.25, -3.668);
        let previous_owner = planted_world - previous_root;
        let owner = release_start_owner_target(
            LocomotionGait::Run,
            Some(previous_owner),
            Some(planted_world),
            next_root,
            Quat::IDENTITY,
            Vec3::ZERO,
        );
        let transported = next_root + owner;
        let lifted =
            transported + Vec3::Y * ik_tuning().run_swing_minimum_sole_clearance_metres.metres();
        let root_delta = next_root - previous_root;
        let root_relative_step = (lifted - planted_world - root_delta).length();
        assert!(
            root_relative_step
                <= ik_tuning().run_swing_minimum_sole_clearance_metres.metres() + 0.0001
        );
        assert!(root_relative_step <= 0.095);
        assert!(root_relative_step <= ik_tuning().maximum_knee_step_metres.metres());

        // Captured uphill release f49->50: neither full owner transport nor a
        // literal world hold can combine terrain rise and 5 cm clearance under
        // the 9 cm 3D owner budget. The joint projection selects an
        // intermediate XZ that satisfies both instead of violating continuity.
        let uphill_previous_root = Vec3::new(0.0, 3.103686, -4.2109375);
        let uphill_next_root = Vec3::new(0.0, 3.096167, -4.296875);
        let uphill_plant = Vec3::new(-0.11504457, 2.2510452, -3.7630615);
        let uphill_owner = uphill_plant - uphill_previous_root;
        let uphill_minimum_y = |point: ScenePlanPoint| {
            checked_scene_elevation(
                uphill_plant.y
                    + ik_tuning().run_swing_minimum_sole_clearance_metres.metres()
                    + (uphill_plant.z - point.metres().y).max(0.0) * 0.475,
            ).map(|height| Some(FootClearanceFloor::Minimum(height)))
        };
        let uphill_release = advance_run_airborne_world_target(
            Some(uphill_owner),
            uphill_plant,
            uphill_next_root,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            FootTargetSpeeds::configured().unwrap().run_airborne(true),
            uphill_minimum_y,
        )
        .unwrap();
        let uphill_release_owner = uphill_release - uphill_next_root;
        assert!(
            uphill_release_owner.distance(uphill_owner)
                <= (ik_tuning().run_first_release_owner_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz())
                    / ik_tuning().continuity_sample_hz.hertz()
                    + 0.0001
        );
        assert!(uphill_release.y + 0.0001 >= uphill_minimum_y(ScenePlanPoint::from_metres(uphill_release.xz()).unwrap()).unwrap().unwrap().minimum_height().unwrap().metres());
        assert!(uphill_release.y - uphill_minimum_y(ScenePlanPoint::from_metres(uphill_release.xz()).unwrap()).unwrap().unwrap().minimum_height().unwrap().metres() <= 0.0001);
        assert!(uphill_release.z < uphill_plant.z);
        assert!(uphill_release.z > uphill_plant.z - 5.5 / ik_tuning().continuity_sample_hz.hertz());
        let captured_toe_offset = Vec3::new(-0.0108, 0.0007, -0.1370);
        let uphill_previous_toe = uphill_plant + captured_toe_offset;
        let uphill_release_toe = uphill_release + captured_toe_offset;
        let toe_root_relative_step =
            (uphill_release_toe - uphill_previous_toe - (uphill_next_root - uphill_previous_root))
                .length();
        assert!(toe_root_relative_step <= 0.095);
        assert!(
            FootTargetSpeeds::configured()
                .unwrap()
                .run_airborne(true)
                .metres_per_second()
                / ik_tuning().continuity_sample_hz.hertz()
                < 0.095
        );
        assert_eq!(
            FootTargetSpeeds::configured()
                .unwrap()
                .run_airborne(false)
                .metres_per_second(),
            (ik_tuning().run_airborne_owner_step_metres.metres()
                * ik_tuning().continuity_sample_hz.hertz())
        );
        assert!(
            (ik_tuning().run_airborne_owner_step_metres.metres()
                * ik_tuning().continuity_sample_hz.hertz())
                / ik_tuning().continuity_sample_hz.hertz()
                > 5.5 / 64.0
        );
        assert!(
            (ik_tuning().run_airborne_owner_step_metres.metres()
                * ik_tuning().continuity_sample_hz.hertz())
                / ik_tuning().continuity_sample_hz.hertz()
                < 0.09
        );

        let previous_rotation = Quat::IDENTITY;
        let desired_rotation = Quat::from_rotation_x(30.0_f32.to_radians());
        let released_rotation = advance_airborne_foot_rotation(
            Some(previous_rotation),
            desired_rotation,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            ik_tuning().first_run_release_foot_rotation_speed_degrees_per_second,
        );
        assert!(
            previous_rotation
                .angle_between(released_rotation)
                .to_degrees()
                <= 0.0
        );

        // Walk/stop continue to hold a world plant on release.
        let walk_owner = release_start_owner_target(
            LocomotionGait::Walk,
            Some(previous_owner),
            Some(planted_world),
            next_root,
            Quat::IDENTITY,
            Vec3::ZERO,
        );
        assert!((next_root + walk_owner).distance(planted_world) < 0.0001);
    }

    #[test]
    fn unreachable_run_contact_keeps_flight_floor_until_chain_can_land() {
        let upper_root = Vec3::new(-0.10032953, 2.5767426, -6.794999);
        let contact = Vec3::new(-0.12013094, 1.902308, -7.4767027);
        let reach = terrain_maximum_reach(
            NonnegativeLength::from_metres(0.5230801).unwrap(),
            NonnegativeLength::from_metres(0.42998108).unwrap(),
        );
        assert!(!run_contact_within_leg_reach(contact, upper_root, reach));

        let flight_floor =
            contact + Vec3::Y * ik_tuning().run_swing_minimum_sole_clearance_metres.metres();
        assert!(run_contact_within_leg_reach(
            flight_floor,
            upper_root,
            reach,
        ));
        assert_eq!(
            run_airborne_clearance_for_sample(
                false,
                CycleDelta::from_cycles(0.133).unwrap(),
                Some(UnitInterval::from_fraction(1.0).unwrap()),
                FootDescent::RaiseOnly
            )
            .unwrap()
            .metres(),
            ik_tuning().run_swing_minimum_sole_clearance_metres.metres()
        );
    }

    #[test]
    fn first_run_release_uses_last_propagated_foot_orientation() {
        let analytic = Quat::from_rotation_x(0.18);
        let propagated = Quat::from_rotation_x(-0.07);
        assert_eq!(
            previous_airborne_foot_orientation(Some(analytic), Some(propagated), true),
            Some(propagated)
        );
        assert_eq!(
            previous_airborne_foot_orientation(Some(analytic), Some(propagated), false),
            Some(analytic)
        );
        assert_eq!(
            advance_airborne_foot_rotation(
                previous_airborne_foot_orientation(Some(analytic), Some(propagated), true),
                Quat::IDENTITY,
                AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz())
                    .unwrap(),
                ik_tuning().first_run_release_foot_rotation_speed_degrees_per_second,
            ),
            propagated
        );
    }

    #[test]
    fn airborne_run_limiter_bounds_combined_horizontal_and_clearance_motion() {
        let mut owner_target = Vec3::ZERO;
        let desired_samples = [
            Vec3::new(0.0, 0.05, -0.08),
            Vec3::new(0.0, 0.08, -0.17),
            Vec3::new(0.0, 0.10, -0.26),
            Vec3::new(0.0, 0.08, -0.35),
        ];
        for desired in desired_samples {
            let next = advance_foot_target_at_speed(
                Some(owner_target),
                desired,
                AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz())
                    .unwrap(),
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                    ik_tuning().run_airborne_owner_step_metres.metres()
                        * ik_tuning().continuity_sample_hz.hertz(),
                )
                .unwrap(),
            );
            assert!(
                next.distance(owner_target)
                    <= (ik_tuning().run_airborne_owner_step_metres.metres()
                        * ik_tuning().continuity_sample_hz.hertz())
                        / ik_tuning().continuity_sample_hz.hertz()
                        + 0.0001
            );
            owner_target = next;
        }

        let endpoint = Vec3::new(0.0, 0.0, -0.45);
        for _ in 0..8 {
            owner_target = advance_foot_target_at_speed(
                Some(owner_target),
                endpoint,
                AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz())
                    .unwrap(),
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                    ik_tuning().run_airborne_owner_step_metres.metres()
                        * ik_tuning().continuity_sample_hz.hertz(),
                )
                .unwrap(),
            );
        }
        assert!(owner_target.distance(endpoint) < 0.0001);
    }

    #[test]
    fn high_speed_unplanned_release_uses_run_budget_before_gait_style_catches_up() {
        let before_root = Vec3::new(0.0, 2.831712, -0.171875);
        let after_root = Vec3::new(0.0, 2.84709, -0.2578125);
        let before_solve = Vec3::new(-0.092886, 1.965967, -0.204507);
        let desired_solve = Vec3::new(-0.120672, 1.962317, -0.195115);
        let before_owner = before_solve - before_root;
        let desired_owner = desired_solve - after_root;
        assert!(before_owner.distance(desired_owner) > 0.095);
        let measured_speed = update_measured_owner_planar_speed(
            None,
            Some(before_root),
            after_root,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            true,
            false,
        )
        .unwrap();
        assert!((measured_speed.unwrap().metres_per_second() - 5.5).abs() <= 0.0001);
        assert!(uses_run_airborne_motion_budget(
            LocomotionGait::Walk,
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(0.5)
                .unwrap()
                .max(measured_speed.unwrap()),
        ));
        assert!(!uses_run_airborne_motion_budget(
            LocomotionGait::Walk,
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(2.0).unwrap()
        ));
        assert_eq!(
            update_measured_owner_planar_speed(
                measured_speed,
                Some(after_root),
                after_root + Vec3::X,
                AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz())
                    .unwrap(),
                false,
                false,
            )
            .unwrap(),
            measured_speed,
        );
        assert_eq!(
            update_measured_owner_planar_speed(
                measured_speed,
                Some(after_root),
                after_root + Vec3::X,
                AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz())
                    .unwrap(),
                true,
                true,
            )
            .unwrap(),
            None,
        );

        let resolved = advance_run_airborne_world_target(
            Some(before_owner),
            desired_solve,
            after_root,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                ik_tuning().run_airborne_owner_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz(),
            )
            .unwrap(),
            |_| checked_scene_elevation(-100.0).map(|height| Some(FootClearanceFloor::Minimum(height))),
        )
        .unwrap();
        let resolved_owner = resolved - after_root;
        assert!(
            resolved_owner.distance(before_owner)
                <= (ik_tuning().run_airborne_owner_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz())
                    / ik_tuning().continuity_sample_hz.hertz()
                    + 0.0001
        );
        assert!(resolved_owner.distance(before_owner) <= 0.095);

        let support_path = bound_unacquired_run_support_release_target(
            true,
            false,
            false,
            true,
            Some(before_owner),
            desired_solve,
            after_root,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            FootTargetSpeeds::configured().unwrap().run_airborne,
            |_| checked_scene_elevation(-100.0).map(|height| Some(FootClearanceFloor::Minimum(height))),
        )
        .unwrap();
        assert!(
            (support_path - after_root).distance(before_owner)
                <= (ik_tuning().run_airborne_owner_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz())
                    / ik_tuning().continuity_sample_hz.hertz()
                    + 0.0001
        );
        assert_eq!(
            bound_unacquired_run_support_release_target(
                true,
                false,
                true,
                true,
                Some(before_owner),
                desired_solve,
                after_root,
                Quat::IDENTITY,
                AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz())
                    .unwrap(),
                FootTargetSpeeds::configured().unwrap().run_airborne,
                |_| checked_scene_elevation(-100.0).map(|height| Some(FootClearanceFloor::Minimum(height)))
            )
            .unwrap(),
            desired_solve,
        );
        let bounded_owner = support_path - after_root;
        assert_eq!(
            support_release_diagnostic_goal(true, true, bounded_owner, desired_owner,),
            Some(bounded_owner),
        );
        assert_eq!(
            support_release_diagnostic_goal(true, false, bounded_owner, desired_owner,),
            Some(desired_owner),
        );
        assert_eq!(
            support_release_diagnostic_goal(false, true, bounded_owner, desired_owner,),
            None,
        );

        let steady_before_root = Vec3::new(0.0, 2.8317122, -0.171875);
        let steady_before_end = Vec3::new(0.21052803, 2.0040245, -0.00043848384);
        let steady_after_root = Vec3::new(0.0, 2.8470902, -0.2578125);
        let steady_after_end = Vec3::new(0.20848821, 2.103109, -0.11178008);
        let authored = Vec3::new(0.200671, 1.9489093, -0.12319517);
        let preliminary_target = authored + Vec3::X * 0.01;
        let planted_target = authored + Vec3::NEG_Z * 0.20;
        assert!(unplanned_support_release_is_owned(
            false,
            Some(UnitInterval::from_fraction(0.0).unwrap()),
            Some(UnitInterval::from_fraction(1.0).unwrap()),
            None,
            preliminary_target,
            planted_target,
            authored,
        ));
        assert!(!unplanned_support_release_is_owned(
            false,
            Some(UnitInterval::from_fraction(0.0).unwrap()),
            Some(UnitInterval::from_fraction(0.0).unwrap()),
            None,
            authored,
            authored,
            authored,
        ));
        assert!(unplanned_support_release_is_owned(
            false,
            Some(UnitInterval::from_fraction(0.0).unwrap()),
            Some(UnitInterval::from_fraction(1.0).unwrap()),
            None,
            authored,
            authored,
            authored,
        ));
        let (stored_world, stored_owner) = resolved_unacquired_support_release_ownership(
            true,
            steady_before_end,
            steady_before_root,
            Quat::IDENTITY,
        )
        .unwrap();
        assert_eq!(stored_world, steady_before_end);
        let mut memory = LegIkMemory {
            right_foot_world_target: Some(Vec3::new(9.0, 9.0, 9.0)),
            right_foot_target: Some(Vec3::new(8.0, 8.0, 8.0)),
            right_release_target: Some(Vec3::new(7.0, 7.0, 7.0)),
            right_release_active: true,
            owner_frame: Some(
                ObservedRigFrame::from_native(steady_before_root, Quat::IDENTITY).unwrap(),
            ),
            ..default()
        };
        assert!(airborne_unplanned_release_uses_resolved_end(
            true, None, true
        ));
        assert!(!airborne_unplanned_release_uses_resolved_end(
            true,
            Some(planted_target),
            true,
        ));
        commit_resolved_unplanned_airborne_release(
            &mut memory,
            LeadFoot::Right,
            true,
            None,
            true,
            steady_before_end,
            steady_before_root,
            Quat::IDENTITY,
        );
        assert_eq!(memory.right_foot_world_target, Some(stored_world));
        assert_eq!(memory.right_foot_target, Some(stored_owner));
        assert_eq!(memory.right_release_target, Some(stored_owner));
        let diagnostics = LegIkState(memory).diagnostics();
        let diagnostic_solve = diagnostics
            .right_solve_target
            .expect("the resolved support solve remains diagnostic state");
        let diagnostic_release = diagnostics
            .right_release_target
            .expect("the resolved support release remains diagnostic state");
        assert!(diagnostic_solve.is_finite());
        assert!(diagnostic_release.is_finite());
        assert!(diagnostic_solve.distance(steady_before_end) <= 0.000001);
        assert!(diagnostic_release.distance(steady_before_end) <= 0.000001);
        assert!(diagnostic_release.distance(diagnostic_solve) <= 0.000001);
        assert_eq!(
            run_previous_owner_target(LocomotionGait::Run, None, memory.right_foot_target,),
            Some(stored_owner),
        );
        let (_, next_owner) = resolved_unacquired_support_release_ownership(
            true,
            steady_after_end,
            steady_after_root,
            Quat::IDENTITY,
        )
        .unwrap();
        assert!(next_owner.distance(stored_owner) <= 0.095);
    }

    #[test]
    fn unacquired_run_support_entry_keeps_using_bounded_follower() {
        let previous_owner = Vec3::new(0.1, 0.15, -0.5);
        let frozen_plant = Vec3::new(0.1, 0.085, -0.8);
        let resolved = advance_run_airborne_world_target(
            Some(previous_owner),
            frozen_plant,
            Vec3::ZERO,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                ik_tuning().run_airborne_owner_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz(),
            )
            .unwrap(),
            |_| checked_scene_elevation(0.085).map(|height| Some(FootClearanceFloor::Minimum(height))),
        )
        .unwrap();
        assert!(
            resolved.distance(previous_owner)
                <= (ik_tuning().run_airborne_owner_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz())
                    / ik_tuning().continuity_sample_hz.hertz()
                    + 0.0001
        );
        assert!(resolved.distance(frozen_plant) > 0.1);

        // A completed plan remains on the 5 cm semantic floor throughout raw
        // flight, then may descend to exact contact on the first eligible
        // support sample without bypassing the follower above.
        assert_eq!(
            run_airborne_clearance(
                CycleDelta::from_cycles(0.34).unwrap(),
                Some(UnitInterval::from_fraction(1.0).unwrap()),
                FootDescent::RaiseOnly
            )
            .unwrap()
            .metres(),
            ik_tuning().run_swing_minimum_sole_clearance_metres.metres()
        );
        assert_eq!(
            run_airborne_clearance(
                CycleDelta::from_cycles(0.17).unwrap(),
                Some(UnitInterval::from_fraction(1.0).unwrap()),
                FootDescent::RaiseOnly
            )
            .unwrap()
            .metres(),
            ik_tuning().run_swing_minimum_sole_clearance_metres.metres()
        );
        assert!(
            run_airborne_clearance(
                CycleDelta::from_cycles(0.17).unwrap(),
                Some(UnitInterval::from_fraction(1.0).unwrap()),
                FootDescent::FollowFloor
            )
            .unwrap()
            .metres()
                <= 0.0
        );
    }

    #[test]
    fn run_follower_can_converge_on_fixed_world_contact_at_full_speed() {
        let previous_root = Vec3::new(0.0, 2.0, -4.0);
        let fixed_contact = Vec3::new(0.1, 0.085, -4.5);
        let previous_owner = fixed_contact - previous_root;
        let current_root =
            previous_root + Vec3::NEG_Z * (5.5 / ik_tuning().continuity_sample_hz.hertz());
        assert!(run_contact_within_follower_step(
            Some(previous_owner),
            fixed_contact,
            current_root,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap()
        ));
        let resolved = advance_run_airborne_world_target(
            Some(previous_owner),
            fixed_contact,
            current_root,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                ik_tuning().run_airborne_owner_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz(),
            )
            .unwrap(),
            |_| checked_scene_elevation(0.085).map(|height| Some(FootClearanceFloor::Minimum(height))),
        )
        .unwrap();
        assert!(resolved.distance(fixed_contact) < 0.0001);

        let far_contact = fixed_contact + Vec3::NEG_Z * 0.3;
        assert!(!run_contact_within_follower_motion_step(
            Some(previous_owner),
            far_contact,
            current_root,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap()
        ));
        assert_eq!(
            run_airborne_clearance(
                CycleDelta::from_cycles(0.17).unwrap(),
                Some(UnitInterval::from_fraction(1.0).unwrap()),
                FootDescent::RaiseOnly
            )
            .unwrap()
            .metres(),
            ik_tuning().run_swing_minimum_sole_clearance_metres.metres()
        );
    }

    #[test]
    fn final_run_descent_transports_unacquired_footprint_then_freezes_it() {
        let previous_root = Vec3::new(0.0, 0.0, -4.0);
        let current_root =
            previous_root + Vec3::NEG_Z * (5.5 / ik_tuning().continuity_sample_hz.hertz());
        let fixed_contact = Vec3::new(0.1, measured_ankle_sole_offset_metres().metres(), -4.5);
        let prior_floor =
            fixed_contact + Vec3::Y * ik_tuning().run_swing_minimum_sole_clearance_metres.metres();
        let previous_owner = prior_floor - previous_root;

        // Root travel plus the contact descent is 9.94 cm, so the stationary
        // footprint cannot be reached inside the 9 cm target budget.
        assert!(!run_contact_within_follower_motion_step(
            Some(previous_owner),
            fixed_contact,
            current_root,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap()
        ));
        let transported = retarget_unacquired_run_contact_for_descent(
            Some(previous_owner),
            fixed_contact,
            current_root,
            Quat::IDENTITY,
            FootTrackSide::PositiveX,
            Vec3::new(0.1, 0.9, current_root.z - 0.5),
            ReachLimit::Fixed(NonnegativeLength::from_metres(1.0).unwrap()),
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            |_| Some(SupportElevation::ZERO),
        )
        .expect("the owner-local footprint should remain reachable after its 5 cm descent");
        assert!(
            (transported.z - (fixed_contact.z - 5.5 / ik_tuning().continuity_sample_hz.hertz()))
                .abs()
                < 0.0001
        );
        assert_eq!(transported.y, measured_ankle_sole_offset_metres().metres());
        assert!(run_contact_within_follower_step(
            Some(previous_owner),
            transported,
            current_root,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap()
        ));
        let landed = advance_run_airborne_world_target(
            Some(previous_owner),
            transported,
            current_root,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                ik_tuning().run_airborne_owner_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz(),
            )
            .unwrap(),
            |_| checked_scene_elevation(measured_ankle_sole_offset_metres().metres()).map(|height| Some(FootClearanceFloor::Minimum(height))),
        )
        .unwrap();
        assert!(landed.distance(transported) < 0.0001);
        assert!(
            landed.distance(current_root + previous_owner)
                <= (ik_tuning().run_airborne_owner_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz())
                    / ik_tuning().continuity_sample_hz.hertz()
                    + 0.0001
        );

        // Acquired support bypasses all airborne retargeting and retains the
        // resulting world footprint exactly on subsequent samples.
        let acquired_world_plant = transported;
        assert_eq!(acquired_world_plant, transported);
    }

    #[test]
    fn downhill_rising_contact_retargets_inside_current_leg_reach() {
        // Captured left landing at phase .867: the follower had reached its
        // frozen endpoint inside the motion budget, but the endpoint remained
        // about 1 cm beyond the current analytic leg reach. The rendered sole
        // consequently stayed 1.7 cm high until the following sample.
        let previous_root = Vec3::new(0.0, 2.7854202, -6.703125);
        let current_root = Vec3::new(0.0, 2.7790365, -6.7890625);
        let previous_ankle = Vec3::new(-0.11826715, 1.9728086, -7.4084473);
        let previous_owner = previous_ankle - previous_root;
        let upper_root = Vec3::new(-0.10032953, 2.5767426, -6.794999);
        let frozen_contact = Vec3::new(-0.12020548, 1.9023025, -7.475421);
        let solve_reach = maximum_reach(
            NonnegativeLength::from_metres(0.523).unwrap(),
            NonnegativeLength::from_metres(0.430).unwrap(),
        );
        assert!(run_contact_within_follower_motion_step(
            Some(previous_owner),
            frozen_contact,
            current_root,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap()
        ));
        assert!(frozen_contact.distance(upper_root) > solve_reach.native_metres().unwrap() + 0.001);

        let terrain_height = SupportElevation::from_metres(
            frozen_contact.y - measured_ankle_sole_offset_metres().metres(),
        ).unwrap();
        let reachable_contact = retarget_unacquired_run_contact_for_descent(
            Some(previous_owner),
            frozen_contact,
            current_root,
            Quat::IDENTITY,
            FootTrackSide::NegativeX,
            upper_root,
            solve_reach,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            |_| Some(terrain_height),
        )
        .expect("the final footprint should move just inside current downhill reach");
        assert!(reachable_contact.xz().distance(frozen_contact.xz()) > 0.001);
        assert!(
            reachable_contact.distance(upper_root) <= solve_reach.native_metres().unwrap() + 0.001
        );
        assert!(run_contact_within_follower_motion_step(
            Some(previous_owner),
            reachable_contact,
            current_root,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap()
        ));
        assert_eq!(
            reachable_contact.y,
            terrain_height.metres() + measured_ankle_sole_offset_metres().metres()
        );
        let landed = advance_run_airborne_world_target(
            Some(previous_owner),
            reachable_contact,
            current_root,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                ik_tuning().run_airborne_owner_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz(),
            )
            .unwrap(),
            |_| checked_scene_elevation(reachable_contact.y).map(|height| Some(FootClearanceFloor::Minimum(height))),
        )
        .unwrap();
        assert!(landed.distance(reachable_contact) < 0.0001);
    }
}
