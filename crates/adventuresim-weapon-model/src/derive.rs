//! Gameplay physical values integrated by the canonical weapon construction.
use crate::*;
pub fn derive_properties(design: &WeaponDesign) -> Result<DerivedProperties, Vec<ValidationError>> {
    crate::validation::identity(design)?;
    let model = crate::model::Construction::new(&design.recipe)
        .map_err(|e| vec![ValidationError::Construction(e)])?;
    Ok(crate::mesh::derived(
        design,
        &model.physical,
        model.sources.iter().map(|part| {
            (
                part.component_id.as_str(),
                part.solid.positions.iter().copied(),
            )
        }),
    ))
}
pub fn derive_material_masses(
    design: &WeaponDesign,
) -> Result<Vec<DerivedMaterialMass>, Vec<ValidationError>> {
    crate::validation::identity(design)?;
    let model = crate::model::Construction::new(&design.recipe)
        .map_err(|e| vec![ValidationError::Construction(e)])?;
    let mut masses: Vec<DerivedMaterialMass> = Vec::new();
    for part in model.physical.components {
        if let Some(total) = masses
            .iter_mut()
            .find(|total| total.material == part.material)
        {
            total.mass_kg += part.mass_kg as f32;
        } else {
            masses.push(DerivedMaterialMass {
                material: part.material,
                mass_kg: part.mass_kg as f32,
            });
        }
    }
    Ok(masses)
}
pub fn derive_holder_properties(
    design: &WeaponHolderDesign,
) -> Result<DerivedProperties, Vec<ValidationError>> {
    crate::holders::HolderConstruction::new(design)
        .map(|holder| holder.derived)
        .map_err(|e| vec![ValidationError::Construction(e.to_string())])
}
