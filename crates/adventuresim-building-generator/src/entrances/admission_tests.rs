use super::*;

#[derive(Serialize)]
struct NativeEntrance {
    id: BuildingEntranceId,
    support: BuildingEntranceSupport,
    threshold_metres: Vec2,
    outward: Vec2,
}

#[test]
fn decoding_retains_entrance_identity_and_checks_threshold_and_direction() {
    let mut native = NativeEntrance {
        id: BuildingEntranceId::Opening(OpeningAssemblyId(712)),
        support: BuildingEntranceSupport::ArchitecturalFloor,
        threshold_metres: Vec2::new(-2.0, 3.0),
        outward: Vec2::NEG_Y,
    };
    let wire = postcard::to_allocvec(&native).unwrap();
    let admitted: BuildingEntrance = postcard::from_bytes(&wire).unwrap();
    assert_eq!(postcard::to_allocvec(&admitted).unwrap(), wire);
    native.threshold_metres.x = f32::INFINITY;
    assert!(
        postcard::from_bytes::<BuildingEntrance>(&postcard::to_allocvec(&native).unwrap()).is_err()
    );
    let cause = serde_json::from_str::<BuildingEntrance>(
        r#"{"id":{"opening":712},"support":"architectural_floor","threshold_metres":[-2,3],"outward":[0,2]}"#,
    ).unwrap_err().to_string();
    assert!(
        cause.contains("712") && cause.contains("normalized"),
        "{cause}"
    );
    native.threshold_metres = Vec2::ZERO;
    for direction in [Vec2::ZERO, Vec2::splat(f32::NAN), Vec2::new(0.0, 2.0)] {
        native.outward = direction;
        assert!(
            postcard::from_bytes::<BuildingEntrance>(&postcard::to_allocvec(&native).unwrap())
                .is_err()
        );
    }
}
