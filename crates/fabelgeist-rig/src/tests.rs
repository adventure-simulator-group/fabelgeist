use super::*;

#[test]
fn exact_prefix_material_neither_normalizes_nor_confers_membership() {
    let part = RigJointName::from("LeftArm");
    let name = RigJointPrefix::MIXAMO.apply(&part);
    assert_eq!(name, RigJointName::from("mixamorig:LeftArm"));
    assert_eq!(RigJointPrefix::MIXAMO.remove_from(&name), part);
    let other = RigJointName::from("mixamorig1:LeftArm");
    assert_eq!(
        RigJointPrefix::MIXAMO.membership(&other),
        RigJointMembership::Excluded
    );
    assert_eq!(RigJointPrefix::MIXAMO.remove_from(&other), other);
    let prefix = RigJointPrefix::from(" α:\0");
    assert_eq!(prefix.remove_from(&prefix.apply(&part)), part);
}

#[test]
fn exact_labels_survive_json_without_trimming_or_namespace_changes() {
    let labels = [
        "root",
        "scope:joint",
        "",
        "  joint  ",
        "c_head\u{0}",
        "éye",
        "helper_head",
    ];
    for spelling in labels {
        let name = RigJointName::from(spelling);
        let encoded = serde_json::to_vec(&name).unwrap();
        assert_eq!(encoded, serde_json::to_vec(spelling).unwrap());
        assert_eq!(
            serde_json::from_slice::<RigJointName>(&encoded).unwrap(),
            name
        );
    }
}

#[test]
fn lookup_keeps_first_duplicate_and_retains_the_rejected_identity() {
    let names = [RigJointName::ROOT, RigJointName::C_HEAD, RigJointName::ROOT];
    assert_eq!(
        RigJointName::ROOT.require_in(&names).unwrap(),
        RigJointOrdinal::from(0_usize)
    );
    let absent = RigJointName::from("scope:root");
    let error = absent.require_in(&names).unwrap_err();
    assert_eq!(error.joint, absent);
    assert_eq!(RigJointName::from("").index_in(&names), None);
}

#[test]
fn skin_family_retains_exact_and_unconstrained_twist_prefix_policy() {
    let owner = RigJointName::L_UPARM;
    for label in [
        "l_uparm",
        "l_uparm_twist",
        "l_uparm_twist3_proc",
        "l_uparm_twisted",
    ] {
        assert_eq!(
            RigJointName::from(label).skin_family(&owner),
            RigJointMembership::Included
        );
    }
    for label in [
        "r_uparm",
        "l_upperarm",
        "l_uparm_extra",
        "l_uparm_Twist0",
        "l_uparm ",
    ] {
        assert_eq!(
            RigJointName::from(label).skin_family(&owner),
            RigJointMembership::Excluded
        );
    }
}

#[test]
fn anatomical_fragment_and_family_prefix_queries_remain_distinct() {
    let helper = RigJointName::from("helper_head_twist");
    assert_eq!(
        helper.contains_part(RigJointPart::Head),
        RigJointMembership::Included
    );
    assert_eq!(
        helper.family_prefix(&RigJointName::C_HEAD),
        RigJointMembership::Excluded
    );
    assert_eq!(
        RigJointName::from("l_thumb_null").family_prefix(&RigJointName::L_THUMB),
        RigJointMembership::Included
    );
    assert_eq!(
        RigJointName::sided(RigSide::Right, RigJointPart::Lowleg),
        RigJointName::R_LOWLEG
    );
    assert_eq!(
        RigJointName::sided(RigSide::Left, RigJointPart::Upperarm),
        RigJointName::L_UPPERARM
    );
}
