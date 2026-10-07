//! Botany's renderer drawing into a texture egui shows.

use std::sync::Arc;

use anyhow::{Context as _, Result};
use eframe::egui::TextureId;
use eframe::egui_wgpu::{RenderState, WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew};
use fabelgeist_botany::viewer::camera::OrbitCamera;
use fabelgeist_botany::viewer::profile::Profile;
use fabelgeist_botany::viewer::renderer::{Renderer, View};
use fabelgeist_botany::viewer::sim::Snapshot;

/// The format botany renders the view in, for egui to sample.
const VIEW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// The view's size in physical pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelSize {
    pub width: u32,
    pub height: u32,
}

/// The device eframe draws with, asked for the adapter's own limits: the
/// forest's instance and hierarchy buffers outgrow wgpu's defaults.
pub fn wgpu_options() -> WgpuConfiguration {
    WgpuConfiguration {
        wgpu_setup: WgpuSetup::CreateNew(WgpuSetupCreateNew {
            power_preference: wgpu::PowerPreference::HighPerformance,
            device_descriptor: Arc::new(|adapter| wgpu::DeviceDescriptor {
                label: Some("environment viewer"),
                required_features: adapter.features() & wgpu::Features::FLOAT32_FILTERABLE,
                required_limits: adapter.limits(),
                ..Default::default()
            }),
            ..WgpuSetupCreateNew::without_display_handle()
        }),
        ..Default::default()
    }
}

/// The renderer and the egui texture its target is registered as.
pub struct ViewTexture {
    renderer: Renderer,
    render_state: RenderState,
    texture: Option<TextureId>,
}

impl ViewTexture {
    pub fn new(render_state: &RenderState) -> Result<Self> {
        let renderer = Renderer::new(
            Arc::new(render_state.device.clone()),
            Arc::new(render_state.queue.clone()),
            VIEW_FORMAT,
        )
        .context("botany's renderer could not be created")?;
        Ok(Self {
            renderer,
            render_state: render_state.clone(),
            texture: None,
        })
    }

    pub fn upload(&mut self, snapshot: &Snapshot) {
        self.renderer.upload(snapshot);
    }

    /// Lower and upper corner of everything drawn.
    pub fn bounds(&self) -> [[f32; 3]; 2] {
        self.renderer.bounds()
    }

    /// Render a frame at `size` and return the texture holding it.
    pub fn render(
        &mut self,
        size: PixelSize,
        camera: &OrbitCamera,
        view: &View,
        profile: &Profile,
    ) -> TextureId {
        let (_, target, fresh) = self.renderer.resize(size.width, size.height);
        let mut egui_renderer = self.render_state.renderer.write();
        let device = &self.render_state.device;
        let texture = match self.texture {
            Some(id) if fresh => {
                egui_renderer.update_egui_texture_from_wgpu_texture(
                    device,
                    &target,
                    wgpu::FilterMode::Linear,
                    id,
                );
                id
            }
            Some(id) => id,
            None => {
                egui_renderer.register_native_texture(device, &target, wgpu::FilterMode::Linear)
            }
        };
        drop(egui_renderer);
        self.texture = Some(texture);
        self.renderer.render(camera, view, profile);
        texture
    }
}

/// eframe's wgpu state, which the viewer cannot run without.
pub fn render_state(creation: &eframe::CreationContext<'_>) -> Result<RenderState> {
    creation
        .wgpu_render_state
        .clone()
        .context("the viewer needs eframe's wgpu renderer")
}
