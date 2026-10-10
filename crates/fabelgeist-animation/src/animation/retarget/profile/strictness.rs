//! Whether source roles absent from the target reject retargeting.

use serde::{Deserialize, Serialize};

/// The transfer policy for source roles the target rig does not resolve.
///
/// Both rigs resolve their required joints before this policy is checked.
/// Strict transfer rejects every resolved source role absent from the target,
/// including roles unused by a clip. Serde represents this policy as a Boolean
/// at the profile's `strict` JSON field.
///
/// Select a named policy when editing settings directly:
///
/// ```
/// use fabelgeist_animation::animation::retarget::{RetargetSettings, RetargetStrictness};
///
/// let settings = RetargetSettings {
///     strict: RetargetStrictness::Strict,
///     ..Default::default()
/// };
/// ```
///
/// A raw Boolean cannot be assigned to settings:
///
/// ```compile_fail
/// use fabelgeist_animation::animation::retarget::RetargetSettings;
///
/// let settings = RetargetSettings {
///     strict: true,
///     ..Default::default()
/// };
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "bool", into = "bool")]
pub enum RetargetStrictness {
    /// Transfer shared roles and ignore resolved source roles absent from the target.
    #[default]
    Permissive,
    /// Reject transfer if any resolved source role is absent from the target.
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
        matches!(strictness, RetargetStrictness::Strict)
    }
}
