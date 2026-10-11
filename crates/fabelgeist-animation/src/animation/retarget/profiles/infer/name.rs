//! Lossy ASCII matching names used only by rig inference.
//!
//! Skeleton and profile joint names retain their exact spelling. These private
//! values own side extraction and hint matching, not exact joint identity.

/// The side and normalized matching stem read from one native joint spelling.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct InferredName {
    pub(super) side: Side,
    pub(super) stem: InferenceStem,
}

/// ASCII alphanumerics only, lowercased after side extraction.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct InferenceStem(String);

/// The recognized fragments used by the ordered inference catalogs.
#[derive(Clone, Copy)]
pub(super) enum InferenceKeyword {
    Hips,
    Pelvis,
    Hip,
    Neck,
    Head,
    Clavicle,
    Shoulder,
    Collar,
    UpperArm,
    UpArm,
    Humerus,
    Shldr,
    Arm,
    Forearm,
    LowerArm,
    LowArm,
    Elbow,
    Ulna,
    Hand,
    Wrist,
    UpperLeg,
    UpLeg,
    Thigh,
    Femur,
    LowerLeg,
    LowLeg,
    Shin,
    Calf,
    Knee,
    Tibia,
    Leg,
    Foot,
    Ankle,
    ToeBase,
    Toe,
    Ball,
    Spine,
    Chest,
    Torso,
    Abdomen,
    Waist,
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
    Little,
    Root,
    Reference,
    Armature,
}

/// Which half of the body a joint's name claims.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Side {
    Left,
    Right,
    Center,
}

impl InferredName {
    /// Splits a joint spelling into its side and a normalized matching stem.
    ///
    /// Rigs mark sides in many ways: `LeftArm`, `l_uparm`, `arm.L`,
    /// `LHipJoint` and `lFemur`. Native text is admitted only at this boundary.
    pub(super) fn from_joint_name(name: &str) -> Self {
        let bare = name.rsplit([':', '|']).next().unwrap_or(name);
        let lower = bare.to_ascii_lowercase();
        let strip = |side: Side, rest: String| Self {
            side,
            stem: InferenceStem::from_native_spelling(&rest),
        };
        for side in [Side::Left, Side::Right] {
            let word = match side {
                Side::Left => "left",
                Side::Right => "right",
                Side::Center => continue,
            };
            if let Some(position) = lower.find(word) {
                let mut rest = lower.clone();
                rest.replace_range(position..position + word.len(), "");
                return strip(side, rest);
            }
        }
        let separators = ['_', '-', '.', ' '];
        for side in [Side::Left, Side::Right] {
            let prefix = match side {
                Side::Left => "l",
                Side::Right => "r",
                Side::Center => continue,
            };
            for separator in separators {
                let marker = format!("{prefix}{separator}");
                if lower.starts_with(&marker) {
                    return strip(side, lower[marker.len()..].to_string());
                }
                let marker = format!("{separator}{prefix}");
                if lower.ends_with(&marker) {
                    return strip(side, lower[..lower.len() - marker.len()].to_string());
                }
            }
        }
        // A lone side letter only counts before an ASCII uppercase character.
        let mut characters = bare.chars();
        if let (Some(first), Some(second)) = (characters.next(), characters.next())
            && second.is_ascii_uppercase()
        {
            match first {
                'l' | 'L' => return strip(Side::Left, bare[1..].to_string()),
                'r' | 'R' => return strip(Side::Right, bare[1..].to_string()),
                _ => {}
            }
        }
        Self {
            side: Side::Center,
            stem: InferenceStem::from_native_spelling(bare),
        }
    }
}

impl InferenceStem {
    fn from_native_spelling(name: &str) -> Self {
        Self(
            name.chars()
                .filter(|character| character.is_ascii_alphanumeric())
                .map(|character| character.to_ascii_lowercase())
                .collect(),
        )
    }

    pub(super) fn contains(&self, keyword: InferenceKeyword) -> bool {
        self.0.contains(keyword.spelling())
    }

