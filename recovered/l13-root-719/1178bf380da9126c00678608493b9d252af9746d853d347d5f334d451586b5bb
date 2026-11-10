//! Captured semantic bones and their reduced report hierarchy.

use super::*;
use serde::ser::SerializeMap;

pub(in crate::animation_viewer) const REPORT_BONES: [(BoneRole, &str); 17] = [
    (BoneRole::Pelvis, "pelvis"),
    (BoneRole::Chest, "chest"),
    (BoneRole::Head, "head"),
    (BoneRole::UpperArmLeft, "left_shoulder"),
    (BoneRole::UpperArmRight, "right_shoulder"),
    (BoneRole::ForearmLeft, "left_elbow"),
    (BoneRole::ForearmRight, "right_elbow"),
    (BoneRole::HandLeft, "left_hand"),
    (BoneRole::HandRight, "right_hand"),
    (BoneRole::ThighLeft, "left_hip"),
    (BoneRole::ThighRight, "right_hip"),
    (BoneRole::ShinLeft, "left_knee"),
    (BoneRole::ShinRight, "right_knee"),
    (BoneRole::FootLeft, "left_foot"),
    (BoneRole::FootRight, "right_foot"),
    (BoneRole::ToeLeft, "left_toe"),
    (BoneRole::ToeRight, "right_toe"),
];

/// Report iteration preserves the former lexical-label order, including floating folds and ties.
#[derive(Debug, Clone, Default)]
pub(in crate::animation_viewer) struct CapturedBones(BTreeMap<BoneRole, BoneSample>);

impl CapturedBones {
    pub(in crate::animation_viewer) fn iter(
        &self,
    ) -> impl Iterator<Item = (&BoneRole, &BoneSample)> {
        self.ordered().into_iter()
    }
    pub(in crate::animation_viewer) fn keys(&self) -> impl Iterator<Item = &BoneRole> {
        self.iter().map(|(role, _)| role)
    }
    pub(in crate::animation_viewer) fn values(&self) -> impl Iterator<Item = &BoneSample> {
        self.iter().map(|(_, bone)| bone)
    }
    pub(in crate::animation_viewer) fn get(&self, role: BoneRole) -> Option<&BoneSample> {
        self.0.get(&role)
    }
    #[cfg(test)]
    pub(in crate::animation_viewer) fn get_mut(
        &mut self,
        role: BoneRole,
    ) -> Option<&mut BoneSample> {
        self.0.get_mut(&role)
    }
    pub(in crate::animation_viewer) fn contains_key(&self, role: BoneRole) -> bool {
        self.0.contains_key(&role)
    }
    #[cfg(test)]
    pub(in crate::animation_viewer) fn insert(
        &mut self,
        role: BoneRole,
        bone: BoneSample,
    ) -> Option<BoneSample> {
        self.0.insert(role, bone)
    }
    pub(in crate::animation_viewer) fn len(&self) -> usize {
        self.0.len()
    }
    fn ordered(&self) -> Vec<(&BoneRole, &BoneSample)> {
        let mut entries: Vec<_> = self.0.iter().collect();
        entries.sort_by_key(|(role, _)| report_bone_label(**role));
        entries
    }
}
impl FromIterator<(BoneRole, BoneSample)> for CapturedBones {
    fn from_iter<I: IntoIterator<Item = (BoneRole, BoneSample)>>(entries: I) -> Self {
        Self(entries.into_iter().collect())
    }
}
impl<'a> IntoIterator for &'a CapturedBones {
    type Item = (&'a BoneRole, &'a BoneSample);
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.ordered().into_iter()
    }
}
impl Serialize for CapturedBones {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (role, bone) in self {
            let label = report_bone_label(*role)
                .ok_or_else(|| serde::ser::Error::custom("uncaptured semantic bone in report"))?;
            map.serialize_entry(label, bone)?;
        }
        map.end()
    }
}

impl crate::animation::jitter::JitterBoneIdentity for BoneRole {
    fn report_label(&self) -> Option<&str> {
        report_bone_label(*self)
    }
}

pub(in crate::animation_viewer) fn report_bone_label(role: BoneRole) -> Option<&'static str> {
    REPORT_BONES
        .iter()
        .find_map(|&(candidate, label)| (candidate == role).then_some(label))
}

/// Parent among the reduced captured roles, not the complete authored hierarchy.
pub(in crate::animation_viewer) fn parent_bone(role: &BoneRole) -> Option<BoneRole> {
    match role {
        BoneRole::Chest => Some(BoneRole::Pelvis),
        BoneRole::ThighLeft => Some(BoneRole::Pelvis),
        BoneRole::ThighRight => Some(BoneRole::Pelvis),
        BoneRole::Head => Some(BoneRole::Chest),
        BoneRole::UpperArmLeft => Some(BoneRole::Chest),
        BoneRole::UpperArmRight => Some(BoneRole::Chest),
        BoneRole::ForearmLeft => Some(BoneRole::UpperArmLeft),
        BoneRole::ForearmRight => Some(BoneRole::UpperArmRight),
        BoneRole::HandLeft => Some(BoneRole::ForearmLeft),
        BoneRole::HandRight => Some(BoneRole::ForearmRight),
        BoneRole::ShinLeft => Some(BoneRole::ThighLeft),
        BoneRole::ShinRight => Some(BoneRole::ThighRight),
        BoneRole::FootLeft => Some(BoneRole::ShinLeft),
        BoneRole::FootRight => Some(BoneRole::ShinRight),
        BoneRole::ToeLeft => Some(BoneRole::FootLeft),
        BoneRole::ToeRight => Some(BoneRole::FootRight),
        _ => None,
    }
}

pub(in crate::animation_viewer) fn serialize_report_bone<S: serde::Serializer>(
    role: &BoneRole,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let label = report_bone_label(*role)
        .ok_or_else(|| serde::ser::Error::custom("uncaptured semantic bone in report"))?;
    serializer.serialize_str(label)
}
