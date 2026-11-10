//! Catalog membership and compiled profile consistency.
use super::*;

#[test]
fn every_authored_threat_reaches_identity_profiles_and_loot_projection() {
    for authored in crate::quest_catalog::catalog().monsters() {
        let id = authored.id.parse::<ThreatId>().unwrap();
        assert!(catalog_threats().contains(&id));
        let compiled = profile(id);
        assert_eq!(compiled.id, id);
        assert_eq!(compiled.display_name, authored.name);
        assert_eq!(
            compiled.combat.loot_item_id,
            authored.combat.loot_item_id.as_deref()
        );
    }
}

#[test]
fn added_authored_threat_profiles_need_no_named_constant() {
    let mut authored = crate::quest_catalog::catalog()
        .monster("bandit")
        .unwrap()
        .clone();
    authored.id = "authority_added_threat".into();
    authored.name = "Added threat".into();
    authored.combat.loot_item_id = Some("knife".into());
    let id = ThreatId::try_new(&authored.id).unwrap();
    let compiled = compile_profile(id, Box::leak(Box::new(authored)));
    assert_eq!(compiled.id.as_str(), "authority_added_threat");
    assert_eq!(compiled.display_name, "Added threat");
    assert_eq!(compiled.combat.loot_item_id, Some("knife"));
}
