//! Geometric framing shared by semantic timber proof cameras.
use super::*;
use adventuresim_building_generator::ResolvedSolid;

pub(super) fn target(
    plan: &BuildingPlan,
    view: ViewerView,
    camera_focused: &[&ResolvedSolid],
) -> Vec3 {
    if camera_focused.is_empty() {
        let dimensions = plan.dimensions_metres();
        Vec3::new(
            dimensions.x * 0.5,
            plan.storey_height_metres,
            dimensions.y * 0.5,
        )
    } else if view == ViewerView::TimberJettyUnderside {
        let min = camera_focused
            .iter()
            .map(|solid| solid.centre - solid.size * 0.5)
            .fold(Vec3::splat(f32::INFINITY), Vec3::min);
        let max = camera_focused
            .iter()
            .map(|solid| solid.centre + solid.size * 0.5)
            .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
        (min + max) * 0.5
    } else {
        camera_focused
            .iter()
            .map(|solid| solid.centre)
            .sum::<Vec3>()
            / camera_focused.len() as f32
    }
}

pub(super) fn extent(camera_focused: &[&ResolvedSolid]) -> f32 {
    let min = camera_focused
        .iter()
        .map(|solid| solid.centre - solid.size * 0.5)
        .fold(Vec3::splat(f32::INFINITY), Vec3::min);
    let max = camera_focused
        .iter()
        .map(|solid| solid.centre + solid.size * 0.5)
        .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
    if camera_focused.is_empty() {
        4.0
    } else {
        (max - min).max_element().max(4.0)
    }
}
