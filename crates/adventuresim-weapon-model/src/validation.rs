//! Gameplay identity and holder constraints around canonical manufacturing validation.
use crate::{
    ComponentRole, WeaponDesign, WeaponHolderDesign, WeaponHolderKind, recommended_holder,
};
use thiserror::Error;
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ValidationError {
    #[error("weapon catalog ID is empty or exceeds its transport limit")]
    CatalogIdentity,
    #[error("invalid canonical weapon recipe: {0}")]
    Recipe(#[from] crate::recipe::RecipeError),
    #[error("weapon construction failed: {0}")]
    Construction(String),
    #[error("invalid holder {0}")]
    Holder(&'static str),
}
pub(crate) fn identity(design: &WeaponDesign) -> Result<(), Vec<ValidationError>> {
    if design.catalog_id.is_empty() || design.catalog_id.len() > 128 {
        return Err(vec![ValidationError::CatalogIdentity]);
    }
    design
        .recipe
        .validate()
        .map_err(|error| vec![ValidationError::Recipe(error)])
}
pub fn validate(design: &WeaponDesign) -> Result<(), Vec<ValidationError>> {
    identity(design)?;
    crate::model::Construction::new(&design.recipe)
        .map(|_| ())
        .map_err(|error| vec![ValidationError::Construction(error)])
}
pub fn validate_holder(design: &WeaponHolderDesign) -> Result<(), Vec<ValidationError>> {
    crate::holders::HolderConstruction::new(design)
        .map(|_| ())
        .map_err(|error| vec![ValidationError::Construction(error.to_string())])
}
pub(crate) fn holder_identity(design: &WeaponHolderDesign) -> Result<(), Vec<ValidationError>> {
    identity(&design.fitted_weapon)?;
    let expected = recommended_holder(&design.fitted_weapon.catalog_id);
    if expected != Some(design.kind)
        || !matches!(
            (design.kind, design.catalog_id.as_str()),
            (WeaponHolderKind::BladeSheath, "scabbard")
                | (WeaponHolderKind::HaftLoop, "weapon_loop")
        )
    {
        return Err(vec![ValidationError::Holder("kind")]);
    }
    if !(1..=10).contains(&design.wall_thickness.0)
        || !(2..=20).contains(&design.clearance.0)
        || !(4..=40).contains(&design.throat_length.0)
        || !(6..=60).contains(&design.chape_length.0)
        || design.loop_position.0 > 1000
        || !(2..=12).contains(&design.loop_bar_radius.0)
        || !(20..=120).contains(&design.hanger_width.0)
        || !(30..=180).contains(&design.hanger_height.0)
    {
        return Err(vec![ValidationError::Holder("parameters")]);
    }
    let components = &design.fitted_weapon.recipe.components;
    if !components
        .iter()
        .any(|part| part.role == Some(ComponentRole::Grip))
        || (design.kind == WeaponHolderKind::BladeSheath
            && !components
                .iter()
                .any(|part| matches!(part.shape, crate::recipe::Shape::LoftedBlade(_))))
    {
        return Err(vec![ValidationError::Holder("source geometry")]);
    }
    Ok(())
}
