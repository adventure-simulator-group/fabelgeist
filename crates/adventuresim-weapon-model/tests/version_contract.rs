use adventuresim_weapon_model::*;
use sha2::{Digest, Sha256};

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
    let design = default_design("arming_sword").unwrap();
    let bytes = encode(&design).unwrap();
    let envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(envelope["schema_version"], SCHEMA_VERSION);
    assert_eq!(envelope["generator_version"], GENERATOR_VERSION);
    assert_eq!(decode(&bytes).unwrap(), design);
    let domain = b"fabelgeist.weapon-design\0";
    let current = versioned_hash(domain, SCHEMA_VERSION, GENERATOR_VERSION, &design);
    assert_eq!(design_hash(&design), current);
    assert_eq!(generate(&design).unwrap().design_hash, current);
    assert_ne!(current, versioned_hash(domain, 6, 9, &design));

    let mut previous = envelope.clone();
    previous["schema_version"] = 6.into();
    assert!(matches!(
        decode(&serde_json::to_vec(&previous).unwrap()),
        Err(CodecError::SchemaVersion {
            found: 6,
            expected: SCHEMA_VERSION
        })
    ));
    previous = envelope;
    previous["generator_version"] = 9.into();
    assert!(matches!(
        decode(&serde_json::to_vec(&previous).unwrap()),
        Err(CodecError::GeneratorVersion {
            found: 9,
            expected: GENERATOR_VERSION
        })
    ));
}

#[test]
fn holder_transport_versions_its_embedded_design_and_generated_identity() {
    let weapon = default_design("arming_sword").unwrap();
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
    assert_ne!(current, versioned_hash(domain, 2, 2, &design));

    let mut previous = envelope.clone();
    previous["schema_version"] = 2.into();
    assert!(matches!(
        decode_holder(&serde_json::to_vec(&previous).unwrap()),
        Err(CodecError::SchemaVersion {
            found: 2,
            expected: HOLDER_SCHEMA_VERSION
        })
    ));
    previous = envelope;
    previous["generator_version"] = 2.into();
    assert!(matches!(
        decode_holder(&serde_json::to_vec(&previous).unwrap()),
        Err(CodecError::GeneratorVersion {
            found: 2,
            expected: HOLDER_GENERATOR_VERSION
        })
    ));
}
