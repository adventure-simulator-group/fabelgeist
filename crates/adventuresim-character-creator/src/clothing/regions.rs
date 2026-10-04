//! Anatomical garment regions mapped to the body skeleton.
use super::Region;

#[derive(Clone, Copy)]
pub(super) struct RegionRig {
    pub(super) proximal: &'static str,
    pub(super) distal: &'static str,
    pub(super) joint: fn(&str) -> bool,
}

pub(super) fn region_rig(region: Region) -> RegionRig {
    match region {
        Region::Head => RegionRig {
            proximal: "c_neck",
            distal: "c_head",
            joint: |name| {
                matches!(name, "c_head" | "c_jaw" | "l_eye" | "r_eye")
                    || name.starts_with("c_tongue")
            },
        },
        Region::Neck => RegionRig {
            proximal: "c_spine3",
            distal: "c_neck",
            joint: |name| name == "c_neck" || name.starts_with("c_neck_twist"),
        },
        Region::Chest => RegionRig {
            proximal: "c_spine2",
            distal: "c_neck",
            joint: |name| matches!(name, "c_spine2" | "c_spine3" | "l_clavicle" | "r_clavicle"),
        },
        Region::LeftAxilla => limb_rig("l_clavicle", "l_uparm", |name| name == "l_clavicle"),
        Region::RightAxilla => limb_rig("r_clavicle", "r_uparm", |name| name == "r_clavicle"),
        Region::Stomach => RegionRig {
            proximal: "c_spine0",
            distal: "c_spine2",
            joint: |name| matches!(name, "c_spine0" | "c_spine1"),
        },
        Region::Groin => RegionRig {
            proximal: "c_spine1",
            distal: "c_spine0",
            joint: |name| matches!(name, "c_spine0" | "l_upleg" | "r_upleg"),
        },
        Region::LeftUpperArm => limb_rig("l_uparm", "l_lowarm", left_upper_arm),
        Region::LeftForearm => limb_rig("l_lowarm", "l_wrist", left_forearm),
        Region::RightUpperArm => limb_rig("r_uparm", "r_lowarm", right_upper_arm),
        Region::RightForearm => limb_rig("r_lowarm", "r_wrist", right_forearm),
        Region::LeftThigh => limb_rig("l_upleg", "l_lowleg", left_thigh),
        Region::LeftLowerLeg => limb_rig("l_lowleg", "l_foot", left_lower_leg),
        Region::RightThigh => limb_rig("r_upleg", "r_lowleg", right_thigh),
        Region::RightLowerLeg => limb_rig("r_lowleg", "r_foot", right_lower_leg),
    }
}

const fn limb_rig(
    proximal: &'static str,
    distal: &'static str,
    joint: fn(&str) -> bool,
) -> RegionRig {
    RegionRig {
        proximal,
        distal,
        joint,
    }
}

fn left_upper_arm(name: &str) -> bool {
    name == "l_uparm" || name.starts_with("l_uparm_twist")
}
fn left_forearm(name: &str) -> bool {
    name == "l_lowarm" || name.starts_with("l_lowarm_twist")
}
fn right_upper_arm(name: &str) -> bool {
    name == "r_uparm" || name.starts_with("r_uparm_twist")
}
fn right_forearm(name: &str) -> bool {
    name == "r_lowarm" || name.starts_with("r_lowarm_twist")
}
fn left_thigh(name: &str) -> bool {
    name == "l_upleg" || name.starts_with("l_upleg_twist")
}
fn left_lower_leg(name: &str) -> bool {
    name == "l_lowleg" || name.starts_with("l_lowleg_twist")
}
fn right_thigh(name: &str) -> bool {
    name == "r_upleg" || name.starts_with("r_upleg_twist")
}
fn right_lower_leg(name: &str) -> bool {
    name == "r_lowleg" || name.starts_with("r_lowleg_twist")
}

pub(super) fn waist_surface_joint(name: &str) -> bool {
    matches!(
        name,
        "root" | "c_spine0" | "c_spine1" | "l_upleg" | "r_upleg"
    ) || name.starts_with("l_upleg_twist")
        || name.starts_with("r_upleg_twist")
}
