//! Catalog-specific hilt proportions; attachment insertions follow grip changes.
use crate::*;

pub(super) fn fit_catalog_grip(design: &mut WeaponDesign, length: Millimeters) {
    let grip = design
        .components
        .iter_mut()
        .find(|part| part.role == ComponentRole::Grip)
        .expect("configured blade has a grip");
    let old_length = grip.shape.axial_length();
    let grip_id = grip.id.clone();
    match &mut grip.shape {
        ComponentShape::OvalGrip(value) => value.length = length,
        ComponentShape::SlabGrip(value) => value.length = length,
        _ => unreachable!("catalog blade grip"),
    }
    for part in &mut design.components {
        if let Attachment::TopOf {
            component,
            insertion,
        } = &mut part.attachment
            && *component == grip_id
            && *insertion == old_length
        {
            *insertion = length;
        }
        if let ComponentShape::KnuckleBow(value) = &mut part.shape {
            value.length = length;
        }
    }
}
