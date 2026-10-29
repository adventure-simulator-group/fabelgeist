use super::*;
use fabelgeist_rig::RigJointOrdinal;

fn skeleton() -> Skeleton {
    Skeleton {
        names: vec!["root".into(), "child".into()].into(),
        parents: vec![None, Some(RigJointOrdinal::from(0usize))].into(),
        translation_offsets: vec![[0.0; 3]; 2].into(),
        prerotations: vec![[0.0, 0.0, 0.0, 1.0]; 2].into(),
    }
}

fn parse(body: &str) -> ParameterTransform {
    let text = format!("Momentum Model Definition V1.0\n[ParameterTransform]\n{body}");
    parse_model_definition(&text, &skeleton()).unwrap()
}

#[test]
fn parses_weights_and_parameter_order() {
    let pt = parse("root.tx = 10.0 * root_tx\nchild.rz = 1.0 * bend + 0.5 * lean\n");
    assert_eq!(
        pt.names.iter().cloned().collect::<Vec<_>>(),
        ["root_tx", "bend", "lean"].map(ModelParameterName::from)
    );
    assert_eq!(
        pt.row(&RigJointName::from("root"), JointChannel::TranslationX)
            .unwrap()
            .as_slice(),
        [10.0, 0.0, 0.0]
    );
    // child.rz is row 1 * 7 + 5.
    assert_eq!(
        pt.row(&RigJointName::from("child"), JointChannel::RotationZ)
            .unwrap()
            .as_slice(),
        [0.0, 1.0, 0.5]
    );
    assert!(pt.active_joint_parameters.row(0usize.into()).unwrap()[JointChannel::TranslationX]);
    assert!(!pt.active_joint_parameters.row(0usize.into()).unwrap()[JointChannel::TranslationY]);
}

#[test]
fn a_joint_reference_copies_scaled_terms() {
    let pt = parse("root.rx = 1.0 * twist + 0.25 * lean\nchild.rx = -0.5 * root.rx\n");
    assert_eq!(
        pt.names.iter().cloned().collect::<Vec<_>>(),
        ["twist", "lean"].map(ModelParameterName::from)
    );
    assert_eq!(
        pt.row(&RigJointName::from("root"), JointChannel::RotationX)
            .unwrap()
            .as_slice(),
        [1.0, 0.25]
    );
    assert_eq!(
        pt.row(&RigJointName::from("child"), JointChannel::RotationX)
            .unwrap()
            .as_slice(),
        [-0.5, -0.125]
    );
}

#[test]
fn duplicate_assignments_accumulate() {
    let pt = parse("root.tx = 1.0 * a\nroot.tx = 2.0 * a\n");
    assert_eq!(
        pt.row(&RigJointName::from("root"), JointChannel::TranslationX)
            .unwrap()
            .as_slice(),
        [3.0]
    );
}

#[test]
fn a_bare_constant_becomes_an_offset() {
    let pt = parse("root.ty = 1.0 * a + 0.75\n");
    assert_eq!(
        pt.offsets.row(0usize.into()).unwrap()[JointChannel::TranslationY],
        0.75
    );
}

#[test]
fn parses_sets_and_limits() {
    let text = "Momentum Model Definition V1.0\n\
         [ParameterTransform]\n\
         root.tx = 1.0 * a\n\
         root.ty = 1.0 * b\n\
         [ParameterSets]\n\
         parameterset rigid a\n\
         [Limits]\n\
         limit b minmax [-0.5, 1.5]\n\
         limit a minmax [-0.25, 0.25] 0.1\n";
    let pt = parse_model_definition(text, &skeleton()).unwrap();
    assert_eq!(
        pt.parameter_sets[&ParameterSetName::from("rigid")]
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [true, false]
    );
    assert_eq!(pt.limits.len(), 2);
    assert_eq!(pt.limits[0].weight, 1.0);
    // The trailing token is a solver weight, not a third bound.
    assert_eq!(pt.limits[1].weight, 0.1);

    let mut parameters = [0.0, 3.0];
    pt.apply_limits(&mut parameters);
    assert_eq!(parameters, [0.0, 1.5]);
}

#[test]
fn blend_shape_columns_are_appended_without_joint_influence() {
    let mut pt = parse("root.tx = 10.0 * root_tx\n");
    append_blend_shape_parameters(&mut pt, NpyDimension::from(2)).unwrap();
    assert_eq!(
        pt.names.iter().cloned().collect::<Vec<_>>(),
        ["root_tx", "blend_0", "blend_1"].map(ModelParameterName::from)
    );
    assert_eq!(
        pt.row(&RigJointName::from("root"), JointChannel::TranslationX)
            .unwrap()
            .as_slice(),
        [10.0, 0.0, 0.0]
    );
}
