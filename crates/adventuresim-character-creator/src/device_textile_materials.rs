//! Textile and structural component materials at the generated mesh boundary.

use crate::armor_recipes::ParametricDesign;
use anyhow::Result;
use fabelgeist_armor::{ArmorComponentRole, GeneratedArmor};

pub(crate) fn apply(design: &ParametricDesign, armor: &mut GeneratedArmor) -> Result<()> {
    if let ParametricDesign::PuffAndSlash(puff) = design {
        for component in &mut armor.components {
            component.material = match component.role {
                ArmorComponentRole::Undercloth => Some(puff.pattern.undercloth_color.material()),
                ArmorComponentRole::OuterFabric => Some(puff.pattern.outer_color.material()),
                _ => component.material,
            };
        }
    }
    if let ParametricDesign::Skirt(skirt) = design {
        for component in &mut armor.components {
            component.material = Some(match component.role {
                ArmorComponentRole::Boning | ArmorComponentRole::Hoops => skirt
                    .support
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("support component without a support design"))?
                    .color
                    .material(),
                ArmorComponentRole::Undercloth => skirt
                    .puff_and_slash
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("lining without a textile pattern"))?
                    .undercloth_color
                    .material(),
                ArmorComponentRole::OuterFabric => skirt
                    .puff_and_slash
                    .as_ref()
                    .map_or(skirt.color, |pattern| pattern.outer_color)
                    .material(),
                _ => skirt.color.material(),
            });
        }
    }
    Ok(())
}
