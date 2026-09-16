//! Bounds for interpolated visible grip dimensions and their material inset.
use super::*;

pub(super) fn check(p: &ProfileGripParameters) -> Checked {
    positive(p.length.get())?;
    require(
        (2..=MAX_GRIP_PROFILE_STATIONS).contains(&p.profile.len()),
        RecipeError::Budget,
    )?;
    require(
        p.profile[0].at.get() == 0.0 && p.profile.last().unwrap().at.get() == 1.0,
        RecipeError::Profile,
    )?;
    require(
        p.profile.windows(2).all(|s| s[1].at.get() > s[0].at.get()),
        RecipeError::Profile,
    )?;
    for s in &p.profile {
        positive(s.width.get())?;
        positive(s.depth.get())?;
        require(
            s.width.get() <= MAX_SWORD_GRIP_WIDTH && s.depth.get() <= MAX_SWORD_GRIP_THICKNESS,
            RecipeError::Grip,
        )?;
    }
    if let Some(cover) = &p.cover {
        positive(cover.thickness.get())?;
        // Monotone interpolation has no interior extrema beyond its endpoints.
        // This conservative interval bound also keeps the ellipse inset below
        // its smallest radius of curvature throughout every profile interval.
        for pair in p.profile.windows(2) {
            let small = pair
                .iter()
                .flat_map(|s| [s.width.get(), s.depth.get()])
                .fold(f64::INFINITY, f64::min)
                / 2.0;
            let large = pair
                .iter()
                .flat_map(|s| [s.width.get(), s.depth.get()])
                .fold(0.0, f64::max)
                / 2.0;
            clearance(cover.thickness.get() < small * small / large)?;
        }
    }
    Ok(())
}
