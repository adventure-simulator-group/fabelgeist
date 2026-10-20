use bevy::prelude::*;

use super::TreeReviewSpecimen;
use crate::presentation::{
    TacticalTreeLeafCardMaterial, TreeLeafRepresentation, UnderstoryReviewSpecimen,
};

/// Specimens have explicit per-view visibility. Production suppression must
/// not queue deferred visibility writes that overwrite that specimen state.
pub(super) type ProductionLeaves = (
    Without<TreeReviewSpecimen>,
    Without<UnderstoryReviewSpecimen>,
    Or<(
        With<TreeLeafRepresentation>,
        With<MeshMaterial3d<TacticalTreeLeafCardMaterial>>,
    )>,
);

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_building_generator::spatial_geometry::Position;

    #[test]
    fn production_visibility_excludes_isolated_review_specimens() {
        let mut world = World::new();
        let production = world.spawn(TreeLeafRepresentation::AlphaCard).id();
        world.spawn((TreeLeafRepresentation::AlphaCard, TreeReviewSpecimen));
        world.spawn((
            TreeLeafRepresentation::AlphaCard,
            UnderstoryReviewSpecimen {
                common_name: "common hazel",
                focus: Position::ORIGIN,
            },
        ));
        let mut query = world.query_filtered::<Entity, ProductionLeaves>();
        assert_eq!(query.iter(&world).collect::<Vec<_>>(), vec![production]);
    }
}
