use crate::{baking::Baker, document::Document};
use adventuresim_procedural_textures::{BakeResolution, BakedRecipe, MapChannel};
use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

#[derive(Resource)]
pub(crate) struct Studio {
    pub document: Document,
    pub controls: serde_json::Value,
    pub defaults: serde_json::Value,
    pub current: Option<BakedRecipe>,
    pub pinned: Option<(BakedRecipe, Document)>,
    pub revision: u64,
    pub last_edit: f64,
    pub applied: Option<(u64, BakeResolution)>,
    pub failed_revision: Option<u64>,
    pub scene_dirty: bool,
    pub channel: Option<MapChannel>,
    pub status: String,
    pub search: String,
    pub undo: Vec<String>,
    pub redo: Vec<String>,
    pub transaction: Option<String>,
    pub viewport: bevy_egui::egui::Rect,
    pub screenshot: bool,
    pub save_due: bool,
    pub library: std::collections::BTreeMap<String, String>,
    pub parameter_error: Option<String>,
}

impl Studio {
    pub fn new(document: Document) -> Self {
        Self {
            controls: serde_json::to_value(&document.texture).unwrap(),
            defaults: serde_json::to_value(
                adventuresim_procedural_textures::TextureParameters::default(),
            )
            .unwrap(),
            document,
            current: None,
            pinned: None,
            revision: 1,
            last_edit: -1.0,
            applied: None,
            failed_revision: None,
            scene_dirty: true,
            channel: None,
            status: "Preparing texture…".into(),
            search: String::new(),
            undo: vec![],
            redo: vec![],
            transaction: None,
            viewport: bevy_egui::egui::Rect::NOTHING,
            screenshot: false,
            save_due: false,
            library: crate::exchange::load_library(),
            parameter_error: None,
        }
    }
    pub fn replace(&mut self, document: Document, now: f64) {
        let texture_changed = self.document.recipe != document.recipe
            || serde_json::to_value(&self.document.texture).unwrap()
                != serde_json::to_value(&document.texture).unwrap();
        self.controls = serde_json::to_value(&document.texture).unwrap();
        self.document = document;
        self.parameter_error = None;
        if texture_changed {
            self.edited(now);
        }
        self.scene_dirty = true;
        self.save_due = true;
    }
    pub fn edited(&mut self, now: f64) {
        self.revision += 1;
        self.failed_revision = None;
        self.last_edit = now;
        self.scene_dirty = true;
        self.save_due = true;
    }
    pub fn remember(&mut self, previous: String) {
        if self.undo.last() != Some(&previous) {
            self.undo.push(previous);
            if self.undo.len() > 64 {
                self.undo.remove(0);
            }
        }
        self.redo.clear();
    }
}

pub fn run() {
    let document = crate::exchange::load_saved()
        .and_then(|s| Document::from_json(&s).ok())
        .unwrap_or_default();
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Fabelgeist · Texture Studio".into(),
            canvas: Some("#studio".into()),
            fit_canvas_to_parent: true,
            resolution: (1600, 1000).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(EguiPlugin::default())
    .add_plugins(adventuresim_procedural_materials::ProceduralMaterialsPlugin)
    .insert_resource(Studio::new(document))
    .insert_non_send(Baker::new())
    .add_systems(Startup, crate::scene::setup)
    .add_systems(EguiPrimaryContextPass, crate::ui::draw)
    .add_systems(
        Update,
        (
            crate::baking::update,
            crate::scene::update,
            crate::exchange::update,
        )
            .chain(),
    );
    app.run();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restoring_viewing_conditions_preserves_the_completed_bake_revision() {
        let mut studio = Studio::new(Document::default());
        let revision = studio.revision;
        studio.applied = Some((revision, BakeResolution::Full));
        let mut document = studio.document.clone();
        document.environment.exposure_ev = 1.0;
        studio.replace(document, 1.0);
        assert_eq!(studio.revision, revision);
        assert_eq!(studio.applied, Some((revision, BakeResolution::Full)));
        let mut document = studio.document.clone();
        document.texture.seed += 1;
        studio.replace(document, 2.0);
        assert!(studio.revision > revision);
    }
}
