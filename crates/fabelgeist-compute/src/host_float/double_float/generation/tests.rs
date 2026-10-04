use super::*;

#[test]
fn constants_sum_to_their_values() {
    let quarter_turn: f64 = QUARTER_TURN.into_iter().map(f64::from).sum();
    let ln_2: f64 = LN_2.into_iter().map(f64::from).sum();
    assert_eq!(quarter_turn, std::f64::consts::FRAC_PI_2);
    assert_eq!(ln_2, std::f64::consts::LN_2);
}

#[test]
fn literals_parse_back_to_their_floats() {
    for value in QUARTER_TURN.into_iter().chain(LN_2).chain([
        0.5,
        -1.0,
        -0.0,
        3.0e38,
        1.0e-30,
        f32::from_bits(1),
    ]) {
        let text = WgslFloatLiteral::try_from(value).unwrap().to_string();
        let parsed: f32 = text.trim_end_matches('f').parse().unwrap();
        assert_eq!(parsed.to_bits(), value.to_bits(), "{text}");
    }
}

#[test]
fn literal_admission_distinguishes_nonfinite_overflow_and_underflow() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            DoubleFloatLiteral::try_from(value),
            Err(LiteralAdmissionError::NonFinite { .. })
        ));
    }
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(matches!(
            WgslFloatLiteral::try_from(value),
            Err(LiteralAdmissionError::NonFinite { .. })
        ));
    }
    let value = f64::from(f32::MAX) * 2.0;
    assert!(matches!(
        DoubleFloatLiteral::try_from(value),
        Err(LiteralAdmissionError::LeadingOverflow { value: found }) if found == value
    ));
    let value = f64::from(f32::from_bits(1)) / 4.0;
    assert!(matches!(
        DoubleFloatLiteral::try_from(value),
        Err(LiteralAdmissionError::Underflow { value: found }) if found == value
    ));
    for value in [0.0, -0.0, f64::from(f32::from_bits(1)), f64::from(f32::MAX)] {
        let literal = DoubleFloatLiteral::try_from(value).unwrap();
        assert_eq!(literal.high.0.to_bits(), (value as f32).to_bits());
        assert_eq!(f64::from(literal.high.0) + f64::from(literal.low.0), value);
    }
}

#[test]
fn series_terms_support_mixed_direction_without_duplication() {
    for series in [
        PolynomialSeries::Sine,
        PolynomialSeries::Cosine,
        PolynomialSeries::Atanh,
        PolynomialSeries::Exponential,
    ] {
        let expected: Vec<SeriesTerm> = PolynomialTerms::from(series).collect();
        let mut terms = PolynomialTerms::from(series);
        let mut found = Vec::new();
        while let Some(front) = terms.next() {
            found.push(front);
            if let Some(back) = terms.next_back() {
                found.push(back);
            }
        }
        assert!(terms.next().is_none());
        assert!(terms.next_back().is_none());
        for term in &expected {
            let mut occurrences = 0;
            for candidate in &found {
                if candidate == term {
                    occurrences += 1;
                }
            }
            assert_eq!(occurrences, 1);
        }
        assert_eq!(found.len(), expected.len());
        for term in expected {
            let coefficient = DoubleFloatLiteral::from(SeriesCoefficient::from(term));
            assert!(coefficient.high.0.is_finite());
            assert!(coefficient.low.0.is_finite());
        }
    }
}

#[test]
fn admitted_terms_follow_their_series_coefficient_law() {
    for (series, expected) in [
        (PolynomialSeries::Sine, [1.0, -1.0 / 6.0, 1.0 / 120.0]),
        (PolynomialSeries::Cosine, [1.0, -0.5, 1.0 / 24.0]),
        (PolynomialSeries::Atanh, [2.0, 2.0 / 3.0, 2.0 / 5.0]),
        (PolynomialSeries::Exponential, [1.0, 1.0, 0.5]),
    ] {
        for (term, expected) in PolynomialTerms::from(series).zip(expected) {
            assert_eq!(SeriesCoefficient::from(term).0, expected);
        }
    }
}
