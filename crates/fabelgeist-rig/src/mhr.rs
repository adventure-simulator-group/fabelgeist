//! The authored Momentum Human Rig naming convention, separate from identity.

use crate::{RigJointMembership, RigJointName, RigJointPart, RigSide};

/// Canonical world/center/sided label membership; this does not prove existence.
pub fn membership(name: &RigJointName) -> RigJointMembership {
    RigJointMembership::from(
        center_membership(name) == RigJointMembership::Included
            || name.as_ref().starts_with("l_")
            || name.as_ref().starts_with("r_"),
    )
}

pub fn center_membership(name: &RigJointName) -> RigJointMembership {
    RigJointMembership::from(
        name == &RigJointName::BODY_WORLD
            || name == &RigJointName::ROOT
            || name.as_ref().starts_with("c_"),
    )
}

/// Exact authored right-side spelling; receiving-rig membership is separate.
pub fn right_partner(name: &RigJointName) -> Option<RigJointName> {
    name.as_ref()
        .strip_prefix("l_")
        .map(|suffix| RigJointName::from(format!("r_{suffix}")))
}

pub fn sided(side: RigSide, part: RigJointPart) -> RigJointName {
    let prefix = match side {
        RigSide::Left => "l",
        RigSide::Right => "r",
    };
    RigJointName::from(format!("{prefix}_{part}"))
}
