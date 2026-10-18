use adventuresim_character_creator::garment::DrapeSettings;
use fabelgeist_cloth::Fabric;
use fabelgeist_shell::SubstepCount;

#[test]
fn scalar_serde_preserves_admission_and_distinct_stage_policies() {
    enum ValidationOutcome {
        Accepted,
        Rejected,
    }
    struct Fixture {
        substeps: SubstepCount,
        host_interval: SubstepCount,
        validation: ValidationOutcome,
    }
    for fixture in [
        Fixture {
            substeps: 0.into(),
            host_interval: 0.into(),
            validation: ValidationOutcome::Rejected,
        },
        Fixture {
            substeps: 1.into(),
            host_interval: 0.into(),
            validation: ValidationOutcome::Accepted,
        },
        Fixture {
            substeps: 1.into(),
            host_interval: 1.into(),
            validation: ValidationOutcome::Accepted,
        },
        Fixture {
            substeps: 32.into(),
            host_interval: 32.into(),
            validation: ValidationOutcome::Accepted,
        },
        Fixture {
            substeps: 33.into(),
            host_interval: 0.into(),
            validation: ValidationOutcome::Rejected,
        },
        Fixture {
            substeps: 2.into(),
            host_interval: 3.into(),
            validation: ValidationOutcome::Rejected,
        },
        Fixture {
            substeps: u32::MAX.into(),
            host_interval: u32::MAX.into(),
            validation: ValidationOutcome::Rejected,
        },
    ] {
        let mut settings = DrapeSettings::for_fabric(Fabric::default());
        settings.sewing.substeps = fixture.substeps;
        settings.sewing.host_contact_interval = fixture.host_interval;
        let value = serde_json::to_value(settings).unwrap();
        assert_eq!(value["sewing"]["substeps"], u32::from(fixture.substeps));
        assert_eq!(
            value["sewing"]["host_contact_interval"],
            u32::from(fixture.host_interval)
        );
        let decoded: DrapeSettings = serde_json::from_value(value).unwrap();
        assert_eq!(decoded, settings);
        assert_eq!(
            decoded.validate().is_ok(),
            matches!(fixture.validation, ValidationOutcome::Accepted)
        );
    }
}

#[test]
fn scalar_deserialization_keeps_u32_rejections_and_typed_range_diagnostics() {
    let settings = DrapeSettings::for_fabric(Fabric::default());
    for native in [
        serde_json::json!(-1),
        serde_json::json!(4294967296u64),
        serde_json::json!(1.5),
        serde_json::json!("1"),
        serde_json::Value::Null,
    ] {
        let mut value = serde_json::to_value(settings).unwrap();
        value["sewing"]["substeps"] = native;
        assert!(serde_json::from_value::<DrapeSettings>(value).is_err());
    }
    let mut outside = settings;
    outside.sewing.substeps = SubstepCount::from(0);
    assert_eq!(
        outside.validate().unwrap_err().to_string(),
        "sewing substeps must be within 1..=32"
    );
    outside.sewing.substeps = SubstepCount::from(1);
    outside.sewing.host_contact_interval = SubstepCount::from(2);
    assert_eq!(
        outside.validate().unwrap_err().to_string(),
        "sewing host contacts cannot be spaced further apart than one step"
    );
}
