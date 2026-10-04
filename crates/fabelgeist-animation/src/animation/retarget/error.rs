//! Stable resolution failures retain anatomical roles and exact candidate labels.
use super::{RigProfileName, semantic::HumanoidJoint};
use fabelgeist_rig::RigJointName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissingRequiredJoint {
    pub role: HumanoidJoint,
    pub candidates: Vec<RigJointName>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RetargetError {
    RequiredJointsMissing {
        profile: RigProfileName,
        bindings: Vec<MissingRequiredJoint>,
    },
    StrictTargetRolesMissing {
        roles: Vec<HumanoidJoint>,
    },
}
impl std::fmt::Display for MissingRequiredJoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (expected ", self.role)?;
        for (index, candidate) in self.candidates.iter().enumerate() {
            if index != 0 {
                f.write_str(" | ")?;
            }
            write!(f, "{candidate}")?;
        }
        f.write_str(")")
    }
}
impl std::fmt::Display for RetargetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequiredJointsMissing { profile, bindings } => {
                write!(
                    f,
                    "rig profile {profile:?} requires joints the skeleton does not have: "
                )?;
                for (index, binding) in bindings.iter().enumerate() {
                    if index != 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{binding}")?;
                }
                Ok(())
            }
            Self::StrictTargetRolesMissing { roles } => {
                f.write_str("strict retargeting: the target rig has no joint for ")?;
                for (index, role) in roles.iter().enumerate() {
                    if index != 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{role}")?;
                }
                Ok(())
            }
        }
    }
}
impl std::error::Error for RetargetError {}
