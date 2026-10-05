use super::*;

#[test]
#[cfg(target_pointer_width = "64")]
fn preserves_main_host_growth_before_native_narrowing() {
    for line in include_str!("../../../tests/fixtures/collider-capacity.txt").lines() {
        let values: Vec<u64> = line
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();
        let count = ColliderCount::from(values[0] as usize);
        let capacity = match ColliderCapacity::INITIAL.fit(count) {
            ColliderCapacityFit::Retained => ColliderCapacity::INITIAL,
            ColliderCapacityFit::GrowthRequired => ColliderCapacity::for_count(count),
        };
        assert_eq!(capacity.0 as u64, values[1]);
        assert_eq!(u64::from(capacity.byte_length()), values[2]);
    }
}
