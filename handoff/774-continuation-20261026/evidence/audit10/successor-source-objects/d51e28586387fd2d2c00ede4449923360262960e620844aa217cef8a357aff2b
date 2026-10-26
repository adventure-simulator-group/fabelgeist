//! Dimension and sampling bounds for shields constructions.
use super::*;
pub(super) fn round_shield(p: &RoundShieldParameters) -> Checked {
    positive(p.radius.get())?;
    positive(p.thickness.get())?;
    require(p.rings.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    require(
        p.radial_segments.0 <= MAX_SAMPLING_REQUEST,
        RecipeError::Budget,
    )?;
    bounded(p.outer_curve.get())?;
    bounded(p.center_curve.get())?;
    bounded(p.center_radius.get())?;
    if let Some(n) = &p.rim_radius {
        bounded(n.get())?;
    }
    if let Some(n) = &p.boss_radius {
        bounded(n.get())?;
    }
    if let Some(n) = &p.boss_height {
        bounded(n.get())?;
    }
    if let Some(n) = &p.fitting_angle {
        require(n.get().abs() <= 3600.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.grip_length {
        positive(n.get())?;
    }
    if let Some(n) = &p.grip_radius {
        positive(n.get())?;
    }
    if let Some(n) = &p.fitting_spacing {
        bounded(n.get())?;
    }
    if let Some(n) = &p.fitting_clearance {
        bounded(n.get())?;
    }
    if let Some(n) = &p.strap_width {
        positive(n.get())?;
    }
    if let Some(n) = &p.strap_thickness {
        positive(n.get())?;
    }

    Ok(())
}
pub(super) fn shaped_shield(p: &ShapedShieldParameters) -> Checked {
    positive(p.width.get())?;
    positive(p.height.get())?;
    positive(p.thickness.get())?;
    require(
        p.edge_segments.0 <= MAX_SAMPLING_REQUEST,
        RecipeError::Budget,
    )?;
    bounded(p.top_depth.get())?;
    bounded(p.bottom_depth.get())?;
    {
        let n = &p.top_roundness;
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    {
        let n = &p.bottom_roundness;
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    {
        let n = &p.side_taper;
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    bounded(p.corner_radius.get())?;
    bounded(p.cylindrical_curve.get())?;
    bounded(p.center_curve.get())?;
    bounded(p.center_width.get())?;
    bounded(p.center_height.get())?;
    if let Some(n) = &p.rim_radius {
        bounded(n.get())?;
    }
    if let Some(n) = &p.boss_radius {
        bounded(n.get())?;
    }
    if let Some(n) = &p.boss_height {
        bounded(n.get())?;
    }
    if let Some(n) = &p.fitting_angle {
        require(n.get().abs() <= 3600.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.grip_length {
        positive(n.get())?;
    }
    if let Some(n) = &p.grip_radius {
        positive(n.get())?;
    }
    if let Some(n) = &p.fitting_spacing {
        bounded(n.get())?;
    }
    if let Some(n) = &p.fitting_clearance {
        bounded(n.get())?;
    }
    if let Some(n) = &p.strap_width {
        positive(n.get())?;
    }
    if let Some(n) = &p.strap_thickness {
        positive(n.get())?;
    }

    Ok(())
}
