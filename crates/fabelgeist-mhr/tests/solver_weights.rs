use std::error::Error;
use std::num::ParseFloatError;

use fabelgeist_mhr::character::Skeleton;
use fabelgeist_mhr::model_def::{
    ParameterLimit, ParameterTransform, SolverLimitWeight, SolverLimitWeightAdmissionError,
    parse_model_definition,
};

fn skeleton() -> Skeleton {
    Skeleton {
        names: vec!["root".into()],
        parents: vec![-1],
        translation_offsets: vec![[0.0; 3]],
        prerotations: vec![[0.0, 0.0, 0.0, 1.0]],
    }
}

fn definition(declaration: &str) -> String {
    format!(
        "Momentum Model Definition V1.0\n[ParameterTransform]\nroot.tx = 1 * b\n[Limits]\n{declaration}\n"
    )
}

struct NativeTokenCase {
    token: &'static str,
    expected_bits: u32,
}

const NATIVE_TOKENS: [NativeTokenCase; 26] = [
    NativeTokenCase {
        token: "",
        expected_bits: 0x3f800000,
    },
    NativeTokenCase {
        token: " \t",
        expected_bits: 0x3f800000,
    },
    NativeTokenCase {
        token: "# omitted",
        expected_bits: 0x3f800000,
    },
    NativeTokenCase {
        token: "1",
        expected_bits: 0x3f800000,
    },
    NativeTokenCase {
        token: "-1",
        expected_bits: 0xbf800000,
    },
    NativeTokenCase {
        token: "0",
        expected_bits: 0,
    },
    NativeTokenCase {
        token: "-0",
        expected_bits: 0x80000000,
    },
    NativeTokenCase {
        token: "+0",
        expected_bits: 0,
    },
    NativeTokenCase {
        token: "0.1",
        expected_bits: 0x3dcccccd,
    },
    NativeTokenCase {
        token: "-1e-45",
        expected_bits: 0x80000001,
    },
    NativeTokenCase {
        token: "1e-45",
        expected_bits: 1,
    },
    NativeTokenCase {
        token: "3.4028235e38",
        expected_bits: 0x7f7fffff,
    },
    NativeTokenCase {
        token: "-3.4028235e38",
        expected_bits: 0xff7fffff,
    },
    NativeTokenCase {
        token: "1e39",
        expected_bits: 0x7f800000,
    },
    NativeTokenCase {
        token: "-1e39",
        expected_bits: 0xff800000,
    },
    NativeTokenCase {
        token: "NaN",
        expected_bits: 0x7fc00000,
    },
    NativeTokenCase {
        token: "nan",
        expected_bits: 0x7fc00000,
    },
    NativeTokenCase {
        token: "-NaN",
        expected_bits: 0xffc00000,
    },
    NativeTokenCase {
        token: "+NaN",
        expected_bits: 0x7fc00000,
    },
    NativeTokenCase {
        token: "inf",
        expected_bits: 0x7f800000,
    },
    NativeTokenCase {
        token: "-inf",
        expected_bits: 0xff800000,
    },
    NativeTokenCase {
        token: "+inf",
        expected_bits: 0x7f800000,
    },
    NativeTokenCase {
        token: "infinity",
        expected_bits: 0x7f800000,
    },
    NativeTokenCase {
        token: "Infinity",
        expected_bits: 0x7f800000,
    },
    NativeTokenCase {
        token: "-infinity",
        expected_bits: 0xff800000,
    },
    NativeTokenCase {
        token: "-0 # authored",
        expected_bits: 0x80000000,
    },
];

