//! Rig profiles that ship with the engine, and optional rig detection.
//!
//! A profile here has no privileged status — it is the same data any project
//! can write for its own rig, and the retargeter cannot tell the difference.

use crate::skeleton::Skeleton;
use crate::skeleton::mixamo::MixamoRig;

use super::profile::RigProfile;

pub mod infer;
mod inference_name;
pub mod mixamo;
mod mixamo_finger;

/// The built-in source profiles, in detection order.
pub fn builtin() -> Vec<RigProfile> {
    vec![MixamoRig::profile()]
}

/// Guesses which built-in profile a skeleton belongs to.
///
/// Detection is a convenience for tooling and import defaults. It is never
/// required: passing an explicit profile always works, and an unrecognized rig
/// is not an error — it just means the caller has to say which profile to use.
pub fn detect(skeleton: &Skeleton) -> Option<RigProfile> {
    builtin()
        .into_iter()
        .find(|profile| profile.matches(skeleton))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skeleton::Joint;
    use crate::skeleton::SkinJointOrdinal;
    use fabelgeist_math::matrix::Mat4;
    use fabelgeist_rig::RigJointOrdinal;

    #[test]
    fn a_known_rig_is_detected() {
        let detected = detect(&MixamoRig::skeleton()).expect("the Mixamo rig should be detected");
        assert_eq!(detected.name, "Mixamo".into());
    }

    #[test]
    fn an_unknown_rig_simply_does_not_detect() {
        let skeleton = Skeleton::new(vec![Joint::new(
            "bone".into(),
            RigJointOrdinal::from(0_usize),
            None,
            Mat4::identity(),
            Default::default(),
            Some(SkinJointOrdinal::from(0_usize)),
        )]);
        assert!(detect(&skeleton).is_none());
    }
}
