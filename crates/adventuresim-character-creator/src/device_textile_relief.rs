//! Derive physical cloth relief from fitted material charts.
use crate::armor_recipes::ParametricDesign;
use anyhow::Result;
use fabelgeist_armor::GeneratedArmor;

pub(crate) fn apply(_design: &ParametricDesign, armor: &mut GeneratedArmor) -> Result<()> {
    for component in &mut armor.components {
        if let Some(relief) = component
            .textile
            .as_mut()
            .and_then(|fabric| fabric.relief.as_mut())
        {
            relief.chart = fabelgeist_armor::TextileChartSize::from_surface(
                &armor.positions,
                &armor.texcoords,
                &armor.indices[component.indices.clone()],
            )?;
        }
    }
    for component in &mut armor.components {
        if let Some(needlework) = component
            .textile
            .as_mut()
            .and_then(|f| f.needlework.as_mut())
        {
            needlework.chart = fabelgeist_armor::TextileChartSize::from_surface(
                &armor.positions,
                &armor.texcoords,
                &armor.indices[component.indices.clone()],
            )?;
        }
    }
    Ok(())
}
