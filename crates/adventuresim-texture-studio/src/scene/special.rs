//! Use the production bark and front/back leaf shaders, including their packed maps.
use super::*;
use adventuresim_procedural_materials::{
    TacticalTreeBarkExtension, TacticalTreeBarkMaterial, TacticalTreeLeafCardMaterial,
};
use adventuresim_procedural_textures::{BakedRecipe, MapChannel, TextureRecipeId};

pub(super) fn spawn(
    world: &mut World,
    assets: &mut SceneAssets,
    bake: &BakedRecipe,
    document: &Document,
    mesh: &Handle<Mesh>,
    x: f32,
) -> bool {
    if bake.map(MapChannel::FrontAlbedo).is_some() {
        let material = leaf(world, assets, bake, document);
        assets.leaves.push(material.clone());
        assets.entities.push(
            world
                .spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_xyz(x, 0.0, 0.0),
                ))
                .id(),
        );
        return true;
    }
    if bake.recipe == TextureRecipeId::OakBark && document.view.displacement == 0.0 {
        let material = bark(world, assets, bake, document);
        assets.bark.push(material.clone());
        assets.entities.push(
            world
                .spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_xyz(x, 0.0, 0.0),
                ))
                .id(),
        );
        return true;
    }
    false
}

fn leaf(
    world: &mut World,
    assets: &mut SceneAssets,
    bake: &BakedRecipe,
    document: &Document,
) -> Handle<TacticalTreeLeafCardMaterial> {
    let mut map = |channel| {
        maps::upload(
            world,
            assets,
            bake.map(channel).expect("leaf contract"),
            false,
        )
    };
    let material = TacticalTreeLeafCardMaterial {
        opacity: map(MapChannel::Opacity),
        front_albedo: map(MapChannel::FrontAlbedo),
        back_albedo: map(MapChannel::BackAlbedo),
        front_normal: map(MapChannel::FrontNormal),
        back_normal: map(MapChannel::BackNormal),
        arm: map(MapChannel::Arm),
        parameters: Vec4::new(1.0, 0.0, 0.0, 0.0),
        surface_parameters: Vec4::new(
            0.5,
            document.surface.normal_strength,
            document.surface.ao_strength,
            document.surface.leaf_transmission,
        ),
        physical_parameters: Vec4::new(
            document.surface.roughness,
            document.surface.leaf_thickness_metres,
            0.0,
            0.0,
        ),
    };
    world
        .resource_mut::<Assets<TacticalTreeLeafCardMaterial>>()
        .add(material)
}

fn bark(
    world: &mut World,
    assets: &mut SceneAssets,
    bake: &BakedRecipe,
    document: &Document,
) -> Handle<TacticalTreeBarkMaterial> {
    let map = maps::upload(
        world,
        assets,
        bake.map(MapChannel::HeightAo).unwrap(),
        false,
    );
    let pigment = Color::srgb_from_array(document.surface.pigment)
        .to_linear()
        .to_vec4()
        .xyz();
    let preview_metres = 1.5 / document.view.repeats;
    let azimuth = document.environment.azimuth_degrees.to_radians();
    let incidence = document.environment.incidence_degrees.to_radians();
    let lighting = Vec3::new(
        azimuth.sin() * incidence.cos(),
        azimuth.cos() * incidence.cos(),
        incidence.sin(),
    );
    let material = TacticalTreeBarkMaterial {
        base: StandardMaterial {
            base_color: Color::srgb_from_array(document.surface.pigment),
            perceptual_roughness: 0.84 * document.surface.roughness,
            cull_mode: None,
            ..default()
        },
        extension: TacticalTreeBarkExtension {
            height_ao: map.clone(),
            relief: Vec4::new(
                1.0 / preview_metres,
                bake.height_range_metres * preview_metres / bake.tile_metres,
                document.surface.normal_strength,
                document.surface.ao_strength,
            ),
            uv_offset: Vec4::new(document.view.offset[0], document.view.offset[1], 0.0, 0.0),
            projection: Vec4::new(
                document.surface.bark_projection_sharpness,
                document.surface.bark_branch_alignment,
                document.surface.bark_parallax,
                document.surface.bark_fade_metres,
            ),
            lighting: lighting.extend(1.0),
            surface: pigment.extend(0.84 * document.surface.roughness),
            soil_surface: pigment.extend(0.84),
            deposition: Vec4::new(-100.0, -99.0, 0.045, 0.007),
            terrain_surface: Vec4::new(1.0, 1.0, -100.0, -99.0),
            soil_response: Vec4::new(1.0, 0.0, 0.0, 0.0),
            soil_optics: Vec4::new(0.35, 0.0, 0.0, 0.0),
            terrain_heightmap: map.clone(),
            soil_height_ao: map,
        },
    };
    world
        .resource_mut::<Assets<TacticalTreeBarkMaterial>>()
        .add(material)
}
