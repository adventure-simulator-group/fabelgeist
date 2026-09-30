//! Rotation from equatorial star coordinates into the local sky.

use adventuresim_world_schema::calendar::StrategicMinute;
use adventuresim_world_schema::coordinates::{LatitudeMicrodegrees, LongitudeMicrodegrees};
use bevy::math::{Mat3, Mat4, Vec3};

pub(super) fn equatorial_to_world(
    absolute_minute: StrategicMinute,
    latitude: LatitudeMicrodegrees,
    longitude: LongitudeMicrodegrees,
) -> Mat4 {
    let latitude = (latitude.degrees() as f32).to_radians();
    let longitude = (longitude.degrees() as f32).to_radians();
    let day = absolute_minute.days_since_epoch_f32();
    let sidereal = (4.383_4 + day * core::f32::consts::TAU * 1.002_737_9 + longitude)
        .rem_euclid(core::f32::consts::TAU);
    let (sin_latitude, cos_latitude) = latitude.sin_cos();
    let (sin_sidereal, cos_sidereal) = sidereal.sin_cos();
    Mat4::from_mat3(Mat3::from_cols(
        Vec3::new(
            -sin_sidereal,
            cos_latitude * cos_sidereal,
            sin_latitude * cos_sidereal,
        ),
        Vec3::new(0.0, sin_latitude, -cos_latitude),
        Vec3::new(
            cos_sidereal,
            cos_latitude * sin_sidereal,
            sin_latitude * sin_sidereal,
        ),
    ))
}
