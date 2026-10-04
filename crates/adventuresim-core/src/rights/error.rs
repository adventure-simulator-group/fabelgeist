//! Rights-question refusals remain inspectable through downstream error chains.

use super::RightsQuestionError;

impl std::fmt::Display for RightsQuestionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ResourceJurisdictionMismatch => "Rights resource and jurisdiction do not match",
            Self::SelfContainment => "Rights question cannot place an object inside itself",
        })
    }
}

impl std::error::Error for RightsQuestionError {}
