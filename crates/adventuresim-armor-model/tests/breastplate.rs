use adventuresim_armor_model::{
    BreastplateDesign, DesignError, FluteCount, Millimeters, Permille, breastplate_design_hash,
    validate_breastplate,
};

fn presets() -> [BreastplateDesign; 5] {
    [
        BreastplateDesign::default(),
        BreastplateDesign::globose(),
        BreastplateDesign::tapul(),
        BreastplateDesign::peascod(),
        BreastplateDesign::fluted(),
    ]
}

#[test]
fn presets_are_valid_and_have_distinct_recipe_identities() {
    let mut hashes = presets()
        .iter()
        .map(|design| breastplate_design_hash(design).unwrap())
        .collect::<Vec<_>>();
    hashes.sort();
    hashes.dedup();
    assert_eq!(hashes.len(), presets().len());
}

#[test]
fn new_controls_reject_invalid_spacing_and_round_trip() {
    let mut design = BreastplateDesign::fluted();
    let hash = breastplate_design_hash(&design).unwrap();
    let json = serde_json::to_vec(&design).unwrap();
    assert_eq!(
        serde_json::from_slice::<BreastplateDesign>(&json).unwrap(),
        design
    );
    design.fluting.as_mut().unwrap().width = Permille(800);
    assert_ne!(breastplate_design_hash(&design).unwrap(), hash);
    design.fluting.as_mut().unwrap().end = Permille(200);
    assert_eq!(
        validate_breastplate(&design),
        Err(DesignError::PlateFluting)
    );
    assert!(breastplate_design_hash(&design).is_err());
    let mut unknown = serde_json::to_value(BreastplateDesign::fluted()).unwrap();
    unknown["fluting"]["widht"] = serde_json::json!(700);
    assert!(serde_json::from_value::<BreastplateDesign>(unknown).is_err());
}

#[test]
fn flute_extremes_within_their_ranges_are_valid() {
    for (count, width, depth) in [(2, 850, 4), (24, 350, 1), (24, 850, 4)] {
        let mut design = BreastplateDesign::fluted();
        let flutes = design.fluting.as_mut().unwrap();
        flutes.count = FluteCount(count);
        flutes.width = Permille(width);
        flutes.depth = Millimeters(depth);
        validate_breastplate(&design).unwrap();
    }
    let mut too_many = BreastplateDesign::fluted();
    too_many.fluting.as_mut().unwrap().count = FluteCount(25);
    assert_eq!(
        validate_breastplate(&too_many),
        Err(DesignError::PlateFluting)
    );
}

#[test]
fn out_of_range_shape_and_edges_report_their_constraint() {
    let with = |edit: fn(&mut BreastplateDesign)| {
        let mut design = BreastplateDesign::default();
        edit(&mut design);
        design
    };
    for (design, expected) in [
        (
            with(|d| d.profile.projection = Millimeters(81)),
            DesignError::BreastplateShape,
        ),
        (
            with(|d| d.profile.fullness = Permille(300)),
            DesignError::BreastplateShape,
        ),
        (
            with(|d| d.catalog_id = String::new()),
            DesignError::EmptyCatalogId,
        ),
        (
            with(|d| d.neck_width = Permille(1_301)),
            DesignError::BreastplateEdges,
        ),
        (
            with(|d| d.wall_thickness = Millimeters(0)),
            DesignError::BreastplateEdges,
        ),
        (
            with(|d| d.back_clearance = Millimeters(5)),
            DesignError::BreastplateEdges,
        ),
        (
            with(|d| d.skirt_flare = Millimeters(71)),
            DesignError::BreastplateEdges,
        ),
    ] {
        assert_eq!(validate_breastplate(&design), Err(expected), "{design:?}");
        assert!(breastplate_design_hash(&design).is_err());
    }
}

#[test]
fn example_recipes_match_editor_presets() {
    for (json, expected) in [
        (
            include_str!("../review/breastplate/designs/rounded.json"),
            BreastplateDesign::globose(),
        ),
        (
            include_str!("../review/breastplate/designs/tapul.json"),
            BreastplateDesign::tapul(),
        ),
        (
            include_str!("../review/breastplate/designs/peascod.json"),
            BreastplateDesign::peascod(),
        ),
        (
            include_str!("../review/breastplate/designs/fluted.json"),
            BreastplateDesign::fluted(),
        ),
    ] {
        assert_eq!(
            serde_json::from_str::<BreastplateDesign>(json).unwrap(),
            expected
        );
    }
}
