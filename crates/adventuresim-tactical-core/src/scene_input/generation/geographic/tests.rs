use super::*;

#[test]
fn complete_geographic_preparation_preserves_source_identity_assets_and_relief() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/tactical-scenes/garden-review.json");
    let mut input = TacticalSceneInput::load(&path).unwrap();
    for (index, height) in input.playable.heights_metres.iter_mut().enumerate() {
        let x = index % usize::from(input.playable.width);
        *height = x as f32 * input.playable.spacing_metres * 0.1;
    }
    let before = input.clone();
    let prepared = input
        .prepare_geographic_terrain(&mut GeneratedBuildingRecipes::default())
        .unwrap();
    assert_eq!(input, before);
    assert_eq!(prepared.buildings.len(), input.buildings.len());
    assert_eq!(prepared.repairs.levelled_building_samples, 0);
    for (building, placement) in prepared.buildings.iter().zip(&input.buildings) {
        assert_eq!(&building.placement, placement);
    }
    let half = prepared.terrain.width() * 0.5;
    let left = prepared.terrain.height_at(Vec2::new(-half, 0.0)).unwrap();
    let right = prepared.terrain.height_at(Vec2::new(half, 0.0)).unwrap();
    assert!(right - left > prepared.terrain.width() * 0.09);
    let repeat = input
        .prepare_geographic_terrain(&mut GeneratedBuildingRecipes::default())
        .unwrap();
    assert_eq!(prepared.terrain, repeat.terrain);
    assert_eq!(prepared.obstacles, repeat.obstacles);
    assert!(!input.gardens.is_empty());
}
