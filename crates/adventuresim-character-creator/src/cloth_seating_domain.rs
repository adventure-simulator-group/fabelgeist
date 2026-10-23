//! Final rest-pose support for released torso cloth and complete sewn sleeves.
use crate::skirt_join::SkirtJoinPlan;

pub(crate) fn mask(plan: &SkirtJoinPlan) -> Vec<u32> {
    let mut mask = vec![0; plan.patch.point_count as usize];
    // Fitted source positions and normals are shared by the visible torso and
    // its concealed layers. Preserve that already seated field. Released cloth
    // has been reshaped since source fitting and needs final body support.
    if plan.release_weights.iter().any(|&weight| weight > 0.) {
        for flag in mask
            .iter_mut()
            .take(plan.table.len() / crate::surface_cut::POINT_WORDS)
        {
            *flag = 1;
        }
    }
    mask
}
