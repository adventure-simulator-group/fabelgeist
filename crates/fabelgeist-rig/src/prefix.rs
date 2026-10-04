//! Exact authored material preceding a rig label, independent of that label.
use super::{RigJointMembership, RigJointName};
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RigJointPrefix(Cow<'static, str>);

impl RigJointPrefix {
    pub const MIXAMO: Self = Self(Cow::Borrowed("mixamorig:"));

    pub fn apply(&self, name: &RigJointName) -> RigJointName {
        RigJointName(Cow::Owned(format!("{}{name}", self.0)))
    }

    pub fn membership(&self, name: &RigJointName) -> RigJointMembership {
        RigJointMembership::from(name.0.starts_with(self.0.as_ref()))
    }

    pub fn remove_from(&self, name: &RigJointName) -> RigJointName {
        RigJointName::from(name.0.strip_prefix(self.0.as_ref()).unwrap_or(&name.0))
    }
}

impl From<&str> for RigJointPrefix {
    fn from(prefix: &str) -> Self {
        Self(Cow::Owned(prefix.to_owned()))
    }
}
impl std::fmt::Display for RigJointPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
