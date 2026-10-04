//! Profile admission choices retain the existing serialized boolean fields.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "bool", into = "bool")]
pub enum JointRequirement {
    #[default]
    Optional,
    Required,
}
impl From<bool> for JointRequirement {
    fn from(required: bool) -> Self {
        if required {
            Self::Required
        } else {
            Self::Optional
        }
    }
}
impl From<JointRequirement> for bool {
    fn from(requirement: JointRequirement) -> Self {
        requirement == JointRequirement::Required
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "bool", into = "bool")]
pub enum RetargetStrictness {
    #[default]
    Permissive,
    Strict,
}
impl From<bool> for RetargetStrictness {
    fn from(strict: bool) -> Self {
        if strict {
            Self::Strict
        } else {
            Self::Permissive
        }
    }
}
impl From<RetargetStrictness> for bool {
    fn from(strictness: RetargetStrictness) -> Self {
        strictness == RetargetStrictness::Strict
    }
}
