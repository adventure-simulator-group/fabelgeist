//! Local presentation of the same scene document consumed by the tactical server.
//! These entities carry asset descriptors only; authority arrives through replication.
use super::{RetainedScene, SceneRoot, buildings, protocol::StrategicView};
use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_netcode::prelude::SceneVistaBundle;
use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct SceneDocument {
    location: String,
    input: Result<TacticalSceneInput, String>,
}
impl SceneDocument {
    pub(crate) fn parse(location: String, json: &str) -> Self {
        let input = serde_json::from_str::<TacticalSceneInput>(json)
            .map_err(|error| error.to_string())
            .and_then(|input| {
                input
                    .validate()
                    .map(|()| input)
                    .map_err(|error| error.to_string())
            });
        Self { location, input }
    }
}

pub(super) fn retain_scene(
    mut commands: Commands,
    requested: Option<Res<StrategicView>>,
    document: Option<Res<SceneDocument>>,
    mut scene: ResMut<RetainedScene>,
    materials: Option<Res<crate::presentation::TacticalBuildingMaterials>>,
    textures: Res<adventuresim_procedural_textures::ProceduralTextureResidency>,
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
                if let Err(error) = prepare(&mut commands, input, &view, &mut scene) {
                    scene.error = Some(error);
                }
            }
            Err(error) => scene.error = Some(error.clone()),
        }
    }
    if !scene.pending.is_empty() {
        let building = scene.pending.pop_front().expect("pending building");
        let root = scene.root.expect("prepared scene root");
        super::instances::spawn_building(
            &mut commands,
            building,
            root,
            document.input.as_ref().ok(),
        );
    }
}

fn prepare(
    commands: &mut Commands,
    input: &TacticalSceneInput,
    view: &StrategicView,
    retained: &mut RetainedScene,
) -> Result<(), String> {
    let mut generated = input.generate().map_err(|e| e.to_string())?;
    retained.venues = buildings::prepare_venues(input, view, &mut generated, &mut retained.street)?;
    let root = commands
        .spawn((SceneRoot, Transform::default(), Visibility::Inherited))
        .id();
    retained.root = Some(root);
    if let Some(street) = &retained.street {
        street.spawn_ground(commands, root);
    }
    let half_extent = Vec2::new(generated.terrain.width(), generated.terrain.depth()) * 0.5;
    super::instances::spawn_obstacles(commands, input, &generated, root);
    super::instances::spawn_props(commands, &mut generated, root);
    let mut terrain = commands.spawn((
        SceneId(input.scene_key.clone()),
        input.environment_snapshot(generated.digest.clone()),
        generated.ground,
        generated.terrain,
        Transform::default(),
        ChildOf(root),
    ));
    if let Some(landform) = input.landform {
        terrain.insert(landform);
    }
    commands.insert_resource(crate::presentation::StreamCityTraffic);
    let recipes = std::mem::take(&mut generated.building_recipes);
    commands.queue(move |world: &mut World| {
        world
            .resource_mut::<crate::presentation::TacticalBuildingMeshCache>()
            .recipes = recipes;
    });
    commands.trigger(SceneVistaBundle {
        scene_digest: generated.digest,
        playable_half_extent_metres: half_extent,
        distant_buildings: input
            .distant_buildings
            .iter()
            .filter(|b| !generated.buildings.iter().any(|p| p.placement.id == b.id))
            .copied()
            .collect(),
        establishments: input.establishments.clone(),
        streets: input.streets.clone(),
        yards: input.yards.clone(),
        parishes: input.parishes.clone(),
        compounds: input.compounds.clone(),
        gardens: input.gardens.clone(),
        furniture_groups: generated.furniture.groups,
        distant_furniture: generated.furniture.distant_instances,
        lods: input.vista.lods.clone(),
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
        input.seed = u64::MAX;
        let document = SceneDocument::parse("city".into(), &serde_json::to_string(&input).unwrap());
        assert_eq!(document.input.unwrap().seed, u64::MAX);
        assert!(SceneDocument::parse("city".into(), "{}").input.is_err());
    }
}
