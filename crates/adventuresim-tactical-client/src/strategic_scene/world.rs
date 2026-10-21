//! Local presentation of the same scene document consumed by the tactical server.
//! These entities carry asset descriptors only; authority arrives through replication.
use super::preparation::{PreparationError, PreparationResult, SharedPreparationResult};
use super::{RetainedScene, SceneRoot, buildings, protocol::StrategicView};
use adventuresim_tactical_core::{prelude::*, scene_input::SceneInputError};
use bevy::prelude::*;
use std::sync::Arc;

#[derive(Resource)]
pub(crate) struct SceneDocument {
    location: String,
    input: SharedPreparationResult<TacticalSceneInput>,
    #[cfg(target_family = "wasm")]
    preparation: crate::presentation::generation::PreparationTicket,
}
impl SceneDocument {
    pub(crate) fn parse(
        location: String,
        json: &str,
        #[cfg(target_family = "wasm")]
        preparation: crate::presentation::generation::PreparationTicket,
    ) -> Self {
        let input = serde_json::from_str::<TacticalSceneInput>(json)
            .map_err(SceneInputError::from)
            .and_then(|input| {
                input.validate()?;
                Ok(input)
            })
            .map_err(PreparationError::from)
            .map_err(Arc::new);
        Self {
            location,
            input,
            #[cfg(target_family = "wasm")]
            preparation,
        }
    }

    fn generated_scene(
        &self,
    ) -> SharedPreparationResult<crate::presentation::PreparedStrategicScene> {
        let input = self.input.as_ref().map_err(Arc::clone)?;
        #[cfg(target_family = "wasm")]
        let generated = crate::presentation::generation::activate(self.preparation, input);
        #[cfg(not(target_family = "wasm"))]
        let generated = crate::presentation::PreparedStrategicScene::generate(input);
        generated.map_err(PreparationError::from).map_err(Arc::new)
    }
}

pub(super) fn retain_scene(
    mut commands: Commands,
    requested: Option<Res<StrategicView>>,
    document: Option<Res<SceneDocument>>,
    mut scene: ResMut<RetainedScene>,
    materials: Option<Res<crate::presentation::TacticalBuildingMaterials>>,
    textures: Res<adventuresim_procedural_textures::ProceduralTextureResidency>,
    settings: Res<crate::presentation::TacticalGraphicsSettings>,
) {
    let (Some(view), Some(document), Some(_)) = (requested, document, materials) else {
        return;
    };
    if document.location != view.location
        || !textures.is_ready(
            adventuresim_procedural_textures::PROCEDURAL_TEXTURE_CATALOGUE
                .iter()
                .map(|recipe| recipe.id),
        )
    {
        return;
    }
    if scene.location == view.location && scene.error.is_some() {
        return;
    }
    if scene.location != view.location || scene.root.is_none() {
        let generated = match document.generated_scene() {
            Ok(generated) => generated,
            Err(error) => {
                scene.error = Some(error);
                return;
            }
        };
        if let Some(root) = scene.root {
            commands.entity(root).despawn();
        }
        commands.queue(crate::presentation::clear_scene_entities);
        *scene = RetainedScene {
            location: view.location.clone(),
            ..default()
        };
        match &document.input {
            Ok(input) => {
                if let Err(error) = prepare(
                    &mut commands,
                    input,
                    &view,
                    &mut scene,
                    generated,
                    settings.config.rendering.vista.maximum_lods,
                ) {
                    scene.error = Some(Arc::new(error));
                }
            }
            Err(error) => scene.error = Some(error.clone()),
        }
    }
    if !scene.pending.is_empty() {
        let (Some(building), Some(root)) = (scene.pending.pop_front(), scene.root) else {
            return;
        };
        if let Err(error) = super::instances::spawn_building(
            &mut commands,
            building,
            root,
            document.input.as_ref().ok(),
        ) {
            scene.error = Some(Arc::new(PreparationError::Presentation(error)));
        }
    }
}

fn prepare(
    commands: &mut Commands,
    input: &TacticalSceneInput,
    view: &StrategicView,
    retained: &mut RetainedScene,
    mut generated: crate::presentation::PreparedStrategicScene,
    maximum_lods: usize,
) -> PreparationResult<()> {
    let backdrop = crate::presentation::StrategicBackdrop::new(input, maximum_lods)?;
    let backdrop = crate::presentation::PreparedBackdrop::new(
        backdrop, input.environment_snapshot(generated.digest.clone()),
    )?;
    retained.venues = buildings::prepare_venues(
        input, view, &mut generated, &mut retained.street, backdrop.backdrop(),
    )?;
    retained.digest = generated.digest.clone();
    let root = commands
        .spawn((SceneRoot, Transform::default(), Visibility::Inherited))
        .id();
    retained.root = Some(root);
    if let Some(street) = &retained.street {
        street.spawn_ground(commands, root);
    }
    super::instances::spawn_props(commands, &mut generated, root);
    backdrop.spawn(commands, root);
    crate::presentation::queue_strategic_city(commands, input, &generated.buildings);
    let recipes = std::mem::take(&mut generated.building_recipes);
    commands.queue(move |world: &mut World| {
        world
            .resource_mut::<crate::presentation::TacticalBuildingMeshCache>()
            .recipes = recipes;
    });
    retained.pending = generated.buildings.into();
    retained.next_place = view.places.len();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_scene_document_preserves_tactical_seed_and_rejects_invalid_input() {
        let text = include_str!("../../../../assets/tactical-scenes/massive-city.json");
        let document = SceneDocument::parse("city".into(), text);
        let mut input = document.input.unwrap();
        input.seed = (u64::MAX).into();
        let document = SceneDocument::parse("city".into(), &serde_json::to_string(&input).unwrap());
        assert_eq!(document.input.unwrap().seed, (u64::MAX).into());
        assert!(SceneDocument::parse("city".into(), "{}").input.is_err());
    }
}
