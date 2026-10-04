//! Capture dimensions are physical pixels, independent of desktop scaling.
use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResolution},
};

const STANDARD_CAPTURE_PIXELS: UVec2 = UVec2::new(1280, 720);
pub(super) const CAPTURE_ASPECT_RATIO: f32 =
    STANDARD_CAPTURE_PIXELS.x as f32 / STANDARD_CAPTURE_PIXELS.y as f32;
// Match the frozen terrain comparison images. This is a capture setting, not
// a geographic measurement or a production gameplay resolution constraint.
const TERRAIN_COMPARISON_PIXELS: UVec2 = UVec2::new(2240, 1260);

pub(super) fn physical_pixels(profile: &str) -> UVec2 {
    if profile == super::fixed_city_cameras::PROFILE {
        TERRAIN_COMPARISON_PIXELS
    } else {
        STANDARD_CAPTURE_PIXELS
    }
}

/// Start with the requested physical dimensions before native window creation.
pub(super) fn window_resolution(profile: &str) -> WindowResolution {
    let pixels = physical_pixels(profile);
    WindowResolution::new(pixels.x, pixels.y).with_scale_factor_override(1.0)
}

fn restore_resolution(profile: &str, resolution: &mut WindowResolution) {
    let pixels = physical_pixels(profile);
    resolution.set_physical_resolution(pixels.x, pixels.y);
}

/// Native window creation applies the host scale to physical dimensions even
/// with an override. Restore the capture size after creation, before readbacks.
pub(super) fn apply(
    state: Res<super::capture_state::SceneCaptureState>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    restore_resolution(&state.profile, &mut window.resolution);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_restores_physical_dimensions_after_native_host_scaling() {
        for profile in [super::super::fixed_city_cameras::PROFILE, "city-review"] {
            for host_scale in [1.0, 1.75, 2.0] {
                let expected = physical_pixels(profile);
                let mut resolution = window_resolution(profile);
                resolution.set_scale_factor_and_apply_to_physical_size(host_scale);
                if host_scale != 1.0 {
                    assert_ne!(resolution.physical_width(), expected.x);
                }
                restore_resolution(profile, &mut resolution);
                assert_eq!(resolution.physical_width(), expected.x);
                assert_eq!(resolution.physical_height(), expected.y);
                assert_eq!(resolution.scale_factor_override(), Some(1.0));
            }
        }
    }
}
