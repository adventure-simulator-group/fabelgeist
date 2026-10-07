//! The live viewer: the simulation growing on its own thread, its snapshots
//! drawn as they arrive, and the panel steering it.

use adventuresim_environment::render::{ForestDetail, ForestFeed};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use bevy_panorbit_camera::PanOrbitCamera;
use fabelgeist_botany::scenes::Scene;
use fabelgeist_botany::viewer::profile::Profile;
use fabelgeist_botany::viewer::sim::{Command, Simulation};

use crate::frame_times::FrameTimes;
use crate::profile_controls;
use crate::scene_controls::{Request, START_SPEED, SceneControls};
use crate::sowing::Sowing;
use crate::stage::{self, Sun};
use crate::stats::Stats;

/// The panel's width, logical pixels.
const PANEL_WIDTH: f32 = 320.0;
/// A gap cut in the canopy, as a share of the scene's width, and at least
/// this many metres across.
const GAP_SHARE: f32 = 0.12;
const GAP_MINIMUM: f32 = 4.0;
/// A press that moves the pointer less than this is a click, not an orbit,
/// logical pixels.
const CLICK_SLACK: f32 = 5.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Scene,
    Profile,
}

#[derive(Resource)]
pub struct Viewer {
    simulation: Simulation,
    /// The scene generation the camera was last framed for.
    framed: u64,
    /// The last snapshot's bounds: lower and upper corner.
    bounds: [[f32; 3]; 2],
    stats: Stats,
    controls: SceneControls,
    profile: Profile,
    tab: Tab,
    sowing: Sowing,
    frame_times: FrameTimes,
    requests: Vec<Request>,
    /// Where the left button went down, if it is down.
    press: Option<Vec2>,
}

/// Run the live viewer on `scene`.
pub fn install(app: &mut App, scene: Scene) {
    let profile = profile_controls::drawable(Profile::default());
    let simulation = Simulation::start(scene, profile);
    simulation.send(Command::Speed(START_SPEED));
    app.insert_resource(Viewer {
        simulation,
        framed: 0,
        bounds: [[0.0; 3]; 2],
        stats: Stats::default(),
        controls: SceneControls::new(scene),
        profile,
        tab: Tab::Scene,
        sowing: Sowing::default(),
        frame_times: FrameTimes::new(),
        requests: Vec::new(),
        press: None,
    })
    .add_systems(Update, (receive, shortcuts, sow, apply).chain())
    .add_systems(EguiPrimaryContextPass, panel);
}

/// Draw the simulation's latest snapshot, framing a new scene.
fn receive(
    mut viewer: ResMut<Viewer>,
    mut feed: ResMut<ForestFeed>,
    mut cameras: Query<&mut PanOrbitCamera>,
) {
    viewer.frame_times.tick();
    let Some(snapshot) = viewer.simulation.take() else {
        return;
    };
    viewer.bounds = snapshot.bounds;
    if snapshot.scene_generation != viewer.framed {
        viewer.framed = snapshot.scene_generation;
        for mut camera in &mut cameras {
            stage::frame(&mut camera, snapshot.bounds);
        }
    }
    viewer.stats.record(&snapshot);
    feed.show(snapshot);
}

fn shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut viewer: ResMut<Viewer>,
) {
    let typing = contexts
        .ctx_mut()
        .is_ok_and(|ctx| ctx.egui_wants_keyboard_input());
    if typing {
        return;
    }
    if keys.just_pressed(KeyCode::Space) {
        let play = viewer.controls.toggle_play();
        viewer.requests.push(Request::Send(play));
    }
    if keys.just_pressed(KeyCode::KeyG) {
        viewer.requests.push(Request::CutGap);
    }
    if keys.just_pressed(KeyCode::KeyR) {
        let scene = viewer.controls.scene;
        viewer.requests.push(Request::Reset(scene));
    }
    if keys.just_pressed(KeyCode::KeyS) {
        viewer.sowing.toggle();
    }
}

