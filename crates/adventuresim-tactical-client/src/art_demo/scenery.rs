//! Curated views of the existing city and sparse-woodland fixtures.

use super::{camera::OrbitView, exhibits::ExhibitId};
use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_netcode::prelude::SceneVistaBundle;
use bevy::prelude::*;

struct PreparedSceneryInput {
    input: TacticalSceneInput,
    furniture: Option<super::district::PreparedOutdoorFurniture>,
    cloud_asset_digest: String,
}

pub(super) fn spawn(world: &mut World, id: ExhibitId) -> Result<(), String> {
    let PreparedSceneryInput {
        mut input,
        furniture: prepared_furniture,
        cloud_asset_digest,
    } = prepare_input(id)?;
    if id == ExhibitId::City {
        world.insert_resource(crate::presentation::StreamCityTraffic);
    }
    // Building-anchored furniture is prepared alongside the city offline.
    // Keep its accepted instances and reservations instead of regenerating
    // from a scene stripped of the buildings that own those activity groups.
    let parishes = std::mem::take(&mut input.parishes);
    let compounds = std::mem::take(&mut input.compounds);
    let gardens = std::mem::take(&mut input.gardens);
    let distant_buildings = std::mem::take(&mut input.distant_buildings);
    let generated = input.generate().map_err(|error| error.to_string())?;
    let environment = input.environment_snapshot(generated.digest.clone());
    let (asset_scene_digest, rgba8): (&str, &'static [u8]) = if id == ExhibitId::Oak {
        (
            include_str!("../../../../assets/clouds/art-demo/oak.scene-digest").trim(),
            include_bytes!("../../../../assets/clouds/art-demo/oak.rgba8"),
        )
    } else {
        (
            include_str!("../../../../assets/clouds/art-demo/city.scene-digest").trim(),
            include_bytes!("../../../../assets/clouds/art-demo/city.rgba8"),
        )
    };
    if asset_scene_digest != cloud_asset_digest {
        return Err(format!(
            "prebaked cloud asset belongs to scene {asset_scene_digest}, not {cloud_asset_digest}"
        ));
    }
    world.insert_resource(crate::presentation::PrebakedCloudEnvironment { rgba8 });
    let half_extent = Vec2::new(generated.terrain.width(), generated.terrain.depth()) * 0.5;
    if id == ExhibitId::Oak {
        spawn_oak(world, &input, &generated, &environment)?;
    }
    let collider = generated.terrain.collider();
    world.spawn((
        SceneId(input.scene_key.clone()),
        environment,
        generated.ground,
        generated.terrain,
        RigidBody::Static,
        CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL),
        collider,
        Transform::IDENTITY,
    ));
    world.flush();
    let furniture =
        prepared_furniture.unwrap_or_else(|| super::district::PreparedOutdoorFurniture {
            instances: generated
                .furniture
                .instances
                .into_iter()
                .chain(generated.furniture.distant_instances)
                .collect(),
            groups: generated.furniture.groups,
        });
    world.trigger(SceneVistaBundle {
        scene_digest: generated.digest,
        playable_half_extent_metres: half_extent,
        distant_buildings,
        streets: input.streets,
        yards: input.yards,
        parishes,
        compounds,
        gardens,
        furniture_groups: furniture.groups,
        distant_furniture: furniture.instances,
        lods: input.vista.lods,
    });
    if id == ExhibitId::City {
        let mut view = world.resource_mut::<OrbitView>();
        view.pitch = 0.55;
        const STREET_INSPECTION_DISTANCE_METRES: f32 = 1.0;
        const CITY_OVERVIEW_DISTANCE_METRES: f32 = 2500.0;
        view.limits = STREET_INSPECTION_DISTANCE_METRES..=CITY_OVERVIEW_DISTANCE_METRES;
    }
    Ok(())
}

fn prepare_input(id: ExhibitId) -> Result<PreparedSceneryInput, String> {
    let json = if id == ExhibitId::Oak {
        include_str!("../../../../assets/tactical-scenes/sparse-woodland.json")
    } else {
        include_str!("../../../../assets/tactical-scenes/massive-city.json")
    };
    let mut input: TacticalSceneInput =
        serde_json::from_str(json).map_err(|error| error.to_string())?;
    // City curation replaces its layout and later moves vista-only collections
    // out before tactical generation. Clouds depend on the authored fixture's
    // environment, so key their offline bake before those city transformations.
    let cloud_asset_digest = input.digest().map_err(|error| error.to_string())?;
    let prepared_furniture = if id == ExhibitId::City {
        let furniture = super::district::curate(&mut input)?;
        Some(furniture)
    } else {
        None
    };
    Ok(PreparedSceneryInput {
        input,
        furniture: prepared_furniture,
        cloud_asset_digest,
    })
}

