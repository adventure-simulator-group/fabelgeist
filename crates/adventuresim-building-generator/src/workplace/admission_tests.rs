use super::*;

#[derive(Serialize)]
struct NativePassage {
    id: WorkplacePassageId,
    purpose: WorkplacePassagePurpose,
    min: Vec3,
    max: Vec3,
}

#[test]
fn passage_decoding_preserves_wire_and_requires_positive_clearance() {
    let mut native = NativePassage {
        id: WorkplacePassageId(83),
        purpose: WorkplacePassagePurpose::ServiceClearance,
        min: Vec3::new(-1.0, -0.2, 2.0),
        max: Vec3::new(1.0, 2.2, 4.0),
    };
    let wire = postcard::to_allocvec(&native).unwrap();
    let passage: WorkplacePassage = postcard::from_bytes(&wire).unwrap();
    assert_eq!(postcard::to_allocvec(&passage).unwrap(), wire);
    for max in [
        native.min,
        Vec3::new(-2.0, 2.2, 4.0),
        Vec3::splat(f32::INFINITY),
    ] {
        native.max = max;
        assert!(
            postcard::from_bytes::<WorkplacePassage>(&postcard::to_allocvec(&native).unwrap())
                .is_err()
        );
    }
    let error = serde_json::from_str::<WorkplacePassage>(
        r#"{"id":83,"purpose":"service_clearance","min":[0,0,0],"max":[1,0,1]}"#,
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains("83") && error.contains("positive"),
        "{error}"
    );
}
