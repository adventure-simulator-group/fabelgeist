use super::*;
use crate::{
    equipment_catalog::authored,
    garment::{ClothLayer, FabricPreset, pattern::shapes},
};

fn catalog_item(inventory: &mut Inventory, item: &str, placement: &str) -> InventoryItemId {
    inventory.add(Article::Catalog(CatalogArticle::new(item, placement)))
}

fn ids<'a>(pieces: &[CatalogPiece<'a>]) -> Vec<&'a str> {
    pieces.iter().map(|p| p.item.id.as_str()).collect()
}

fn worn(inventory: &Inventory) -> Vec<InventoryItemId> {
    inventory.worn().map(|item| item.id).collect()
}

#[test]
fn default_outfit_fits_and_splits_by_generator() {
    let catalog = authored();
    let recipe = crate::CharacterRecipe::default();
    let loadout = recipe.inventory.loadout(&catalog).unwrap();
    assert_eq!(ids(&loadout.clothing), ["linen_tunic", "linen_breeches"]);
    let fitted = loadout.fitted.iter().map(|p| p.piece).collect::<Vec<_>>();
    assert_eq!(ids(&fitted), ["leather_boot", "leather_boot"]);
    assert!(loadout.draped.is_empty());
}

#[test]
fn wearing_replaces_whatever_fills_the_same_layer() {
    let catalog = authored();
    let mut inventory = crate::CharacterRecipe::default().inventory;
    let tunic = inventory.items()[0].id;
    let shirt = inventory.add(Article::Draped(GarmentSelection::default()));
    assert_eq!(inventory.wear(shirt, &catalog).unwrap(), [tunic]);
    assert!(!inventory.get(tunic).unwrap().worn);
    assert!(inventory.loadout(&catalog).is_ok());

    // Mail is a separate layer, so it goes over the cloth shirt.
    let hauberk = inventory.add(Article::Draped(GarmentSelection::chainmail()));
    assert_eq!(inventory.wear(hauberk, &catalog).unwrap(), []);
    let mail_shirt = catalog_item(&mut inventory, "mail_shirt", "worn");
    assert_eq!(inventory.wear(mail_shirt, &catalog).unwrap(), [hauberk]);
}

#[test]
fn articulated_plates_share_a_limb_but_not_a_fit_zone() {
    let catalog = authored();
    let mut inventory = Inventory::default();
    let vambrace = catalog_item(&mut inventory, "vambrace", "left");
    let rerebrace = catalog_item(&mut inventory, "rerebrace", "left");
    let right = catalog_item(&mut inventory, "vambrace", "right");
    for id in [vambrace, rerebrace, right] {
        assert_eq!(inventory.wear(id, &catalog).unwrap(), []);
    }
    let spare = catalog_item(&mut inventory, "vambrace", "left");
    assert_eq!(inventory.wear(spare, &catalog).unwrap(), [vambrace]);
    assert_eq!(worn(&inventory), [rerebrace, right, spare]);
}

#[test]
fn attached_mail_needs_its_support_and_comes_off_with_it() {
    let catalog = authored();
    let mut inventory = Inventory::default();
    let voiders = catalog_item(&mut inventory, "mail_voiders", "worn");
    assert!(matches!(
        inventory.wear(voiders, &catalog),
        Err(EquipConflict::Unsupported { .. })
    ));
    assert!(!inventory.get(voiders).unwrap().worn);

    let doublet = catalog_item(&mut inventory, "arming_doublet", "worn");
    inventory.wear(doublet, &catalog).unwrap();
    inventory.wear(voiders, &catalog).unwrap();
    let graph = inventory.fit(&catalog).unwrap();
    assert_eq!(graph.dependents(doublet).collect::<Vec<_>>(), [voiders]);

    // The doublet's single voider point is taken.
    let spare = catalog_item(&mut inventory, "mail_voiders", "worn");
    assert!(inventory.wear(spare, &catalog).is_err());

    assert_eq!(inventory.take_off(doublet, &catalog), [doublet, voiders]);
    assert!(worn(&inventory).is_empty());
}

#[test]
fn sided_attachments_bind_only_to_their_own_limb() {
    let catalog = authored();
    let mut inventory = Inventory::default();
    let hose = catalog_item(&mut inventory, "padded_chausses", "left");
    inventory.wear(hose, &catalog).unwrap();
    let right_knee = catalog_item(&mut inventory, "mail_knee_voider", "right");
    assert!(inventory.wear(right_knee, &catalog).is_err());
    let left_knee = catalog_item(&mut inventory, "mail_knee_voider", "left");
    inventory.wear(left_knee, &catalog).unwrap();
    assert_eq!(inventory.remove(hose, &catalog), [hose, left_knee]);
    assert!(inventory.get(hose).is_none());
}

