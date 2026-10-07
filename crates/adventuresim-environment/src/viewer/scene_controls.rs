//! The scene tab: what grows, how fast, in what climate, and how it is lit.

use std::f32::consts::TAU;

use eframe::egui::{self, Slider, Ui};
use fabelgeist_botany::scenes::Scene;
use fabelgeist_botany::viewer::renderer::View;
use fabelgeist_botany::viewer::sim::Command;

/// Years a second at which the speed slider means "as fast as it goes".
const UNLIMITED_SPEED: f32 = 200.0;
/// The speed a scene starts at, simulated years a wall-clock second.
pub const START_SPEED: f32 = 20.0;

/// What the panel asks of the viewer beyond the simulation's commands.
pub enum Request {
    Send(Command),
    Reset(Scene),
    CutGap,
}

pub struct SceneControls {
    pub scene: Scene,
    pub playing: bool,
    speed: f32,
    temperature: f32,
    precipitation: f32,
    global_shadowing: bool,
    /// Whether plants spread seed, and whether plants and vines die; kept
    /// from scene to scene.
    seeding: bool,
    mortality: bool,
    pub view: View,
}

impl SceneControls {
    pub fn new(scene: Scene) -> Self {
        let mut controls = Self {
            scene,
            playing: true,
            speed: START_SPEED,
            temperature: 0.0,
            precipitation: 0.0,
            global_shadowing: true,
            seeding: true,
            mortality: true,
            view: View {
                sun_azimuth: 0.9,
                sun_elevation: 0.75,
                shadows: true,
                leaves: true,
            },
        };
        controls.adopt_climate(scene);
        controls
    }

    /// The climate controls start from a scene's own climate.
    pub fn adopt_climate(&mut self, scene: Scene) {
        let climate = scene.build(1).climate;
        self.scene = scene;
        self.temperature = climate.temperature;
        self.precipitation = climate.precipitation;
    }

    /// What a new scene is told so its life follows the switches.
    pub fn life(&self) -> [Command; 2] {
        [
            Command::Seeding(self.seeding),
            Command::Mortality(self.mortality),
        ]
    }

    pub fn toggle_play(&mut self) -> Command {
        self.playing = !self.playing;
        Command::Play(self.playing)
    }

    pub fn show(&mut self, ui: &mut Ui, requests: &mut Vec<Request>) {
        ui.strong("Scene");
        ui.horizontal(|ui| {
            let mut scene = self.scene;
            egui::ComboBox::from_id_salt("scene")
                .selected_text(scene.name())
                .show_ui(ui, |ui| {
                    for choice in Scene::ALL {
                        ui.selectable_value(&mut scene, choice, choice.name());
                    }
                });
            let reset = ui.button("Reset").clicked();
            if scene != self.scene || reset {
                requests.push(Request::Reset(scene));
            }
        });
        self.time(ui, requests);
        self.climate(ui, requests);
        let cut_gap = ui.button("Cut a gap (G)").clicked();
        if cut_gap {
            requests.push(Request::CutGap);
        }
        let shading = ui.checkbox(&mut self.global_shadowing, "Canopy shading (simulation)");
        if shading.changed() {
            requests.push(Request::Send(Command::GlobalShadowing(
                self.global_shadowing,
            )));
        }
        let seeding = ui.checkbox(&mut self.seeding, "Plants spread seeds");
        if seeding.changed() {
            requests.push(Request::Send(Command::Seeding(self.seeding)));
        }
        let mortality = ui.checkbox(&mut self.mortality, "Plants die (off: kept as they were)");
        if mortality.changed() {
            requests.push(Request::Send(Command::Mortality(self.mortality)));
        }
        ui.checkbox(&mut self.view.shadows, "Sun shadows");
        ui.checkbox(&mut self.view.leaves, "Foliage");
        ui.weak("Sun: direction, height");
        ui.add(Slider::new(&mut self.view.sun_azimuth, 0.0..=TAU).text("direction"));
        ui.add(Slider::new(&mut self.view.sun_elevation, 0.08..=1.5).text("height"));
    }

    fn time(&mut self, ui: &mut Ui, requests: &mut Vec<Request>) {
        ui.horizontal(|ui| {
            let play_label = if self.playing { "Pause" } else { "Play" };
            let play = ui.button(play_label).clicked();
            if play {
                requests.push(Request::Send(self.toggle_play()));
            }
            let step = ui
                .add_enabled(!self.playing, egui::Button::new("Step"))
                .clicked();
            if step {
                requests.push(Request::Send(Command::Step));
            }
        });
        let label = if self.speed >= UNLIMITED_SPEED {
            "Speed: as fast as it goes".to_owned()
        } else {
            format!("Speed: {:.0} years/s", self.speed)
        };
        ui.weak(label);
        if ui
            .add(Slider::new(&mut self.speed, 1.0..=UNLIMITED_SPEED).show_value(false))
            .changed()
        {
            let speed = if self.speed >= UNLIMITED_SPEED {
                f32::INFINITY
            } else {
                self.speed
            };
            requests.push(Request::Send(Command::Speed(speed)));
        }
    }

    fn climate(&mut self, ui: &mut Ui, requests: &mut Vec<Request>) {
        ui.strong("Climate");
        ui.weak(format!(
            "Temperature at the valley floor: {:.1} C",
            self.temperature
        ));
        let temperature = ui.add(
            Slider::new(&mut self.temperature, -12.0..=32.0)
                .step_by(0.5)
                .show_value(false),
        );
        ui.weak(format!("Precipitation: {:.0} mm", self.precipitation));
        let precipitation = ui.add(
            Slider::new(&mut self.precipitation, 50.0..=4000.0)
                .step_by(25.0)
                .show_value(false),
        );
        if temperature.changed() || precipitation.changed() {
            requests.push(Request::Send(Command::Climate {
                temperature: self.temperature,
                precipitation: self.precipitation,
            }));
        }
    }
}
