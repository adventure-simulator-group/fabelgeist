use fabelgeist_gpu::prelude::*;

fn values() -> PassParameters {
    let mut parameters = PassParameters::new();
    parameters.insert("float".into(), (-0.0f64).into());
    parameters.insert("word".into(), (-7i32).into());
    parameters.insert("vec2".into(), Vec2::new(1.25, -3.0).into());
    parameters.insert("vec3".into(), Vec3::new(-0.0, 5.5, 9.0).into());
    parameters.insert("vec4".into(), Vec4::new(1.0, 2.0, 3.0, 4.0).into());
    parameters.insert(
        "mat2".into(),
        Mat2 {
            columns: [[1.0, 2.0], [3.0, 4.0]],
        }
        .into(),
    );
    parameters.insert(
        "mat3".into(),
        Mat3 {
            columns: [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
        }
        .into(),
    );
    parameters.insert(
        "mat4".into(),
        Mat4 {
            columns: [
                [1.0, 2.0, 3.0, 4.0],
                [5.0, 6.0, 7.0, 8.0],
                [9.0, 10.0, 11.0, 12.0],
                [13.0, 14.0, 15.0, 16.0],
            ],
        }
        .into(),
    );
    parameters.insert("transform".into(), Transform::default().into());
    parameters.insert("unsupported".into(), Sampler::default().into());
    parameters
}

#[test]
fn general_and_cached_bytes_and_failures_match_original_implementation() {
    let frozen: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/uniform_packing.json")).unwrap();
    let general = [
        ("float", 0, 4),
        ("word", 4, 4),
        ("vec2", 8, 8),
        ("vec3", 16, 12),
        ("vec4", 32, 16),
        ("mat2", 48, 16),
        ("mat3", 64, 48),
        ("mat4", 112, 64),
        ("transform", 176, 64),
        ("absent", 240, 4),
        ("unsupported", 244, 4),
        ("vec4", 248, 16),
    ];
    let cached = [
        ("float", 0, 4),
        ("word", 4, 4),
        ("vec2", 8, 8),
        ("vec3", 16, 12),
        ("vec4", 32, 16),
        ("mat4", 48, 64),
        ("vec4", 120, 16),
    ];
    let parameters = values();
    for (policy, members, expected) in [
        (
            UniformPackingPolicy::General,
            general.as_slice(),
            &frozen["general"],
        ),
        (
            UniformPackingPolicy::Cached,
            cached.as_slice(),
            &frozen["cached"],
        ),
    ] {
        let members: Vec<UniformMember> = members
            .iter()
            .map(
                |&(name, offset, size): &(&str, u32, u32)| -> UniformMember {
                    UniformMember {
                        name: name.into(),
                        offset: BufferByteOffset::from(offset),
                        size: BufferByteLength::from(size),
                    }
                },
            )
            .collect();
        let expected: Vec<u8> = serde_json::from_value(expected.clone()).unwrap();
        let mut actual = vec![0u8; expected.len()];
        UniformBytes::from(actual.as_mut_slice())
            .pack(&members, &parameters, policy)
            .unwrap();
        assert_eq!(actual, expected);
    }
    for name in ["absent", "unsupported", "mat2", "mat3", "transform"] {
        let member = UniformMember {
            name: name.into(),
            offset: BufferByteOffset::START,
            size: BufferByteLength::from(64u32),
        };
        let mut bytes = [0u8; 256];
        let error = UniformBytes::from(bytes.as_mut_slice())
            .pack(&[member], &parameters, UniformPackingPolicy::Cached)
            .unwrap_err();
        assert_eq!(error.to_string(), frozen["errors"][name].as_str().unwrap());
        match error {
            UniformPackingError::Missing { member } => {
                assert_eq!(member, PassParameterName::from("absent"))
            }
            UniformPackingError::Unsupported { member, .. } => {
                assert_eq!(member, PassParameterName::from(name))
            }
            UniformPackingError::HostLayout { .. } => panic!("fixture layouts fit the host"),
        }
    }
}

#[test]
fn scalar_casts_preserve_ieee_words_and_signed_admission() {
    let frozen: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/uniform_packing.json")).unwrap();
    for (name, value) in [
        ("nan", f64::NAN),
        ("infinity", f64::INFINITY),
        ("rounding", 1.000_000_059_604_644_8_f64),
    ] {
        let mut parameters = PassParameters::new();
        parameters.insert(name.into(), value.into());
        let member = UniformMember {
            name: name.into(),
            offset: BufferByteOffset::START,
            size: BufferByteLength::from(4u32),
        };
        let mut actual = [0u8; 4];
        UniformBytes::from(actual.as_mut_slice())
            .pack(&[member], &parameters, UniformPackingPolicy::Cached)
            .unwrap();
        let expected: Vec<u8> = serde_json::from_value(frozen["specials"][name].clone()).unwrap();
        assert_eq!(actual.as_slice(), expected);
    }
}

#[test]
fn packing_leaves_matrix_padding_and_skipped_regions_untouched() {
    let members = [
        UniformMember {
            name: "mat3".into(),
            offset: BufferByteOffset::from(4u32),
            size: BufferByteLength::from(48u32),
        },
        UniformMember {
            name: "absent".into(),
            offset: BufferByteOffset::from(52u32),
            size: BufferByteLength::from(4u32),
        },
    ];
    let mut bytes = [0xabu8; 56];
    UniformBytes::from(bytes.as_mut_slice())
        .pack(&members, &values(), UniformPackingPolicy::General)
        .unwrap();
    assert_eq!(&bytes[..4], &[0xab; 4]);
    for padding in [16..20, 32..36, 48..52, 52..56] {
        assert_eq!(&bytes[padding], &[0xab; 4]);
    }
}

#[test]
fn parameter_identity_preserves_spelling_replacement_and_order() {
    let mut parameters = PassParameters::new();
    let name = PassParameterName::from(" Δ \0");
    parameters.insert(name.clone(), 3u32.into());
    parameters.insert("second".into(), 4u32.into());
    parameters.insert(name.clone(), 5u32.into());
    assert!(parameters.get(&PassParameterName::from("Δ")).is_none());
    let keys: Vec<_> = parameters
        .iter()
        .map(|(key, _): (&PassParameterName, &PassParameter)| -> PassParameterName { key.clone() })
        .collect();
    assert_eq!(keys, [name.clone(), PassParameterName::from("second")]);
    assert!(
        matches!(parameters.get(&name), Some(PassParameter::Unsigned(value)) if *value == UniformUnsigned::from(5u32))
    );
    assert_eq!(ShaderBindingName::from(" Δ \0").parameter_name(), &name);
    parameters.overlay(PassParameters::from([
        ("second".into(), 9u32.into()),
        (name.clone(), 6u32.into()),
        ("third".into(), 7u32.into()),
    ]));
    let keys: Vec<_> = parameters
        .iter()
        .map(|(key, _): (&PassParameterName, &PassParameter)| -> PassParameterName { key.clone() })
        .collect();
    assert_eq!(keys, [name.clone(), "second".into(), "third".into()]);
    assert!(
        matches!(parameters.get(&name), Some(PassParameter::Unsigned(value)) if *value == UniformUnsigned::from(6u32))
    );
}
