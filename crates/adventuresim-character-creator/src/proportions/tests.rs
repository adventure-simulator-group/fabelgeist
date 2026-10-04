use super::binding::ProportionParameterBinding;
use super::*;
use fabelgeist_fs::FileText;
use fabelgeist_mhr::{ModelParameterName, ParameterTransform, Skeleton};

#[derive(Clone, Copy)]
enum ProportionFixture {
    Valid,
    MissingParameter,
    MissingLimit,
    WrongLimit,
    Rotation,
    Scale,
}
impl ProportionFixture {
    fn transform(self, proportion: BodyProportion) -> ParameterTransform {
        let name = proportion.mhr_parameter();
        let mut source = "Momentum Model Definition V1.0\n[ParameterTransform]\n".to_owned();
        if !matches!(self, Self::MissingParameter) {
            source.push_str(&format!("root.tx = 10 * {name}\nroot.ty = -20 * {name}\nroot.tz = 30 * {name}\nchild.ty = 40 * {name}\n"));
        }
        match self {
            Self::Rotation => source.push_str(&format!("root.rz = 1 * {name}\n")),
            Self::Scale => source.push_str(&format!("root.sc = 1 * {name}\n")),
            _ => (),
        }
        if !matches!(self, Self::MissingLimit | Self::MissingParameter) {
            let minimum = if matches!(self, Self::WrongLimit) {
                0.0
            } else {
                -proportion.limit()
            };
            source.push_str(&format!(
                "[Limits]\nlimit {name} minmax [{minimum}, {}]\n",
                proportion.limit()
            ));
        }
        ParameterTransform::from_definition(
            &FileText::from(source),
            &Skeleton {
                names: vec!["root".into(), "child".into()],
                ..Default::default()
            },
        )
        .unwrap()
    }
}
fn bases() -> Vec<JointProportionBasis> {
    vec![
        JointProportionBasis {
            reference: CharacterProportions::default(),
            translation_metres: [[0.0; 3]; BODY_PROPORTION_COUNT]
        };
        2
    ]
}
#[test]
fn every_proportion_binding_projects_the_same_centimetre_to_metre_translation() {
    for proportion in BodyProportion::ALL {
        let transform = ProportionFixture::Valid.transform(proportion);
        let binding = ProportionParameterBinding::lookup(&transform, proportion)
            .unwrap()
            .admit_basis(&transform)
            .unwrap();
        let mut projected = bases();
        binding.project_into(&transform, &mut projected).unwrap();
        assert_eq!(
            projected[0].translation_metres[proportion.index()],
            [0.1, -0.2, 0.3]
        );
        assert_eq!(
            projected[1].translation_metres[proportion.index()],
            [0.0, 0.4, 0.0]
        );
        for other in BodyProportion::ALL {
            if other != proportion {
                assert_eq!(projected[0].translation_metres[other.index()], [0.0; 3]);
            }
        }
    }
}
#[test]
fn binding_admission_classifies_missing_parameter_missing_limit_and_contract_mismatch() {
    let proportion = BodyProportion::HipWidth;
    let transform = ProportionFixture::MissingParameter.transform(proportion);
    assert!(
        matches!(ProportionParameterBinding::lookup(&transform,proportion),Err(ProportionBasisError::MissingParameter(name)) if name==ModelParameterName::from(proportion.mhr_parameter()))
    );
    for (fixture, missing) in [
        (ProportionFixture::MissingLimit, true),
        (ProportionFixture::WrongLimit, false),
    ] {
        let transform = fixture.transform(proportion);
        let failure = match ProportionParameterBinding::lookup(&transform, proportion)
            .unwrap()
            .admit_basis(&transform)
        {
            Err(error) => error,
            Ok(_) => panic!("bad basis contract admitted"),
        };
        match failure {
            ProportionBasisError::MissingLimit(actual) => {
                assert!(missing);
                assert_eq!(actual, proportion);
            }
            ProportionBasisError::LimitContract(actual) => {
                assert!(!missing);
                assert_eq!(actual, proportion);
            }
            _ => panic!("unexpected contract classification"),
        }
    }
}
#[test]
fn rotation_and_scale_drivers_cannot_enter_a_portable_translation_basis() {
    for fixture in [ProportionFixture::Rotation, ProportionFixture::Scale] {
        let proportion = BodyProportion::UpperArmLength;
        let transform = fixture.transform(proportion);
        let binding = ProportionParameterBinding::lookup(&transform, proportion)
            .unwrap()
            .admit_basis(&transform)
            .unwrap();
        assert!(
            matches!(binding.project_into(&transform,&mut bases()),Err(ProportionBasisError::NonTranslationChannel(actual)) if actual==proportion)
        );
    }
}
