use super::*;

#[test]
fn object_names_keep_first_separator_lossy_decode_and_last_namespace_policy() {
    let cases: [(&[u8], &str, &str); 8] = [
        (b"scope:joint\0\x01LimbNode", "scope:joint", "joint"),
        (b"a:b:joint", "a:b:joint", "joint"),
        (b"scope:", "scope:", ""),
        (b" name ", " name ", " name "),
        (b"scope:\xff\0\x01X\0\x01Y", "scope:�", "�"),
        (b"a\0b", "a\0b", "a\0b"),
        (b"\0\x01Class", "", ""),
        (b"", "", ""),
    ];
    for (encoded, expected_qualified, expected_normalized) in cases {
        let qualified = FbxQualifiedName::from(StorageView::from(encoded));
        let normalized = FbxObjectName::from(&qualified);
        assert_eq!(qualified.to_string(), expected_qualified);
        assert_eq!(normalized.to_string(), expected_normalized);
    }
}

#[test]
fn missing_or_nontext_names_are_empty_without_dropping_object_identity() {
    for property in [
        None,
        Some(Prop::Bool(true)),
        Some(Prop::Raw(b"scope:raw".to_vec())),
    ] {
        let mut props = vec![Prop::I64(7)];
        props.extend(property.clone());
        let scene = Scene::from_roots(vec![Node {
            name: FbxRecordName::OBJECTS,
            props: Vec::new(),
            children: vec![Node {
                name: FbxRecordName::MODEL,
                props,
                children: Vec::new(),
            }],
        }]);
        let object = scene.get(FbxObjectId::from(7)).unwrap();
        let expected = if matches!(property, Some(Prop::Raw(_))) {
            "raw"
        } else {
            ""
        };
        assert_eq!(object.name.to_string(), expected);
    }
}
