//! Parameter lookup, shared-limit admission, and portable translation projection.
use super::ProportionBasisError;
use adventuresim_core::character_proportions::{BodyProportion, JointProportionBasis};
use fabelgeist_mhr::model_def::{ParameterBounds, ParameterBoundsError, ParameterLimit};
use fabelgeist_mhr::{
    JointParameterChannel, ModelParameterIndex, ModelParameterName, ParameterTransform,
};
use fabelgeist_rig::RigJointOrdinal;
const CENTIMETRES_PER_METRE: f32 = 100.0;

pub(super) struct ProportionParameterBinding {
    pub(super) column: ModelParameterIndex,
    proportion: BodyProportion,
}
impl ProportionParameterBinding {
    pub(super) fn lookup(
        transform: &ParameterTransform,
        proportion: BodyProportion,
    ) -> Result<Self, ProportionBasisError> {
        let name = ModelParameterName::from(proportion.mhr_parameter());
        let column = transform
            .parameter_index(&name)
            .ok_or(ProportionBasisError::MissingParameter(name))?;
        Ok(Self { column, proportion })
    }
    pub(super) fn admit_basis(
        self,
        transform: &ParameterTransform,
    ) -> Result<ProportionBasisBinding, ProportionBasisError> {
        let limits = transform
            .limits()
            .find(|limit: &&ParameterLimit| -> bool { limit.parameter() == self.column })
            .ok_or(ProportionBasisError::MissingLimit(self.proportion))?;
        let expected =
            ParameterBounds::try_from((-self.proportion.limit(), self.proportion.limit()))
                .map_err(|source: ParameterBoundsError| -> ProportionBasisError {
                    ProportionBasisError::SharedLimitBounds {
                        proportion: self.proportion,
                        source,
                    }
                })?;
        if limits.bounds() != expected {
            return Err(ProportionBasisError::LimitContract(self.proportion));
        }
        Ok(ProportionBasisBinding(self))
    }
}
pub(super) struct ProportionBasisBinding(ProportionParameterBinding);
impl ProportionBasisBinding {
    pub(super) fn project_into(
        self,
        transform: &ParameterTransform,
        bases: &mut [JointProportionBasis],
    ) -> Result<(), ProportionBasisError> {
        let ProportionParameterBinding { column, proportion } = self.0;
        for (joint, basis) in bases.iter_mut().enumerate() {
            let joint = RigJointOrdinal::from(joint);
            for (axis, channel) in JointParameterChannel::TRANSLATIONS.into_iter().enumerate() {
                basis.translation_metres[proportion.index()][axis] =
                    transform.row(channel.row(joint))[usize::from(column)] / CENTIMETRES_PER_METRE;
            }
            if !JointParameterChannel::ROTATIONS_AND_SCALE.into_iter().all(
                |channel: JointParameterChannel| -> bool {
                    transform.row(channel.row(joint))[usize::from(column)] == 0.0
                },
            ) {
                return Err(ProportionBasisError::NonTranslationChannel(proportion));
            }
        }
        Ok(())
    }
}
