use super::*;
use fabelgeist_rig::RigJointName;

fn skeleton() -> Skeleton {
    Skeleton {
        names: vec!["root".into(), "child".into()],
        parents: vec![-1, 0],
        translation_offsets: vec![[0.0; 3]; 2],
        prerotations: vec![[0.0, 0.0, 0.0, 1.0]; 2],
    }
}

struct DefinitionFixture(FileText);
impl From<&str> for DefinitionFixture {
    fn from(body: &str) -> Self {
        Self(FileText::from(format!(
            "Momentum Model Definition V1.0\n[ParameterTransform]\n{body}"
        )))
    }
}
impl DefinitionFixture {
    fn admit(self) -> Result<ParameterTransform, ModelDefinitionError> {
        ParameterTransform::from_definition(&self.0, &skeleton())
    }
    fn parse(self) -> ParameterTransform {
        self.admit().unwrap()
    }
}

#[test]
fn parses_weights_and_parameter_order() {
    let pt =
        DefinitionFixture::from("root.tx = 10.0 * root_tx\nchild.rz = 1.0 * bend + 0.5 * lean\n")
            .parse();
    assert_eq!(
        pt.names,
        [
            ModelParameterName::from("root_tx"),
            ModelParameterName::from("bend"),
            ModelParameterName::from("lean")
        ]
    );
    assert_eq!(pt.row(JointParameterRow::from(0)), [10.0, 0.0, 0.0]);
    // child.rz is row 1 * 7 + 5.
    assert_eq!(pt.row(JointParameterRow::from(12)), [0.0, 1.0, 0.5]);
    assert!(pt.active_joint_parameters[0]);
    assert!(!pt.active_joint_parameters[1]);
}

#[test]
fn a_joint_reference_copies_scaled_terms() {
    let pt =
        DefinitionFixture::from("root.rx = 1.0 * twist + 0.25 * lean\nchild.rx = -0.5 * root.rx\n")
            .parse();
    assert_eq!(
        pt.names,
        [
            ModelParameterName::from("twist"),
            ModelParameterName::from("lean")
        ]
    );
    assert_eq!(pt.row(JointParameterRow::from(3)), [1.0, 0.25]);
    assert_eq!(pt.row(JointParameterRow::from(10)), [-0.5, -0.125]);
}

#[test]
fn duplicate_assignments_accumulate() {
    let pt = DefinitionFixture::from("root.tx = 1.0 * a\nroot.tx = 2.0 * a\n").parse();
    assert_eq!(pt.row(JointParameterRow::from(0)), [3.0]);
}

#[test]
fn a_bare_constant_becomes_an_offset() {
    let pt = DefinitionFixture::from("root.ty = 1.0 * a + 0.75\n").parse();
    assert_eq!(pt.offsets[1], 0.75);
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
    let pt =
        ParameterTransform::from_definition(&FileText::from(text.to_owned()), &skeleton()).unwrap();
    assert_eq!(
        pt.parameter_sets[&ParameterSetName::from("rigid")],
        [true, false]
    );
    assert_eq!(pt.limits.len(), 2);
    assert_eq!(pt.limits[0].weight(), SolverLimitWeight::from(1.0));
    // The trailing token is a solver weight, not a third bound.
    assert_eq!(pt.limits[1].weight(), SolverLimitWeight::from(0.1));

    let mut parameters = [0.0, 3.0];
    pt.apply_limits(&mut parameters);
    assert_eq!(parameters, [0.0, 1.5]);
}

