//! Endpoint admission and public parser/clamping contract regressions.

use fabelgeist_mhr::model_def::{
    ParameterBounds, ParameterBoundsAdmissionError, ParameterBoundsError, ParameterLimit,
};
use fabelgeist_mhr::{ParameterTransform, Skeleton, parse_model_definition};

fn skeleton() -> Skeleton {
    Skeleton {
        names: vec!["root".into()],
        parents: vec![-1],
        translation_offsets: vec![[0.0; 3]],
        prerotations: vec![[0.0, 0.0, 0.0, 1.0]],
    }
}

fn model_definition(limits: &str) -> String {
    format!(
        "Momentum Model Definition V1.0\n[ParameterTransform]\n\
         root.tx=1*bend\n[Limits]\n{limits}\n"
    )
}

#[test]
fn constructor_rejects_nan_and_reversed_endpoints() {
    let payload_nan = f32::from_bits(0x7fc0_0042);
    for endpoints in [
        (payload_nan, 1.0),
        (0.0, payload_nan),
        (payload_nan, payload_nan),
    ] {
        assert_eq!(
            ParameterBounds::try_from(endpoints),
            Err(ParameterBoundsError::NaNEndpoint)
        );
    }
    for endpoints in [(3.0, -2.0), (f32::INFINITY, f32::NEG_INFINITY)] {
        assert_eq!(
            ParameterBounds::try_from(endpoints),
            Err(ParameterBoundsError::Reversed)
        );
    }
}

#[test]
fn admitted_intervals_preserve_endpoint_bits() {
    for endpoints in [
        (-0.5, 1.5),
        (0.25, 0.25),
        (f32::NEG_INFINITY, f32::INFINITY),
        (f32::NEG_INFINITY, f32::NEG_INFINITY),
        (f32::INFINITY, f32::INFINITY),
        (-0.0, 0.0),
        (0.0, -0.0),
        (f32::from_bits(1), f32::from_bits(2)),
    ] {
        let bounds = ParameterBounds::try_from(endpoints).unwrap();
        let (minimum, maximum): (f32, f32) = bounds.into();
        assert_eq!(minimum.to_bits(), endpoints.0.to_bits());
        assert_eq!(maximum.to_bits(), endpoints.1.to_bits());
    }
}

#[test]
fn parser_rejects_invalid_endpoints_with_classification_and_source_context() {
    for (endpoints, cause) in [
        ("NaN,1", ParameterBoundsError::NaNEndpoint),
        ("0,NaN", ParameterBoundsError::NaNEndpoint),
        ("NaN,NaN", ParameterBoundsError::NaNEndpoint),
        ("3,-2", ParameterBoundsError::Reversed),
        ("inf,-inf", ParameterBoundsError::Reversed),
    ] {
        let line = format!("limit bend minmax [{endpoints}] 0.1");
        let error = parse_model_definition(&model_definition(&line), &skeleton()).unwrap_err();
        let admission = error
            .downcast_ref::<ParameterBoundsAdmissionError>()
            .unwrap();
        assert_eq!(admission.parameter(), "bend");
        assert_eq!(admission.line(), line);
        assert_eq!(admission.cause(), cause);
        assert_eq!(
            error
                .chain()
                .find_map(|cause| cause.downcast_ref::<ParameterBoundsError>()),
            Some(&cause)
        );
        assert!(error.to_string().contains("parameter bend"));
        assert!(error.to_string().contains(&line));
        assert_eq!(error.root_cause().to_string(), cause.to_string());
    }
}

#[test]
fn parser_keeps_unsupported_unknown_and_malformed_limit_behavior() {
    let pt = parse_model_definition(
        &model_definition(
            "limit missing minmax [NaN,1]\n\
             limit bend linear [3,-2]\n\
             limit bend minmax [bad,0,1] invalid\n\
             limit bend minmax [0,bad]\n\
             limit bend minmax [0,1,2]\n\
             limit bend minmax\n\
             limit bend minmax 0,1\n\
             limit bend minmax [0,1",
        ),
        &skeleton(),
    )
    .unwrap();
    assert_eq!(pt.limits.len(), 1);
    assert_eq!(
        pt.limits[0].bounds,
        ParameterBounds::try_from((0.0, 1.0)).unwrap()
    );
    assert_eq!(pt.limits[0].weight, 1.0);
}

#[test]
fn clamp_keeps_inclusive_equal_infinite_and_signed_zero_behavior() {
    for (endpoints, input, expected) in [
        ((-0.5, 1.5), -0.5, -0.5),
        ((-0.5, 1.5), 1.5, 1.5),
        ((-0.5, 1.5), -3.0, -0.5),
        ((-0.5, 1.5), 3.0, 1.5),
        ((0.25, 0.25), -3.0, 0.25),
        ((f32::NEG_INFINITY, f32::INFINITY), 3.0, 3.0),
        ((f32::INFINITY, f32::INFINITY), 0.0, f32::INFINITY),
        (
            (f32::NEG_INFINITY, f32::NEG_INFINITY),
            0.0,
            f32::NEG_INFINITY,
        ),
        ((-0.0, 0.0), -1.0, -0.0),
        ((-0.0, 0.0), 1.0, 0.0),
        ((0.0, -0.0), -1.0, 0.0),
        ((0.0, -0.0), 1.0, -0.0),
        ((-0.0, 0.0), -0.0, -0.0),
        ((0.0, -0.0), 0.0, 0.0),
    ] {
        let pt = ParameterTransform {
            limits: vec![ParameterLimit {
                parameter: 0,
                bounds: ParameterBounds::try_from(endpoints).unwrap(),
                weight: 0.0,
            }],
            ..Default::default()
        };
        let mut values = [input];
        pt.apply_limits(&mut values);
        assert_eq!(values[0].to_bits(), expected.to_bits());
    }
    let pt =
        parse_model_definition(&model_definition("limit bend minmax [-1,1]"), &skeleton()).unwrap();
    let mut values = [f32::from_bits(0x7fc0_0042)];
    pt.apply_limits(&mut values);
    assert_eq!(values[0].to_bits(), 0x7fc0_0042);
}

#[test]
fn limits_apply_in_file_order_without_solver_weight_or_missing_slot_effects() {
    let pt = parse_model_definition(
        &model_definition("limit bend minmax [1,2] 0\nlimit bend minmax [-2,0] NaN"),
        &skeleton(),
    )
    .unwrap();
    assert_eq!(pt.limits.len(), 2);
    assert!(pt.limits[1].weight.is_nan());
    let mut values = [3.0, -0.0];
    pt.apply_limits(&mut values);
    assert_eq!(values[0].to_bits(), 0.0_f32.to_bits());
    assert_eq!(values[1].to_bits(), (-0.0_f32).to_bits());
    pt.apply_limits(&mut []);
}
