//! Exposure follows interior daylight gradually, while preserving the authored night response.
use super::{InteriorField, daylight_response};
use crate::presentation::{PresentedCelestialLighting, TacticalGameplayCamera};
use bevy::{camera::Exposure, prelude::*};

const MAX_INTERIOR_EXPOSURE_STOPS: f32 = 2.0;
const DARK_ADAPTATION_SECONDS: f32 = 1.5;
const LIGHT_ADAPTATION_SECONDS: f32 = 0.35;
const WELL_LIT_SAMPLE: f32 = 0.4;
const EXPOSURE_SETTLED_TOLERANCE_STOPS: f32 = 0.005;

#[derive(Resource, Default)]
pub(super) struct InteriorExposure {
    compensation: f32,
    scene: Option<Entity>,
}

fn approach(current: f32, target: f32, elapsed: f32) -> f32 {
    let seconds = if target > current {
        DARK_ADAPTATION_SECONDS
    } else {
        LIGHT_ADAPTATION_SECONDS
    };
    let next = current + (target - current) * (1.0 - (-elapsed / seconds).exp());
    if (next - target).abs() <= EXPOSURE_SETTLED_TOLERANCE_STOPS {
        target
    } else {
        next
    }
}

pub(super) fn adapt_exposure(
    camera: Option<Single<(&GlobalTransform, &mut Exposure), With<TacticalGameplayCamera>>>,
    fields: Query<(&InteriorField, &GlobalTransform)>,
    celestial: Res<PresentedCelestialLighting>,
    // Eye adaptation follows the camera even when the simulation is paused.
    time: Res<Time<Real>>,
    mut state: ResMut<InteriorExposure>,
) {
    let Some(camera) = camera else {
        return;
    };
    let (transform, mut exposure) = camera.into_inner();
    let Some(snapshot) = &celestial.snapshot else {
        *state = InteriorExposure::default();
        exposure.ev100 = Exposure::SUNLIGHT.ev100;
        return;
    };
    if state.scene != Some(snapshot.scene) {
        state.compensation = 0.0;
        state.scene = Some(snapshot.scene);
    }
    let mut target: f32 = 0.0;
    for (field, building_transform) in &fields {
        let point = building_transform
            .affine()
            .inverse()
            .transform_point3(transform.translation());
        if let Some(sample) = field.sample(point) {
            let brightness =
                (sample.positive.truncate() + sample.negative.truncate()).element_sum() / 6.0;
            let shelter = 1.0 - (brightness / WELL_LIT_SAMPLE).clamp(0.0, 1.0);
            target =
                target.max(MAX_INTERIOR_EXPOSURE_STOPS * shelter * daylight_response(&celestial).w);
        }
    }
    state.compensation = approach(state.compensation, target, time.delta_secs());
    exposure.ev100 = snapshot.exposure_ev100
        - state
            .compensation
            .min(MAX_INTERIOR_EXPOSURE_STOPS * daylight_response(&celestial).w);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptation_is_frame_rate_independent_and_does_not_overshoot() {
        let mut fine = 0.0;
        for _ in 0..60 {
            fine = approach(fine, 2.0, 1.0 / 60.0);
        }
        let coarse = approach(0.0, 2.0, 1.0);
        assert!((fine - coarse).abs() < 0.00001);
        assert!(coarse > 0.0 && coarse < 2.0);
        assert!(approach(2.0, 0.0, 1.0) < 2.0 - coarse);
    }
}
