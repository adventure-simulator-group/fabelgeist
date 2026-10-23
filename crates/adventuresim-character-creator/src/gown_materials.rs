//! Each gown construction selects its own cloth without replacing fitted data.
use crate::device_textile_materials::{bodice_fabric, skirt_fabric};
use anyhow::Result;
use fabelgeist_armor::{ArmorComponent, ArmorComponentRole, GownDesign, TextileAppearance};

pub(crate) fn fabric<'a>(
    gown: &'a GownDesign,
    component: &mut ArmorComponent,
) -> Result<Option<&'a TextileAppearance>> {
    Ok(match component.role {
        ArmorComponentRole::BodiceFabric
        | ArmorComponentRole::Lining
        | ArmorComponentRole::Interlining => bodice_fabric(&gown.bodice, component.role)?,
        _ => skirt_fabric(&gown.skirt, component)?,
    })
}
