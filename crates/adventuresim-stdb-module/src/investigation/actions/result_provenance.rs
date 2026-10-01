//! Validate captured generation provenance before publishing an action lead.
use super::*;
use adventuresim_core::quest_generation::GeneratedActionOutput;

/// Temporary validated provenance from immutable manifest and output records.
/// Rebuilt for each publication; current action results own their own chronology.
pub(super) struct ActionResultProvenance {
    pub public_case_id: String,
    pub kind: InvestigationActionKind,
    pub generated_outputs: Option<Vec<GeneratedActionOutput>>,
}

impl ActionResultProvenance {
    pub fn from_capability(
        ctx: &ReducerContext,
        capability: &InvestigationActionCapability,
    ) -> Result<Self, String> {
        let public_case_id = generated_authority_reducer(ctx, capability)
            .map_err(|()| "Generated action authority is invalid")?
            .map(|(manifest, _)| {
                serde_json::from_str::<adventuresim_core::quest_generation::GeneratedCase>(
                    &manifest,
                )
                .map(|generated| generated.public_case_id)
                .map_err(|_| "Validated generated manifest became invalid")
            })
            .transpose()?
            .unwrap_or_else(|| capability.case_id.clone());
        let kind = capability
            .method
            .parse::<InvestigationActionKind>()
            .map_err(|error| error.to_string())?;
        let generated_outputs = ctx
            .db
            .investigation_generated_action_output()
            .capability_id()
            .find(&capability.id)
            .map(|row| {
                serde_json::from_str::<
                    Vec<adventuresim_core::quest_generation::GeneratedActionOutput>,
                >(&row.outputs_json)
                .map_err(|_| "Generated action output authority is invalid")
            })
            .transpose()?;
        Ok(Self {
            public_case_id,
            kind,
            generated_outputs,
        })
    }
}