    pub(super) fn starts_with(&self, keyword: InferenceKeyword) -> bool {
        self.0.starts_with(keyword.spelling())
    }

    pub(super) fn is_root_hint(&self) -> bool {
        [
            InferenceKeyword::Root,
            InferenceKeyword::Reference,
            InferenceKeyword::Armature,
        ]
        .iter()
        .any(|keyword| self.0 == keyword.spelling())
    }
}

impl InferenceKeyword {
    /// ASCII token length is the existing intrinsic specificity measure.
    pub(super) fn length(self) -> usize {
        self.spelling().len()
    }

    fn spelling(self) -> &'static str {
        match self {
            Self::Hips => "hips",
            Self::Pelvis => "pelvis",
            Self::Hip => "hip",
            Self::Neck => "neck",
            Self::Head => "head",
            Self::Clavicle => "clavicle",
            Self::Shoulder => "shoulder",
            Self::Collar => "collar",
            Self::UpperArm => "upperarm",
            Self::UpArm => "uparm",
            Self::Humerus => "humerus",
            Self::Shldr => "shldr",
            Self::Arm => "arm",
            Self::Forearm => "forearm",
            Self::LowerArm => "lowerarm",
            Self::LowArm => "lowarm",
            Self::Elbow => "elbow",
            Self::Ulna => "ulna",
            Self::Hand => "hand",
            Self::Wrist => "wrist",
            Self::UpperLeg => "upperleg",
            Self::UpLeg => "upleg",
            Self::Thigh => "thigh",
            Self::Femur => "femur",
            Self::LowerLeg => "lowerleg",
            Self::LowLeg => "lowleg",
            Self::Shin => "shin",
            Self::Calf => "calf",
            Self::Knee => "knee",
            Self::Tibia => "tibia",
            Self::Leg => "leg",
            Self::Foot => "foot",
            Self::Ankle => "ankle",
            Self::ToeBase => "toebase",
            Self::Toe => "toe",
            Self::Ball => "ball",
            Self::Spine => "spine",
            Self::Chest => "chest",
            Self::Torso => "torso",
            Self::Abdomen => "abdomen",
            Self::Waist => "waist",
            Self::Thumb => "thumb",
            Self::Index => "index",
            Self::Middle => "middle",
            Self::Ring => "ring",
            Self::Pinky => "pinky",
            Self::Little => "little",
            Self::Root => "root",
            Self::Reference => "reference",
            Self::Armature => "armature",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sides_are_read_however_a_rig_spells_them() {
        assert_eq!(
            InferredName::from_joint_name("LeftUpLeg"),
            InferredName {
                side: Side::Left,
                stem: InferenceStem("upleg".into())
            }
        );
        assert_eq!(
            InferredName::from_joint_name("mixamorig:RightArm"),
            InferredName {
                side: Side::Right,
                stem: InferenceStem("arm".into())
            }
        );
        assert_eq!(
            InferredName::from_joint_name("l_uparm"),
            InferredName {
                side: Side::Left,
                stem: InferenceStem("uparm".into())
            }
        );
        assert_eq!(
            InferredName::from_joint_name("upperarm.R"),
            InferredName {
                side: Side::Right,
                stem: InferenceStem("upperarm".into())
            }
        );
        assert_eq!(
            InferredName::from_joint_name("LHipJoint"),
            InferredName {
                side: Side::Left,
                stem: InferenceStem("hipjoint".into())
            }
        );
        assert_eq!(
            InferredName::from_joint_name("lFemur"),
            InferredName {
                side: Side::Left,
                stem: InferenceStem("femur".into())
            }
        );
        // Words that merely start with l or r are not sides.
        assert_eq!(
            InferredName::from_joint_name("LowerLeg"),
            InferredName {
                side: Side::Center,
                stem: InferenceStem("lowerleg".into())
            }
        );
        assert_eq!(
            InferredName::from_joint_name("Hips"),
            InferredName {
                side: Side::Center,
                stem: InferenceStem("hips".into())
            }
        );
    }
}
