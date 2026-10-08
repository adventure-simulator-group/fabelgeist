use super::*;

#[test]
fn preserves_main_hash_layout_boundaries() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/hash-layouts.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let requested = case["requested"].as_u64().unwrap() as u32;
        let capacity = ParticleCapacity::from(requested);
        let expected = &case["layout"];
        if expected.is_null() {
            #[cfg(debug_assertions)]
            assert!(
                std::panic::catch_unwind(|| CollisionTableSize::for_capacity(capacity)).is_err()
            );
            continue;
        }
        let size = CollisionTableSize::for_capacity(capacity);
        assert_eq!(
            serde_json::json!({
                "capacity":u32::from(capacity),
                "buckets":u32::from(size),
                "starts_bytes":u64::from(size.starts_bytes()),
                "clear_items":u32::from(size)+1,
            }),
            *expected
        );
    }
}
