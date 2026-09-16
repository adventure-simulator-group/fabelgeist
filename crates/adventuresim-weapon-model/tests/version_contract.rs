use adventuresim_weapon_model::*;
use sha2::{Digest, Sha256};

fn pointed_design() -> WeaponDesign {
    let mut design = default_design("rondel_dagger").unwrap();
    let study: serde_json::Value =
        serde_json::from_str(include_str!("../review/museum/london-80.157.json")).unwrap();
    design.recipe = serde_json::from_value(study["definition"].clone()).unwrap();
    design
}

#[test]
fn arming_sword_round_trip_preserves_mortise_wheel_and_ribbed_cover() {
    let mut design = default_design("arming_sword").unwrap();
    let study: serde_json::Value =
        serde_json::from_str(include_str!("../review/museum/met-14.25.1096.json")).unwrap();
    design.recipe = serde_json::from_value(study["definition"].clone()).unwrap();
    let restored = decode(&encode(&design).unwrap()).unwrap();
    assert_eq!(restored, design);
    assert_eq!(generate(&restored).unwrap(), generate(&design).unwrap());
}

#[test]
fn triangular_museum_recipe_round_trip_preserves_the_complete_assembly() {
    let mut design = pointed_design();
    let study: serde_json::Value =
        serde_json::from_str(include_str!("../review/museum/cma-1916.686.json")).unwrap();
    design.recipe = serde_json::from_value(study["definition"].clone()).unwrap();
    let restored = decode(&encode(&design).unwrap()).unwrap();
    assert_eq!(restored, design);
    assert_eq!(generate(&restored).unwrap(), generate(&design).unwrap());
}

fn versioned_hash(
    domain: &[u8],
    schema: u16,
    generator: u16,
    design: &impl serde::Serialize,
) -> DesignHash {
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update(schema.to_le_bytes());
    hash.update(generator.to_le_bytes());
    hash.update(serde_json::to_vec(design).unwrap());
    DesignHash(hash.finalize().into())
}

#[test]
fn weapon_transport_rejects_previous_versions_and_emits_current_identity() {
    let design = pointed_design();
    let bytes = encode(&design).unwrap();
    let envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(envelope["schema_version"], SCHEMA_VERSION);
    assert_eq!(envelope["generator_version"], GENERATOR_VERSION);
    assert_eq!(decode(&bytes).unwrap(), design);
    let domain = b"fabelgeist.weapon-design\0";
    let current = versioned_hash(domain, SCHEMA_VERSION, GENERATOR_VERSION, &design);
    assert_eq!(design_hash(&design), current);
    assert_eq!(generate(&design).unwrap().design_hash, current);
    assert_ne!(current, versioned_hash(domain, 17, 20, &design));

    let mut previous = envelope.clone();
    previous["schema_version"] = 17.into();
    assert!(matches!(
        decode(&serde_json::to_vec(&previous).unwrap()),
        Err(CodecError::SchemaVersion {
            found: 17,
            expected: SCHEMA_VERSION
        })
    ));
    previous = envelope;
    previous["generator_version"] = 20.into();
    assert!(matches!(
        decode(&serde_json::to_vec(&previous).unwrap()),
        Err(CodecError::GeneratorVersion {
            found: 20,
            expected: GENERATOR_VERSION
        })
    ));
}

#[test]
fn museum_mace_round_trip_preserves_receiving_faces_and_material_properties() {
    let mut design = default_design("flanged_mace").unwrap();
    let study: serde_json::Value =
        serde_json::from_str(include_str!("../review/museum/cma-1916.1589.json")).unwrap();
    design.recipe = serde_json::from_value(study["definition"].clone()).unwrap();
    let restored = decode(&encode(&design).unwrap()).unwrap();
    assert_eq!(restored, design);
    assert!(generate(&restored).unwrap() == generate(&design).unwrap());
}

#[test]
fn gothic_mace_round_trip_preserves_profiles_crown_and_supported_cord() {
    let mut design = default_design("flanged_mace").unwrap();
    let study: serde_json::Value =
        serde_json::from_str(include_str!("../review/museum/khm-a297.json")).unwrap();
    design.recipe = serde_json::from_value(study["definition"].clone()).unwrap();
    let restored = decode(&encode(&design).unwrap()).unwrap();
    assert_eq!(restored, design);
    assert!(generate(&restored).unwrap() == generate(&design).unwrap());
}

