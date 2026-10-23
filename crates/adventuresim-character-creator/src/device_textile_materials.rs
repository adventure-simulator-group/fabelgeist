//! Textile appearance at the fitted mesh boundary; skin correspondence is done.
use crate::armor_recipes::ParametricDesign;
use anyhow::Result;
use fabelgeist_armor::{ArmorComponentRole, GeneratedArmor};

pub(crate) fn apply(design: &ParametricDesign, armor: &mut GeneratedArmor) -> Result<()> {
    for component in &mut armor.components {
        let fabric = match (design, component.role) {
            (ParametricDesign::PuffAndSlash(puff), ArmorComponentRole::Undercloth) => {
                Some(&puff.pattern.undercloth_fabric)
            }
            (ParametricDesign::PuffAndSlash(puff), ArmorComponentRole::OuterFabric) => {
                Some(&puff.pattern.outer_fabric)
            }
            (ParametricDesign::Skirt(skirt), ArmorComponentRole::Undercloth) => Some(
                &skirt
                    .puff_and_slash
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("lining without a textile construction"))?
                    .undercloth_fabric,
            ),
            (ParametricDesign::Skirt(skirt), ArmorComponentRole::OuterFabric) => Some(
                skirt
                    .puff_and_slash
                    .as_ref()
                    .map_or(&skirt.fabric, |p| &p.outer_fabric),
            ),
            (
                ParametricDesign::Skirt(skirt),
                ArmorComponentRole::Boning | ArmorComponentRole::Hoops,
            ) => {
                component.material = Some(
                    skirt
                        .support
                        .as_ref()
                        .ok_or_else(|| {
                            anyhow::anyhow!("support component without a support design")
                        })?
                        .color
                        .material(),
                );
                None
            }
            _ => None,
        };
        if let Some(fabric) = fabric {
            component.material = Some(fabric.material());
            component.textile = Some(fabric.clone());
        }
    }
    crate::device_textile_relief::apply(design, armor)?;
    armor.split_textile_seams()?;
    Ok(())
}
