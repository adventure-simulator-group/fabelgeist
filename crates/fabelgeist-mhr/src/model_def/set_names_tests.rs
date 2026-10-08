use super::append_blend_shape_parameters;
use crate::{ParameterSetName, ParameterTransform, Skeleton, parse_model_definition};

const NON_ASCII_SET_NAME: &str = "β";
const EMPTY_SET_NAME: &str = "";
const NUL_SET_NAME: &str = "\0";

const THREE_PARAMETER_MODEL_PREFIX: &str = concat!(
    "Momentum Model Definition V1.0\n",
    "[ParameterTransform]\n",
    "root.tx=1*a + 2*b + 3*A\n",
    "[ParameterSets]\n",
);
const EXACT_NAMES_AND_DUPLICATE_MEMBERS: &str = concat!(
    "parameterset group a a missing\n",
    "parameterset Group b\n",
    "parameterset β A\n",
    "parameterset x.y a b\n",
);
const REPLACED_KEYS_AND_UNKNOWN_MEMBERS: &str = concat!(
    "parameterset group a\n",
    "parameterset group b\n",
    "parameterset missing unknown\n",
    "parameterset a\n",
);
const MALFORMED_LINES_AND_UNUSUAL_NAMES: &str = concat!(
    "wrong group a\n",
    "parameterset\n",
    "parameterset group\n",
    "parameterset β unknown\n",
    "parameterset \0 a\n",
);
const REPEATED_PARAMETER_SET_SECTIONS: &str = concat!(
    "parameterset group a\n",
    "[Other]\n",
    "parameterset Group b\n",
    "[ParameterSets]\n",
    "parameterset group A\n",
);
const CASE_SENSITIVE_KEYS_AND_MEMBERS: &str = concat!(
    "parameterset group a\n",
    "parameterset Group A\n",
    "parameterset β b\n",
);
const NO_OP_THEN_REPEATED_BLEND_EXTENSIONS: [usize; 3] = [0, 2, 1];

fn one_joint_skeleton() -> Skeleton {
    Skeleton {
        names: vec!["root".into()],
        parents: vec![-1],
        translation_offsets: vec![[0.; 3]],
        prerotations: vec![[0., 0., 0., 1.]],
    }
}

fn parameter_sets_snapshot(transform: &ParameterTransform) -> String {
    let mut membership_masks = transform.parameter_sets.iter().collect::<Vec<_>>();
    // HashMap iteration order must not affect the expected snapshot.
    membership_masks.sort_by(|a, b| a.0.cmp(b.0));
    let queries = [
        "group",
        "Group",
        NON_ASCII_SET_NAME,
        "x.y",
        EMPTY_SET_NAME,
        NUL_SET_NAME,
        "missing",
        "a",
    ];
    let lookup_results = queries
        .iter()
        .map(|key| {
            (
                *key,
                transform.parameter_sets.get(&ParameterSetName::from(*key)),
            )
        })
        .collect::<Vec<_>>();
    format!(
        "names={:?};matrix={:?};sets={membership_masks:?};lookups={lookup_results:?}\n",
        transform.names,
        // Freeze exact matrix values independently of float display formatting.
        transform
            .transform
            .iter()
            .copied()
            .map(f32::to_bits)
            .collect::<Vec<_>>()
    )
}

fn observe_parsing_lookup_and_extension() -> String {
    let parser_cases = [
        EXACT_NAMES_AND_DUPLICATE_MEMBERS,
        REPLACED_KEYS_AND_UNKNOWN_MEMBERS,
        MALFORMED_LINES_AND_UNUSUAL_NAMES,
        REPEATED_PARAMETER_SET_SECTIONS,
        CASE_SENSITIVE_KEYS_AND_MEMBERS,
    ];
    let mut observations = String::new();
    for (index, parameter_set_lines) in parser_cases.iter().enumerate() {
        let text = format!("{THREE_PARAMETER_MODEL_PREFIX}{parameter_set_lines}");
        let mut transform = parse_model_definition(&text, &one_joint_skeleton()).unwrap();
        observations.push_str(&format!(
            "case{index}:{}",
            parameter_sets_snapshot(&transform)
        ));
        // Existing keys survive each addition; new blend columns are non-members.
        for added_blend_shapes in NO_OP_THEN_REPEATED_BLEND_EXTENSIONS {
            append_blend_shape_parameters(&mut transform, added_blend_shapes);
            observations.push_str(&format!(
                "append{added_blend_shapes}:{}",
                parameter_sets_snapshot(&transform)
            ));
        }
    }
    // Public construction preserves unusual names even outside the file parser.
    let manually_constructed = ParameterTransform {
        parameter_sets: std::collections::HashMap::from([
            (EMPTY_SET_NAME.into(), vec![true]),
            (NUL_SET_NAME.into(), vec![false]),
            (NON_ASCII_SET_NAME.into(), vec![true, false]),
        ]),
        ..Default::default()
    };
    observations.push_str(&format!(
        "manual:{}",
        parameter_sets_snapshot(&manually_constructed)
    ));
    observations
}

#[test]
fn parsing_lookup_and_extension_preserve_parameter_set_identity() {
    assert_eq!(
        observe_parsing_lookup_and_extension(),
        include_str!("fixtures/sets.txt")
    );
}
