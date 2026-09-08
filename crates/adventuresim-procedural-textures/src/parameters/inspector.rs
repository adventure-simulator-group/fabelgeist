//! Catalogue-specific reachable control roots; nested values retain their typed document shape.
use crate::TextureRecipeId;
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Clone, serde::Deserialize)]
#[serde(transparent)]
pub struct ControlPath(String);
impl ControlPath {
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn group(&self) -> &str {
        self.0.split('/').nth(1).expect("validated control path")
    }
}
impl TextureRecipeId {
    pub fn control_paths(self) -> &'static [ControlPath] {
        static CATALOGUE: OnceLock<BTreeMap<String, Vec<ControlPath>>> = OnceLock::new();
        CATALOGUE
            .get_or_init(|| {
                serde_json::from_str(include_str!("active.json"))
                    .expect("validated control catalogue")
            })
            .get(self.slug())
            .expect("every recipe has control metadata")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_catalogue_recipe_exposes_existing_controls() {
        let defaults = serde_json::to_value(crate::TextureParameters::default()).unwrap();
        for descriptor in crate::PROCEDURAL_TEXTURE_CATALOGUE {
            assert!(!descriptor.id.control_paths().is_empty());
            for path in descriptor.id.control_paths() {
                assert!(
                    defaults.pointer(path.as_str()).is_some(),
                    "{}: {}",
                    descriptor.id.slug(),
                    path.as_str()
                );
            }
        }
    }
}
