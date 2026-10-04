use adventuresim_core::character_proportions::BodyProportion;
use fabelgeist_mhr::ModelParameterName;
use fabelgeist_mhr::model_def::ParameterBoundsError;
#[derive(Debug)]
pub enum ProportionBasisError {
    MissingParameter(ModelParameterName),
    MissingLimit(BodyProportion),
    LimitContract(BodyProportion),
    SharedLimitBounds {
        proportion: BodyProportion,
        source: ParameterBoundsError,
    },
    NonTranslationChannel(BodyProportion),
}
impl std::fmt::Display for ProportionBasisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingParameter(name) => write!(f, "MHR is missing parameter {name}"),
            Self::MissingLimit(proportion) => write!(
                f,
                "MHR body proportion {} has no limits",
                proportion.mhr_parameter()
            ),
            Self::LimitContract(proportion) => write!(
                f,
                "MHR body proportion {} limits differ from the shared contract",
                proportion.mhr_parameter()
            ),
            Self::SharedLimitBounds { proportion, source } => write!(
                f,
                "shared body proportion {} has invalid bounds: {source}",
                proportion.mhr_parameter()
            ),
            Self::NonTranslationChannel(proportion) => write!(
                f,
                "skeletal translation basis cannot encode rotation or scale for {}",
                proportion.mhr_parameter()
            ),
        }
    }
}
impl std::error::Error for ProportionBasisError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SharedLimitBounds { source, .. } => Some(source),
            _ => None,
        }
    }
}
