//! Completed mapping diagnostics shared by resolution and retargeting.

use std::fmt;

use crate::skeleton::Skeleton;

use super::resolve::ResolvedProfile;
use super::semantic::{HumanoidChain, HumanoidJoint};

/// Rendered diagnostics for a source-to-target joint mapping.
///
/// Obtain a report from [`ResolvedProfile::report`] or
/// [`super::Retargeter::report`]. Keep this artifact typed until printing or
/// another native text boundary. Reports retain their original joint spellings,
/// ordering and whitespace; they neither parse nor normalize names.
///
/// ```
/// use fabelgeist_animation::animation::retarget::{
///     RetargetProfile, RetargetReport, RigProfile, RigProfileName,
/// };
/// use fabelgeist_animation::Skeleton;
/// # fn main() -> anyhow::Result<()> {
/// let rig = Skeleton::new(Vec::new());
/// let profile = RetargetProfile::new(
///     RigProfile::new(RigProfileName::from("source")),
///     RigProfile::new(RigProfileName::from("target")),
/// );
/// let report: RetargetReport = profile.resolve(&rig, &rig)?.report(&rig, &rig);
/// println!("{report}");
/// # Ok(())
/// # }
/// ```
///
/// Arbitrary text is not a completed mapping report:
///
/// ```compile_fail
/// use fabelgeist_animation::animation::retarget::RetargetReport;
/// let report: RetargetReport = String::from("unresolved text");
/// ```
///
/// A rig-profile label describes a recognition recipe, not its report:
///
/// ```compile_fail
/// use fabelgeist_animation::animation::retarget::{RetargetReport, RigProfileName};
/// let report: RetargetReport = RigProfileName::from("my rig");
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct RetargetReport(String);

impl fmt::Debug for RetargetReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl fmt::Display for RetargetReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

impl RetargetReport {
    /// Append formatted diagnostics at the native text rendering boundary.
    pub(super) fn append(&mut self, arguments: fmt::Arguments<'_>) {
        self.0.push_str(&arguments.to_string());
    }
}

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
        let name = |skeleton: &Skeleton, index: Option<usize>| match index {
            Some(index) => skeleton.joints[index].name.clone(),
            None => "<unmapped>".to_string(),
        };

        report.push_str(&format!("profile: {}\n", self.name));
        report.push_str(&format!(
            "source rig: {} ({} joints)\n",
            self.source.profile,
            source_skeleton.joints.len()
        ));
        report.push_str(&format!(
            "target rig: {} ({} joints)\n\n",
            self.target.profile,
            target_skeleton.joints.len()
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
            if source.len() == target.len() {
                continue;
            }
            let joints = |skeleton: &Skeleton, indices: &[usize]| {
                indices
                    .iter()
                    .map(|index| skeleton.joints[*index].name.as_str())
                    .collect::<Vec<_>>()
                    .join(" -> ")
            };
            report.push_str(&format!(
                "\nchain {chain} ({} source joints, {} target joints, motion distributed)\n",
                source.len(),
                target.len()
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
        RetargetReport(report)
    }
}
