//! Rendered mapping diagnostics, without acquiring rig-label lookup authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetargetReport(String);
impl From<String> for RetargetReport {
    fn from(text: String) -> Self {
        Self(text)
    }
}
impl std::fmt::Display for RetargetReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

use super::RigJointChain;
use super::{HumanoidChain, HumanoidJoint, ResolvedProfile, Retargeter};
use crate::skeleton::Skeleton;
use fabelgeist_rig::RigJointOrdinal;

impl ResolvedProfile {
    /// A human-readable dump of the mapping, for checking a new rig.
    ///
    /// ```text
    /// Pelvis:
    ///   source = mixamorig:Hips
    ///   target = root
    /// ```
    pub fn report(&self, source_skeleton: &Skeleton, target_skeleton: &Skeleton) -> RetargetReport {
        let mut report = String::new();
        let name = |skeleton: &Skeleton, index: Option<RigJointOrdinal>| match index {
            Some(index) => skeleton.joints[index].name.to_string(),
            None => "<unmapped>".to_string(),
        };

        report.push_str(&format!("profile: {}\n", self.name));
        report.push_str(&format!(
            "source rig: {} ({} joints)\n",
            self.source.profile,
            source_skeleton.joints.count()
        ));
        report.push_str(&format!(
            "target rig: {} ({} joints)\n\n",
            self.target.profile,
            target_skeleton.joints.count()
        ));

        for role in HumanoidJoint::ALL {
            let source = self.source.joint(*role);
            let target = self.target.joint(*role);
            if source.is_none() && target.is_none() {
                continue;
            }
            report.push_str(&format!("{role}:\n"));
            report.push_str(&format!("  source = {}\n", name(source_skeleton, source)));
            report.push_str(&format!("  target = {}\n", name(target_skeleton, target)));
        }

        for chain in HumanoidChain::ALL {
            let source = self.source.chains.get(chain);
            let target = self.target.chains.get(chain);
            let (Some(source), Some(target)) = (source, target) else {
                continue;
            };
            if source.count() == target.count() {
                continue;
            }
            let joints = |skeleton: &Skeleton, indices: &RigJointChain| {
                indices
                    .iter()
                    .map(|index| skeleton.joints[*index].name.to_string())
                    .collect::<Vec<_>>()
                    .join(" -> ")
            };
            report.push_str(&format!(
                "\nchain {chain} ({} source joints, {} target joints, motion distributed)\n",
                source.count(),
                target.count()
            ));
            report.push_str(&format!("  source = {}\n", joints(source_skeleton, source)));
            report.push_str(&format!("  target = {}\n", joints(target_skeleton, target)));
        }

        if !self.source.missing.is_empty() {
            let missing: Vec<String> = self
                .source
                .missing
                .iter()
                .map(ToString::to_string)
                .collect();
            report.push_str(&format!(
                "\nsource rig is missing: {}\n",
                missing.join(", ")
            ));
        }
        if !self.target.missing.is_empty() {
            let missing: Vec<String> = self
                .target
                .missing
                .iter()
                .map(ToString::to_string)
                .collect();
            report.push_str(&format!("target rig is missing: {}\n", missing.join(", ")));
        }
        RetargetReport::from(report)
    }
}

impl Retargeter<'_> {
    /// The mapping as text, for inspecting a rig that misbehaves.
    pub fn report(&self) -> RetargetReport {
        let mut report = self
            .resolved
            .report(self.source_skeleton, self.target_skeleton)
            .to_string();
        report.push_str(&format!(
            "\nscale: {:.4} target units per source unit\n",
            self.scale
        ));
        let effectors = self.resolved.target.end_effectors();
        if !effectors.is_empty() {
            let names: Vec<String> = effectors
                .iter()
                .map(|(role, index)| format!("{role}={}", self.target_skeleton.joints[*index].name))
                .collect();
            report.push_str(&format!("end effectors: {}\n", names.join(", ")));
        }
        RetargetReport::from(report)
    }
}
