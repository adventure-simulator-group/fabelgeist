//! Bounded per-view values for diagnosing the shared renderer in browser captures.
use bevy::{camera::Exposure, ecs::system::SystemParam, prelude::*};
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct ViewLighting {
    position: Vec3,
    exposure_ev100: f32,
    camera_daylight: Vec3,
    subject_daylight: Vec3,
}

#[derive(SystemParam)]
pub(super) struct LightingDiagnostics<'w, 's> {
    pub gpu: Res<'w, crate::presentation::interior_lighting::InteriorLightingGpu>,
    cameras: Query<
        'w,
        's,
        (&'static Camera, &'static GlobalTransform, &'static Exposure),
        With<super::views::StrategicCamera>,
    >,
}

impl LightingDiagnostics<'_, '_> {
    pub fn views(&self) -> Vec<ViewLighting> {
        self.cameras
            .iter()
            .filter(|(camera, ..)| camera.is_active)
            .map(|(_, transform, exposure)| ViewLighting {
                position: transform.translation(),
                exposure_ev100: exposure.ev100,
                camera_daylight: self.gpu.sample_at(transform.translation()),
                subject_daylight: self.gpu.sample_at(
                    transform.translation()
                        + *transform.forward() * super::staging::CONVERSATION_DISTANCE_METRES,
                ),
            })
            .collect()
    }
}
