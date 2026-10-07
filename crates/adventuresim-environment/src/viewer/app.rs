//! The window: botany's view of the ecosystem, and the panel steering it.

use anyhow::Result;
use eframe::egui::{self, Key, Sense, Ui, Vec2};
use fabelgeist_botany::scenes::Scene;
use fabelgeist_botany::viewer::camera::OrbitCamera;
use fabelgeist_botany::viewer::profile::Profile;
use fabelgeist_botany::viewer::sim::{Command, Simulation};

use crate::frame_times::FrameTimes;
use crate::navigation;
use crate::profile_controls;
use crate::scene_controls::{Request, START_SPEED, SceneControls};
use crate::stats::Stats;
use crate::view_texture::{self, PixelSize, ViewTexture};

/// The panel's width, logical pixels.
const PANEL_WIDTH: f32 = 320.0;
/// A gap cut in the canopy, as a share of the scene's width, and at least
/// this many metres across.
const GAP_SHARE: f32 = 0.12;
const GAP_MINIMUM: f32 = 4.0;
/// The camera before the first snapshot frames the scene, metres.
const FIRST_FRAMING: [[f32; 3]; 2] = [[-30.0, 0.0, -30.0], [30.0, 20.0, 30.0]];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Scene,
    Profile,
}

pub struct EnvironmentViewer {
    simulation: Simulation,
    view: ViewTexture,
    camera: OrbitCamera,
    /// The scene generation the camera was last framed for.
    framed: u64,
    stats: Stats,
    controls: SceneControls,
    profile: Profile,
    tab: Tab,
    frame_times: FrameTimes,
}

impl EnvironmentViewer {
    pub fn new(creation: &eframe::CreationContext<'_>, scene: Scene) -> Result<Self> {
        let render_state = view_texture::render_state(creation)?;
        let profile = Profile::default();
        let simulation = Simulation::start(scene, profile);
        simulation.send(Command::Speed(START_SPEED));
        let [lo, hi] = FIRST_FRAMING;
        Ok(Self {
            simulation,
            view: ViewTexture::new(&render_state)?,
            camera: OrbitCamera::framing(lo, hi),
            framed: 0,
            stats: Stats::default(),
            controls: SceneControls::new(scene),
            profile,
            tab: Tab::Scene,
            frame_times: FrameTimes::new(),
        })
    }

    /// Take the simulation's latest snapshot, if it has a new one.
    fn receive(&mut self) {
        let Some(snapshot) = self.simulation.take() else {
            return;
        };
        self.view.upload(&snapshot);
        if snapshot.scene_generation != self.framed {
            let [lo, hi] = snapshot.bounds;
            self.camera = OrbitCamera::framing(lo, hi);
            self.framed = snapshot.scene_generation;
        }
        self.stats.record(&snapshot);
    }

    fn handle(&mut self, request: Request) {
        match request {
            Request::Send(command) => self.simulation.send(command),
            Request::Reset(scene) => {
                self.controls.adopt_climate(scene);
                self.stats.forget_history();
                self.simulation.send(Command::Reset(scene));
                self.simulation.send(Command::Play(self.controls.playing));
            }
            Request::CutGap => {
                let [lo, hi] = self.view.bounds();
                let size = (hi[0] - lo[0]).max(hi[2] - lo[2]);
                self.simulation.send(Command::Gap {
                    x: self.camera.target[0],
                    z: self.camera.target[2],
                    radius: (size * GAP_SHARE).max(GAP_MINIMUM),
                });
            }
        }
    }

    fn shortcuts(&mut self, ui: &Ui, requests: &mut Vec<Request>) {
        if ui.ctx().egui_wants_keyboard_input() {
            return;
        }
        ui.input(|input| {
            if input.key_pressed(Key::Space) {
                requests.push(Request::Send(self.controls.toggle_play()));
            }
            if input.key_pressed(Key::G) {
                requests.push(Request::CutGap);
            }
            if input.key_pressed(Key::R) {
                requests.push(Request::Reset(self.controls.scene));
            }
        });
    }

    fn panel(&mut self, ui: &mut Ui, requests: &mut Vec<Request>) {
        ui.heading("Synthetic Silviculture");
        ui.weak("Makowski et al. 2019, live");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tab, Tab::Scene, "Scene");
            ui.selectable_value(&mut self.tab, Tab::Profile, "Profile");
        });
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| match self.tab {
            Tab::Scene => {
                self.controls.show(ui, requests);
                ui.separator();
                self.stats.show(ui, self.frame_times.rate());
                ui.separator();
                ui.weak("Drag orbit, right-drag pan, wheel zoom");
                ui.weak("Space play/pause, G gap, R reset");
            }
            Tab::Profile => {
                if profile_controls::show(ui, &mut self.profile) {
                    requests.push(Request::Send(Command::Profile(self.profile)));
                }
            }
        });
    }

    fn scene_view(&mut self, ui: &mut Ui) {
        let size = ui.available_size();
        let scale = ui.ctx().pixels_per_point();
        let pixels = PixelSize {
            width: (size.x * scale).round().max(1.0) as u32,
            height: (size.y * scale).round().max(1.0) as u32,
        };
        let texture = self
            .view
            .render(pixels, &self.camera, &self.controls.view, &self.profile);
        let image = egui::Image::new(egui::load::SizedTexture::new(texture, size))
            .sense(Sense::click_and_drag());
        let response = ui.add_sized(Vec2::new(size.x, size.y), image);
        navigation::steer(ui, &response, &mut self.camera);
    }
}

impl eframe::App for EnvironmentViewer {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.frame_times.tick();
        self.receive();
        let mut requests = Vec::new();
        self.shortcuts(ui, &mut requests);
        egui::Panel::right("controls")
            .exact_size(PANEL_WIDTH)
            .show_inside(ui, |ui| self.panel(ui, &mut requests));
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| self.scene_view(ui));
        for request in requests {
            self.handle(request);
        }
        // The forest keeps growing between inputs.
        ui.ctx().request_repaint();
    }
}
