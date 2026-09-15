//! Canonical controlling hand frame selection.
use super::*;
use std::collections::BTreeMap;
pub(super) fn grip_point(recipe: &Recipe, frames: &BTreeMap<String, Point>) -> Point {
    if let Some(&grip) = frames.get(SHIELD_GRIP_FRAME) {
        return grip;
    }
    if let (Some(clearance), Some(&base), Some(&top)) = (
        recipe.grip_clearance,
        frames.get(GRIP_BASE_FRAME),
        frames.get(GRIP_TOP_FRAME),
    ) {
        return add(top, mul(normalize(sub(base, top)), clearance.get()));
    }
    if let Some(grip) = recipe
        .components
        .iter()
        .rev()
        .find(|c| c.role == Some(crate::ComponentRole::Grip))
        .and_then(|c| c.id.as_ref())
        .and_then(|id| frames.get(&format!("{id}.center")))
    {
        return *grip;
    }
    if let Some(&grip) = frames.get(GRIP_CENTER_FRAME) {
        return grip;
    }
    if let (Some(&bottom), Some(&top)) =
        (frames.get(SHAFT_BOTTOM_FRAME), frames.get(SHAFT_TOP_FRAME))
    {
        let length = magnitude(sub(top, bottom));
        return lerp(bottom, top, (length * 0.2).clamp(0.18, 0.45) / length);
    }
    [0.0; 3]
}
