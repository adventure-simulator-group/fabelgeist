//! Whether a rig binding must resolve to a skeleton joint.

use serde::{Deserialize, Serialize};

/// The admission role of a joint binding during rig resolution.
///
/// Optional bindings may remain unresolved. A missing required binding rejects
/// the rig before chains and root motion are resolved. Serde represents this
/// role as a Boolean at the profile's existing `required` JSON field.
///
/// Select a named role when editing a binding directly:
///
/// ```
/// use fabelgeist_animation::animation::retarget::{JointBinding, JointRequirement};
///
/// let mut binding = JointBinding::new("hips");
/// binding.required = JointRequirement::Required;
/// ```
///
/// A raw Boolean cannot be assigned to the binding:
///
/// ```compile_fail
/// use fabelgeist_animation::animation::retarget::JointBinding;
///
/// let mut binding = JointBinding::new("hips");
/// binding.required = true;
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "bool", into = "bool")]
pub enum JointRequirement {
    /// A missing joint is reported without rejecting the rig.
    #[default]
    Optional,
    /// A missing joint rejects the rig.
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
        matches!(requirement, JointRequirement::Required)
    }
}
