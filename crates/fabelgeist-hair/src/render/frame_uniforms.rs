//! Projection and native uniform construction for one renderer frame.

use super::{
    Camera, Globals, HairRenderer, Light, Lighting, Mat4, look_at, math, multiply, orthographic,
};
use crate::gpu::error::Result;
use crate::lod::{HairShading, PixelSpan, Quality};
use crate::material::HairMaterial;
use fabelgeist_math::units::Metres;

pub(super) struct FrameProjection {
    pub pixels: PixelSpan,
    view_proj: Mat4,
    camera: [f32; 4],
    viewport: [f32; 4],
    light_view_proj: Mat4,
    radius: Metres,
}

struct LightUniform {
    direction: [f32; 4],
    color: [f32; 4],
}

impl FrameProjection {
    pub(super) fn new(
        renderer: &HairRenderer,
        camera: &Camera,
        lighting: &Lighting,
        extent: wgpu::Extent3d,
    ) -> Self {
        let (width, height) = (extent.width, extent.height);
        let aspect = width as f32 / height as f32;
        let distance = math::length(math::sub(renderer.center, camera.eye)).max(1e-3);
        let pixel_angle = 2.0 * (camera.fov_y * 0.5).tan() / height as f32;
        let pixels = PixelSpan::from_pixels(0.25 / (distance * pixel_angle));
        // The key light's frustum: a box around everything, from the light.
        let [mut lo, mut hi] = renderer.bounds;
        // Moving hair may leave its rest bounds: room for it in the light's
        // frustum.
        if renderer.dynamics.is_some() {
            lo = math::sub(lo, [0.12; 3]);
            hi = math::add(hi, [0.12; 3]);
        }
        let center = math::scale(math::add(lo, hi), 0.5);
        let radius = math::length(math::sub(hi, lo)) * 0.5;
        let key = math::normalize(lighting.key.direction);
        let light_up = if key[1].abs() > 0.99 {
            [0.0, 0.0, 1.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let light_eye = math::add(center, math::scale(key, radius * 2.0));
        let light_view_proj = multiply(
            &orthographic(-radius, radius, -radius, radius, radius, radius * 3.0),
            &look_at(light_eye, center, light_up),
        );
        Self {
            pixels,
            view_proj: camera.view_projection(aspect),
            camera: [camera.eye[0], camera.eye[1], camera.eye[2], pixel_angle],
            viewport: [
                width as f32,
                height as f32,
                1.0 / width as f32,
                1.0 / height as f32,
            ],
            light_view_proj,
            radius: Metres::from_metres(radius),
        }
    }
}

impl Globals {
    pub(super) fn for_frame(
        geometry: &FrameProjection,
        renderer: &HairRenderer,
        lighting: &Lighting,
        material: &HairMaterial,
        quality: Quality,
    ) -> Result<Self> {
        let shadow_resolution = quality
            .shadows
            .size()
            .unwrap_or(renderer.shadows.resolution);
        let radius = geometry.radius.metres();
        let texel = 2.0 * radius / shadow_resolution.texels() as f32;
        let base = material.base_color();
        let darkness = 1.0 - (base[0] * 0.3 + base[1] * 0.59 + base[2] * 0.11).clamp(0.0, 1.0);
        let key = LightUniform::from(&lighting.key);
        let fill = LightUniform::from(&lighting.fill);
        let rim = LightUniform::from(&lighting.rim);
        Self {
            view_proj: geometry.view_proj,
            light_view_proj: geometry.light_view_proj,
            head: renderer.head_matrix,
            camera: geometry.camera,
            jitter: [0.0, 0.0, 0.0, lighting.exposure],
            viewport: geometry.viewport,
            key_dir: key.direction,
            key_color: key.color,
            fill_dir: fill.direction,
            fill_color: fill.color,
            rim_dir: rim.direction,
            rim_color: rim.color,
            sky_top: pack(lighting.sky_top, 0.0),
            sky_bottom: pack(lighting.sky_bottom, 0.0),
            base_color: [base[0], base[1], base[2], material.random_color],
            material: [
                material.roughness,
                material.radial_roughness,
                material.cuticle_tilt,
                material.random_roughness,
            ],
            taper: [material.taper.tip, material.taper.shape, 0.35, 0.45],
            dom: [0.003, 0.01, 0.03, 2.0 * radius],
            shadow: [
                texel,
                0.9,
                0.35 + 0.5 * darkness,
                if quality.shadows.enabled() { 1.0 } else { 0.0 },
            ],
            mode: [
                if quality.shading == HairShading::Marschner {
                    1.0
                } else {
                    0.0
                },
                renderer.atlas.variants.count() as f32,
                material.tip_lightness,
                material.grey,
            ],
            // The key's softness: a 12 mm penumbra, in shadow-map UV.
            skin: pack(lighting.skin, 0.012 / (2.0 * radius)),
            roots: [
                material.root_darkness_start,
                material.root_darkness_end,
                material.root_darkness_strength,
                material.wave_strength,
            ],
            waves: [
                material.wave_scale,
                material.wave_phase.to_radians(),
                0.0,
                0.0,
            ],
        }
        .validate()
    }
}

impl From<&Light> for LightUniform {
    fn from(light: &Light) -> Self {
        let direction = math::normalize(light.direction);
        Self {
            direction: pack(direction, light.intensity),
            color: pack(light.color, 0.0),
        }
    }
}

/// Pack a native three-component shader vector and its scalar fourth lane.
fn pack(vector: [f32; 3], fourth: f32) -> [f32; 4] {
    [vector[0], vector[1], vector[2], fourth]
}
