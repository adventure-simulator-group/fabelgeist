use super::*;

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

#[test]
fn case_insensitive_queries_do_not_change_exact_identity_or_sorting() {
    let upper = RigJointName::from("ROOT");
    assert_ne!(upper, RigJointName::ROOT);
    assert!(upper.eq_ignore_ascii_case(&RigJointName::ROOT));
    assert!(!RigJointName::from("root ").eq_ignore_ascii_case(&RigJointName::ROOT));
    let mut names = [RigJointName::ROOT, RigJointName::from(""), upper];
    names.sort();
    assert_eq!(
        serde_json::to_value(&names).unwrap(),
        serde_json::json!(["", "ROOT", "root"])
    );
}

#[test]
fn mhr_topology_queries_preserve_case_and_unmatched_side_labels() {
    for label in ["body_world", "root", "c_helper", "l_", "r_unfamiliar"] {
        assert_eq!(
            RigJointName::from(label).mhr_membership(),
            RigJointMembership::Included
        );
    }
    for label in ["ROOT", "scope:root", "left_foot", "l"] {
        assert_eq!(
            RigJointName::from(label).mhr_membership(),
            RigJointMembership::Excluded
        );
    }
    assert_eq!(
        RigJointName::L_FOOT.mhr_right_partner(),
        Some(RigJointName::R_FOOT)
    );
    assert_eq!(
        RigJointName::from("l_").mhr_right_partner(),
        Some(RigJointName::from("r_"))
    );
    assert_eq!(RigJointName::R_FOOT.mhr_right_partner(), None);
    assert_eq!(
        RigJointName::L_FOOT.mhr_center_membership(),
        RigJointMembership::Excluded
    );
    assert_eq!(
        RigJointName::from("c_helper").mhr_center_membership(),
        RigJointMembership::Included
    );
}
