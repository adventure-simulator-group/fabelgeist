use super::*;

#[test]
fn detail_admission_accepts_only_runtime_levels_and_retains_rejections() {
    for value in 0..=u8::MAX {
        match CharacterLod::try_from(value) {
            Ok(lod) => {
                assert!((4..=6).contains(&value));
                assert_eq!(lod.to_string(), value.to_string());
            }
            Err(error) => {
                assert!(!(4..=6).contains(&value));
                assert_eq!(error.violation(), CharacterLodViolation::UnsupportedDetail);
                assert!(error.to_string().contains(&value.to_string()));
                assert!(std::error::Error::source(&error).is_none());
            }
        }
    }
    for spelling in ["not a detail", "256", "-1", ""] {
        let error = spelling.parse::<CharacterLod>().unwrap_err();
        assert_eq!(error.violation(), CharacterLodViolation::InvalidEncoding);
        assert!(std::error::Error::source(&error).is_some());
        assert!(error.to_string().contains(&format!("{spelling:?}")));
    }
    assert_eq!(
        "04".parse::<CharacterLod>().unwrap(),
        CharacterLod::Detailed
    );
    let error = "03".parse::<CharacterLod>().unwrap_err();
    assert_eq!(error.violation(), CharacterLodViolation::UnsupportedDetail);
    assert!(error.to_string().contains("03"));
}

#[test]
fn configuration_round_trips_existing_wire_values_and_rejects_invalid_detail() {
    for lod in CharacterLod::ALL {
        for policy in [
            PoseCorrectivePolicy::Enabled,
            PoseCorrectivePolicy::Disabled,
        ] {
            let config = MhrConfig {
                lod,
                pose_correctives: policy,
            };
            let wire = serde_json::to_value(config).unwrap();
            assert_eq!(wire["lod"], lod.to_string().parse::<u8>().unwrap());
            assert_eq!(
                wire["pose_correctives"],
                policy == PoseCorrectivePolicy::Enabled
            );
            assert_eq!(serde_json::from_value::<MhrConfig>(wire).unwrap(), config);
        }
    }
    for wire in [
        r#"{"lod":3,"pose_correctives":true}"#,
        r#"{"lod":7,"pose_correctives":false}"#,
        r#"{"lod":"4","pose_correctives":true}"#,
        r#"{"lod":4,"pose_correctives":"false"}"#,
    ] {
        assert!(serde_json::from_str::<MhrConfig>(wire).is_err());
    }
}

#[test]
fn defaults_and_native_asset_catalog_preserve_the_runtime_contract() {
    assert_eq!(
        MhrConfig::default(),
        MhrConfig {
            lod: CharacterLod::Detailed,
            pose_correctives: PoseCorrectivePolicy::Enabled,
        }
    );
    for (lod, detail, vertices) in [
        (CharacterLod::Detailed, 4, 2_461),
        (CharacterLod::Reduced, 5, 971),
        (CharacterLod::Minimal, 6, 595),
    ] {
        assert_eq!(lod.rig_filename(), format!("lod{detail}.fbx"));
        assert_eq!(
            lod.corrective_basis_filename(),
            format!("corrective_blendshapes_lod{detail}.npz")
        );
        assert_eq!(lod.vertices().to_string(), vertices.to_string());
    }
}
