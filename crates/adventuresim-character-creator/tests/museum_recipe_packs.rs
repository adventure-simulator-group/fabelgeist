//! Museum presets must remain readable by the same parsers as the exporter.
use std::path::Path;

use adventuresim_character_creator::{
    CharacterRecipe, armor_design_input, design_input, fasteners,
};

#[test]
fn museum_recipe_packs_use_the_current_export_schema() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../adventuresim-armor-model/review/museum");
    for museum in ["henry", "nuremberg"] {
        let directory = root.join(museum);
        let body_path = directory.join("body.json");
        let body: CharacterRecipe = serde_json::from_slice(
            &std::fs::read(&body_path).expect("checked-in museum body recipe"),
        )
        .unwrap_or_else(|error| panic!("{}: {error}", body_path.display()));
        body.validate()
            .unwrap_or_else(|error| panic!("{}: {error}", body_path.display()));

        armor_design_input::load(Some(&directory.join("armor.json")))
            .unwrap_or_else(|error| panic!("{museum} armor: {error:#}"));
        design_input::load_breastplate_design(Some(&directory.join("breastplate.json")))
            .unwrap_or_else(|error| panic!("{museum} breastplate: {error:#}"));
        design_input::load_bracer_design(Some(&directory.join("vambrace.json")))
            .unwrap_or_else(|error| panic!("{museum} vambrace: {error:#}"));
        fasteners::catalog::load(Some(&directory.join("fasteners.json")))
            .unwrap_or_else(|error| panic!("{museum} fasteners: {error:#}"));
    }
}
