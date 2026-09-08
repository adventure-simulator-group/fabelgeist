use super::*;
use adventuresim_procedural_textures::{BakeResolution, BakedRecipe};

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
