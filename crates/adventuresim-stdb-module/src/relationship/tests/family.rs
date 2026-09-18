use crate::relationship::seeded_household_plan;
use crate::{HouseholdRole, KinshipKind};

#[test]
fn seeded_family_contract_has_unique_roles_and_canonical_edges() {
    let plan = seeded_household_plan("test-settlement", &[10, 11, 12, 13])
        .unwrap()
        .unwrap();
    assert_eq!(plan.household_id, "household:seeded:test-settlement:10");
    assert_eq!(plan.family_key, "seeded:test-settlement:10");
    assert_eq!(
        plan.members,
        vec![
            (10, HouseholdRole::Head),
            (11, HouseholdRole::Spouse),
            (12, HouseholdRole::AdultChild),
            (13, HouseholdRole::AdultChild),
        ]
    );
    assert_eq!(plan.kinships.len(), 10);
    for child in [12, 13] {
        for parent in [10, 11] {
            assert!(plan.kinships.contains(&(child, parent, KinshipKind::Parent)));
            assert!(plan.kinships.contains(&(parent, child, KinshipKind::Child)));
        }
    }
    assert!(plan.kinships.contains(&(12, 13, KinshipKind::Sibling)));
    assert!(plan.kinships.contains(&(13, 12, KinshipKind::Sibling)));

    let incomplete = seeded_household_plan("test-settlement", &[20, 21])
        .unwrap()
        .unwrap();
    assert!(incomplete.kinships.is_empty());
    assert!(seeded_household_plan("test-settlement", &[1, 2, 3, 4, 5]).is_err());
}

#[test]
fn marriage_preserves_birth_family_and_birth_copies_it_to_the_child() {
    let source = crate::production_source(crate::relationship::RELATIONSHIP_SOURCE);
    let wedding = source
        .split("pub fn settle_due_weddings")
        .nth(1)
        .unwrap()
        .split("pub fn settle_due_weddings_global")
        .next()
        .unwrap();
    assert!(!wedding.contains("ensure_character_family_role"));
    assert!(!wedding.contains("delete_character_social_roles"));
    let birth = source
        .split("pub fn settle_due_births")
        .nth(1)
        .unwrap()
        .split("pub fn settle_due_births_global")
        .next()
        .unwrap();
    assert!(birth.contains("insert_character_with_origin"));
    assert!(birth.contains("copy_birth_family_roles"));
}
