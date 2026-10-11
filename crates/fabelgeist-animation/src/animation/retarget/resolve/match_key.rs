//! The resolver's lossy equality key for native joint spellings.
//!
//! Keep the final namespace segment, ASCII letters and digits, and fold ASCII
//! case. Side markers and digits stay intact. Distinct spellings can collide,
//! including at the empty key; JointIndex owns exact and first-joint
//! precedence.

#[derive(Debug, PartialEq, Eq, Hash)]
pub(super) struct RetargetMatchKey(String);

impl RetargetMatchKey {
    pub(super) fn from_joint_spelling(name: &str) -> Self {
        let name = name.rsplit([':', '|']).next().unwrap_or(name);
        Self(
            name.chars()
                .filter(|character| character.is_ascii_alphanumeric())
                .map(|character| character.to_ascii_lowercase())
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespaces_and_separators_do_not_defeat_lookup() {
        assert_eq!(
            RetargetMatchKey::from_joint_spelling("mixamorig:LeftUpLeg"),
            RetargetMatchKey("leftupleg".to_owned())
        );
        assert_eq!(
            RetargetMatchKey::from_joint_spelling("mixamorig1:Left_Up_Leg"),
            RetargetMatchKey("leftupleg".to_owned())
        );
        assert_eq!(
            RetargetMatchKey::from_joint_spelling("Armature|LeftUpLeg"),
            RetargetMatchKey("leftupleg".to_owned())
        );
        assert_ne!(
            RetargetMatchKey::from_joint_spelling("l_upleg"),
            RetargetMatchKey::from_joint_spelling("r_upleg")
        );
    }
}
