//! Translate the pinned MHR parameter definition into the portable skeleton basis.

use adventuresim_core::character_proportions::{
    BODY_PROPORTION_COUNT, BodyProportion, CharacterProportions, JointProportionBasis,
};
mod binding;
mod error;
#[cfg(test)]
mod tests;
use binding::ProportionParameterBinding;
pub use error::ProportionBasisError;
use fabelgeist_mhr::Mhr;

pub fn model_parameters(
    model: &Mhr,
    proportions: CharacterProportions,
) -> Result<Vec<f32>, ProportionBasisError> {
    let mut parameters = vec![0.0; usize::from(model.num_model_parameters())];
    for proportion in BodyProportion::ALL {
        let binding = ProportionParameterBinding::lookup(model.parameter_transform(), proportion)?;
        parameters[usize::from(binding.column)] = proportions.get(proportion);
    }
    Ok(parameters)
}

pub fn joint_bases(
    model: &Mhr,
    reference: CharacterProportions,
) -> Result<Vec<JointProportionBasis>, ProportionBasisError> {
    let transform = model.parameter_transform();
    let mut bases = vec![
        JointProportionBasis {
            reference,
            translation_metres: [[0.0; 3]; BODY_PROPORTION_COUNT],
        };
        model.num_joints()
    ];
    for proportion in BodyProportion::ALL {
        ProportionParameterBinding::lookup(transform, proportion)?
            .admit_basis(transform)?
            .project_into(transform, &mut bases)?;
    }
    Ok(bases)
}
