//! Static gable bays retain their complete structural frame and fixed glass.
use crate::{BuildingPlan, ClosureState, ResolvedItemId, WallSourceId};

pub(super) fn solids(plan: &BuildingPlan) -> Vec<ResolvedItemId> {
    let mut ids = Vec::new();
    for opening in plan
        .opening_assemblies
        .iter()
        .filter(|opening| matches!(opening.host_source, WallSourceId::RoofGable { .. }))
    {
        if opening.closure.state == ClosureState::Closed {
            ids.extend(&opening.closure_solids);
        }
        if let Some(frame) = &plan.timber_frame {
            for bay in frame
                .bays
                .iter()
                .filter(|bay| bay.opening == Some(opening.id))
            {
                ids.extend(
                    frame
                        .members
                        .iter()
                        .filter(|member| bay.member_ids.contains(&member.id))
                        .map(|member| member.solid),
                );
            }
        }
    }
    ids
}
