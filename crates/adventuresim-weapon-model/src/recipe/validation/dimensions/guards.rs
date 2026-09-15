//! Dimension and sampling bounds for guards constructions.
use super::*;
pub(super) fn guard(p: &GuardParameters) -> Checked {
    positive(p.width.get())?;
    positive(p.height.get())?;
    positive(p.thickness.get())?;
    if let Some(n) = &p.sweep {
        bounded(n.get())?;
    }
    if let Some(n) = &p.tip_scale {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.terminal_swell {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.left_length {
        bounded(n.get())?;
    }
    if let Some(n) = &p.right_length {
        bounded(n.get())?;
    }
    if let Some(n) = &p.left_sweep {
        bounded(n.get())?;
    }
    if let Some(n) = &p.right_sweep {
        bounded(n.get())?;
    }
    if let Some(n) = &p.left_set {
        bounded(n.get())?;
    }
    if let Some(n) = &p.right_set {
        bounded(n.get())?;
    }
    if let Some(n) = &p.section_width {
        bounded(n.get())?;
    }
    if let Some(n) = &p.section_depth {
        bounded(n.get())?;
    }
    if let Some(n) = &p.section_twist {
        require(n.get().abs() <= 3600.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.terminal_size {
        bounded(n.get())?;
    }

    Ok(())
}
pub(super) fn guard_assembly(p: &GuardAssemblyParameters) -> Checked {
    {
        let n = &p.members;
        require(n.len() <= MAX_AUTHORED_STATIONS, RecipeError::Budget)?;
    }
    if let Some(n) = &p.plates {
        require(n.len() <= MAX_AUTHORED_STATIONS, RecipeError::Budget)?;
    }

    Ok(())
}
pub(super) fn knuckle_bow(p: &KnuckleBowParameters) -> Checked {
    positive(p.width.get())?;
    positive(p.length.get())?;
    if let Some(n) = &p.bar {
        bounded(n.get())?;
    }
    if let Some(n) = &p.thickness {
        positive(n.get())?;
    }
    if let Some(n) = &p.bulge {
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
pub(super) fn ring_guard(p: &RingGuardParameters) -> Checked {
    positive(p.radius.get())?;
    if let Some(n) = &p.bar {
        bounded(n.get())?;
    }
    if let Some(n) = &p.arc_start {
        require(n.get().abs() <= 3600.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.arc_end {
        require(n.get().abs() <= 3600.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.samples {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }
    if let Some(n) = &p.radial_segments {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }

    Ok(())
}
pub(super) fn tube(p: &TubeParameters) -> Checked {
    {
        let n = &p.points;
        require(n.len() <= MAX_AUTHORED_STATIONS, RecipeError::Budget)?;
    }
    positive(p.radius.get())?;
    if let Some(n) = &p.radial_segments {
        require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
    }

    Ok(())
}
pub(super) fn figure_eight(p: &FigureEightParameters) -> Checked {
    positive(p.width.get())?;
    if let Some(n) = &p.height {
        positive(n.get())?;
    }
    if let Some(n) = &p.bar {
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
