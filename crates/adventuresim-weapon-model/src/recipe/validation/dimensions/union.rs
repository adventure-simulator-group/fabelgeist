//! Dimension and sampling bounds for union constructions.
use super::*;
pub(super) fn bent_bar(p: &BentBarParameters) -> Checked {
    positive(p.radius.get())?;
    require(
        p.samples.0 <= MAX_SAMPLING_REQUEST && p.radial_segments.0 <= MAX_SAMPLING_REQUEST,
        RecipeError::Budget,
    )?;
    match p.centerline {
        BarCenterline::Opposed { span, sweep } => {
            positive(span.get())?;
            bounded(sweep.get())?;
        }
        BarCenterline::Arch {
            width,
            length,
            bulge,
            ..
        } => {
            positive(width.get())?;
            positive(length.get())?;
            bounded(bulge.get())?;
        }
    }
    Ok(())
}
pub(super) fn spatial_tube(p: &SpatialTubeParameters) -> Checked {
    positive(p.radius.get())?;
    require(
        (2..=MAX_AUTHORED_STATIONS).contains(&p.points.len())
            && p.radial_segments.0 <= MAX_SAMPLING_REQUEST,
        RecipeError::Budget,
    )?;
    for point in &p.points {
        for n in point {
            bounded(n.get())?;
        }
    }

    Ok(())
}
pub(super) fn lofted_blade(p: &LoftedBladeParameters) -> Checked {
    positive(p.length.get())?;
    positive(p.width.get())?;
    positive(p.thickness.get())?;
    bounded(p.curvature.get())?;
    proportion(
        (0.0..p.length.get()).contains(&p.ricasso.get())
            && p.taper.get() > 0.0
            && p.belly.get() > -1.0,
    )?;
    require(p.samples.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;

    Ok(())
}
pub(super) fn shaft(p: &Shaft) -> Checked {
    positive(p.length.get())?;
    positive(p.radius.get())?;
    require(
        p.segments.is_none_or(|n| n.0 <= MAX_SAMPLING_REQUEST),
        RecipeError::Budget,
    )?;

    Ok(())
}
