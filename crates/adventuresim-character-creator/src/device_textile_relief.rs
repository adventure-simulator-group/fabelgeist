//! Derive physical cloth relief from fitted material charts.
use crate::armor_recipes::ParametricDesign;
use anyhow::Result;
use fabelgeist_armor::{ArmorComponentRole, GeneratedArmor, SkirtDesign};

pub(crate) fn apply(design: &ParametricDesign, armor: &mut GeneratedArmor) -> Result<()> {
    crate::chart_column_spacing::even_out(armor);
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
            let indices = &armor.indices[component.indices.clone()];
            needlework.chart = fabelgeist_armor::TextileChartSize::from_surface(
                &armor.positions,
                &armor.texcoords,
                indices,
            )?;
            // Each region that carries ornament is measured where it lies.
            needlework.regions = needlework
                .ornamented_regions()
                .into_iter()
                .filter_map(|region| {
                    let size = fabelgeist_armor::TextileChartSize::of_region(
                        &armor.positions,
                        &armor.texcoords,
                        indices,
                        &region,
                    )?;
                    Some(fabelgeist_armor::TextileRegionSize { region, size })
                })
                .collect();
        }
    }
    if let ParametricDesign::Partlet(partlet) = design {
        for component in &mut armor.components {
            if component.role == ArmorComponentRole::CollarFabric
                && let Some(pleats) = partlet
                    .collar
                    .as_ref()
                    .and_then(|collar| collar.pleats.as_ref())
                && let Some(fabric) = &mut component.textile
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
    match design {
        ParametricDesign::Skirt(skirt) => waist_gathers(skirt, armor),
        ParametricDesign::Gown(gown) => waist_gathers(&gown.skirt, armor),
        _ => Ok(()),
    }
}

/// Fine gathering at a skirt's waist, as relief on the cloth that hangs from
/// it. The fitted surface supplies the chart and how far it narrows there.
fn waist_gathers(skirt: &SkirtDesign, armor: &mut GeneratedArmor) -> Result<()> {
    let Some(pleats) = skirt.pleats.as_ref().filter(|p| p.waist_depth.0 > 0) else {
        return Ok(());
    };
    let carrier = if skirt.puff_and_slash.is_some() {
        ArmorComponentRole::Undercloth
    } else {
        ArmorComponentRole::OuterFabric
    };
    for component in &mut armor.components {
        if component.role != carrier {
            continue;
        }
        let Some(fabric) = &mut component.textile else {
            continue;
        };
        let indices = &armor.indices[component.indices.clone()];
        fabric.relief = Some(Box::new(fabelgeist_armor::TextilePleatRelief {
            pleats: fabelgeist_armor::ClothPleats {
                count: fabelgeist_armor::TextileRepeatCount(pleats.waist_count.0),
                depth: pleats.waist_depth,
                profile: pleats.style.cloth_profile(),
            },
            chart: fabelgeist_armor::TextileChartSize::from_surface(
                &armor.positions,
                &armor.texcoords,
                indices,
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
                waist_width: fabelgeist_armor::TextileChartSize::start_width(
                    &armor.positions,
                    &armor.texcoords,
                    indices,
                )?,
            },
        }));
    }
    Ok(())
}