#[test]
fn blend_shape_columns_are_appended_without_joint_influence() {
    let mut pt = DefinitionFixture::from("root.tx = 10.0 * root_tx\n").parse();
    let pose_columns = crate::model::PoseParameterCount::from(pt.num_parameters());
    assert_eq!(pt.num_parameters(), ModelParameterCount::from(1));
    pt.append_blend_shapes(BlendShapeParameterCount::from(2))
        .unwrap();
    assert_eq!(pt.num_parameters(), ModelParameterCount::from(3));
    assert_eq!(pose_columns, crate::model::PoseParameterCount::from(1));
    assert_eq!(
        pt.names,
        [
            ModelParameterName::from("root_tx"),
            ModelParameterName::from("blend_0"),
            ModelParameterName::from("blend_1")
        ]
    );
    assert_eq!(pt.row(JointParameterRow::from(0)), [10.0, 0.0, 0.0]);
}
#[test]
fn bad_headers_and_targets_retain_their_exact_source_line_and_identity() {
    let empty = FileText::from("# only comments\n\n".to_owned());
    assert!(matches!(
        ParameterTransform::from_definition(&empty, &skeleton()),
        Err(ModelDefinitionError::MissingHeader)
    ));
    let invalid = FileText::from("# comment\n\nFuture model version\n".to_owned());
    match ParameterTransform::from_definition(&invalid, &skeleton()).unwrap_err() {
        ModelDefinitionError::InvalidHeader(line) => {
            assert_eq!(line, DefinitionLine::from((2, "Future model version")))
        }
        _ => panic!("expected bad header"),
    }
    assert!(matches!(
        DefinitionFixture::from("root = 1 * a").admit(),
        Err(ModelDefinitionError::InvalidTarget(_))
    ));
    match DefinitionFixture::from("absent.tx = 1 * a")
        .admit()
        .unwrap_err()
    {
        ModelDefinitionError::UnknownJoint { line, joint } => {
            assert_eq!(line, DefinitionLine::from((2, "absent.tx = 1 * a")));
            assert_eq!(joint, RigJointName::from("absent"));
        }
        _ => panic!("expected joint identity"),
    }
    assert!(matches!(
        DefinitionFixture::from("root.zz = 1 * a").admit(),
        Err(ModelDefinitionError::UnknownChannel { .. })
    ));
}
#[test]
fn numeric_errors_keep_the_weight_role_source_token_and_parse_cause() {
    let failure = DefinitionFixture::from("root.tx = bad * a")
        .admit()
        .unwrap_err();
    assert!(
        std::error::Error::source(&failure)
            .unwrap()
            .is::<std::num::ParseFloatError>()
    );
    match failure {
        ModelDefinitionError::Number {
            line, role, token, ..
        } => {
            assert_eq!(role, DefinitionNumberRole::ExpressionWeight);
            assert_eq!(line, DefinitionLine::from((2, "root.tx = bad * a")));
            assert_eq!(
                token,
                lexical::RejectedDefinitionToken::from(lexical::DefinitionToken::from("bad"))
            );
        }
        _ => panic!("expected numeric weight failure"),
    }
    for (limits, expected_role) in [
        ("limit a minmax [bad, 1]", DefinitionNumberRole::Minimum),
        ("limit a minmax [-1, bad]", DefinitionNumberRole::Maximum),
        (
            "limit a minmax [-1, 1] bad",
            DefinitionNumberRole::SolverWeight,
        ),
    ] {
        let fixture =
            DefinitionFixture::from(format!("root.tx = 1 * a\n[Limits]\n{limits}").as_str());
        assert!(
            matches!(fixture.admit(),Err(ModelDefinitionError::Number { role,.. }) if role==expected_role)
        );
    }
}
#[test]
fn invalid_minmax_records_fail_admission_before_clamping_can_panic() {
    for (bounds, expected) in [
        ("[1, -1]", ParameterBoundsError::Unordered),
        ("[NaN, 1]", ParameterBoundsError::NotNumber),
        ("[-1, NaN]", ParameterBoundsError::NotNumber),
    ] {
        let fixture = DefinitionFixture::from(
            format!("root.tx = 1 * a\n[Limits]\nlimit a minmax {bounds}").as_str(),
        );
        let failure = fixture.admit().unwrap_err();
        assert!(
            std::error::Error::source(&failure)
                .unwrap()
                .is::<ParameterBoundsError>()
        );
        assert!(
            matches!(failure,ModelDefinitionError::LimitBounds { source,.. } if source==expected)
        );
    }
    for bounds in ["", "] [-1, 1", "[-1]", "[-1, 1, 2]", "missing"] {
        let fixture = DefinitionFixture::from(
            format!("root.tx = 1 * a\n[Limits]\nlimit a minmax {bounds}").as_str(),
        );
        assert!(
            matches!(fixture.admit(), Err(ModelDefinitionError::LimitSyntax(_))),
            "bounds {bounds}"
        );
    }
    // Infinite endpoints remain valid f32 clamping bounds.
    let parsed =
        DefinitionFixture::from("root.tx = 1 * a\n[Limits]\nlimit a minmax [-inf, inf]").parse();
    let mut parameters = [1.5f32];
    parsed.apply_limits(&mut parameters);
    assert_eq!(parameters, [1.5]);
}
#[test]
fn repeated_sections_comments_and_first_duplicate_joint_match_keep_order() {
    let fixture = DefinitionFixture::from(
        "# ignored\n[ParameterSets]\nparameterset selected b\n[ParameterTransform]\nroot.tx = 1 * a # note\n[Other]\nignored.content = bad * other\n[ParameterTransform]\nchild.ty = 2 * b",
    );
    let transform = fixture.parse();
    assert_eq!(
        transform.names,
        [ModelParameterName::from("a"), ModelParameterName::from("b")]
    );
    assert_eq!(
        transform.parameter_sets[&ParameterSetName::from("selected")],
        [false, true]
    );
    assert_eq!(transform.row(JointParameterRow::from(8)), [0.0, 2.0]);
    let mut duplicate = skeleton();
    duplicate.names[1] = RigJointName::ROOT;
    let source = DefinitionFixture::from("root.tx = 3 * a");
    let transform = ParameterTransform::from_definition(&source.0, &duplicate).unwrap();
    assert_eq!(transform.row(JointParameterRow::from(0)), [3.0]);
    assert_eq!(transform.row(JointParameterRow::from(7)), [0.0]);
}
#[test]
fn self_reference_copies_a_snapshot_without_offsets_or_reordering_weights() {
    let transform=DefinitionFixture::from("root.rx = 1 * a + 0.75\nroot.rx = 2 * root.rx\nchild.rx = -0.5 * root.rx\nroot.ty = 0 * unused").parse();
    assert_eq!(
        transform.names,
        [
            ModelParameterName::from("a"),
            ModelParameterName::from("unused")
        ]
    );
    assert_eq!(transform.row(JointParameterRow::from(3)), [3.0, 0.0]);
    assert_eq!(transform.row(JointParameterRow::from(10)), [-1.5, 0.0]);
    assert_eq!(transform.offsets[3], 0.75);
    assert_eq!(transform.offsets[10], 0.0);
    assert!(transform.active_joint_parameters[1]);
}
#[test]
fn unsupported_terms_limits_and_rhs_whitespace_keep_the_documented_subset() {
    let transform=DefinitionFixture::from("root . tx = nonnumeric + 1 * root .rx\nroot.rx = 2 * a\n[Limits]\nlimit absent minmax [bad, worse]\nlimit a linear ignored\n[ParameterSets]\nparameterset known a absent").parse();
    assert_eq!(
        transform.names,
        [
            ModelParameterName::from("root .rx"),
            ModelParameterName::from("a")
        ]
    );
    assert_eq!(transform.row(JointParameterRow::from(0)), [1.0, 0.0]);
    assert!(transform.limits.is_empty());
    assert_eq!(
        transform.parameter_sets[&ParameterSetName::from("known")],
        [false, true]
    );
}
#[test]
fn failed_blend_growth_preserves_names_matrix_and_parameter_sets() {
    let mut transform =
        DefinitionFixture::from("root.tx = 2 * a\n[ParameterSets]\nparameterset selected a")
            .parse();
    let before = transform.clone();
    assert!(matches!(
        transform.append_blend_shapes(BlendShapeParameterCount::from(usize::MAX)),
        Err(ModelDefinitionError::LayoutOverflow(
            DefinitionLayout::ParameterColumns
        ))
    ));
    assert_eq!(transform.names, before.names);
    assert_eq!(transform.transform, before.transform);
    assert_eq!(transform.parameter_sets, before.parameter_sets);
    transform
        .append_blend_shapes(BlendShapeParameterCount::from(2))
        .unwrap();
    assert_eq!(
        transform.parameter_sets[&ParameterSetName::from("selected")],
        [true, false, false]
    );
    assert_eq!(transform.row(JointParameterRow::from(0)), [2.0, 0.0, 0.0]);
}
