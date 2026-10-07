//! Botany's named scenes, found by the name a person types.

use fabelgeist_botany::scenes::Scene;

/// No scene's name starts with what was asked for.
#[derive(Debug, thiserror::Error)]
#[error("unknown scene `{0}`")]
pub struct SceneLookupError(String);

/// The scene whose name, lowercased with every run of other characters as
/// one hyphen, starts with `name`: `boreal` finds the boreal forest and
/// `castle-unplanted` the unplanted castle.
pub fn scene_named(name: &str) -> Result<Scene, SceneLookupError> {
    let wanted = name.to_lowercase();
    Scene::ALL
        .into_iter()
        .find(|scene| slug(scene.name()).starts_with(&wanted))
        .ok_or_else(|| SceneLookupError(name.to_owned()))
}

fn slug(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    for letter in name.chars() {
        if letter.is_alphanumeric() {
            slug.extend(letter.to_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug
}
