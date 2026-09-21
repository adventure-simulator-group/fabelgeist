use adventuresim_armor_model::{
    BracerDesign, DesignError, Millimeters, Permille, PlateFluting, decode, design_hash, encode,
    validate,
};

#[test]
fn presets_are_valid_distinct_recipes_that_round_trip() {
    let presets = [
        BracerDesign::bracelet(),
        BracerDesign::default(),
        BracerDesign::full_forearm(),
    ];
    let mut hashes = Vec::new();
    for preset in &presets {
        validate(preset).unwrap();
        let bytes = encode(preset).unwrap();
        assert_eq!(&decode(&bytes).unwrap(), preset);
        hashes.push(design_hash(preset).unwrap());
    }
    hashes.sort();
    hashes.dedup();
    assert_eq!(
        hashes.len(),
        presets.len(),
        "presets share a recipe identity"
    );
}

#[test]
fn thickness_changes_recipe_identity() {
    let thin = BracerDesign::default();
    let mut thick = thin.clone();
    thick.wall_thickness.0 += 7;
    assert_ne!(design_hash(&thin).unwrap(), design_hash(&thick).unwrap());
    assert_ne!(encode(&thin).unwrap(), encode(&thick).unwrap());
    assert_eq!(
        design_hash(&thin).unwrap(),
        design_hash(&thin.clone()).unwrap()
    );
}

#[test]
fn out_of_range_controls_report_their_constraint() {
    let with = |edit: fn(&mut BracerDesign)| {
        let mut design = BracerDesign::default();
        edit(&mut design);
        design
    };
    for (design, expected) in [
        (
            with(|d| d.catalog_id = " ".into()),
            DesignError::EmptyCatalogId,
        ),
        (with(|d| d.coverage = Permille(49)), DesignError::Coverage),
        (
            with(|d| d.coverage = Permille(1_001)),
            DesignError::Coverage,
        ),
        (
            with(|d| {
                d.coverage = Permille(900);
                d.wrist_offset = Permille(200);
            }),
            DesignError::Placement,
        ),
        (
            with(|d| d.wall_thickness = Millimeters(0)),
            DesignError::WallThickness,
        ),
        (
            with(|d| d.wall_thickness = Millimeters(21)),
            DesignError::WallThickness,
        ),
        (
            with(|d| d.clearance = Millimeters(31)),
            DesignError::Clearance,
        ),
        (
            with(|d| d.elbow_flare = Millimeters(16)),
            DesignError::Clearance,
        ),
        (
            with(|d| {
                d.fluting = Some(PlateFluting {
                    depth: Millimeters(9),
                    ..Default::default()
                })
            }),
            DesignError::PlateFluting,
        ),
    ] {
        assert_eq!(validate(&design), Err(expected.clone()), "{design:?}");
        assert_eq!(encode(&design), Err(expected.clone()));
        assert_eq!(design_hash(&design), Err(expected));
    }
}

#[test]
fn decoding_rejects_invalid_and_malformed_recipes() {
    let invalid = BracerDesign {
        coverage: Permille(10),
        ..Default::default()
    };
    let bytes = postcard::to_allocvec(&invalid).unwrap();
    assert_eq!(decode(&bytes), Err(DesignError::Coverage));
    let valid = encode(&BracerDesign::default()).unwrap();
    assert_eq!(
        decode(&valid[..valid.len() / 2]),
        Err(DesignError::Encoding)
    );
}
