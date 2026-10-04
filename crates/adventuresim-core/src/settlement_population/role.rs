//! Population roles with their required generation inputs.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PopulationRole {
    Resident,
    /// A business owner may have no public service ID.
    ServiceProvider {
        profession: String,
        service_id: Option<String>,
    },
}
impl PopulationRole {
    pub(super) const fn is_provider(&self) -> bool {
        matches!(self, Self::ServiceProvider { .. })
    }
}

impl PopulationRole {
    pub(super) fn apply_profession_explanation(
        &self,
        decision: &mut super::RelationDecision,
        local_role: &str,
    ) {
        if let Self::ServiceProvider {
            profession,
            service_id,
        } = self
        {
            decision.decision = profession.clone();
            decision.context = format!(
                "service:{};role:{}",
                service_id.as_deref().unwrap_or("unknown"),
                local_role
            );
        }
    }
}
