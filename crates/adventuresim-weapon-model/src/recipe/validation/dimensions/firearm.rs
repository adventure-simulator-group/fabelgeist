//! Dimension and sampling bounds for firearm constructions.
use super::*;
pub(super) fn firearm(p: &FirearmParameters) -> Checked {
    positive(p.length.get())?;
    require(
        p.barrel_count.0 <= MAX_SAMPLING_REQUEST,
        RecipeError::Budget,
    )?;
    bounded(p.barrel_length.get())?;
    if let Some(n) = &p.secondary_barrel_length {
        bounded(n.get())?;
    }
    bounded(p.bore.get())?;
    bounded(p.barrel_wall.get())?;
    if let Some(n) = &p.barrel_separation {
        bounded(n.get())?;
    }
    {
        let n = &p.octagonal_ratio;
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.muzzle_flare {
        bounded(n.get())?;
    }
    bounded(p.butt_width.get())?;
    bounded(p.waist_width.get())?;
    bounded(p.fore_width.get())?;
    bounded(p.stock_depth.get())?;
    bounded(p.butt_drop.get())?;
    bounded(p.lock_position.get())?;
    bounded(p.lock_wheel_radius.get())?;
    bounded(p.pan_width.get())?;
    bounded(p.trigger_length.get())?;
    bounded(p.guard_width.get())?;
    bounded(p.ramrod_radius.get())?;
    require(p.band_count.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    bounded(p.facing_thickness.get())?;
    if let Some(n) = &p.samples {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }
    if let Some(n) = &p.radial_segments {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }

    Ok(())
}
pub(super) fn lead_ball(p: &LeadBallParameters) -> Checked {
    positive(p.radius.get())?;
    if let Some(n) = &p.segments {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }

    Ok(())
}
pub(super) fn ball_pouch(p: &BallPouchParameters) -> Checked {
    positive(p.width.get())?;
    positive(p.height.get())?;
    bounded(p.depth.get())?;
    positive(p.wall.get())?;
    bounded(p.flap_length.get())?;
    bounded(p.flap_overlap.get())?;
    {
        let n = &p.flap_angle;
        require(n.get().abs() <= 3600.0, RecipeError::Dimension)?;
    }
    bounded(p.belt_loop_width.get())?;
    bounded(p.belt_loop_gap.get())?;

    Ok(())
}
