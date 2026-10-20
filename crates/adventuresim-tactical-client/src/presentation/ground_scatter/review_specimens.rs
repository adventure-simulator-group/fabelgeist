use super::*;
use adventuresim_building_generator::spatial_geometry::Position;
use adventuresim_tactical_core::scene_coordinates::Scene;
use bevy::prelude::{Mesh3d, MeshMaterial3d, Name, Visibility};

/// Production geometry/material specimen used only by deterministic capture
/// views. It avoids coupling review availability to whichever species the
/// habitat sampler happened to place in a fixture.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(crate) struct UnderstoryReviewSpecimen {
    pub(crate) common_name: &'static str,
    pub(crate) focus: Position<Scene>,
}

struct PreparedSpecimen {
    marker: UnderstoryReviewSpecimen,
    branches: Handle<Mesh>,
    bark: Handle<StandardMaterial>,
    leaf_cards: Handle<Mesh>,
    leaves: Handle<TacticalTreeLeafCardMaterial>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum UnderstoryReviewError {
    #[error("understory review assets are incomplete for {common_name}")]
    IncompletePresentation { common_name: &'static str },
}

impl PreparedSpecimen {
    fn from_cache(
        common_name: &'static str,
        presentation: &WoodyUnderstoryPresentation,
        focus: Position<Scene>,
    ) -> Result<Self, UnderstoryReviewError> {
        let (Some(branches), Some(bark), Some(leaf_cards), Some(leaves)) = (
            &presentation.branches,
            &presentation.bark,
            &presentation.leaf_cards,
            &presentation.leaves,
        ) else {
            return Err(UnderstoryReviewError::IncompletePresentation { common_name });
        };
        Ok(Self {
            marker: UnderstoryReviewSpecimen { common_name, focus },
            branches: branches.clone(),
            bark: bark.clone(),
            leaf_cards: leaf_cards.clone(),
            leaves: leaves.clone(),
        })
    }
}

pub(crate) fn spawn_understory_review_specimens(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    standard_materials: &mut Assets<StandardMaterial>,
    leaf_materials: &mut Assets<TacticalTreeLeafCardMaterial>,
    cache: &mut WoodyUnderstoryPresentationCache,
    procedural_assets: &ProceduralTextureAssets,
    origin: Position<Scene>,
) -> Result<(), UnderstoryReviewError> {
    ensure_understory_presentations(
        meshes,
        standard_materials,
        leaf_materials,
        cache,
        procedural_assets,
    );
    let prepared = [
        ("common hazel", &cache.hazel),
        ("blackthorn", &cache.blackthorn),
        ("common hawthorn", &cache.hawthorn),
    ]
    .into_iter()
    .map(|(name, presentation)| PreparedSpecimen::from_cache(name, presentation, origin))
    .collect::<Result<Vec<_>, _>>()?;
    for specimen in prepared {
        let marker = specimen.marker;
        let common_name = marker.common_name;
        commands.spawn((
            Name::new(format!("Capture {common_name} production shrub wood")),
            marker,
            Mesh3d(specimen.branches),
            MeshMaterial3d(specimen.bark),
            Visibility::Hidden,
            Transform::from_translation(origin.metres()),
        ));
        commands.spawn((
            Name::new(format!("Capture {common_name} production leaf cards")),
            marker,
            TreeLeafRepresentation::AlphaCard,
            Mesh3d(specimen.leaf_cards),
            MeshMaterial3d(specimen.leaves),
            Visibility::Hidden,
            Transform::from_translation(origin.metres()),
        ));
    }
    Ok(())
}
