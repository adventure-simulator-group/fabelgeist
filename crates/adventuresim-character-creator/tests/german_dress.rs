//! The shipped dress must survive inventory order, replacement and persistence.
use adventuresim_character_creator::{
    CharacterRecipe,
    equipment_catalog::ItemCatalog,
    equipment_layers::LayerPlan,
    inventory::{Article, CatalogArticle},
    item_design::CatalogDesigns,
};

#[test]
fn gown_survives_catalog_replacement_and_persistence() {
    let catalog = ItemCatalog::load(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/items"),
        CatalogDesigns::authored(),
    )
    .unwrap();
    let mut document: serde_json::Value = serde_json::from_str(include_str!(
        "../../../assets_src/characters/german_dress.json"
    ))
    .unwrap();
    document["inventory"].as_array_mut().unwrap().reverse();
    let mut recipe: CharacterRecipe = serde_json::from_value(document).unwrap();
    let loadout = recipe.inventory.loadout(&catalog).unwrap();
    assert!(loadout.clothing.is_empty() && loadout.draped.is_empty());
    assert_eq!(loadout.fitted.len(), 1);
    let placements = loadout
        .fitted
        .iter()
        .map(|p| p.piece.selection.definition())
        .collect::<Vec<_>>();
    let plan = LayerPlan::new(&placements).unwrap();
    let ids = plan
        .order()
        .iter()
        .map(|&i| {
            loadout.fitted[i]
                .piece
                .selection
                .item()
                .definition()
                .id
                .as_str()
        })
        .collect::<Vec<_>>();
    assert_eq!(ids, ["wool_gown"]);
    let outer = *plan.order().last().unwrap();
    let supports = plan
        .supports(outer)
        .map(|i| {
            loadout.fitted[i]
                .piece
                .selection
                .item()
                .definition()
                .id
                .as_str()
        })
        .collect::<Vec<_>>();
    assert!(supports.is_empty());
    let old_gown = loadout
        .fitted
        .iter()
        .find(|p| p.piece.selection.item().definition().id == "wool_gown")
        .unwrap()
        .piece
        .id;

    // A replacement at the same clothing order displaces only that layer.
    let replacement = recipe
        .inventory
        .add(Article::Catalog(CatalogArticle::new("wool_gown", "worn")));
    assert_eq!(
        recipe.inventory.wear(replacement, &catalog).unwrap(),
        [old_gown]
    );
    assert_eq!(recipe.inventory.loadout(&catalog).unwrap().fitted.len(), 1);
    let saved = serde_json::to_vec(&recipe).unwrap();
    let restored: CharacterRecipe = serde_json::from_slice(&saved).unwrap();
    assert_eq!(
        restored.inventory.loadout(&catalog).unwrap().fitted.len(),
        1
    );
}
