//! Dimension and sampling bounds for blades constructions.
use super::*;
pub(super) fn blade(p: &BladeParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.width.get())?;
    positive(p.thickness.get())?;
    if let Some(n) = &p.curvature {
        bounded(n.get())?;
    }
    if let Some(n) = &p.taper {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.single_edge {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.tip_width {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.belly {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }

    Ok(())
}
pub(super) fn section_blade(p: &SectionBladeParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.width.get())?;
    positive(p.thickness.get())?;
    if let Some(n) = &p.taper {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }

    Ok(())
}
pub(super) fn diamond_blade(p: &DiamondBladeParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.width.get())?;
    positive(p.thickness.get())?;
    if let Some(n) = &p.taper {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }

    Ok(())
}
pub(super) fn axe(p: &AxeParameters) -> Checked {
    positive(p.width.get())?;
    positive(p.height.get())?;
    positive(p.thickness.get())?;
    if let Some(n) = &p.beard {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.curvature {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.root_width {
        positive(n.get())?;
    }
    if let Some(n) = &p.upper_shoulder {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.lower_shoulder {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.flare {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.toe {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.heel {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.beard_drop {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.upper_cusp {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.lower_cusp {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }

    Ok(())
}
pub(super) fn spear(p: &SpearParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.width.get())?;
    positive(p.thickness.get())?;
    if let Some(n) = &p.shoulder {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.root_width {
        positive(n.get())?;
    }
    if let Some(n) = &p.belly_position {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.acuteness {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }

    Ok(())
}
pub(super) fn fork(p: &ForkParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.width.get())?;
    positive(p.base_width.get())?;
    positive(p.thickness.get())?;
    if let Some(n) = &p.crotch {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.tine_width {
        positive(n.get())?;
    }
    if let Some(n) = &p.tine_taper {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.shoulder_blend {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.crotch_round {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }

    Ok(())
}
pub(super) fn partisan(p: &PartisanParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.width.get())?;
    bounded(p.lug_width.get())?;
    positive(p.thickness.get())?;
    if let Some(n) = &p.lug_drop {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.belly_position {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.root_width {
        positive(n.get())?;
    }
    if let Some(n) = &p.lug_sweep {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.acuteness {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }

    Ok(())
}
pub(super) fn glaive(p: &GlaiveParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.width.get())?;
    positive(p.thickness.get())?;
    if let Some(n) = &p.curvature {
        bounded(n.get())?;
    }
    if let Some(n) = &p.root {
        bounded(n.get())?;
    }
    if let Some(n) = &p.edge_curvature {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.spine_curvature {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.belly_position {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.point_length {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.root_length {
        bounded(n.get())?;
    }

    Ok(())
}
pub(super) fn faceted_beak(p: &FacetedBeakParameters) -> Checked {
    positive(p.length.get())?;
    bounded(p.root.get())?;
    if let Some(n) = &p.tip {
        bounded(n.get())?;
    }
    positive(p.thickness.get())?;
    if let Some(n) = &p.set {
        bounded(n.get())?;
    }
    if let Some(n) = &p.bend_position {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.tip_thickness {
        bounded(n.get())?;
    }

    Ok(())
}
pub(super) fn bill(p: &BillParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.width.get())?;
    bounded(p.hook.get())?;
    positive(p.thickness.get())?;
    if let Some(n) = &p.root {
        bounded(n.get())?;
    }
    if let Some(n) = &p.root_length {
        bounded(n.get())?;
    }
    if let Some(n) = &p.belly_position {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.hook_depth {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.hook_curvature {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.point_length {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }

    Ok(())
}
pub(super) fn pick(p: &PickParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.radius.get())?;

    Ok(())
}
pub(super) fn beak(p: &BeakParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.radius.get())?;
    if let Some(n) = &p.curvature {
        bounded(n.get())?;
    }
    if let Some(n) = &p.thickness {
        positive(n.get())?;
    }
    if let Some(n) = &p.root_section {
        positive(n.get())?;
    }
    if let Some(n) = &p.tip_section {
        positive(n.get())?;
    }
    if let Some(n) = &p.bend_position {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.droop {
        bounded(n.get())?;
    }

    Ok(())
}
pub(super) fn hammer(p: &HammerParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.face.get())?;
    positive(p.neck.get())?;
    positive(p.thickness.get())?;
    if let Some(n) = &p.crown {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.neck_ratio {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.face_flare {
        require(n.get().abs() <= 128.0, RecipeError::Dimension)?;
    }
    if let Some(n) = &p.crown_length {
        bounded(n.get())?;
    }
    if let Some(n) = &p.face_thickness {
        positive(n.get())?;
    }

    Ok(())
}
