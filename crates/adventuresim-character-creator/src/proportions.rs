//! Translate the pinned MHR parameter definition into the portable skeleton basis.

use adventuresim_core::character_proportions::{
    BODY_PROPORTION_COUNT, BodyProportion, CharacterProportions, JointProportionBasis,
};
use anyhow::{Context, Result, ensure};
use fabelgeist_mhr::{
    Mhr,
    model_def::{JointChannel, JointParameterRow, ModelParameterName},
};
use fabelgeist_rig::RigJointTable;

const CENTIMETRES_PER_METRE: f32 = 100.0;

pub fn model_parameters(model: &Mhr, proportions: CharacterProportions) -> Result<Vec<f32>> {
    let mut parameters = vec![0.0; model.num_model_parameters()];
    for proportion in BodyProportion::ALL {
        let column = model
            .parameter_transform
            .parameter_index(&ModelParameterName::from(proportion.mhr_parameter()))
            .with_context(|| format!("MHR is missing {}", proportion.mhr_parameter()))?;
        parameters[column.native_index()] = proportions.get(proportion);
    }
    Ok(parameters)
}

pub fn joint_bases(
    model: &Mhr,
    reference: CharacterProportions,
) -> Result<RigJointTable<JointProportionBasis>> {
    let transform = &model.parameter_transform;
    let mut bases = model
        .character
        .skeleton
        .names
        .map(|_| JointProportionBasis {
            reference,
            translation_metres: [[0.0; 3]; BODY_PROPORTION_COUNT],
        });
    for proportion in BodyProportion::ALL {
        let column = transform
            .parameter_index(&ModelParameterName::from(proportion.mhr_parameter()))
            .with_context(|| format!("MHR is missing {}", proportion.mhr_parameter()))?;
        let limits = transform
            .limits
            .iter()
            .find(|limit| limit.parameter == column)
            .context("MHR body proportion has no limits")?;
        ensure!(
            limits.min == -proportion.limit() && limits.max == proportion.limit(),
            "MHR body proportion limits differ from the shared contract"
        );
        for joint in bases.ordinals() {
            let basis = &mut bases[joint];
            for (axis, channel) in JointChannel::TRANSLATIONS.into_iter().enumerate() {
                basis.translation_metres[proportion.index()][axis] = transform
                    .row(JointParameterRow::new(joint, channel))[column]
                    / CENTIMETRES_PER_METRE;
            }
            ensure!(
                JointChannel::ROTATION_AND_SCALE
                    .into_iter()
                    .all(
                        |channel| transform.row(JointParameterRow::new(joint, channel))[column]
                            == 0.0
                    ),
                "skeletal translation basis cannot encode rotation or scale for {}",
                proportion.mhr_parameter()
            );
        }
    }
    Ok(bases)
}
