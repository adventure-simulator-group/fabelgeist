//! Shared leaf morphology adapted from adventure-simulator-group/leaves.
use super::LeafShape;
use std::{collections::BTreeMap, sync::OnceLock};
fn catalogue() -> &'static BTreeMap<String, LeafShape> {
    static PRESETS: OnceLock<BTreeMap<String, LeafShape>> = OnceLock::new();
    PRESETS.get_or_init(|| {
        serde_json::from_str(include_str!("presets.json")).expect("valid leaf presets")
    })
}
impl LeafShape {
    pub fn preset(name: &str) -> Option<Self> {
        catalogue().get(name).cloned()
    }
    pub fn preset_names() -> impl Iterator<Item = &'static str> {
        catalogue().keys().map(String::as_str)
    }
}
