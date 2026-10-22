//! The menu, over a dimmed scene: which scene grows, the world's climate
//! and life, and how it is shown. The world keeps growing behind it.

use std::f32::consts::TAU;

use bevy_egui::egui::{
    self, Align, Context, Id, Layout, Response, RichText, Sense, Slider, Ui, vec2,
};
use fabelgeist_botany::scenes::Scene;
use fabelgeist_botany::viewer::sim::Command;

use crate::live::Viewer;
use crate::world::{self, Request};
use crate::{profile_controls, theme, widgets};

/// The menu's width, logical pixels.
const WIDTH: f32 = 520.0;
/// A scene card's height, logical pixels.
const CARD_HEIGHT: f32 = 76.0;
/// Room inside a scene card, logical pixels.
const CARD_INSET: f32 = 12.0;
/// The menu's title size, logical pixels.
const TITLE_SIZE: f32 = 26.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Page {
    #[default]
    Scenes,
    World,
    Display,
}

#[derive(Default)]
pub struct Menu {
    pub open: bool,
    page: Page,
}

/// Show the menu; true when the player chose to quit.
pub fn show(ctx: &Context, viewer: &mut Viewer) -> bool {
    let mut quit = false;
    let modal = egui::Modal::new(Id::new("menu"))
        .frame(theme::panel().inner_margin(egui::Margin::same(20)))
        .backdrop_color(theme::BACKDROP)
        .show(ctx, |ui| {
            ui.set_width(WIDTH);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Menu").size(TITLE_SIZE).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    for (page, label) in [
                        (Page::Display, "Display"),
                        (Page::World, "World"),
                        (Page::Scenes, "Scenes"),
                    ]
                    .into_iter()
                    {
                        ui.selectable_value(&mut viewer.menu.page, page, label);
                    }
                });
            });
            ui.separator();
            match viewer.menu.page {
                Page::Scenes => scenes(ui, viewer),
                Page::World => world_page(ui, viewer),
                Page::Display => display(ui, viewer),
            }
            ui.add_space(8.0);
            ui.separator();
            ui.horizontal(|ui| {
                quit = ui.button("Quit").clicked();
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let resume = ui.button(RichText::new("Resume").strong()).clicked();
                    if resume {
                        viewer.menu.open = false;
                    }
                });
            });
        });
    if modal.should_close() {
        viewer.menu.open = false;
    }
    quit
}

/// A card for every scene; picking one starts it afresh.
fn scenes(ui: &mut Ui, viewer: &mut Viewer) {
    widgets::section(ui, "Scenes");
    let mut chosen = None;
    egui::Grid::new("scenes")
        .num_columns(2)
        .spacing(vec2(10.0, 10.0))
        .show(ui, |ui| {
            for (index, scene) in Scene::ALL.into_iter().enumerate() {
                let current = viewer.world.scene == scene;
                if card(ui, scene, current).clicked() {
                    chosen = Some(scene);
                }
                if index % 2 == 1 {
                    ui.end_row();
                }
            }
        });
    if let Some(scene) = chosen {
        viewer.requests.push(Request::Load(scene));
        viewer.menu.open = false;
    }
}

fn card(ui: &mut Ui, scene: Scene, current: bool) -> Response {
    let width = (WIDTH - 10.0) * 0.5;
    let (rect, response) = ui.allocate_exact_size(vec2(width, CARD_HEIGHT), Sense::click());
    let fill = if current {
        theme::ACCENT.gamma_multiply(0.25)
    } else if response.hovered() {
        theme::HOVERED
    } else {
        theme::RAISED
    };
    let painter = ui.painter();
    painter.rect_filled(rect, theme::ROUNDING_SMALL, fill);
    if current {
        painter.rect_stroke(
            rect,
            theme::ROUNDING_SMALL,
            egui::Stroke::new(1.5_f32, theme::ACCENT),
            egui::StrokeKind::Inside,
        );
    }
    let inner = rect.shrink(CARD_INSET);
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Min)),
    );
    child.spacing_mut().item_spacing.y = 2.0;
    child.label(RichText::new(scene.name()).strong());
    child.label(
        RichText::new(world::pitch(scene))
            .small()
            .color(theme::MUTED),
    );
    let hover = if current {
        "Growing now: pick it to start over"
    } else {
        "Start this scene"
    };
    response.on_hover_text(hover)
}