fn spawn_oak(
    world: &mut World,
    input: &TacticalSceneInput,
    generated: &GeneratedTacticalScene,
    environment: &SceneEnvironment,
) -> Result<(), String> {
    let terrain = &generated.terrain;
    let position = generated
        .obstacles
        .iter()
        .filter_map(|obstacle| {
            let GeneratedObstacle::Tree { x, z } = obstacle else {
                return None;
            };
            let position = Vec2::new(f32::from(*x), f32::from(*z)) * input.playable.spacing_metres
                - Vec2::new(terrain.width(), terrain.depth()) * 0.5;
            let root = Vec3::new(
                position.x,
                terrain.height_at(position)? + TREE_TRUNK_HEIGHT_METRES * 0.5,
                position.y,
            );
            (crate::presentation::tree_species_for_site(root, environment)
                == crate::presentation::TreePresentationSpecies::EnglishOak)
                .then_some((
                    position,
                    crate::presentation::oak_root_exposure_for_site(root, environment),
                ))
        })
        .max_by(|a, b| {
            a.1.total_cmp(&b.1).then_with(|| {
                root_slope(terrain, a.0)
                    .length_squared()
                    .total_cmp(&root_slope(terrain, b.0).length_squared())
            })
        })
        .map(|(position, _)| position);
    let position = position.ok_or("woodland fixture contains no oak specimen")?;
    {
        let ground = terrain.height_at(position).unwrap_or_default();
        world.spawn((
            SceneObstacle::Tree,
            Transform::from_xyz(
                position.x,
                ground + TREE_TRUNK_HEIGHT_METRES * 0.5,
                position.y,
            ),
        ));
        let mut view = world.resource_mut::<OrbitView>();
        view.focus = Vec3::new(position.x, ground + view.focus.y, position.y);
        view.pitch = 0.06;
        let downhill = root_slope(terrain, position);
        view.yaw = downhill.x.atan2(downhill.y) + std::f32::consts::FRAC_PI_2;
    }
    Ok(())
}

// The local slope ranks exposed sites and orients a lit view across the hill.
fn root_slope(terrain: &SceneTerrain, position: Vec2) -> Vec2 {
    const ROOT_SPREAD_METRES: f32 = 1.5;
    let centre_height = terrain
        .height_at(position)
        .expect("oak lies on its terrain");
    let height = |offset| {
        terrain
            .height_at(position + offset)
            .unwrap_or(centre_height)
    };
    Vec2::new(
        height(-Vec2::X * ROOT_SPREAD_METRES) - height(Vec2::X * ROOT_SPREAD_METRES),
        height(-Vec2::Y * ROOT_SPREAD_METRES) - height(Vec2::Y * ROOT_SPREAD_METRES),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_environment(id: ExhibitId) -> SceneEnvironment {
        let prepared = prepare_input(id).unwrap();
        prepared
            .input
            .environment_snapshot(prepared.cloud_asset_digest)
    }

    #[test]
    fn fixed_exhibits_embed_complete_nonempty_cloud_bakes() {
        for (id, identity, bake) in [
            (
                ExhibitId::Oak,
                include_str!("../../../../assets/clouds/art-demo/oak.scene-digest"),
                include_bytes!("../../../../assets/clouds/art-demo/oak.rgba8").as_slice(),
            ),
            (
                ExhibitId::City,
                include_str!("../../../../assets/clouds/art-demo/city.scene-digest"),
                include_bytes!("../../../../assets/clouds/art-demo/city.rgba8").as_slice(),
            ),
        ] {
            assert_eq!(fixture_environment(id).scene_digest, identity.trim());
            assert_eq!(bake.len(), 1025 * 257 * 4);
            assert!(bake.iter().any(|channel| *channel != 0));
        }
    }

    #[test]
    #[ignore = "offline asset generator performs the full deterministic cloud bakes"]
    fn regenerate_art_demo_cloud_assets() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for (id, output, identity) in [
            (
                ExhibitId::Oak,
                "assets/clouds/art-demo/oak.rgba8",
                "assets/clouds/art-demo/oak.scene-digest",
            ),
            (
                ExhibitId::City,
                "assets/clouds/art-demo/city.rgba8",
                "assets/clouds/art-demo/city.scene-digest",
            ),
        ] {
            let environment = fixture_environment(id);
            let bake = crate::presentation::bake_environment_rgba8(&environment);
            std::fs::write(root.join(output), &bake).unwrap();
            std::fs::write(
                root.join(identity),
                format!("{}\n", environment.scene_digest),
            )
            .unwrap();
            assert_eq!(std::fs::read(root.join(output)).unwrap(), bake);
            println!("wrote {output} for {}", environment.scene_digest);
        }
    }

    #[test]
    fn woodland_exhibit_selects_one_oak_on_nonflat_terrain() {
        let input: TacticalSceneInput = serde_json::from_str(include_str!(
            "../../../../assets/tactical-scenes/sparse-woodland.json"
        ))
        .unwrap();
        let generated = input.generate().unwrap();
        let environment = input.environment_snapshot(generated.digest.clone());
        let mut world = World::new();
        world.init_resource::<OrbitView>();
        spawn_oak(&mut world, &input, &generated, &environment).unwrap();
        let roots = world
            .query_filtered::<&Transform, With<SceneObstacle>>()
            .iter(&world)
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(roots.len(), 1);
        assert_eq!(
            crate::presentation::tree_species_for_site(roots[0].translation, &environment),
            crate::presentation::TreePresentationSpecies::EnglishOak
        );
        assert!(generated.terrain.maximum_height() - generated.terrain.minimum_height() > 1.0);
    }
}
