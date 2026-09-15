//! Dimension and sampling bounds for archery constructions.
use super::*;
pub(super) fn archery_bow(p: &ArcheryBowParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.grip_length.get())?;
    positive(p.limb_width.get())?;
    positive(p.limb_depth.get())?;
    bounded(p.grip_width.get())?;
    bounded(p.grip_depth.get())?;
    {
        let n = &p.tip_scale;
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    bounded(p.reflex.get())?;
    bounded(p.recurve.get())?;
    {
        let n = &p.upper_ratio;
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    bounded(p.brace_height.get())?;
    positive(p.string_radius.get())?;
    bounded(p.loop_radius.get())?;
    bounded(p.loop_gap.get())?;
    if let Some(n) = &p.backing_thickness {
        bounded(n.get())?;
    }
    if let Some(n) = &p.horn_thickness {
        bounded(n.get())?;
    }
    if let Some(n) = &p.samples {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }
    if let Some(n) = &p.radial_segments {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }

    Ok(())
}
pub(super) fn arrow(p: &ArrowParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.shaft_radius.get())?;
    positive(p.head_length.get())?;
    positive(p.head_width.get())?;
    positive(p.head_thickness.get())?;
    positive(p.fletching_length.get())?;
    bounded(p.fletching_height.get())?;
    require(
        p.fletching_count.0 <= MAX_SAMPLING_REQUEST,
        RecipeError::Budget,
    )?;
    positive(p.nock_length.get())?;
    bounded(p.nock_slot_width.get())?;
    bounded(p.maximum_string_radius.get())?;
    bounded(p.nock_clearance.get())?;
    if let Some(n) = &p.segments {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }

    Ok(())
}
pub(super) fn arrow_quiver(p: &ArrowQuiverParameters) -> Checked {
    positive(p.length.get())?;
    bounded(p.mouth_radius.get())?;
    {
        let n = &p.bottom_scale;
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    positive(p.wall.get())?;
    bounded(p.rim_radius.get())?;
    positive(p.strap_width.get())?;
    positive(p.strap_thickness.get())?;
    bounded(p.strap_drop.get())?;
    if let Some(n) = &p.segments {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }

    Ok(())
}
