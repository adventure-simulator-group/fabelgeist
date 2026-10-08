use super::*;
/// The defining property: the weights annihilate any affine image of the
/// rest points, and only that.
#[test]
fn bending_weights_annihilate_affine_maps() {
    let points = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.3, 0.8, 0.0),
        Vec3::new(0.7, -0.9, 0.0),
    ];
    let weights = BendWeights::for_points(BendPoints::from(points))
        .expect("a proper planar quad has weights");

    // Sum to zero, so the measure is translation-invariant.
    assert!(weights.0.iter().sum::<f32>().abs() < 1e-5);

    let combine = |points: [Vec3; 4]| -> Vec3 {
        let mut sum = Vec3::default();
        for (point, weight) in points.iter().zip(weights.0) {
            sum += *point * weight;
        }
        sum
    };
    assert!(combine(points).length() < 1e-5, "rest is not annihilated");

    // Translation, rotation, uniform scale and shear are all affine, so
    // every one of them must still measure zero.
    let translated = points.map(|p: Vec3| -> Vec3 { p + Vec3::new(3.0, -2.0, 7.0) });
    assert!(combine(translated).length() < 1e-5, "translation");

    let rotated = points.map(|p: Vec3| -> Vec3 {
        let (s, c) = 0.7f32.sin_cos();
        Vec3::new(p.x * c - p.y * s, p.x * s + p.y * c, p.z)
    });
    assert!(combine(rotated).length() < 1e-5, "rotation");

    let scaled = points.map(|p: Vec3| -> Vec3 { p * 2.5 });
    assert!(combine(scaled).length() < 1e-5, "uniform scale");

    let sheared = points.map(|p: Vec3| -> Vec3 { Vec3::new(p.x + 0.4 * p.y, p.y, p.z) });
    assert!(combine(sheared).length() < 1e-4, "shear");

    // Folding one wing out of the plane is not affine, and must register.
    let mut folded = points;
    folded[3] = Vec3::new(0.7, -0.4, 0.5);
    assert!(
        combine(folded).length() > 0.05,
        "a fold produced no bending measure"
    );
}

/// The measure has to grow with the fold, or compliance means nothing.
#[test]
fn bending_weights_grow_with_the_fold() {
    let points = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.5, 0.6, 0.0),
        Vec3::new(0.5, -0.6, 0.0),
    ];
    let weights = BendWeights::for_points(BendPoints::from(points)).unwrap();
    let combine =
        |points: [Vec3; 4]| -> BendRestMeasure { weights.measure(BendPoints::from(points)) };

    let mut previous = BendRestMeasure(0.0);
    for lift in [0.05f32, 0.1, 0.2, 0.4] {
        let mut folded = points;
        folded[3] = Vec3::new(0.5, -0.6, lift);
        let measure = combine(folded);
        assert!(measure > previous, "lifting to {lift} did not increase it");
        previous = measure;
    }
}

#[test]
fn bending_weights_reject_degenerate_quads() {
    // All four collinear: no unique dependency.
    let collinear = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(3.0, 0.0, 0.0),
    ];
    assert!(BendWeights::for_points(BendPoints::from(collinear)).is_err());

    // All coincident.
    assert!(BendWeights::for_points(BendPoints::from([Vec3::default(); 4])).is_err());
}

#[test]
fn original_weights_rest_and_record_words_preserve_extreme_geometry() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("../../tests/fixtures/bending_records.json")).unwrap();
    let fixtures = fixtures.as_array().unwrap();
    assert_eq!(fixtures.len(), 36);
    for fixture in fixtures {
        let mut points = [Vec3::default(); 4];
        for (point, row) in points.iter_mut().zip(fixture["points"].as_array().unwrap()) {
            let row = row.as_array().unwrap();
            *point = Vec3::new(
                f32::from_bits(row[0].as_u64().unwrap() as u32),
                f32::from_bits(row[1].as_u64().unwrap() as u32),
                f32::from_bits(row[2].as_u64().unwrap() as u32),
            );
        }
        let points = BendPoints::from(points);
        match BendWeights::for_points(points) {
            Ok(weights) => {
                let mut expected = Vec::new();
                for word in fixture["weights"].as_array().unwrap() {
                    expected.push(word.as_u64().unwrap() as u32);
                }
                assert_eq!(
                    weights.0.map(f32::to_bits).as_slice(),
                    expected,
                    "{}",
                    fixture["name"]
                );
                assert_eq!(
                    weights.measure(points).0.to_bits(),
                    fixture["rest"].as_u64().unwrap() as u32,
                    "{}",
                    fixture["name"]
                );
                let record = weights.observed_rest(points);
                let native: &[u32] = bytemuck::cast_slice(std::slice::from_ref(&record));
                let mut words = expected.clone();
                words.push(fixture["rest"].as_u64().unwrap() as u32);
                words.extend([0; 3]);
                assert_eq!(native, words);
                let flat = weights.flat_rest();
                let native: &[u32] = bytemuck::cast_slice(std::slice::from_ref(&flat));
                let mut words = expected;
                words.extend([0; 4]);
                assert_eq!(native, words);
            }
            Err(_) => assert!(
                fixture["weights"].is_null() && fixture["rest"].is_null(),
                "{}",
                fixture["name"]
            ),
        }
    }
    let slack = BendRecord::slack();
    assert_eq!(
        bytemuck::cast_slice::<_, u32>(std::slice::from_ref(&slack)),
        [0; 8]
    );
    assert_eq!(slack.validity(), BendRecordValidity::Finite);
}

#[test]
fn classification_preserves_the_earliest_degenerate_contract() {
    assert_eq!(
        BendWeights::for_points(BendPoints::from([Vec3::default(); 4])),
        Err(BendGeometryError::CollapsedSpokes)
    );
    let line = [
        Vec3::default(),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(3.0, 0.0, 0.0),
    ];
    assert_eq!(
        BendWeights::for_points(BendPoints::from(line)),
        Err(BendGeometryError::CollinearSpokes)
    );
}

#[test]
fn later_degenerate_stages_and_record_validity_keep_their_roles() {
    let tiny = BendPoints::from([
        Vec3::default(),
        Vec3::new(1e-8, 0.0, 0.0),
        Vec3::new(0.3e-8, 0.8e-8, 0.0),
        Vec3::new(0.7e-8, -0.9e-8, 0.0),
    ]);
    assert_eq!(
        BendWeights::for_points(tiny),
        Err(BendGeometryError::VanishingMinors)
    );
    let zero_hinge = BendPoints::from([
        Vec3::default(),
        Vec3::default(),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    ]);
    assert_eq!(
        BendWeights::for_points(zero_hinge),
        Err(BendGeometryError::NegligibleArea)
    );
    let proper = BendPoints::from([
        Vec3::default(),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.3, 0.8, 0.0),
        Vec3::new(0.7, -0.9, 0.0),
    ]);
    let weights = BendWeights::for_points(proper).unwrap();
    assert_eq!(
        weights.observed_rest(proper).validity(),
        BendRecordValidity::Finite
    );
    let invalid = BendPoints::from([Vec3::new(f32::NAN, 0.0, 0.0); 4]);
    assert_eq!(
        weights.observed_rest(invalid).validity(),
        BendRecordValidity::NonFinite
    );
}
