//! Deterministic force-driven quickstep stimulus and frame selection.

use super::*;

const QUICKSTEP_FIXTURE_BIOLOGICAL_MASS_KG: f32 = 70.0;
const QUICKSTEP_FIXTURE_TOTAL_MASS_KG: f32 = 93.9;
const QUICKSTEP_FIXTURE_LEG_AGILITY: f32 = 4.0;

pub(in crate::animation_viewer) fn quickstep_scenario(
    name: ScenarioMetadata<'static>,
    local_direction: Vec2,
) -> Vec<PlannedFrame> {
    let planar_speeds = quickstep_fixture_planar_speeds(72);
    (0..72)
        .map(|scenario_frame| PlannedFrame {
            scenario: name.clone(),
            scenario_frame,
            speed: planar_speeds[scenario_frame],
            time_seconds: authored_frame_time(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            ),
            local_direction,
            camera_yaw: authored_camera_angle(0.0),
            camera_pitch: authored_camera_angle(0.0),
            action: SkeletonAction::Dodge,
            weapon_guard: WeaponGuardState::Raised,
            lead_foot: LeadFoot::Left,
        })
        .collect()
}

pub(in crate::animation_viewer) fn quickstep_action_ticks() -> AnimationSampleSpan {
    let config = TacticalCombatConfig::default();
    quickstep_action_contact_ticks(config.movement.maneuvers.quickstep_duration_seconds)
        .saturating_mul(2)
}

pub(in crate::animation_viewer) fn quickstep_push_ticks() -> AnimationSampleSpan {
    let config = TacticalCombatConfig::default();
    let duration =
        match quickstep_push_duration(QUICKSTEP_FIXTURE_LEG_AGILITY, &config.movement.motor) {
            Ok(duration) => duration,
            Err(error) => panic!("invalid authored quickstep duration: {error}"),
        };
    AnimationSampleSpan::from((duration.seconds() * locomotion_sample_rate().hertz()).ceil() as u64)
}

pub(in crate::animation_viewer) fn quickstep_fixture_planar_speeds(
    frame_count: usize,
) -> Vec<SignedSpeed> {
    let config = TacticalCombatConfig::default();
    let action_ticks = quickstep_action_ticks().samples() as usize;
    let peak_force = quickstep_peak_horizontal_force_newtons(
        QUICKSTEP_FIXTURE_BIOLOGICAL_MASS_KG,
        QUICKSTEP_FIXTURE_LEG_STRENGTH,
        &config.movement.motor,
    );
    let target_displacement = quickstep_target_displacement_metres(
        CharacterDimensions::default().leg_length_metres,
        &config.movement.motor,
    );
    let duration = authored_frame_time(action_ticks as f32 / locomotion_sample_rate().hertz());
    let mut velocity: f32 = 0.0;
    let mut displacement: f32 = 0.0;
    let mut speeds = Vec::with_capacity(frame_count);
    for frame in 0..frame_count {
        speeds.push(authored_drive_speed(velocity.abs()));
        if frame < action_ticks {
            let target = match quickstep_motion_target(
                (frame + 1) as f32 / action_ticks as f32,
                target_displacement,
                duration,
                config
                    .movement
                    .motor
                    .quickstep_authored_displacement_profile,
            ) {
                Ok(target) => target,
                Err(error) => panic!("invalid authored quickstep speed: {error}"),
            };
            let force = quickstep_tracking_force_newtons(
                displacement,
                authored_drive_speed(velocity),
                target,
                QUICKSTEP_FIXTURE_TOTAL_MASS_KG,
                peak_force,
                authored_frame_time(locomotion_sample_rate().interval().seconds()),
            );
            velocity += force / QUICKSTEP_FIXTURE_TOTAL_MASS_KG / locomotion_sample_rate().hertz();
        } else {
            velocity = 0.0;
        }
        displacement += velocity / locomotion_sample_rate().hertz();
    }
    speeds
}

