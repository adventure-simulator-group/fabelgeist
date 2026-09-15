use adventuresim_heraldry::{
    bake::{Baked, Resolution},
    document::Document,
};
use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass, egui};
use std::sync::Arc;
const HISTORY_LIMIT: usize = 64;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Channel {
    Physical,
    Flat,
    BaseColor,
    Normal,
    Roughness,
    Metallic,
    Coating,
    Height,
}
#[derive(Resource)]
pub(crate) struct Studio {
    pub document: Document,
    pub mixer: crate::ui::mixer::Mixer,
    pub current: Option<Arc<Baked>>,
    pub current_document: Option<Document>,
    pub pinned: Option<(Document, Arc<Baked>)>,
    pub resolution: Resolution,
    pub channel: Channel,
    pub viewport: egui::Rect,
    pub scene_dirty: bool,
    pub status: String,
    pub last_edit: f64,
    pub export_requested: bool,
    pub undo: Vec<Document>,
    pub redo: Vec<Document>,
    pub transaction: Option<Document>,
    pub json: String,
    pub show_json: bool,
    #[cfg(not(target_family = "wasm"))]
    pub exchange_path: String,
    pub screenshot: bool,
    pub save_due: bool,
}
impl Studio {
    pub fn new(document: Document) -> Self {
        Self {
            json: document.to_json().unwrap(),
            document,
            mixer: crate::ui::mixer::Mixer::default(),
            current: None,
            current_document: None,
            pinned: None,
            resolution: Resolution::Preview,
            channel: Channel::Physical,
            viewport: egui::Rect::NOTHING,
            scene_dirty: true,
            status: "Preparing heraldry…".into(),
            last_edit: -1.0,
            export_requested: false,
            undo: vec![],
            redo: vec![],
            transaction: None,
            show_json: false,
            #[cfg(not(target_family = "wasm"))]
            exchange_path: "target/heraldry/arms.json".into(),
            screenshot: false,
            save_due: false,
        }
    }
    pub fn ready(&self) -> bool {
        self.current
            .as_ref()
            .is_some_and(|b| b.matches(&self.document, self.resolution))
    }
    pub fn remember(&mut self, before: Document) {
        if self.undo.last() != Some(&before) {
            self.undo.push(before);
            if self.undo.len() > HISTORY_LIMIT {
                self.undo.remove(0);
            }
        }
        self.redo.clear();
    }
    pub fn replace(&mut self, document: Document, now: f64) {
        if document == self.document {
            return;
        }
        self.document = document;
        self.last_edit = now;
        self.json = self.document.to_json().unwrap();
        self.save_due = true;
    }
}
pub fn run() {
    let d = crate::exchange::load_saved()
        .and_then(|s| Document::from_json(&s).ok())
        .unwrap_or_default();
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Fabelgeist · Heraldry Studio".into(),
                canvas: Some("#studio".into()),
                fit_canvas_to_parent: true,
                resolution: (1600, 1000).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .insert_resource(Studio::new(d))
        .insert_non_send(crate::baking::Baker::new())
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
        )
        .run();
}
