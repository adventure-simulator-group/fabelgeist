use fabelgeist_determinism::{SamplingError, Seed, StreamId};
use std::num::NonZeroU64;

fn seed() -> Seed {
    Seed::derive(
        b"resident:town:7",
        StreamId::new("resident.surname"),
        &[&42_u64.to_le_bytes(), b"north"],
    )
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn complete_derivation_and_sampling_vectors() {
    let mut indices = seed().rng();
    assert_eq!(indices.index(3), 0);
    assert_eq!(indices.index(257), 35);
    assert!(seed().rng().boolean());
    assert_eq!(Seed::from_u64(0).rng().next_u64(), 0xe220_a839_7b1d_cdaf);
    let mut rng = seed().rng();
    let words: Vec<_> = (0..4).map(|_| rng.next_u64()).collect();
    let bounds = [
        1,
        3,
        257,
        u32::MAX as u64,
        u32::MAX as u64 + 1,
        u32::MAX as u64 + 2,
        u64::MAX,
    ];
    let bounded: Vec<_> = bounds
        .into_iter()
        .map(|bound| rng.below(NonZeroU64::new(bound).unwrap()))
        .collect();
    let floats = [
        rng.unit_f32().to_bits() as u64,
        rng.unit_f64().to_bits(),
        rng.inclusive_unit_f32().to_bits() as u64,
        rng.inclusive_unit_f64().to_bits(),
    ];
    let mut permutation = [0, 1, 2, 3, 4, 5, 6, 7];
    rng.shuffle(&mut permutation);
    let weighted: Vec<_> = (0..8)
        .map(|_| rng.weighted_index(&[0, 1, 7, 2]).unwrap())
        .collect();
    let order = rng.weighted_order(&[0, 1, 7, 2]).unwrap();
    assert_eq!(seed().to_le_bytes(), [203, 252, 7, 57, 2, 74, 126, 13]);
    assert_eq!(
        words,
        [
            2887654850984301039,
            2527706837764416413,
            8760451997279430129,
            3381265251239552993
        ]
    );
    assert_eq!(
        bounded,
        [
            0,
            1,
            116,
            403430187,
            205040374,
            3345612165,
            11562767038108464683
        ]
    );
    assert_eq!(
        floats,
        [
            1063537731,
            4587984613187871104,
            1062229215,
            4604477385842690760
        ]
    );
    assert_eq!(permutation, [3, 4, 7, 0, 1, 2, 5, 6]);
    assert_eq!(weighted, [2, 2, 1, 3, 3, 2, 2, 2]);
    assert_eq!(order, [3, 2, 1]);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn framing_and_independent_streams() {
    let purpose = StreamId::new("fixture.choice");
    assert_ne!(
        Seed::derive(b"ab", purpose, &[b"c"]),
        Seed::derive(b"a", purpose, &[b"bc"])
    );
    assert_ne!(
        Seed::derive(b"root", purpose, &[b"ab", b"c"]),
        Seed::derive(b"root", purpose, &[b"a", b"bc"])
    );
    assert_ne!(
        Seed::derive(b"root", purpose, &[]),
        Seed::derive(b"root", purpose, &[b""])
    );
    let mut a = seed().rng();
    let mut unrelated = seed().child(purpose, &[]).rng();
    for _ in 0..100 {
        unrelated.next_u64();
    }
    assert_eq!(a.next_u64(), seed().rng().next_u64());
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn weights_fail_explicitly_and_exclude_zero() {
    let mut rng = seed().rng();
    assert_eq!(rng.weighted_index(&[]), Err(SamplingError::EmptyCandidates));
    assert_eq!(
        rng.weighted_index(&[0, 0]),
        Err(SamplingError::ZeroTotalWeight)
    );
    assert_eq!(
        rng.weighted_index(&[u64::MAX, 1]),
        Err(SamplingError::WeightOverflow)
    );
    assert_eq!(rng.weighted_order(&[0, 5, 0]), Ok(vec![1]));
    let mut counts = [0; 3];
    for _ in 0..30_000 {
        counts[rng.weighted_index(&[0, 1, 2]).unwrap()] += 1;
    }
    assert_eq!(counts[0], 0);
    assert!((9_500..10_500).contains(&counts[1]), "{counts:?}");
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn empty_and_singleton_shuffles_and_float_endpoints() {
    let mut rng = seed().rng();
    let before = rng.clone();
    rng.shuffle::<u8>(&mut []);
    rng.shuffle(&mut [9]);
    assert_eq!(rng, before);
    assert_eq!(fabelgeist_determinism::unit_f32(0).to_bits(), 0);
    assert_eq!(
        fabelgeist_determinism::unit_f32(u64::MAX).to_bits(),
        0x3f7fffff
    );
    assert_eq!(
        fabelgeist_determinism::unit_f64(u64::MAX).to_bits(),
        0x3fefffffffffffff
    );
    assert_eq!(
        fabelgeist_determinism::inclusive_unit_f32(u64::MAX).to_bits(),
        1_f32.to_bits()
    );
    assert_eq!(
        fabelgeist_determinism::inclusive_unit_f64(u64::MAX).to_bits(),
        1_f64.to_bits()
    );
}