/// The climate, and what plants may do.
fn world_page(ui: &mut Ui, viewer: &mut Viewer) {
    let world = &mut viewer.world;
    let requests = &mut viewer.requests;
    widgets::section(ui, "Climate");
    // UI sliders expose native display units; state and commands stay typed.
    let mut temperature_degrees = world.temperature.degrees();
    let mut rainfall_mm = world.precipitation.millimetres();
    let temperature = widgets::setting(
        ui,
        "Temperature",
        format!("{temperature_degrees:.1} °C"),
        Slider::new(&mut temperature_degrees, -12.0..=32.0).step_by(0.5),
    );
    let precipitation = widgets::setting(
        ui,
        "Rainfall",
        format!("{rainfall_mm:.0} mm a year"),
        Slider::new(&mut rainfall_mm, 50.0..=4000.0).step_by(25.0),
    );
    if temperature.changed() || precipitation.changed() {
        world.temperature = fabelgeist_botany::units::Celsius::from_degrees(temperature_degrees);
        world.precipitation = fabelgeist_botany::units::Millimetres::from_millimetres(rainfall_mm);
        requests.push(Request::Send(world.climate()));
    }
    ui.label(
        RichText::new(format!(
            "Makes {}",
            world::biome_name(world.biome()).to_lowercase()
        ))
        .small()
        .color(theme::MUTED),
    );
    widgets::section(ui, "Life");
    let seeding = widgets::switch(ui, &mut world.seeding, "Plants spread seeds");
    if seeding.changed() {
        requests.push(Request::Send(Command::Seeding(world.seeding)));
    }
    let mortality = widgets::switch(ui, &mut world.mortality, "Plants die");
    if mortality.changed() {
        requests.push(Request::Send(Command::Mortality(world.mortality)));
    }
    let shading = widgets::switch(ui, &mut world.canopy_shading, "Tall plants shade the rest");
    if shading.changed() {
        requests.push(Request::Send(Command::GlobalShadowing(
            world.canopy_shading,
        )));
    }
    egui::CollapsingHeader::new("Simulation")
        .default_open(false)
        .show(ui, |ui| {
            if profile_controls::growth(ui, &mut viewer.profile) {
                let profile = profile_controls::drawable(viewer.profile);
                requests.push(Request::Send(Command::Profile(profile)));
            }
        });
}

/// Trees as baked impostors: when scene trees switch, and the woods round
/// the scene.
fn impostors(ui: &mut Ui, viewer: &mut Viewer) {
    widgets::section(ui, "Impostors");
    let settings = &mut viewer.impostors;
    widgets::switch(ui, &mut settings.enabled, "Trees as impostors (I)");
    widgets::setting(
        ui,
        "Switch below",
        if settings.switch >= 1_000.0 {
            "always".to_owned()
        } else {
            format!("{:.0} px", settings.switch)
        },
        Slider::new(&mut settings.switch, 0.0..=1_000.0).logarithmic(true),
    );
    widgets::setting(
        ui,
        "Woods",
        format!("{} trees", crate::hud::grouped(settings.woods as usize)),
        Slider::new(&mut settings.woods, 0..=200_000).logarithmic(true),
    );
    widgets::setting(
        ui,
        "Woods reach",
        format!("{:.0} m", settings.woods_reach),
        Slider::new(&mut settings.woods_reach, 100.0..=4_000.0).logarithmic(true),
    );
    widgets::setting(
        ui,
        "Leaf density",
        format!("×{:.2}", settings.density),
        Slider::new(&mut settings.density, 0.5..=4.0),
    );
    widgets::setting(
        ui,
        "Leaf size, baked",
        format!("×{:.2}", settings.leaf_size),
        Slider::new(&mut settings.leaf_size, 0.5..=3.0),
    );
    widgets::switch(ui, &mut settings.tint, "Tint impostors (T)");
    if ui.button("Bake again (B)").clicked() {
        settings.rebake += 1;
    }
    ui.label(
        RichText::new(crate::hud::impostor_line(viewer))
            .small()
            .color(theme::MUTED),
    );
}

/// The sun, and how finely plants are drawn.
fn display(ui: &mut Ui, viewer: &mut Viewer) {
    let lighting = &mut viewer.lighting;
    widgets::section(ui, "Sun");
    widgets::setting(
        ui,
        "Direction",
        format!("{:.0}°", lighting.sun_azimuth.to_degrees()),
        Slider::new(&mut lighting.sun_azimuth, 0.0..=TAU),
    );
    widgets::setting(
        ui,
        "Height",
        format!("{:.0}°", lighting.sun_elevation.to_degrees()),
        Slider::new(&mut lighting.sun_elevation, 0.08..=1.5),
    );
    widgets::switch(ui, &mut lighting.shadows, "Shadows");
    widgets::section(ui, "Plants");
    widgets::switch(ui, &mut viewer.leaves, "Foliage");
    widgets::switch(ui, &mut viewer.leaf_coverage, "Keep distant leaves (C)");
    impostors(ui, viewer);
    if profile_controls::detail(ui, &mut viewer.profile) {
        let profile = profile_controls::drawable(viewer.profile);
        viewer
            .requests
            .push(Request::Send(Command::Profile(profile)));
    }
}