#[test]
fn accepted_solver_tokens_preserve_native_bits() {
    // Native observations include values that are neither finite nor positive.
    for NativeTokenCase {
        token,
        expected_bits,
    } in NATIVE_TOKENS
    {
        let declaration = format!("limit b minmax [-2, 2] {token}");
        let transform = parse_model_definition(&definition(&declaration), &skeleton()).unwrap();
        let cloned = transform.clone();
        let weight: SolverLimitWeight = cloned.limits[0].weight;
        assert_eq!(f32::from(weight).to_bits(), expected_bits, "{token:?}");
        let mut parameters = [-5.0];
        cloned.apply_limits(&mut parameters);
        assert_eq!(parameters, [-2.0]);
    }
}

#[test]
fn present_malformed_solver_tokens_retain_admission_context() {
    for token in [
        "junk",
        "+",
        "1,0",
        "0x3f800000",
        "1.0 tail",
        "NaN tail",
        "[1]",
        "∞",
    ] {
        let source_line = format!("limit b minmax [-2, 2] {token}");
        let authored = format!("  {source_line}  # author comment");
        let error = parse_model_definition(&definition(&authored), &skeleton()).unwrap_err();
        let admission = error
            .downcast_ref::<SolverLimitWeightAdmissionError>()
            .unwrap();
        assert_eq!(admission.source_line, source_line);
        assert_eq!(admission.parameter_name, "b");
        assert_eq!(admission.rejected_token, token);
        assert!(
            admission
                .source()
                .unwrap()
                .downcast_ref::<ParseFloatError>()
                .is_some()
        );
        assert_eq!(error.chain().count(), 2);
    }
}

#[test]
fn native_construction_and_metadata_preserve_bits_and_clamp_independence() {
    for bits in [
        0, 0x80000000, 0x7fc01234, 0xffc05678, 0x7fa12345, 1, 0x80000001, 0x7f7fffff, 0xff7fffff,
        0x7f800000, 0xff800000,
    ] {
        let native = f32::from_bits(bits);
        let limit = ParameterLimit {
            parameter: 0,
            min: -1.0,
            max: 1.0,
            weight: SolverLimitWeight::from(native),
        };
        let transform = ParameterTransform {
            limits: vec![limit],
            ..Default::default()
        };
        let cloned = transform.clone();
        assert_eq!(f32::from(cloned.limits[0].weight).to_bits(), bits);
        assert_eq!(
            format!("{:?}", cloned.limits[0]),
            format!("ParameterLimit {{ parameter: 0, min: -1.0, max: 1.0, weight: {native:?} }}"),
        );
        for input in [-3.0, -0.0, 0.25, 3.0] {
            let mut parameters = [input];
            cloned.apply_limits(&mut parameters);
            assert_eq!(parameters[0].to_bits(), input.clamp(-1.0, 1.0).to_bits());
        }
    }
}

#[test]
fn unrelated_limit_grammar_and_bounds_admission_remain_unchanged() {
    for declaration in [
        "limit unknown minmax [-1, 1] junk",
        "limit b linear [-1, 1] junk",
        "limit b minmax [junk, junk] junk",
        "limit b minmax [-1, 1, 2] junk",
        "limit b minmax no_bounds junk",
    ] {
        let transform = parse_model_definition(&definition(declaration), &skeleton()).unwrap();
        assert!(transform.limits.is_empty(), "{declaration}");
    }
    let filtered = parse_model_definition(
        &definition("limit b minmax [-1, ignored, 1] -0"),
        &skeleton(),
    )
    .unwrap();
    assert_eq!(filtered.limits[0].min, -1.0);
    assert_eq!(filtered.limits[0].max, 1.0);
    assert_eq!(f32::from(filtered.limits[0].weight).to_bits(), 0x80000000);

    // Interval admission is a separate responsibility. Do not clamp these.
    let nan =
        parse_model_definition(&definition("limit b minmax [NaN, 1] 1"), &skeleton()).unwrap();
    assert!(nan.limits[0].min.is_nan());
    let reversed =
        parse_model_definition(&definition("limit b minmax [2, 1] 1"), &skeleton()).unwrap();
    assert_eq!(reversed.limits[0].min, 2.0);
    assert_eq!(reversed.limits[0].max, 1.0);
}