#[test]
fn war_hammer_round_trip_preserves_square_beak_and_silver_parts() {
    let mut design = default_design("war_hammer").unwrap();
    let study: serde_json::Value =
        serde_json::from_str(include_str!("../review/museum/met-29.158.674.json")).unwrap();
    design.recipe = serde_json::from_value(study["definition"].clone()).unwrap();
    let restored = decode(&encode(&design).unwrap()).unwrap();
    assert_eq!(restored, design);
    assert!(generate(&restored).unwrap() == generate(&design).unwrap());
}

#[test]
fn holder_transport_versions_its_embedded_design_and_generated_identity() {
    let weapon = default_design("rondel_dagger").unwrap();
    let design = default_holder_design(&weapon).unwrap();
    let bytes = encode_holder(&design).unwrap();
    let envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(envelope["schema_version"], HOLDER_SCHEMA_VERSION);
    assert_eq!(envelope["generator_version"], HOLDER_GENERATOR_VERSION);
    assert_eq!(decode_holder(&bytes).unwrap(), design);
    let domain = b"fabelgeist.weapon-holder-design\0";
    let current = versioned_hash(
        domain,
        HOLDER_SCHEMA_VERSION,
        HOLDER_GENERATOR_VERSION,
        &design,
    );
    assert_eq!(holder_design_hash(&design), current);
    assert_eq!(generate_holder(&design).unwrap().design_hash, current);
    assert_ne!(current, versioned_hash(domain, 11, 11, &design));

    let mut previous = envelope.clone();
    previous["schema_version"] = 11.into();
    assert!(matches!(
        decode_holder(&serde_json::to_vec(&previous).unwrap()),
        Err(CodecError::SchemaVersion {
            found: 11,
            expected: HOLDER_SCHEMA_VERSION
        })
    ));
    previous = envelope;
    previous["generator_version"] = 11.into();
    assert!(matches!(
        decode_holder(&serde_json::to_vec(&previous).unwrap()),
        Err(CodecError::GeneratorVersion {
            found: 11,
            expected: HOLDER_GENERATOR_VERSION
        })
    ));
}

#[test]
fn silver_holder_fittings_round_trip_through_an_existing_holder() {
    let mut weapon = default_design("rondel_dagger").unwrap();
    weapon.recipe.components[0].material = Some(Material::Silver);
    let mut design = default_holder_design(&weapon).unwrap();
    design.fitting_material = Material::Silver;
    let restored = decode_holder(&encode_holder(&design).unwrap()).unwrap();
    assert_eq!(restored, design);
    let holder = generate_holder(&restored).unwrap();
    assert!(holder.parts.iter().any(|p| p.material == Material::Silver));
    assert_eq!(holder, generate_holder(&design).unwrap());
}

#[test]
fn thrusting_sections_do_not_enable_generic_blade_scabbards() {
    for section in [
        recipe::ForgedBladeSection::Diamond,
        recipe::ForgedBladeSection::Triangular,
    ] {
        let mut weapon = pointed_design();
        for component in &mut weapon.recipe.components {
            if let recipe::Shape::Blade(blade) = &mut component.shape {
                blade.section = Some(section);
                blade.single_edge = Some(recipe::Ratio::new(0.0).unwrap());
            }
        }
        validate(&weapon).unwrap();
        let holder = default_holder_design(&weapon).unwrap();
        assert!(matches!(
            generate_holder(&holder),
            Err(GenerateError::Invalid(errors))
                if errors == vec![ValidationError::Holder("source geometry")]
        ));
    }
}

#[test]
fn katzbalger_round_trip_preserves_grooves_partitioned_cover_and_shared_seats() {
    let mut design = default_design("katzbalger").unwrap();
    let study: serde_json::Value =
        serde_json::from_str(include_str!("../review/museum/khm-a287.json")).unwrap();
    design.recipe = serde_json::from_value(study["definition"].clone()).unwrap();
    let restored = decode(&encode(&design).unwrap()).unwrap();
    assert_eq!(restored, design);
    assert!(generate(&restored).unwrap() == generate(&design).unwrap());
}

