//! Decoration controls shared by the armory and the inventory: editing an
//! engraving and trim, and saving decorations to and choosing them from the
//! library.
use super::*;
use adventuresim_character_creator::decoration::{Decoration, DecorationLibrary, DecorationName};

/// The decoration library and where it is saved.
pub(super) struct Decorations {
    pub library: DecorationLibrary,
    pub path: String,
}

impl Decorations {
    /// Save `decoration` under `name` and write the library.
    pub(super) fn save(&mut self, name: &str, decoration: &Decoration) -> Result<DecorationName> {
        let name = DecorationName::try_from(name.to_owned()).map_err(anyhow::Error::msg)?;
        let mut library = self.library.clone();
        library.insert(name.clone(), decoration.clone())?;
        library.save(std::path::Path::new(&self.path))?;
        self.library = library;
        Ok(name)
    }

    /// Delete the decoration called `name` and write the library.
    pub(super) fn delete(&mut self, name: &DecorationName) -> Result<()> {
        let mut library = self.library.clone();
        library.remove(name);
        library.save(std::path::Path::new(&self.path))?;
        self.library = library;
        Ok(())
    }
}

/// Engraving and trim editors. Returns whether the decoration changed.
pub(super) fn edit(ui: &mut egui::Ui, decoration: &mut Decoration) -> bool {
    let engraving = ui
        .collapsing("Engraving", |ui| {
            metal_controls::engraving(ui, &mut decoration.engraving)
        })
        .body_returned
        .unwrap_or(false);
    let trim = ui
        .collapsing("Trim", |ui| metal_controls::trim(ui, &mut decoration.trim))
        .body_returned
        .unwrap_or(false);
    engraving || trim
}

/// The library entry `decoration` matches, for display.
pub(super) fn label(library: &DecorationLibrary, decoration: &Decoration) -> String {
    if decoration.is_plain() {
        return "Plain".into();
    }
    library
        .name_of(decoration)
        .map_or_else(|| "Custom".into(), ToString::to_string)
}

/// Pick a saved decoration, or none, for `decoration`. Returns the name of
/// the saved decoration chosen, or `Some(None)` for plain, when the choice changed.
pub(super) fn choose(
    ui: &mut egui::Ui,
    id_salt: impl std::hash::Hash,
    library: &DecorationLibrary,
    decoration: &mut Decoration,
) -> Option<Option<DecorationName>> {
    let mut chosen = None;
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(label(library, decoration))
        .width(ui.available_width() * 0.6)
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(decoration.is_plain(), "Plain")
                .clicked()
            {
                chosen = Some(None);
            }
            for (name, saved) in library.iter() {
                if ui
                    .selectable_label(saved == decoration, name.to_string())
                    .clicked()
                {
                    chosen = Some(Some(name.clone()));
                }
            }
        });
    let choice = chosen?;
    let replacement = choice
        .as_ref()
        .and_then(|name| library.get(name))
        .cloned()
        .unwrap_or_default();
    (replacement != *decoration).then(|| {
        *decoration = replacement;
        choice
    })
}
