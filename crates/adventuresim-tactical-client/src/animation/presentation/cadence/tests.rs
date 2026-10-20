use super::*;
use adventuresim_tactical_core::animation::{CadenceDemand, StepCadence};

#[test]
fn rejected_recalibration_clears_only_the_changed_motion() {
    let measurement = AuthoredStrideMeasurement {
        step_distance: NonnegativeLength::from_metres(1.0).unwrap(),
        maximum_stance_slip: NonnegativeLength::from_metres(0.0).unwrap(),
    };
    let mut strides = AuthoredLocomotionStrides {
        walk: Some(measurement),
        run: Some(measurement),
        ..default()
    };
    strides.phase_curves.insert(
        super::CalibrationMotion::Walk,
        AuthoredPhaseCurve::from_samples(
            vec![0.25, 1.25]
                .into_iter()
                .map(|phase| CyclePhase::from_cycles(phase).unwrap())
                .collect(),
        )
        .unwrap(),
    );

    strides.clear_motion(super::CalibrationMotion::Walk);

    assert!(strides.walk.is_none());
    assert!(strides.run.is_some());
    assert!(
        !strides
            .phase_curves
            .contains_key(&super::CalibrationMotion::Walk)
    );
}

#[test]
fn authored_walk_stride_controls_visual_cycle_rate() {
    let state = SkeletonState::default().with_local_velocity(Vec3::NEG_Z * 2.0);
    let mut presented = PresentedSkeleton::new(state.clone(), None);
    let strides = AuthoredLocomotionStrides {
        walk: Some(AuthoredStrideMeasurement {
            step_distance: NonnegativeLength::from_metres(1.0).unwrap(),
            maximum_stance_slip: NonnegativeLength::from_metres(0.0).unwrap(),
        }),
        run: Some(AuthoredStrideMeasurement {
            step_distance: NonnegativeLength::from_metres(2.0).unwrap(),
            maximum_stance_slip: NonnegativeLength::from_metres(0.0).unwrap(),
        }),
        ..default()
    };

    advance_presented_skeleton_with_strides(
        &mut presented,
        &state,
        AnimationElapsed::from_seconds(0.1).unwrap(),
        &strides,
    )
    .unwrap();

    // One cycle owns two one-metre steps: 2 m/s advances 0.1 cycle in 0.1 s.
    assert!((presented.gait_phase.cycles() - 0.1).abs() < 0.0001);
}

#[test]
fn diagonal_combat_cadence_blends_strafe_and_skip_stride() {
    let state = SkeletonState::default()
        .with_weapon_guard(WeaponGuardState::Raised)
        .with_local_velocity(Vec3::new(1.0, 0.0, 1.0))
        .with_raised_locomotion(RaisedLocomotionIntent::moving(
            Vec2::ONE,
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(2.0)
                .unwrap(),
        ));
    let mut presented = PresentedSkeleton::new(state.clone(), None);
    let strides = AuthoredLocomotionStrides {
        strafe: Some(AuthoredStrideMeasurement {
            step_distance: NonnegativeLength::from_metres(0.25).unwrap(),
            maximum_stance_slip: NonnegativeLength::from_metres(0.0).unwrap(),
        }),
        skip: Some(AuthoredStrideMeasurement {
            step_distance: NonnegativeLength::from_metres(0.75).unwrap(),
            maximum_stance_slip: NonnegativeLength::from_metres(0.0).unwrap(),
        }),
        ..default()
    };

    advance_presented_skeleton_with_strides(
        &mut presented,
        &state,
        AnimationElapsed::from_seconds(0.1).unwrap(),
        &strides,
    )
    .unwrap();

    assert!((presented.gait_phase.cycles() - 0.2).abs() < 0.0001);
}

#[test]
fn authored_cadence_reports_and_caps_unrepresentable_speed() {
    let state = SkeletonState::default().with_local_velocity(Vec3::NEG_Z * 9.0);
    let mut presented = PresentedSkeleton::new(state.clone(), None);
    let strides = AuthoredLocomotionStrides {
        walk: Some(AuthoredStrideMeasurement {
            step_distance: NonnegativeLength::from_metres(1.0).unwrap(),
            maximum_stance_slip: NonnegativeLength::from_metres(0.0).unwrap(),
        }),
        run: Some(AuthoredStrideMeasurement {
            step_distance: NonnegativeLength::from_metres(1.5).unwrap(),
            maximum_stance_slip: NonnegativeLength::from_metres(0.04).unwrap(),
        }),
        ..default()
    };

    advance_presented_skeleton_with_strides(
        &mut presented,
        &state,
        AnimationElapsed::from_seconds(0.1).unwrap(),
        &strides,
    )
    .unwrap();

    assert!((presented.gait_phase.cycles() - 0.25).abs() < 0.0001);
    assert!(
        presented
            .authored_cadence
            .is_some_and(|cadence| cadence.limit != AuthoredCadenceLimit::Unrestricted)
    );
}

#[test]
fn authored_cadence_cap_has_a_smooth_bounded_transition() {
    let cap = playback_tuning().authored_step_cadence;
    assert_eq!(
        cap.select(CadenceDemand::Finite(StepCadence::from_per_second(4.0).unwrap()))
            .unwrap().applied().per_second(),
        4.0,
    );
    assert_eq!(
        cap.select(CadenceDemand::Finite(StepCadence::from_per_second(5.5).unwrap()))
            .unwrap().applied().per_second(),
        5.0,
    );
    assert_eq!(
        cap.select(CadenceDemand::Finite(StepCadence::from_per_second(7.0).unwrap()))
            .unwrap().applied().per_second(),
        5.0,
    );
    let near_cap = cap.select(CadenceDemand::Finite(StepCadence::from_per_second(5.0).unwrap()))
        .unwrap().applied().per_second();
    assert!(near_cap < 5.0);
    assert!(near_cap > 4.8);

    let epsilon = 0.001;
    let slope_before = (cap.select(CadenceDemand::Finite(StepCadence::from_per_second(4.5).unwrap()))
        .unwrap().applied().per_second()
        - cap.select(CadenceDemand::Finite(StepCadence::from_per_second(4.5 - epsilon).unwrap()))
            .unwrap().applied().per_second())
        / epsilon;
    let slope_after = (cap.select(CadenceDemand::Finite(StepCadence::from_per_second(5.5 + epsilon).unwrap()))
        .unwrap().applied().per_second()
        - cap.select(CadenceDemand::Finite(StepCadence::from_per_second(5.5).unwrap()))
            .unwrap().applied().per_second())
        / epsilon;
    assert!((slope_before - 1.0).abs() < 0.001);
    assert!(slope_after.abs() < 0.001);
}
