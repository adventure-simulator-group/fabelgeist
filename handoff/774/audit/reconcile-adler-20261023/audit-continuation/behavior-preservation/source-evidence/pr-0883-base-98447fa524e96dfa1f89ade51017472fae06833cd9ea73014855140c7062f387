fn main() {
    let target = std::env::args().nth(1).unwrap_or_else(|| "all".to_owned());
    if !matches!(target.as_str(), "all" | "items" | "encounters") {
        eprintln!("unsupported content target {target:?}; expected all, items, or encounters");
        std::process::exit(2);
    }

    if matches!(target.as_str(), "all" | "encounters") {
        let definitions = adventuresim_core::road_encounter_catalog::definitions();
        adventuresim_core::road_encounter_catalog::validate_definitions(definitions)
            .expect("invalid embedded encounter catalog");
        println!(
            "encounters: {} definitions, revision {}, digest {}",
            definitions.len(),
            adventuresim_core::road_encounter_catalog::CATALOG_REVISION,
            adventuresim_core::road_encounter_catalog::digest()
        );
    }
    if target == "encounters" {
        return;
    }
    let catalog = adventuresim_core::item_catalog::catalog();
    if let Err(missing) = adventuresim_core::item_references::validate_gameplay_references(
        adventuresim_core::quest_catalog::catalog(),
    ) {
        panic!(
            "missing required gameplay item references: {:?}",
            missing.ids
        );
    }
    println!(
        "items: {} definitions, revision {}",
        catalog.len(),
        adventuresim_core::item_catalog::revision()
    );
}
