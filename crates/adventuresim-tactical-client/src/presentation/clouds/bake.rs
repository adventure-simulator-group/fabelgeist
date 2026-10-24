//! Native dome ray integration shared by endpoint baking and bounded checks.
use super::super::cloud_bake_assets::{
    CLOUD_BAKE_AZIMUTH_SEGMENTS, CLOUD_BAKE_CHANNELS, CLOUD_BAKE_ELEVATION_SEGMENTS,
    CLOUD_BAKE_TEXTURE_HEIGHT, CLOUD_BAKE_TEXTURE_WIDTH, image_from_rgba8,
};
use super::{
    ActiveCloudLayerOrdinal, CLOUD_BAKE_REFERENCE_EYE_METRES, CLOUD_BAKE_VERTICAL_SAMPLES,
    CloudBakeRequest, baked_cloud_density, cloud_bake_lighting_variation, cloud_ray_layer_interval,
    cloud_shell_altitude,
};
use bevy::prelude::*;

impl CloudBakeRequest {
    /// Integrate a native unit dome ray in the metre-based, Y-up bake frame.
    /// The RGBA8 result is the image boundary; no extra ray normalization changes
    /// the authored angular lattice or the integration arithmetic.
    pub(super) fn integrated_pixel(&self, direction: Vec3) -> [u8; 4] {
        let reference_eye = Vec3::new(0.0, CLOUD_BAKE_REFERENCE_EYE_METRES, 0.0);
        let advection_metres = self.advection_metres();
        let evolution = self.evolution();
        let mut optical_depth = 0.0;
        let mut lighting = 0.0;
        let mut variation = 0.0;
        for (slot, layer) in self.key.layers.iter().flatten().enumerate() {
            let Some((start, end)) = cloud_ray_layer_interval(reference_eye, direction, *layer)
            else {
                continue;
            };
            let step = (end - start) / CLOUD_BAKE_VERTICAL_SAMPLES as f32;
            for sample in 0..CLOUD_BAKE_VERTICAL_SAMPLES {
                let distance = start + (sample as f32 + 0.5) * step;
                let position = reference_eye + direction * distance;
                let height = ((cloud_shell_altitude(position) - layer.bottom_metres)
                    / layer.thickness_metres)
                    .clamp(0.0, 1.0);
                let density = baked_cloud_density(
                    position.xz() - advection_metres,
                    height,
                    *layer,
                    self.key.seed,
                    ActiveCloudLayerOrdinal::new(slot),
                    evolution,
                );
                let contribution = density * step * 0.001_45;
                optical_depth += contribution;
                let detail = cloud_bake_lighting_variation(
                    position.xz() - advection_metres,
                    height,
                    *layer,
                    self.key.seed,
                    ActiveCloudLayerOrdinal::new(slot),
                    evolution,
                );
                lighting += contribution * (0.18 + height * 0.76) * (0.18 + detail * 0.82);
                variation += contribution * detail;
            }
        }
        let alpha = 1.0 - (-optical_depth).exp();
        let lighting = if optical_depth > 0.0001 {
            lighting / optical_depth
        } else {
            0.0
        };
        let variation = if optical_depth > 0.0001 {
            variation / optical_depth
        } else {
            0.0
        };
        let sun_transmission = (-optical_depth * (0.45 + variation * 0.55)).exp();
        [
            (alpha * 255.0).round() as u8,
            (lighting.clamp(0.0, 1.0) * 255.0).round() as u8,
            (sun_transmission * 255.0).round() as u8,
            (variation.clamp(0.0, 1.0) * 255.0).round() as u8,
        ]
    }
}

/// Bake every ray of the production dome, retaining row-major pixel order.
pub(super) fn cloud_bake_image(request: &CloudBakeRequest) -> Image {
    let mut pixels = Vec::with_capacity(
        CLOUD_BAKE_TEXTURE_WIDTH as usize
            * CLOUD_BAKE_TEXTURE_HEIGHT as usize
            * CLOUD_BAKE_CHANNELS,
    );
    for y in 0..CLOUD_BAKE_TEXTURE_HEIGHT {
        for x in 0..CLOUD_BAKE_TEXTURE_WIDTH {
            pixels.extend(request.integrated_pixel(cloud_bake_direction(x, y)));
        }
    }
    image_from_rgba8(pixels)
}

/// Maps the same equirectangular UV as the dome mesh: U is azimuth and V is
/// elevation. The first and final columns intentionally evaluate the exact
/// same ray, allowing clamp sampling to cross the visible seam continuously.
pub(super) fn cloud_bake_direction(x: u32, y: u32) -> Vec3 {
    let azimuth_index = if x == CLOUD_BAKE_AZIMUTH_SEGMENTS {
        0
    } else {
        x
    };
    let azimuth =
        (azimuth_index as f32 / CLOUD_BAKE_AZIMUTH_SEGMENTS as f32 - 0.5) * core::f32::consts::TAU;
    let elevation = y as f32 / CLOUD_BAKE_ELEVATION_SEGMENTS as f32 * core::f32::consts::FRAC_PI_2;
    let horizontal = elevation.cos();
    Vec3::new(
        horizontal * azimuth.cos(),
        elevation.sin(),
        horizontal * azimuth.sin(),
    )
}
