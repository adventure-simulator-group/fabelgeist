use adventuresim_weapon_model::*;
use sha2::{Digest, Sha256};

fn pointed_design() -> WeaponDesign {
    let mut design = default_design("rondel_dagger").unwrap();
    let study: serde_json::Value =
        serde_json::from_str(include_str!("../review/museum/london-80.157.json")).unwrap();
    design.recipe = serde_json::from_value(study["definition"].clone()).unwrap();
    design
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
    assert_ne!(current, versioned_hash(domain, 12, 15, &design));

    let mut previous = envelope.clone();
    previous["schema_version"] = 12.into();
    assert!(matches!(
        decode(&serde_json::to_vec(&previous).unwrap()),
        Err(CodecError::SchemaVersion {
            found: 12,
            expected: SCHEMA_VERSION
        })
    ));
    previous = envelope;
    previous["generator_version"] = 15.into();
    assert!(matches!(
        decode(&serde_json::to_vec(&previous).unwrap()),
        Err(CodecError::GeneratorVersion {
            found: 15,
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
    assert_ne!(current, versioned_hash(domain, 6, 6, &design));

    let mut previous = envelope.clone();
    previous["schema_version"] = 6.into();
    assert!(matches!(
        decode_holder(&serde_json::to_vec(&previous).unwrap()),
        Err(CodecError::SchemaVersion {
            found: 6,
            expected: HOLDER_SCHEMA_VERSION
        })
    ));
    previous = envelope;
    previous["generator_version"] = 6.into();
    assert!(matches!(
        decode_holder(&serde_json::to_vec(&previous).unwrap()),
        Err(CodecError::GeneratorVersion {
            found: 6,
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
fn diamond_sections_do_not_enable_generic_blade_scabbards() {
    let mut weapon = pointed_design();
    for component in &mut weapon.recipe.components {
        if let recipe::Shape::Blade(blade) = &mut component.shape {
            blade.section = Some(recipe::ForgedBladeSection::Diamond);
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