/// With sowing on, a click on the scene throws a seed from the eye through
/// the clicked pixel; a drag still orbits.
fn sow(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut contexts: EguiContexts,
    mut viewer: ResMut<Viewer>,
) {
    let cursor = windows.iter().find_map(Window::cursor_position);
    let over_panel = contexts
        .ctx_mut()
        .is_ok_and(|ctx| ctx.egui_wants_pointer_input() || ctx.is_pointer_over_egui());
    if buttons.just_pressed(MouseButton::Left) {
        viewer.press = cursor.filter(|_| !over_panel);
    }
    if !buttons.just_released(MouseButton::Left) {
        return;
    }
    let (Some(pressed), Some(released)) = (viewer.press.take(), cursor) else {
        return;
    };
    if !viewer.sowing.active() || pressed.distance(released) > CLICK_SLACK {
        return;
    }
    let ray = cameras
        .iter()
        .find_map(|(camera, transform)| camera.viewport_to_world(transform, released).ok());
    if let Some(ray) = ray {
        let seed = viewer.sowing.throw(ray);
        viewer.simulation.send(seed);
    }
}

/// Carry out what the panel and the keys asked for, and keep the forest's
/// detail and the sun in step with the panel.
fn apply(
    mut viewer: ResMut<Viewer>,
    mut detail: ResMut<ForestDetail>,
    cameras: Query<&PanOrbitCamera>,
    mut sun: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
) {
    let focus = cameras
        .iter()
        .next()
        .map_or(Vec3::ZERO, |camera| camera.focus);
    let requests = std::mem::take(&mut viewer.requests);
    for request in requests {
        viewer.handle(request, focus);
    }
    let wanted = ForestDetail {
        tube_sides: viewer.profile.tube_sides,
        tube_spans: viewer.profile.tube_spans,
        leaf_blades: viewer.profile.leaf_blades,
        leaves: viewer.controls.leaves,
    };
    detail.set_if_neq(wanted);
    let lighting = viewer.controls.lighting;
    for (mut transform, mut light) in &mut sun {
        transform.set_if_neq(stage::sun_transform(lighting));
        if light.shadow_maps_enabled != lighting.shadows {
            light.shadow_maps_enabled = lighting.shadows;
        }
    }
}

impl Viewer {
    fn handle(&mut self, request: Request, focus: Vec3) {
        match request {
            Request::Send(command) => self.simulation.send(command),
            Request::Reset(scene) => {
                self.controls.adopt_climate(scene);
                self.stats.forget_history();
                self.sowing.forget_species();
                self.simulation.send(Command::Reset(scene));
                self.simulation.send(Command::Play(self.controls.playing));
                for command in self.controls.life() {
                    self.simulation.send(command);
                }
            }
            Request::CutGap => {
                let [low, high] = self.bounds;
                let size = (high[0] - low[0]).max(high[2] - low[2]);
                self.simulation.send(Command::Gap {
                    x: focus.x,
                    z: focus.z,
                    radius: (size * GAP_SHARE).max(GAP_MINIMUM),
                });
            }
        }
    }
}

#[expect(
    deprecated,
    reason = "egui's replacement requires a parent Ui, but this is the top-level panel"
)]
fn panel(mut contexts: EguiContexts, mut viewer: ResMut<Viewer>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let viewer = &mut *viewer;
    egui::SidePanel::right("controls")
        .exact_width(PANEL_WIDTH)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("Synthetic Silviculture");
            ui.weak("Makowski et al. 2019, live");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut viewer.tab, Tab::Scene, "Scene");
                ui.selectable_value(&mut viewer.tab, Tab::Profile, "Profile");
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| match viewer.tab {
                Tab::Scene => {
                    viewer.sowing.show(ui, viewer.stats.species());
                    viewer.controls.show(ui, &mut viewer.requests);
                    ui.separator();
                    viewer.stats.show(ui, viewer.frame_times.rate());
                    ui.separator();
                    ui.weak("Drag orbit, right-drag pan, wheel zoom");
                    ui.weak("Space play/pause, G gap, R reset, S sow");
                }
                Tab::Profile => {
                    if profile_controls::show(ui, &mut viewer.profile) {
                        let profile = profile_controls::drawable(viewer.profile);
                        viewer
                            .requests
                            .push(Request::Send(Command::Profile(profile)));
                    }
                }
            });
        });
}
