use super::append_blend_shape_parameters;
use crate::{ParameterSetName, ParameterTransform, Skeleton, parse_model_definition};
fn skeleton() -> Skeleton {
    Skeleton {
        names: vec!["root".into()],
        parents: vec![-1],
        translation_offsets: vec![[0.; 3]],
        prerotations: vec![[0., 0., 0., 1.]],
    }
}
fn snapshot(pt: &ParameterTransform) -> String {
    let mut sets = pt.parameter_sets.iter().collect::<Vec<_>>();
    sets.sort_by(|a, b| a.0.cmp(b.0));
    let queries = ["group", "Group", "β", "x.y", "", "\0", "missing", "a"];
    let lookups = queries
        .iter()
        .map(|key| (*key, pt.parameter_sets.get(&ParameterSetName::from(*key))))
        .collect::<Vec<_>>();
    format!(
        "names={:?};matrix={:?};sets={sets:?};lookups={lookups:?}\n",
        pt.names,
        pt.transform
            .iter()
            .copied()
            .map(f32::to_bits)
            .collect::<Vec<_>>()
    )
}
fn observe() -> String {
    let sections = [
        "parameterset group a a missing\nparameterset Group b\nparameterset β A\nparameterset x.y a b\n",
        "parameterset group a\nparameterset group b\nparameterset missing unknown\nparameterset a\n",
        "wrong group a\nparameterset\nparameterset group\nparameterset β unknown\nparameterset \0 a\n",
        "parameterset group a\n[Other]\nparameterset Group b\n[ParameterSets]\nparameterset group A\n",
        "parameterset group a\nparameterset Group A\nparameterset β b\n",
    ];
    let mut out = String::new();
    for (index, sets) in sections.iter().enumerate() {
        let text = format!(
            "Momentum Model Definition V1.0\n[ParameterTransform]\nroot.tx=1*a + 2*b + 3*A\n[ParameterSets]\n{sets}"
        );
        let mut pt = parse_model_definition(&text, &skeleton()).unwrap();
        out.push_str(&format!("case{index}:{}", snapshot(&pt)));
        for count in [0, 2, 1] {
            append_blend_shape_parameters(&mut pt, count);
            out.push_str(&format!("append{count}:{}", snapshot(&pt)));
        }
    }
    let pt = ParameterTransform {
        parameter_sets: std::collections::HashMap::from([
            ("".into(), vec![true]),
            ("\0".into(), vec![false]),
            ("β".into(), vec![true, false]),
        ]),
        ..Default::default()
    };
    out.push_str(&format!("manual:{}", snapshot(&pt)));
    out
}
#[test]
fn parsed_sets_and_typed_lookup_preserve_actual_main_behavior() {
    assert_eq!(observe(), include_str!("fixtures/sets.txt"));
}
