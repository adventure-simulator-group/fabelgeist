use super::*;
use adventuresim_procedural_textures::{BakeResolution, BakedRecipe};

#[test]
fn crenellation_preview_preserves_alpha_instead_of_using_masonry_red_as_coverage() {
    use adventuresim_procedural_textures::{MapChannel, TextureRecipeId};
    let document = Document::default();
    let bake = BakedRecipe::generate(TextureRecipeId::CrenellationMask, &document.texture);
    let source = bake.map(MapChannel::Opacity).unwrap();
    let display = maps::image(source, true);
    let displayed = display.data.unwrap();
    assert!(displayed.as_chunks::<4>().0.iter().any(|p| p[0] == 255));
    assert!(displayed.as_chunks::<4>().0.iter().any(|p| p[0] == 0));
    let mut world = World::new();
    world.init_resource::<Assets<Image>>();
    world.init_resource::<Assets<StandardMaterial>>();
    let mut assets = SceneAssets::default();
    let handle = maps::material(&mut world, &mut assets, &bake, &document, None);
    let material = world
        .resource::<Assets<StandardMaterial>>()
        .get(&handle)
        .unwrap();
    assert_eq!(
        material.diffuse_transmission, 0.0,
        "a masonry silhouette is opaque"
    );
    let image = world
        .resource::<Assets<Image>>()
        .get(material.base_color_texture.as_ref().unwrap())
        .unwrap();
    assert_eq!(image.data.as_ref().unwrap(), &source.bytes);
}

#[test]
fn replacing_and_pinning_materials_releases_previous_gpu_assets() {
    let mut world = World::new();
    world.init_resource::<Assets<Image>>();
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<StandardMaterial>>();
    world.init_resource::<Assets<adventuresim_procedural_materials::TacticalTreeBarkMaterial>>();
    world
        .init_resource::<Assets<adventuresim_procedural_materials::TacticalTreeLeafCardMaterial>>();
    world.init_resource::<SceneAssets>();
    let mut document = Document::default();
    document.texture.resolution = BakeResolution::Draft;
    let mut studio = Studio::new(document);
    studio.current = Some(BakedRecipe::generate(
        studio.document.recipe,
        &studio.document.texture,
    ));
    world.insert_resource(studio);
    rebuild(&mut world);
    let counts = asset_counts(&world);
    for _ in 0..8 {
        rebuild(&mut world);
        assert_eq!(asset_counts(&world), counts);
    }
    let mut studio = world.resource_mut::<Studio>();
    studio.pinned = studio
        .current
        .clone()
        .map(|bake| (bake, studio.document.clone()));
    rebuild(&mut world);
    assert_eq!(asset_counts(&world), counts.map(|count| count * 2));
    world.resource_mut::<Studio>().pinned = None;
    rebuild(&mut world);
    assert_eq!(asset_counts(&world), counts);
}

fn asset_counts(world: &World) -> [usize; 3] {
    [
        world.resource::<Assets<Image>>().len(),
        world.resource::<Assets<Mesh>>().len(),
        world.resource::<Assets<StandardMaterial>>().len(),
    ]
}

#[test]
fn pinned_crown_strips_do_not_intersect() {
    let mut document = Document::default();
    document.view.shape = crate::document::Shape::CrownStrip;
    document.texture.resolution = BakeResolution::Draft;
    let bake = BakedRecipe::generate(
        adventuresim_procedural_textures::TextureRecipeId::CrenellationMask,
        &document.texture,
    );
    let mesh = geometry::mesh(&bake, &document.view);
    let Some(bevy::mesh::VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions");
    };
    let left_edge = positions.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
    let right_edge = positions
        .iter()
        .map(|p| p[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let offset = geometry::comparison_offset(document.view.shape);
    assert!(
        right_edge - offset < left_edge + offset,
        "pinned strips must have a clear gap"
    );
}
