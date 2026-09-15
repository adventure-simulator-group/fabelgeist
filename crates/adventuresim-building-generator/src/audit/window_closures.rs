/// Closure-policy checks kept separate from structural opening geometry.
fn window_closure_is_legal(
    opening: &crate::OpeningAssembly,
    archetype: BuildingArchetype,
) -> bool {
    use crate::{ClosureKind, ClosureState};

    if opening.closure.layers == [ClosureKind::TimberShutter] {
        return archetype != BuildingArchetype::Cathedral
            && opening.closure.state == ClosureState::Operable
            && opening.closure.swing_clearance_metres > 0.0;
    }
    matches!(
        opening.closure.layers.as_slice(),
        [ClosureKind::LeadedGlazing] | [ClosureKind::IronBars, ClosureKind::LeadedGlazing]
    ) && (archetype != BuildingArchetype::Cathedral
        || opening.closure.state == ClosureState::Closed)
}
