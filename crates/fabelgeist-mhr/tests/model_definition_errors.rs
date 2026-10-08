use std::error::Error;
use std::num::ParseFloatError;

use burn::tensor::Device;
use fabelgeist_mhr::{Mhr, MhrConfig, ModelDefinitionError, ParameterTransform, Skeleton};

fn skeleton() -> Skeleton {
    Skeleton {
        names: vec!["root".into()],
        parents: vec![-1],
        translation_offsets: vec![[0.0; 3]],
        prerotations: vec![[0.0, 0.0, 0.0, 1.0]],
    }
}

fn assignment(body: &str) -> String {
    format!("Momentum Model Definition V1.0\n[ParameterTransform]\n{body}")
}

#[test]
fn distinguishes_missing_and_invalid_headers_with_native_provenance() {
    let result: Result<ParameterTransform, ModelDefinitionError> =
        ParameterTransform::from_model_definition(" \n# only a comment\n", &skeleton());
    let missing = result.unwrap_err();
    assert!(matches!(missing, ModelDefinitionError::MissingHeader));
    assert_eq!(
        missing.to_string(),
        "invalid model definition file; missing the version header"
    );
    assert!(missing.source().is_none());

    let invalid = ParameterTransform::from_model_definition(
        "  wrong\0命名 header # comment removed\n",
        &skeleton(),
    )
    .unwrap_err();
    assert!(matches!(
        &invalid,
        ModelDefinitionError::InvalidHeader { line } if line == "wrong\0命名 header"
    ));
    assert_eq!(
        invalid.to_string(),
        "invalid model definition file; got \"wrong\\0命名 header\""
    );
}

#[test]
fn classifies_targets_before_expression_coefficients() {
    let invalid = ParameterTransform::from_model_definition(
        &assignment("  no_separator = bad * a # comment removed\n"),
        &skeleton(),
    )
    .unwrap_err();
    assert!(matches!(
        &invalid,
        ModelDefinitionError::InvalidTarget { line, target }
            if line == "no_separator = bad * a" && target == "no_separator"
    ));
    assert_eq!(
        invalid.to_string(),
        "unknown joint name in expression: no_separator = bad * a"
    );

    let unknown_joint = ParameterTransform::from_model_definition(
        &assignment(" ghost\0命名 . tx = bad * a\nroot.nope = bad * b"),
        &skeleton(),
    )
    .unwrap_err();
    assert!(matches!(
        &unknown_joint,
        ModelDefinitionError::UnknownJoint { line, joint }
            if line == "ghost\0命名 . tx = bad * a" && joint == "ghost\0命名"
    ));
    assert_eq!(
        unknown_joint.to_string(),
        "unknown joint name in expression: ghost\0命名 . tx = bad * a"
    );
    assert!(unknown_joint.source().is_none());

    let unknown_channel = ParameterTransform::from_model_definition(
        &assignment("root . nope\0命名 = bad * a"),
        &skeleton(),
    )
    .unwrap_err();
    assert!(matches!(
        &unknown_channel,
        ModelDefinitionError::UnknownChannel { line, channel }
            if line == "root . nope\0命名 = bad * a" && channel == "nope\0命名"
    ));
    assert_eq!(
        unknown_channel.to_string(),
        "unknown channel name in expression: root . nope\0命名 = bad * a"
    );
}

#[test]
fn retains_the_coefficient_token_and_native_parse_cause() {
    let error = ParameterTransform::from_model_definition(
        &assignment(" root.tx = 1 * a + nope\0命名 * b # removed\n"),
        &skeleton(),
    )
    .unwrap_err();
    let ModelDefinitionError::InvalidExpressionCoefficient {
        line,
        token,
        source,
    } = &error
    else {
        panic!("wrong rejection: {error:?}");
    };
    assert_eq!(line, "root.tx = 1 * a + nope\0命名 * b");
    assert_eq!(token, "nope\0命名");
    let native = error.source().unwrap().downcast_ref::<ParseFloatError>();
    assert!(std::ptr::eq(native.unwrap(), source));
    assert_eq!(source.to_string(), "invalid float literal");
    assert_eq!(
        error.to_string(),
        "could not parse weight in: root.tx = 1 * a + nope\0命名 * b"
    );
}

#[test]
fn keeps_skips_duplicates_reference_order_and_native_limit_policy() {
    let text = assignment(
        "not an assignment\n\
         root.tx = unparseable + bad * too * many\n\
         root.rx = 1 * a + 0.25 * b\n\
         root.rx = 2 * a\n\
         root.ry = -0.5 * root.rx\n\
         root.tz = -0\n\
         [Ignored]\n\
         missing.channel = bad * coefficient\n\
         [Limits]\n\
         limit a minmax [-2, 2] malformed\n\
         limit b minmax [NaN, 1] -inf\n\
         limit b minmax [2, -2] -0\n",
    );
    let value = ParameterTransform::from_model_definition(&text, &skeleton()).unwrap();
    assert_eq!(value.names, ["a", "b"]);
    assert_eq!(value.row(3), [3.0, 0.25]);
    assert_eq!(value.row(4), [-1.5, -0.125]);
    assert_eq!(value.offsets[2].to_bits(), (-0.0f32).to_bits());
    assert!(value.active_joint_parameters[0]);
    assert_eq!(value.limits.len(), 3);
    assert_eq!(value.limits[0].weight, 1.0);
    assert!(value.limits[1].min.is_nan());
    assert_eq!(value.limits[1].weight, f32::NEG_INFINITY);
    assert_eq!(value.limits[2].min, 2.0);
    assert_eq!(value.limits[2].max, -2.0);
    assert_eq!(value.limits[2].weight.to_bits(), (-0.0f32).to_bits());
}

#[test]
fn public_model_loader_preserves_parser_identity_and_native_source() {
    let bytes = include_bytes!("fixtures/model-error-rig.fbx");
    let device = Device::default();
    let error = Mhr::from_asset_bytes(
        bytes,
        "Momentum Model Definition V1.0\n[ParameterTransform]\nitem1.tx = nope * a",
        None,
        MhrConfig {
            lod: 4,
            pose_correctives: false,
        },
        &device,
    )
    .err()
    .unwrap();
    assert_eq!(error.to_string(), "parsing the MHR model definition");
    let typed = error.downcast_ref::<ModelDefinitionError>().unwrap();
    let through_chain = error
        .chain()
        .find_map(|cause| cause.downcast_ref::<ModelDefinitionError>())
        .unwrap();
    assert!(std::ptr::eq(typed, through_chain));
    assert!(matches!(
        typed,
        ModelDefinitionError::InvalidExpressionCoefficient { line, token, .. }
            if line == "item1.tx = nope * a" && token == "nope"
    ));
    assert!(
        error
            .chain()
            .any(|cause| cause.downcast_ref::<ParseFloatError>().is_some())
    );
}
