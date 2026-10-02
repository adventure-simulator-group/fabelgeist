//! Catalog fixtures for the two museum assemblies, using gameplay variation.
// Supporting garments precede dependent equipment in each fixture.
pub(super) const HENRY_ITEMS: &[&str] = &[
    "arming_doublet",
    "padded_chausses",
    "breastplate",
    "burgonet",
    "gorget",
    "tassets",
    "leather_boot",
    "pauldron",
    "rerebrace",
    "couter",
    "vambrace",
    "mitten_gauntlet",
    "poleyn",
];

pub(super) const NUREMBERG_ITEMS: &[&str] = &[
    "arming_doublet",
    "padded_chausses",
    "mail_brayette",
    "breastplate",
    "close_helmet",
    "gorget",
    "fauld",
    "spaulder",
    "rerebrace",
    "couter",
    "vambrace",
    "mitten_gauntlet",
    "cuisse",
    "poleyn",
    "greave",
    "sabaton",
];

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn museum_fixtures_select_distinct_constructions_and_require_morphs() {
        for (name, helmet, shoulder, waist, excluded) in [
            ("museum-henry", "burgonet", "pauldron", "tassets", "greave"),
            (
                "museum-nuremberg",
                "close_helmet",
                "spaulder",
                "fauld",
                "leather_boot",
            ),
        ] {
            let fixture = ArmorHarness::from_str(name, false).unwrap();
            let items = fixture.item_ids().collect::<BTreeSet<_>>();
            for item in [helmet, shoulder, waist, "breastplate", "arming_doublet"] {
                assert!(items.contains(item));
            }
            for item in [excluded, "morion", "cuirass"] {
                assert!(!items.contains(item));
            }
            assert_eq!(items.len(), fixture.item_ids().count());
            let required = fixture.visual_requirements();
            assert_eq!(
                required.morph_targets,
                Some(
                    adventuresim_core::character_morph::IDENTITY_MORPH_COUNT
                        + adventuresim_core::skeletal_fit::SkeletalFitMorph::ALL.len()
                )
            );
            // Helmet-only component names must not block other fixture items.
            assert!(required.names.is_empty());
        }
    }
}
