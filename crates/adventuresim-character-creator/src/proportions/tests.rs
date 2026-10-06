use fabelgeist_mhr::{Skeleton, parse_model_definition};

use super::*;

fn parsed_bounds(proportion: BodyProportion, minimum: f32, maximum: f32) -> ParameterBounds {
    let skeleton = Skeleton {
        names: vec!["root".into()],
        parents: vec![-1],
        translation_offsets: vec![[0.0; 3]],
        prerotations: vec![[0.0, 0.0, 0.0, 1.0]],
    };
    let parameter = proportion.mhr_parameter();
    let text = format!(
        "Momentum Model Definition V1.0\n[ParameterTransform]\n\
         root.tx=1*{parameter}\n[Limits]\n\
         limit {parameter} minmax [{minimum},{maximum}]\n"
    );
    parse_model_definition(&text, &skeleton).unwrap().limits[0].bounds
}

#[test]
fn parsed_intervals_match_every_shared_proportion_contract() {
    for proportion in BodyProportion::ALL {
        let bounds = parsed_bounds(proportion, -proportion.limit(), proportion.limit());
        ensure_shared_proportion_bounds(bounds, proportion).unwrap();
    }
}

#[test]
fn checked_intervals_with_different_endpoints_fail_the_shared_contract() {
    for proportion in BodyProportion::ALL {
        for (minimum, maximum) in [
            (-proportion.limit(), 0.0),
            (0.0, proportion.limit()),
            (f32::NEG_INFINITY, f32::INFINITY),
        ] {
            let bounds = parsed_bounds(proportion, minimum, maximum);
            assert!(ensure_shared_proportion_bounds(bounds, proportion).is_err());
        }
    }
}
