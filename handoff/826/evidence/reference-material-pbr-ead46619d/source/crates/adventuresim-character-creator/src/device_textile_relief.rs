//! Derive physical cloth relief from fitted material charts.
use crate::armor_recipes::ParametricDesign;
use anyhow::Result;
use fabelgeist_armor::{ArmorComponentRole, GeneratedArmor};

pub(crate) fn apply(design: &ParametricDesign, armor: &mut GeneratedArmor) -> Result<()> {
    for component in &mut armor.components {
        if let Some(relief) = component
            .appearance
            .textile_mut()
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
            .appearance
            .textile_mut()
            .and_then(|f| f.needlework.as_mut())
        {
            needlework.chart = fabelgeist_armor::TextileChartSize::from_surface(
                &armor.positions,
                &armor.texcoords,
                &armor.indices[component.indices.clone()],
            )?;
        }
    }
    if let ParametricDesign::Partlet(partlet) = design {
        for component in &mut armor.components {
            if component.role == ArmorComponentRole::CollarFabric
                && let Some(pleats) = partlet
                    .collar
                    .as_ref()
                    .and_then(|collar| collar.pleats.as_ref())
                && let Some(fabric) = component.appearance.textile_mut()
            {
                fabric.relief = Some(Box::new(fabelgeist_armor::TextilePleatRelief {
                    pleats: pleats.clone(),
                    layout: fabelgeist_armor::TextilePleatLayout::FreeEdge,
                    zone: None,
                    chart: fabelgeist_armor::TextileChartSize::from_surface(
                        &armor.positions,
                        &armor.texcoords,
                        &armor.indices[component.indices.clone()],
                    )?,
                }));
            }
        }
    }
    let skirt = match design {
        ParametricDesign::Skirt(skirt) => Some(skirt),
        ParametricDesign::Gown(gown) => Some(&gown.skirt),
        _ => None,
    };
    if let Some(skirt) = skirt
        && let Some(pleats) = &skirt.pleats
        && pleats.waist_depth.0 > 0
    {
        for component in &mut armor.components {
            let carrier = if skirt.puff_and_slash.is_some() {
                ArmorComponentRole::Undercloth
            } else {
                ArmorComponentRole::OuterFabric
            };
            if component.role != carrier {
                continue;
            }
            if let Some(fabric) = component.appearance.textile_mut() {
                fabric.relief = Some(Box::new(fabelgeist_armor::TextilePleatRelief {
                    pleats: fabelgeist_armor::ClothPleats {
                        count: fabelgeist_armor::TextileRepeatCount(pleats.waist_count.0),
                        depth: pleats.waist_depth,
                        profile: pleats.style.cloth_profile(),
                    },
                    chart: fabelgeist_armor::TextileChartSize::from_surface(
                        &armor.positions,
                        &armor.texcoords,
                        &armor.indices[component.indices.clone()],
                    )?,
                    zone: pleats
                        .front_style
                        .as_ref()
                        .map(fabelgeist_armor::SkirtFrontPleats::relief_zone),
                    layout: fabelgeist_armor::TextilePleatLayout::Waist {
                        front_panel: pleats.front_panel,
                        front_depth: pleats.front_depth,
                        rotation: pleats.rotation,
                        falloff: pleats.gather_falloff,
                        retention: pleats.gather_retention,
                        release: skirt.fitted_fraction,
                    },
                }));
            }
        }
    }
    Ok(())
}
