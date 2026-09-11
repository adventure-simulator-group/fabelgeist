//! Fit independent cranial, jaw and neck sections to the reference wearer.
use crate::armor_frames::{FitRegion, Wearer};
use adventuresim_armor_model::{
    CloseHelmetDesign, CloseHelmetProfile, PartMesh, generate_close_helmet,
};
use anyhow::Result;

pub fn fit(design: &CloseHelmetDesign, wearer: &Wearer<'_>) -> Result<PartMesh> {
    let frame = wearer.frame(FitRegion::Head)?;
    let mut support = wearer.support_indices(FitRegion::Head)?;
    support.extend(wearer.support_indices(FitRegion::Neck)?);
    support.sort_unstable();
    support.dedup();
    let samples: Vec<_> = support.iter().map(|i| wearer.positions[*i]).collect();
    let profile = CloseHelmetProfile::from_samples(design, &frame, &samples)?;
    Ok(generate_close_helmet(design, &frame, &profile)?)
}
