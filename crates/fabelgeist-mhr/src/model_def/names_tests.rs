use crate::{ModelParameterName, ParameterTransform, Skeleton, parse_model_definition};
fn skeleton() -> Skeleton {
    Skeleton {
        names: vec!["root".into(), "child".into()],
        parents: vec![-1, 0],
        translation_offsets: vec![[0.; 3]; 2],
        prerotations: vec![[0., 0., 0., 1.]; 2],
    }
}
fn bits(values: &[f32]) -> Vec<u32> {
    values.iter().copied().map(f32::to_bits).collect()
}
fn snapshot(pt: &ParameterTransform) -> String {
    let mut sets = pt.parameter_sets.iter().collect::<Vec<_>>();
    sets.sort_by(|a, b| a.0.cmp(b.0));
    let limits = pt
        .limits
        .iter()
        .map(|l| {
            (
                l.parameter,
                l.min.to_bits(),
                l.max.to_bits(),
                l.weight.to_bits(),
            )
        })
        .collect::<Vec<_>>();
    let names = pt.names.iter().map(|n| n.to_string()).collect::<Vec<_>>();
    let queries = [
        "a",
        "A",
        "β",
        "x.y",
        "blend_0",
        "blend_1",
        "root.tx",
        "",
        "a\0",
        "missing",
        "scale_hip_width",
    ];
    let lookups = queries
        .iter()
        .map(|n| (*n, pt.parameter_index(&ModelParameterName::from(*n))))
        .collect::<Vec<_>>();
    format!(
        "names={names:?};name-debug={:?};count={};rows={};matrix={:?};offsets={:?};active={:?};sets={sets:?};limits={limits:?};lookups={lookups:?}\n",
        pt.names,
        pt.num_parameters(),
        pt.num_joint_parameters,
        bits(&pt.transform),
        bits(&pt.offsets),
        pt.active_joint_parameters
    )
}
fn observe() -> String {
    let cases = [
        "[ParameterTransform]\nroot.tx = 1 * a + 0 * zero\nchild.rz = -2 * β + 0.5 * A\nroot.tx = 2 * a\n",
        "[ParameterTransform]\nroot.rx = 1 * a + 0.25 * b + 0.75\nchild.rx = -0.5 * root.rx\nroot.tx = 1 * a + 2 * root.tx + 3 * b\n",
        "[ParameterTransform]\nroot.ty = 1 * a + 0.75 + -0.25\nroot.tz = bad\nchild.sc = 1 * x.y + 0 * a\n",
        "[ParameterTransform]\nroot.tx = -0 * a + NaN * n + inf * i + -inf * j\n",
        "[ParameterTransform]\nroot.tx=1*a + 1*A + 1*β + 1*a\0\n[ParameterSets]\nparameterset group a missing β\nparameterset group A\nparameterset empty missing\n[Limits]\nlimit a minmax [-0.5, 1.5]\nlimit A minmax [-0.25,0.25] 0.1\nlimit missing minmax [-1,1]\nlimit β minmax [NaN, -1] invalid\nlimit a minmax [3,-2] 4\n",
        "[ParameterTransform]\nroot.tx=1*blend_0\n[ParameterSets]\nparameterset blend blend_0\n",
        "[ParameterTransform]\nnot-assignment\nroot.tx=1*a*b + 2 + nope\n[Unknown]\nignored\n",
        "[ParameterSets]\nparameterset empty missing\n[Limits]\nlimit missing minmax [0,1]\n",
        "[ParameterTransform]\nroot.xx=1*a\n",
        "[ParameterTransform]\nmissing.tx=1*a\n",
        "[ParameterTransform]\nroot.tx=nope*a\n",
        "[ParameterTransform]\nroot=1*a\n",
    ];
    let mut out = String::new();
    for (index, body) in cases.iter().enumerate() {
        let text = format!("Momentum Model Definition V1.0\n{body}");
        match parse_model_definition(&text, &skeleton()) {
            Ok(mut pt) => {
                out.push_str(&format!("case{index}:{}", snapshot(&pt)));
                for count in [0, 2, 1] {
                    pt.append_blend_shape_parameters(count);
                    out.push_str(&format!("append{count}:{}", snapshot(&pt)));
                }
            }
            Err(e) => out.push_str(&format!("case{index}:error={e:#}\n")),
        }
    }
    for text in [
        "",
        "#comment\n",
        "wrong header\n",
        "  Momentum Model Definition V1.0 # header\n[ParameterTransform]\nroot.tx=1*a",
    ] {
        out.push_str(&format!(
            "header:{text:?};{:?}\n",
            parse_model_definition(text, &skeleton())
                .map(|p| snapshot(&p))
                .map_err(|e| format!("{e:#}"))
        ));
    }
    let duplicate = ParameterTransform {
        names: vec![
            "root.tx".into(),
            "a".into(),
            "a".into(),
            "".into(),
            "β".into(),
        ],
        ..Default::default()
    };
    out.push_str(&format!("manual:{}", snapshot(&duplicate)));
    out
}
#[test]
fn parsed_names_and_lookups_match_original_native_behavior() {
    assert_eq!(observe(), include_str!("fixtures/names.txt"));
}
