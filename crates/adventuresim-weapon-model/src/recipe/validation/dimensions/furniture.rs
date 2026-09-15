//! Dimension and sampling bounds for furniture constructions.
use super::*;
pub(super) fn cuboid(p: &BoxParameters) -> Checked {
    let n = &p.size;
    for v in n {
        positive(v.get())?;
    }

    Ok(())
}
pub(super) fn socket(p: &SocketParameters) -> Checked {
    {
        let n = &p.profile;
        require(n.len() <= MAX_AUTHORED_STATIONS, RecipeError::Budget)?;
    }
    if let Some(n) = &p.segments {
        require(
            (3..=MAX_SAMPLING_REQUEST).contains(&n.0),
            RecipeError::Budget,
        )?;
    }
    if let Some(n) = &p.wall {
        positive(n.get())?;
    }

    Ok(())
}
pub(super) fn pommel(p: &PommelParameters) -> Checked {
    if let Some(n) = &p.profile {
        require(n.len() <= MAX_AUTHORED_STATIONS, RecipeError::Budget)?;
    }
    if let Some(n) = &p.segments {
        require(
            (3..=MAX_SAMPLING_REQUEST).contains(&n.0),
            RecipeError::Budget,
        )?;
    }
    if let Some(n) = &p.width_scale {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.length_scale {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.height {
        positive(n.get())?;
    }
    if let Some(n) = &p.diameter {
        bounded(n.get())?;
    }
    if let Some(n) = &p.thickness {
        positive(n.get())?;
    }
    if let Some(n) = &p.face_convexity {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.rim_bevel {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.facets {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }
    if let Some(n) = &p.flute_count {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }
    if let Some(n) = &p.flute_depth {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.twist {
        require(n.get().abs() <= 3600.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.notch_depth {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.lobe_spread {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.shoulder_width {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.ornaments {
        require(n.len() <= MAX_AUTHORED_STATIONS, RecipeError::Budget)?;
    }

    Ok(())
}
pub(super) fn collar(p: &CollarParameters) -> Checked {
    positive(p.width.get())?;
    positive(p.radius.get())?;
    if let Some(n) = &p.segments {
        require(
            (3..=MAX_SAMPLING_REQUEST).contains(&n.0),
            RecipeError::Budget,
        )?;
    }

    Ok(())
}
pub(super) fn sleeve(p: &SleeveParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.radius.get())?;
    if let Some(n) = &p.top_radius {
        bounded(n.get())?;
    }
    if let Some(n) = &p.segments {
        require(
            (3..=MAX_SAMPLING_REQUEST).contains(&n.0),
            RecipeError::Budget,
        )?;
    }
    if let Some(n) = &p.wall {
        positive(n.get())?;
    }

    Ok(())
}
pub(super) fn mace(p: &MaceParameters) -> Checked {
    positive(p.length.get())?;
    bounded(p.root_radius.get())?;
    bounded(p.shoulder_radius.get())?;
    bounded(p.cusp_radius.get())?;
    if let Some(n) = &p.cusp_height {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.concavity {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.crown_length {
        bounded(n.get())?;
    }
    require(p.flanges.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    positive(p.flange_thickness.get())?;
    if let Some(n) = &p.profile_samples {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }
    if let Some(n) = &p.segments {
        require(
            (3..=MAX_SAMPLING_REQUEST).contains(&n.0),
            RecipeError::Budget,
        )?;
    }
    if let Some(n) = &p.waist {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.flange_depth {
        bounded(n.get())?;
    }

    Ok(())
}
pub(super) fn grip(p: &GripParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.radius.get())?;
    if let Some(n) = &p.bottom_scale {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.top_scale {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.wraps {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }
    if let Some(n) = &p.segments {
        require(
            (3..=MAX_SAMPLING_REQUEST).contains(&n.0),
            RecipeError::Budget,
        )?;
    }

    Ok(())
}
pub(super) fn oval_grip(p: &OvalGripParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.width.get())?;
    positive(p.thickness.get())?;
    if let Some(n) = &p.bottom_scale {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.top_scale {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.segments {
        require(
            (3..=MAX_SAMPLING_REQUEST).contains(&n.0),
            RecipeError::Budget,
        )?;
    }

    Ok(())
}
pub(super) fn slab_grip(p: &SlabGripParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.width.get())?;
    positive(p.thickness.get())?;
    if let Some(n) = &p.scale_thickness {
        bounded(n.get())?;
    }

    Ok(())
}