pub(in crate::animation_viewer) fn quickstep_release_frame() -> usize {
    let config = TacticalCombatConfig::default();
    let motor = &config.movement.motor;
    let ticks = quickstep_push_ticks().samples() as usize;
    let acceleration = quickstep_peak_horizontal_force_newtons(
        QUICKSTEP_FIXTURE_BIOLOGICAL_MASS_KG,
        QUICKSTEP_FIXTURE_LEG_STRENGTH,
        motor,
    ) / QUICKSTEP_FIXTURE_TOTAL_MASS_KG;
    let mut speed = 0.0;
    let mut displacement = 0.0;
    for tick in 0..ticks {
        speed += acceleration * quickstep_force_curve((tick as f32 + 0.5) / ticks as f32)
            / locomotion_sample_rate().hertz();
        displacement += speed / locomotion_sample_rate().hertz();
        if displacement >= motor.quickstep_maximum_supported_root_displacement_metres {
            return tick + 1;
        }
    }
    ticks
}

pub(in crate::animation_viewer) fn quickstep_landing_frame() -> usize {
    ((quickstep_release_frame() + 1)
        ..(quickstep_release_frame() + locomotion_sample_rate().hertz() as usize))
        .find(|&frame| quickstep_fixture_vertical_state(frame).0 <= 0.0)
        .expect("the quickstep fixture must return to ground within one second")
}

pub(in crate::animation_viewer) fn quickstep_fixture_vertical_state(
    scenario_frame: usize,
) -> (f32, f32) {
    let config = TacticalCombatConfig::default();
    let motor = &config.movement.motor;
    let push_ticks = quickstep_push_ticks().samples() as usize;
    let release_frame = quickstep_release_frame();
    let peak_acceleration = quickstep_peak_horizontal_force_newtons(
        QUICKSTEP_FIXTURE_BIOLOGICAL_MASS_KG,
        QUICKSTEP_FIXTURE_LEG_STRENGTH,
        motor,
    ) / QUICKSTEP_FIXTURE_TOTAL_MASS_KG
        * motor.quickstep_takeoff_angle_degrees.to_radians().tan();
    let mut height = 0.0;
    let mut velocity = 0.0;
    for tick in 0..scenario_frame {
        if tick < release_frame {
            velocity += peak_acceleration
                * quickstep_force_curve((tick as f32 + 0.5) / push_ticks as f32)
                / locomotion_sample_rate().hertz();
        } else {
            velocity -= motor.gravity_metres_per_second_squared / locomotion_sample_rate().hertz();
        }
        height += velocity / locomotion_sample_rate().hertz();
    }
    (height, velocity)
}

pub(in crate::animation_viewer) fn quickstep_fixture_action_distance_metres() -> f32 {
    quickstep_scenario(
        ScenarioMetadata::authored(ScenarioId::QuickstepRight),
        Vec2::X,
    )
    .iter()
    .take(quickstep_action_ticks().samples() as usize + 1)
    .map(|frame| frame.speed.metres_per_second() / locomotion_sample_rate().hertz())
    .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quickstep_fixture_keeps_feet_airborne_until_action_end() {
        assert_eq!(quickstep_push_ticks().samples(), 25);
        assert_eq!(quickstep_release_frame(), 11);
        assert_eq!(quickstep_landing_frame(), 22);
        assert!(quickstep_landing_frame() < quickstep_action_ticks().samples() as usize);
    }
    #[test]
    fn johns_quickstep_fixture_covers_about_one_metre_in_half_a_second() {
        let duration = quickstep_action_ticks().samples() as f32 / locomotion_sample_rate().hertz();
        let distance = quickstep_fixture_action_distance_metres();
        assert!((0.49..=0.51).contains(&duration), "duration={duration}");
        assert!((0.90..=1.10).contains(&distance), "distance={distance}");
    }
}
