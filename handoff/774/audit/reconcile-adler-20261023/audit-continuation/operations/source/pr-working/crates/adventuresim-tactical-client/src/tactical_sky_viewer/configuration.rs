//! Deterministic lighting and camera presets for sky captures.

use super::*;

pub(super) fn sky_view_configuration(view: SkyView) -> SkyViewConfiguration {
    let absolute_minute = match view {
        SkyView::Sun => StrategicMinute::day_start_for_index(172).saturating_add_minutes(12 * 60),
        // Clear midsummer low Sun: demonstrates atmospheric extinction and
        // aureole structure without changing the production solar model.
        SkyView::SunDetail => {
            StrategicMinute::day_start_for_index(172).saturating_add_minutes(19 * 60)
        }
        SkyView::Twilight => {
            StrategicMinute::day_start_for_index(80).saturating_add_minutes(18 * 60)
        }
        // Day 249 23:00: verified moonlit slot, +24.6 degrees and 99% illuminated.
        SkyView::Moon => StrategicMinute::new(359_940),
        // Canonical new moon at 23:00 on day 77.
        SkyView::Stars => StrategicMinute::new(637_860),
        SkyView::CloudCumulus
        | SkyView::CloudStratocumulus
        | SkyView::CloudCirrus
        | SkyView::CloudOvercast
        | SkyView::CloudStorm => {
            StrategicMinute::day_start_for_index(172).saturating_add_minutes(15 * 60)
        }
    };
    let celestial = celestial_directions(absolute_minute, LATITUDE, LONGITUDE);
    let sun = to_bevy_direction(celestial.sun);
    let moon = to_bevy_direction(celestial.moon);
    let view_direction = match view {
        SkyView::Sun => horizon_view(sun, 0.5),
        SkyView::SunDetail => sun,
        SkyView::Twilight => horizon_view(sun, 0.03),
        SkyView::Moon => moon,
        SkyView::Stars => Vec3::new(0.15, 0.55, -0.82).normalize(),
        SkyView::CloudCumulus
        | SkyView::CloudStratocumulus
        | SkyView::CloudCirrus
        | SkyView::CloudOvercast
        | SkyView::CloudStorm => horizon_view(sun, 0.34),
    };
    SkyViewConfiguration {
        absolute_minute,
        sun_altitude_degrees: celestial.sun[1].asin().to_degrees(),
        moon_altitude_degrees: celestial.moon[1].asin().to_degrees(),
        lunar_illumination: celestial.lunar_illumination,
        camera_translation: Vec3::new(0.0, 2.0, 8.0),
        camera_direction: view_direction,
        vertical_fov_degrees: if matches!(view, SkyView::Moon) {
            12.0
        } else if matches!(view, SkyView::SunDetail) {
            20.0
        } else if is_cloud_view(view) {
            72.0
        } else {
            80.0
        },
    }
}
