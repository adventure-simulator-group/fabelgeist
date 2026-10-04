//! Replanning refusals retain their precise classification through error chains.

use super::CommitRejection;

impl std::fmt::Display for CommitRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ForgedPlan => "Commit attempt does not match the planned action",
            Self::IdempotencyConflict => "Commit attempt conflicts with an earlier receipt",
            Self::StaleSnapshot => "Action authority changed after planning",
            Self::PrerequisitesChanged => "Action prerequisites changed after planning",
            Self::CalculationChanged => "Action calculation changed after planning",
        })
    }
}

impl std::error::Error for CommitRejection {}
