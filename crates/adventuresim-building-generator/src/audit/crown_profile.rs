use super::*;
pub(super) fn audit(crown: &crate::CrownAssembly, issues: &mut Vec<AuditIssue>) {
    let p = crown.profile;
    let merlon_top = p.breastwork_height_metres + p.merlon_height_metres + p.coping_height_metres;
    if !(0.8..=1.0).contains(&p.breastwork_height_metres)
        || !(1.5..=1.8).contains(&merlon_top)
        || p.thickness_metres < 0.35
        || !(0.35..=0.6).contains(&p.crenel_width_metres)
        || p.walk_clear_width_metres < 0.9
        || p.inner_guard_height_metres < 0.9
        || p.firing_height_metres <= p.breastwork_height_metres
        || p.firing_height_metres >= merlon_top
    {
        issues.push(issue(
            "unsafe_crown_profile",
            format!(
                "crown owner {} violates the declared cover/clearance envelope",
                crown.owner.0
            ),
        ));
    }
}
