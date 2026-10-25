//! Validate stored action vocabulary and current route structure together.
use super::*;

/// Temporary parsed capability fields admitted against the current route graph.
/// Capability rows remain the authority; each request rebuilds this projection.
pub(super) struct ValidatedActionRoute {
    pub kind: InvestigationActionKind,
    pub target_terrain: Terrain,
}

impl ValidatedActionRoute {
    pub fn from_capability(
        ctx: &ReducerContext,
        capability: &InvestigationActionCapability,
    ) -> Result<Self, String> {
        let kind = capability
            .method
            .parse::<InvestigationActionKind>()
            .map_err(|error| error.to_string())?;
        let target_terrain = capability
            .target_terrain
            .parse::<Terrain>()
            .map_err(|error| error.to_string())?;
        validate_action_route_graph(ctx, capability.owner_character_id, &capability.case_id)?;
        Ok(Self {
            kind,
            target_terrain,
        })
    }
}
