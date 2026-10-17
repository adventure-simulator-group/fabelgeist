//! Textile appearance at the fitted mesh boundary; skin correspondence is done.
use crate::armor_recipes::ParametricDesign;
use anyhow::{Context, Result};
use fabelgeist_armor::{
    ArmorComponent, ArmorComponentRole, BodiceDesign, GeneratedArmor, SkirtDesign,
    TextileAppearance,
};

pub(crate) fn apply(design: &ParametricDesign, armor: &mut GeneratedArmor) -> Result<()> {
    for component in &mut armor.components {
        let fabric = match design {
            ParametricDesign::Bodice(bodice) => bodice_fabric(bodice, component.role)?,
            ParametricDesign::Partlet(partlet) => match component.role {
                ArmorComponentRole::OuterFabric => Some(&partlet.fabric),
                ArmorComponentRole::CollarFabric => partlet.collar.as_ref().map(|c| &c.fabric),
                ArmorComponentRole::EdgeBinding => partlet.binding.as_ref().map(|b| &b.fabric),
                ArmorComponentRole::ClothTies => partlet.ties.as_ref().map(|t| &t.fabric),
                ArmorComponentRole::Neckband => partlet
                    .collar
                    .as_ref()
                    .and_then(|c| c.neckband.as_ref())
                    .map(|b| &b.fabric),
                ArmorComponentRole::CollarTrim => partlet
                    .collar
                    .as_ref()
                    .and_then(|c| c.trim.as_ref())
                    .map(|b| &b.fabric),
                _ => None,
            },
            ParametricDesign::Gown(gown) => match component.role {
                ArmorComponentRole::BodiceFabric
                | ArmorComponentRole::Lining
                | ArmorComponentRole::Interlining => bodice_fabric(&gown.bodice, component.role)?,
                _ => skirt_fabric(&gown.skirt, component)?,
            },
            ParametricDesign::Skirt(skirt) => skirt_fabric(skirt, component)?,
            ParametricDesign::PuffAndSlash(puff) => match component.role {
                ArmorComponentRole::Undercloth => Some(&puff.pattern.undercloth_fabric),
                ArmorComponentRole::OuterFabric => Some(&puff.pattern.outer_fabric),
                _ => None,
            },
            _ => None,
        };
        if let Some(fabric) = fabric {
            component.material = Some(fabric.material());
            component.textile = Some(fabric.clone());
        }
    }
    armor.split_textile_seams()?;
    crate::device_textile_relief::apply(design, armor)?;
    Ok(())
}

fn bodice_fabric(
    bodice: &BodiceDesign,
    role: ArmorComponentRole,
) -> Result<Option<&TextileAppearance>> {
    Ok(match role {
        ArmorComponentRole::OuterFabric | ArmorComponentRole::BodiceFabric => Some(&bodice.fabric),
        ArmorComponentRole::Lining => Some(
            &bodice
                .lining
                .as_ref()
                .context("lining without a bodice lining design")?
                .fabric,
        ),
        ArmorComponentRole::Interlining => Some(
            &bodice
                .interlining
                .as_ref()
                .context("interlining without a bodice stiffening design")?
                .fabric,
        ),
        _ => None,
    })
}

fn skirt_fabric<'a>(
    skirt: &'a SkirtDesign,
    component: &mut ArmorComponent,
) -> Result<Option<&'a TextileAppearance>> {
    Ok(match component.role {
        ArmorComponentRole::BodiceFabric => Some(&skirt.fabric),
        ArmorComponentRole::Undercloth => Some(
            &skirt
                .puff_and_slash
                .as_ref()
                .context("undercloth without a textile construction")?
                .undercloth_fabric,
        ),
        ArmorComponentRole::OuterFabric => Some(
            skirt
                .puff_and_slash
                .as_ref()
                .map_or(&skirt.fabric, |p| &p.outer_fabric),
        ),
        ArmorComponentRole::Boning | ArmorComponentRole::Hoops => {
            component.material = Some(
                skirt
                    .support
                    .as_ref()
                    .context("support component without a support design")?
                    .color
                    .material(),
            );
            None
        }
        _ => None,
    })
}
