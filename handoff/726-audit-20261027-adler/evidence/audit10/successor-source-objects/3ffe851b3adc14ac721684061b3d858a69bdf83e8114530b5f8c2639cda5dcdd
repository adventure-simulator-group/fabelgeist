use super::*;

fn selection(item: &str, side: &str) -> LayerSelection {
    LayerSelection::admit(item, side).unwrap()
}

fn key(outfit: Vec<LayerSelection>, item: &str, placement: &str) -> generation::FitKey {
    let plan = OutfitPlan::new(outfit).unwrap();
    generation::FitKey {
        shape: BodyShapeKey::new(&[], None, Default::default(), Default::default()),
        selection: selection(item, placement),
        layers: plan.ancestors(&selection(item, placement)).unwrap(),
    }
}

#[test]
fn fitting_keys_ignore_outfit_order_and_unrelated_limbs() {
    let mut outfit = vec![
        selection("vambrace", "left"),
        selection("mail_sleeve", "left"),
        selection("quilted_sleeve", "left"),
    ];
    let fitted = key(outfit.clone(), "vambrace", "left");
    outfit.reverse();
    outfit.push(selection("mail_sleeve", "right"));
    assert_eq!(fitted, key(outfit.clone(), "vambrace", "left"));
    outfit.retain(|item| item.selection.item().id != "quilted_sleeve");
    assert_ne!(fitted, key(outfit, "vambrace", "left"));
    let mail = fitted
        .support_keys()
        .unwrap()
        .into_iter()
        .find(|key| key.selection.selection.item().id == "mail_sleeve")
        .unwrap();
    assert_eq!(mail.layers, [selection("quilted_sleeve", "left")]);
}

#[test]
fn evicted_fit_rebuilds_innermost_support_and_propagates_failure() {
    let fitted = key(
        vec![
            selection("puffed_sleeve", "left"),
            selection("quilted_sleeve", "left"),
        ],
        "puffed_sleeve",
        "left",
    );
    let mut cache = RuntimeEquipmentBodyCache::default();
    let first = cache.next_fit(&fitted).unwrap();
    assert_eq!(first.selection.selection.item().id, "quilted_sleeve");
    assert!(first.layers.is_empty());
    cache
        .models
        .insert(first, Err("invalid support geometry".into()));
    assert!(
        cache
            .next_fit(&fitted)
            .unwrap_err()
            .to_string()
            .contains("invalid support geometry")
    );
}
