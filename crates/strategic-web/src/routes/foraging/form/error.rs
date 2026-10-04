//! Admission failures retain the field role and native numeric decoder cause.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::routes::foraging) enum ForageScalarField {
    Hours,
    ReturnTo,
}

#[derive(Debug, PartialEq, Eq)]
pub(in crate::routes::foraging) enum ForageFormError {
    BodyTooLarge,
    TooManyPairs,
    TooManySources,
    SourceTooLong,
    ReturnHintTooLong,
    DuplicateScalar(ForageScalarField),
    MissingScalar(ForageScalarField),
    Hours(std::num::ParseIntError),
}
impl std::fmt::Display for ForageFormError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BodyTooLarge => formatter.write_str("Forage form exceeds its byte limit"),
            Self::TooManyPairs => formatter.write_str("Forage form contains too many pairs"),
            Self::TooManySources => formatter.write_str("Forage form contains too many sources"),
            Self::SourceTooLong => {
                formatter.write_str("Forage source token exceeds its byte limit")
            }
            Self::ReturnHintTooLong => {
                formatter.write_str("Forage return hint exceeds its byte limit")
            }
            Self::DuplicateScalar(field) => {
                write!(formatter, "Duplicate forage scalar field: {field:?}")
            }
            Self::MissingScalar(field) => {
                write!(formatter, "Missing forage scalar field: {field:?}")
            }
            Self::Hours(source) => write!(formatter, "Invalid submitted forage hours: {source}"),
        }
    }
}
impl std::error::Error for ForageFormError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Hours(source) => Some(source),
            _ => None,
        }
    }
}
