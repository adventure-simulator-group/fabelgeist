//! Dimension and sampling bounds for crossbow constructions.
use super::*;
pub(super) fn crossbow(p: &CrossbowParameters) -> Checked {
    positive(p.length.get())?;
    bounded(p.butt_width.get())?;
    bounded(p.waist_width.get())?;
    bounded(p.nose_width.get())?;
    bounded(p.stock_thickness.get())?;
    bounded(p.butt_drop.get())?;
    bounded(p.lock_table_height.get())?;
    bounded(p.fore_end_rise.get())?;
    bounded(p.facing_thickness.get())?;
    bounded(p.prod_position.get())?;
    bounded(p.prod_span.get())?;
    bounded(p.prod_depth.get())?;
    bounded(p.prod_thickness.get())?;
    bounded(p.prod_sweep.get())?;
    {
        let n = &p.prod_tip_scale;
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.horn_thickness {
        bounded(n.get())?;
    }
    if let Some(n) = &p.sinew_thickness {
        bounded(n.get())?;
    }
    positive(p.string_radius.get())?;
    bounded(p.serving_width.get())?;
    bounded(p.tip_loop_clearance.get())?;
    bounded(p.bridle_spacing.get())?;
    bounded(p.bridle_radius.get())?;
    bounded(p.nut_position.get())?;
    bounded(p.nut_radius.get())?;
    bounded(p.nut_width.get())?;
    bounded(p.nut_thickness.get())?;
    bounded(p.rail_height.get())?;
    bounded(p.trigger_length.get())?;
    bounded(p.groove_width.get())?;
    bounded(p.stirrup_width.get())?;
    bounded(p.stirrup_length.get())?;
    bounded(p.stirrup_bar.get())?;
    bounded(p.spanning_bar.get())?;
    if let Some(n) = &p.samples {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }
    if let Some(n) = &p.radial_segments {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }

    Ok(())
}
pub(super) fn crossbow_bolt(p: &CrossbowBoltParameters) -> Checked {
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
    bounded(p.butt_length.get())?;
    bounded(p.butt_width.get())?;
    bounded(p.butt_height.get())?;
    if let Some(n) = &p.segments {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }

    Ok(())
}
pub(super) fn bolt_quiver(p: &BoltQuiverParameters) -> Checked {
    positive(p.length.get())?;
    bounded(p.bottom_width.get())?;
    bounded(p.mouth_width.get())?;
    bounded(p.depth.get())?;
    positive(p.wall.get())?;
    bounded(p.lining.get())?;
    bounded(p.hide_cover.get())?;
    positive(p.strap_width.get())?;
    positive(p.strap_thickness.get())?;
    bounded(p.strap_drop.get())?;

    Ok(())
}