#[test]
fn draped_garments_drape_from_the_innermost_layer() {
    let catalog = authored();
    let mut inventory = Inventory::default();
    let mail = inventory.add(Article::Draped(GarmentSelection::chainmail()));
    let trousers = inventory.add(draped(shapes::TROUSERS));
    let coif = inventory.add(Article::Draped(GarmentSelection::chainmail_coif()));
    for id in [mail, trousers, coif] {
        inventory.wear(id, &catalog).unwrap();
    }
    let order = |inventory: &Inventory| {
        inventory
            .loadout(&catalog)
            .unwrap()
            .draped
            .iter()
            .map(|piece| piece.id)
            .collect::<Vec<_>>()
    };
    assert_eq!(order(&inventory), [trousers, mail, coif]);
    // Within one layer, inventory order is draping order.
    inventory.shift(coif, false);
    inventory.shift(coif, false);
    assert_eq!(inventory.items()[0].id, coif);
    assert_eq!(order(&inventory), [trousers, coif, mail]);
}

#[test]
fn own_designs_must_keep_the_catalog_construction() {
    let catalog = authored();
    let mut inventory = Inventory::default();
    let morion = catalog_item(&mut inventory, "morion", "worn");
    inventory.wear(morion, &catalog).unwrap();
    let Article::Catalog(article) = &mut inventory.get_mut(morion).unwrap().article else {
        unreachable!()
    };
    article.design = catalog.design("barbute");
    assert!(inventory.loadout(&catalog).is_err());
    let Article::Catalog(article) = &mut inventory.get_mut(morion).unwrap().article else {
        unreachable!()
    };
    article.design = catalog.design("morion");
    assert!(inventory.loadout(&catalog).is_ok());
}

#[test]
fn inventory_round_trips_and_rejects_repeated_ids() {
    let catalog = authored();
    let mut inventory = crate::CharacterRecipe::default().inventory;
    let coif = inventory.add(Article::Draped(GarmentSelection {
        fabric: FabricPreset::Chainmail,
        ..GarmentSelection::chainmail_coif()
    }));
    inventory.wear(coif, &catalog).unwrap();
    let parsed: Inventory =
        serde_json::from_slice(&serde_json::to_vec(&inventory).unwrap()).unwrap();
    assert_eq!(parsed, inventory);
    assert!(parsed.validate().is_ok());

    let mut repeated = inventory.clone();
    repeated.items.push(repeated.items[0].clone());
    assert!(repeated.validate().is_err());
}

fn draped(shape: shapes::Shape) -> Article {
    Article::Draped(GarmentSelection::from_shape(&shape))
}

#[test]
fn cloth_stacks_from_base_through_padding_and_mail_to_outerwear() {
    use shapes::{GAMBESON, SHIRT, SURCOAT};
    let catalog = authored();
    let mut inventory = Inventory::default();
    let shirt = inventory.add(draped(SHIRT));
    let gambeson = inventory.add(draped(GAMBESON));
    let mail = inventory.add(Article::Draped(GarmentSelection::chainmail()));
    let surcoat = inventory.add(draped(SURCOAT));
    // Worn outermost first: the layer, not the order of dressing, decides.
    for id in [surcoat, mail, gambeson, shirt] {
        assert!(inventory.wear(id, &catalog).unwrap().is_empty());
    }
    let order: Vec<_> = inventory
        .loadout(&catalog)
        .unwrap()
        .draped
        .iter()
        .map(|piece| piece.id)
        .collect();
    assert_eq!(order, [shirt, gambeson, mail, surcoat]);
}

#[test]
fn outer_garments_displace_each_other_but_not_the_layers_beneath() {
    use shapes::{HOUPPELANDE, SURCOAT};
    let catalog = authored();
    let mut inventory = Inventory::default();
    let coif = inventory.add(Article::Draped(GarmentSelection::chainmail_coif()));
    let surcoat = inventory.add(draped(SURCOAT));
    let houppelande = inventory.add(draped(HOUPPELANDE));
    for id in [coif, surcoat] {
        assert!(inventory.wear(id, &catalog).unwrap().is_empty());
    }
    assert_eq!(inventory.wear(houppelande, &catalog).unwrap(), [surcoat]);
    assert_eq!(worn(&inventory), [coif, houppelande]);
}

#[test]
fn a_garment_fills_what_its_pattern_covers_in_its_chosen_layer() {
    use crate::garment::{
        Construction,
        pattern::{Lower, Pattern},
    };
    let catalog = authored();
    let mut inventory = Inventory::default();
    let tunic = inventory.add(draped(shapes::TUNIC));
    let trousers = inventory.add(draped(shapes::TROUSERS));
    // A long tunic has no lower garment, so trousers go under it.
    for id in [tunic, trousers] {
        assert!(inventory.wear(id, &catalog).unwrap().is_empty());
    }
    // Giving the tunic legs makes it compete with the trousers.
    let Article::Draped(garment) = &mut inventory.get_mut(tunic).unwrap().article else {
        unreachable!()
    };
    garment.construction = Construction::Sewn(Pattern {
        lower: Some(Lower::TROUSERS),
        ..shapes::TUNIC.pattern
    });
    assert!(inventory.loadout(&catalog).is_err());
    // Moving it to the outer layer resolves the conflict.
    let Article::Draped(garment) = &mut inventory.get_mut(tunic).unwrap().article else {
        unreachable!()
    };
    garment.layer = ClothLayer::Outerwear;
    assert!(inventory.loadout(&catalog).is_ok());
}
