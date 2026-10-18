//! The live viewer: the simulation growing on its own thread, its snapshots
//! drawn as they arrive, and a game's HUD over them -- the year and the
//! population, the time controls, a hotbar of tools, and a menu for the
//! rest.

use adventuresim_environment::render::{ForestDetail, ForestFeed};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use bevy_panorbit_camera::PanOrbitCamera;
use fabelgeist_botany::scenes::Scene;
use fabelgeist_botany::viewer::profile::Profile;
use fabelgeist_botany::viewer::sim::{Command, Simulation};

use crate::frame_times::FrameTimes;
use crate::menu::Menu;
use crate::stage::{self, Lighting, Orbit, Sun};
use crate::stats::Stats;
use crate::tools::Tool;
use crate::world::{Request, World};
use crate::{hotbar, hud, menu, pointer, profile_controls, theme};

/// A gap cleared in the canopy, as a share of the scene's width, and at
/// least this many metres across.
const GAP_SHARE: f32 = 0.08;
const GAP_MINIMUM: f32 = 4.0;

#[derive(Resource)]
pub struct Viewer {
    pub simulation: Simulation,
    /// The scene generation the camera was last framed for.
    framed: u64,
    /// The last snapshot's bounds: lower and upper corner.
    bounds: [[f32; 3]; 2],
    pub stats: Stats,
    pub world: World,
    pub lighting: Lighting,
    /// Whether the foliage is drawn.
    pub leaves: bool,
    pub profile: Profile,
    pub tool: Tool,
    pub menu: Menu,
    /// Whether the figures for developers show.
    pub debug: bool,
    pub frame_times: FrameTimes,
    pub requests: Vec<Request>,
}

/// Run the live viewer on `scene`.
pub fn install(app: &mut App, scene: Scene) {
    let profile = profile_controls::drawable(Profile::default());
    let simulation = match Simulation::start(scene, profile) {
        Ok(simulation) => simulation,
        Err(error) => {
            error!("Could not start the botanical viewer: {error}");
            return;
        }
    };
    let world = World::new(scene);
    simulation.send(Command::Speed(world.pace.years_per_second()));
    app.insert_resource(Viewer {
        simulation,
        framed: 0,
        bounds: [[0.0; 3]; 2],
        stats: Stats::default(),
        world,
        lighting: Lighting::default(),
        leaves: true,
        profile,
        tool: Tool::Look,
        menu: Menu::default(),
        debug: false,
        frame_times: FrameTimes::new(),
        requests: Vec::new(),
    })
    .add_systems(Startup, (spawn_camera, stage::sun))
    .add_systems(Update, (receive, apply).chain())
    .add_systems(EguiPrimaryContextPass, interface);
    pointer::install(app);
}

/// An orbiting camera, turned by the mouse.
fn spawn_camera(mut commands: Commands) {
    let mut orbit = PanOrbitCamera::default();
    stage::first_orbit().apply(&mut orbit);
    commands.spawn((stage::camera(), orbit));
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
        let orbit = Orbit::framing(snapshot.bounds);
        for mut camera in &mut cameras {
            orbit.apply(&mut camera);
        }
    }
    viewer.stats.record(&snapshot);
    feed.show(snapshot);
}

/// Carry out what the interface asked for, and keep the forest's detail and
/// the sun in step with the settings.
fn apply(
    mut viewer: ResMut<Viewer>,
    mut detail: ResMut<ForestDetail>,
    mut sun: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
) {
    let requests = std::mem::take(&mut viewer.requests);
    for request in requests {
        viewer.handle(request);
    }
    let wanted = ForestDetail {
        tube_sides: viewer.profile.tube_sides,
        tube_spans: viewer.profile.tube_spans,
        leaf_blades: viewer.profile.leaf_blades,
        full_pixels: viewer.profile.full_pixels,
        middle_pixels: viewer.profile.middle_pixels,
        leaves: viewer.leaves,
    };
    detail.set_if_neq(wanted);
    let lighting = viewer.lighting;
    for (mut transform, mut light) in &mut sun {
        transform.set_if_neq(stage::sun_transform(lighting));
        if light.shadow_maps_enabled != lighting.shadows {
            light.shadow_maps_enabled = lighting.shadows;
        }
    }
}

/// The HUD, the hotbar, the menu when it is open, and the keys.
fn interface(
    mut contexts: EguiContexts,
    mut viewer: ResMut<Viewer>,
    mut styled: Local<bool>,
    mut exit: MessageWriter<AppExit>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    if !*styled {
        theme::apply(ctx);
        *styled = true;
    }
    let viewer = &mut *viewer;
    let menu_was_open = viewer.menu.open;
    if menu_was_open && menu::show(ctx, viewer) {
        exit.write(AppExit::Success);
    }
    hud::show(ctx, viewer);
    hotbar::show(ctx, viewer);
    if !menu_was_open {
        keys(ctx, viewer);
    }
}

/// Space plays and pauses, the digits pick a plant to sow, G clears, Esc
/// puts the tool down or opens the menu, F3 shows the figures.
fn keys(ctx: &egui::Context, viewer: &mut Viewer) {
    if ctx.egui_wants_keyboard_input() {
        return;
    }
    let pressed = |key| ctx.input(|input| input.key_pressed(key));
    if pressed(egui::Key::Space) {
        let play = viewer.world.toggle_play();
        viewer.requests.push(Request::Send(play));
    }
    if pressed(egui::Key::G) {
        viewer.tool = Tool::Clear;
    }
    if pressed(egui::Key::F3) {
        viewer.debug = !viewer.debug;
    }
    if pressed(egui::Key::Escape) {
        if viewer.tool == Tool::Look {
            viewer.menu.open = true;
        } else {
            viewer.tool = Tool::Look;
        }
    }
    let species = viewer.stats.species().len();
    for (index, key) in hotbar::SPECIES_KEYS.into_iter().enumerate() {
        if index < species && pressed(key) {
            viewer.tool = Tool::Sow(index);
        }
    }
}

impl Viewer {
    fn handle(&mut self, request: Request) {
        match request {
            Request::Send(command) => self.simulation.send(command),
            Request::Load(scene) => {
                self.world.adopt_climate(scene);
                self.stats.forget_history();
                self.tool = Tool::Look;
                self.simulation.send(Command::Reset(scene));
                for command in self.world.start() {
                    self.simulation.send(command);
                }
            }
        }
    }

    /// The radius of a gap cleared by hand, metres.
    pub fn gap_radius(&self) -> f32 {
        let [low, high] = self.bounds;
        let size = (high[0] - low[0]).max(high[2] - low[2]);
        (size * GAP_SHARE).max(GAP_MINIMUM)
    }
}
