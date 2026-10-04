//! Loose retargeting keys never replace a skeleton's exact authored identity.
use fabelgeist_rig::RigJointName;
use std::borrow::Cow;

/// Last colon/pipe namespace segment, ASCII alphanumerics, lowercase.
/// Distinct labels can collide; resolution gives exact labels precedence,
/// then retains the first skeleton joint for a loose-key collision.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct RetargetMatchKey(String);
impl From<&RigJointName> for RetargetMatchKey {
    fn from(name: &RigJointName) -> Self {
        let spelling: Cow<'static, str> = name.clone().into();
        let bare = spelling.rsplit([':', '|']).next().unwrap_or(&spelling);
        Self(
            bare.chars()
                .filter(|character: &char| -> bool { character.is_ascii_alphanumeric() })
                .map(|character: char| -> char { character.to_ascii_lowercase() })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespaces_and_separators_do_not_defeat_lookup() {
        let expected = RetargetMatchKey("leftupleg".to_owned());
        assert_eq!(
            RetargetMatchKey::from(&RigJointName::from("mixamorig:LeftUpLeg")),
            expected
        );
        assert_eq!(
            RetargetMatchKey::from(&RigJointName::from("mixamorig1:Left_Up_Leg")),
            expected
        );
        assert_eq!(
            RetargetMatchKey::from(&RigJointName::from("Armature|LeftUpLeg")),
            expected
        );
        assert_ne!(
            RetargetMatchKey::from(&RigJointName::from("l_upleg")),
            RetargetMatchKey::from(&RigJointName::from("r_upleg"))
        );
    }
}