#[test]
fn halberd_round_trip_preserves_curved_boundary_ridges_and_finite_seats() {
    let mut design = default_design("halberd").unwrap();
    let study: serde_json::Value =
        serde_json::from_str(include_str!("../review/museum/met-96.5.23.json")).unwrap();
    design.recipe = serde_json::from_value(study["definition"].clone()).unwrap();
    let restored = decode(&encode(&design).unwrap()).unwrap();
    assert_eq!(restored, design);
    assert_eq!(generate(&restored).unwrap(), generate(&design).unwrap());
}

#[test]
fn partisan_round_trip_preserves_physical_facets_and_relieved_sections() {
    let mut design = default_design("halberd").unwrap();
    let study: serde_json::Value =
        serde_json::from_str(include_str!("../review/museum/met-08.261.2.json")).unwrap();
    design.recipe = serde_json::from_value(study["definition"].clone()).unwrap();
    let restored = decode(&encode(&design).unwrap()).unwrap();
    assert_eq!(restored, design);
    assert_eq!(generate(&restored).unwrap(), generate(&design).unwrap());
}

#[test]
fn fork_round_trip_preserves_multiple_ridge_tracks_and_isolated_points() {
    let mut design = default_design("halberd").unwrap();
    let study: serde_json::Value =
        serde_json::from_str(include_str!("../review/museum/met-14.25.116.json")).unwrap();
    design.recipe = serde_json::from_value(study["definition"].clone()).unwrap();
    let restored = decode(&encode(&design).unwrap()).unwrap();
    assert_eq!(restored, design);
    assert_eq!(generate(&restored).unwrap(), generate(&design).unwrap());
}

#[test]
fn holder_transport_retains_an_embedded_contoured_plate() {
    let mut weapon = default_design("war_hammer").unwrap();
    weapon.recipe = serde_json::from_value(serde_json::json!({"components":[
        {"id":"grip","kind":"box","role":"Grip","material":"wood","size":[0.024,0.3,0.022],"attach":{"to":"weapon.root","at":"base"}},
        {"id":"plate","kind":"contouredPlate","width":0.09,"length":0.07,"start":[-0.5,0],
        "boundary":[{"kind":"line","to":[0.5,0]},{"kind":"line","to":[0.5,1]},{"kind":"line","to":[-0.5,1]},{"kind":"line","to":[-0.5,0]}],
        "surface":{"kind":"ridge","stations":[{"at":0,"edge":0.002,"ridge":0.022,"ridgeHalfWidth":0.035,"flatHalfWidth":0.012,"hollowDepth":0.003},
        {"at":1,"edge":0.002,"ridge":0.022,"ridgeHalfWidth":0.035,"flatHalfWidth":0.012,"hollowDepth":0.003}]},"attach":{"to":"grip.top","at":"base"}}
    ]})).unwrap();
    let holder = default_holder_design(&weapon).unwrap();
    let restored = decode_holder(&encode_holder(&holder).unwrap()).unwrap();
    assert_eq!(restored, holder);
    assert_eq!(
        generate_holder(&restored).unwrap(),
        generate_holder(&holder).unwrap()
    );
    assert!(default_holder_design(&default_design("halberd").unwrap()).is_none());
    let recipe::Shape::ContouredPlate(plate) = &mut weapon.recipe.components[1].shape else {
        panic!("expected an embedded plate");
    };
    plate.surface = serde_json::from_value(serde_json::json!({"kind":"profile","stations":[
        {"at":0,"profile":[{"across":-1,"thickness":0.010},{"across":1,"thickness":0.018}]},
        {"at":1,"profile":[{"across":-1,"thickness":0.014},{"across":1,"thickness":0.012}]}
    ]}))
    .unwrap();
    let holder = default_holder_design(&weapon).unwrap();
    let restored = decode_holder(&encode_holder(&holder).unwrap()).unwrap();
    assert_eq!(restored, holder);
    assert_eq!(
        generate_holder(&restored).unwrap(),
        generate_holder(&holder).unwrap()
    );
}
