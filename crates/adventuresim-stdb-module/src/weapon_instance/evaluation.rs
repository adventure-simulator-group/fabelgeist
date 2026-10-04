//! Authenticate persisted projections with one canonical construction evaluation.
use super::*;
use adventuresim_weapon_model::{EvaluatedHolder, EvaluatedWeapon};
impl WeaponInstance {
    pub(super) fn from_design(
        physical_object_id: u64,
        design: &WeaponDesign,
    ) -> Result<Self, WeaponProjectionError> {
        let evaluated =
            EvaluatedWeapon::new(design.clone()).map_err(WeaponProjectionError::Evaluation)?;
        Self::from_evaluation(physical_object_id, &evaluated)
    }
    fn from_evaluation(
        physical_object_id: u64,
        evaluated: &EvaluatedWeapon,
    ) -> Result<Self, WeaponProjectionError> {
        let design = evaluated.design();
        let derived = evaluated.derived();
        let recipe = evaluated
            .encode()
            .map_err(WeaponProjectionError::Encoding)?;
        if recipe.len() > MAX_WEAPON_RECIPE_BYTES {
            return Err(WeaponProjectionError::RecipeTooLarge(
                WeaponRecipeKind::Weapon,
            ));
        }
        let mass_grams =
            checked_scaled_u32(derived.mass_kg, 1_000.0, WeaponProjectionField::Mass)?.max(1);
        let length_mm =
            checked_scaled_u32(derived.length_m, 1_000.0, WeaponProjectionField::Length)?.max(1);
        let grip_to_tip_mm = checked_scaled_u32(
            derived.grip_to_tip_m,
            1_000.0,
            WeaponProjectionField::GripToTip,
        )?;
        Ok(WeaponInstance {
            physical_object_id,
            generator_version: GENERATOR_VERSION,
            design_hash: design_hash(design).0.to_vec(),
            recipe,
            mass_grams,
            length_mm,
            grip_to_tip_mm,
        })
    }
}
impl WeaponHolderInstance {
    pub(super) fn from_design(
        physical_object_id: u64,
        design: &WeaponHolderDesign,
    ) -> Result<Self, WeaponProjectionError> {
        let evaluated =
            EvaluatedHolder::new(design.clone()).map_err(WeaponProjectionError::Evaluation)?;
        Self::from_evaluation(physical_object_id, &evaluated)
    }
    fn from_evaluation(
        physical_object_id: u64,
        evaluated: &EvaluatedHolder,
    ) -> Result<Self, WeaponProjectionError> {
        let design = evaluated.design();
        let recipe = evaluated
            .encode()
            .map_err(WeaponProjectionError::Encoding)?;
        if recipe.len() > MAX_WEAPON_RECIPE_BYTES {
            return Err(WeaponProjectionError::RecipeTooLarge(
                WeaponRecipeKind::Holder,
            ));
        }
        let derived = evaluated.derived();
        Ok(WeaponHolderInstance {
            physical_object_id,
            generator_version: HOLDER_GENERATOR_VERSION,
            design_hash: holder_design_hash(design).0.to_vec(),
            recipe,
            mass_grams: checked_scaled_u32(
                derived.mass_kg,
                1_000.0,
                WeaponProjectionField::HolderMass,
            )?
            .max(1),
            length_mm: checked_scaled_u32(
                derived.length_m,
                1_000.0,
                WeaponProjectionField::HolderLength,
            )?
            .max(1),
            grip_to_tip_mm: checked_scaled_u32(
                derived.grip_to_tip_m,
                1_000.0,
                WeaponProjectionField::HolderGripToTip,
            )?,
        })
    }
}

pub(super) fn evaluate_instance(
    instance: &WeaponInstance,
    expected_catalog_id: &adventuresim_core::item_catalog::ItemDefinitionId,
) -> Result<EvaluatedWeapon, WeaponAuthenticationError> {
    if instance.generator_version != GENERATOR_VERSION {
        return Err(WeaponAuthenticationError::GeneratorVersion(
            WeaponRecipeKind::Weapon,
        ));
    }
    if instance.design_hash.len() != 32 {
        return Err(WeaponAuthenticationError::DigestLength(
            WeaponRecipeKind::Weapon,
        ));
    }
    if instance.recipe.len() > MAX_WEAPON_RECIPE_BYTES {
        return Err(WeaponAuthenticationError::RecipeTooLarge(
            WeaponRecipeKind::Weapon,
        ));
    }
    let evaluated =
        EvaluatedWeapon::decode(&instance.recipe).map_err(WeaponAuthenticationError::Decode)?;
    if evaluated.design().catalog_id != expected_catalog_id.as_str() {
        return Err(WeaponAuthenticationError::Chassis {
            expected: expected_catalog_id.clone(),
            actual: (&evaluated.design().catalog_id).into(),
        });
    }
    if WeaponInstance::from_evaluation(instance.physical_object_id, &evaluated)? != *instance {
        return Err(WeaponAuthenticationError::ProjectionMismatch(
            WeaponRecipeKind::Weapon,
        ));
    }
    Ok(evaluated)
}
pub(super) fn evaluate_holder_instance(
    instance: &WeaponHolderInstance,
    expected_catalog_id: &adventuresim_core::item_catalog::ItemDefinitionId,
) -> Result<EvaluatedHolder, WeaponAuthenticationError> {
    if instance.generator_version != HOLDER_GENERATOR_VERSION {
        return Err(WeaponAuthenticationError::GeneratorVersion(
            WeaponRecipeKind::Holder,
        ));
    }
    if instance.design_hash.len() != 32 {
        return Err(WeaponAuthenticationError::DigestLength(
            WeaponRecipeKind::Holder,
        ));
    }
    if instance.recipe.len() > MAX_WEAPON_RECIPE_BYTES {
        return Err(WeaponAuthenticationError::RecipeTooLarge(
            WeaponRecipeKind::Holder,
        ));
    }
    let evaluated =
        EvaluatedHolder::decode(&instance.recipe).map_err(WeaponAuthenticationError::Decode)?;
    if evaluated.design().catalog_id != expected_catalog_id.as_str() {
        return Err(WeaponAuthenticationError::Chassis {
            expected: expected_catalog_id.clone(),
            actual: (&evaluated.design().catalog_id).into(),
        });
    }
    if WeaponHolderInstance::from_evaluation(instance.physical_object_id, &evaluated)? != *instance
    {
        return Err(WeaponAuthenticationError::ProjectionMismatch(
            WeaponRecipeKind::Holder,
        ));
    }
    Ok(evaluated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authentication_preserves_decode_causes_and_distinguishes_projection_forgery() {
        use std::error::Error;
        let key = "longsword".into();
        let design = default_design("longsword").unwrap();
        let mut malformed = WeaponInstance::from_design(42, &design).unwrap();
        malformed.recipe.clear();
        let error = evaluate_instance(&malformed, &key)
            .err()
            .expect("malformed recipe must fail");
        assert!(matches!(
            &error,
            WeaponAuthenticationError::Decode(adventuresim_weapon_model::CodecError::Json(_))
        ));
        assert!(
            error
                .source()
                .unwrap()
                .is::<adventuresim_weapon_model::CodecError>()
        );

        let mut forged = WeaponInstance::from_design(42, &design).unwrap();
        forged.mass_grams += 1;
        assert!(matches!(
            evaluate_instance(&forged, &key),
            Err(WeaponAuthenticationError::ProjectionMismatch(
                WeaponRecipeKind::Weapon
            ))
        ));
    }

    #[test]
    fn authentication_checks_envelope_before_recipe_and_retains_chassis_identity() {
        let design = default_holder_design(&default_design("longsword").unwrap()).unwrap();
        let mut instance = WeaponHolderInstance::from_design(42, &design).unwrap();
        let wrong_key = "weapon_loop".into();
        assert!(matches!(evaluate_holder_instance(&instance, &wrong_key),
            Err(WeaponAuthenticationError::Chassis { expected, actual })
            if expected == wrong_key && actual.as_str() == "scabbard"));
        instance.recipe.clear();
        instance.design_hash.clear();
        instance.generator_version += 1;
        assert!(matches!(
            evaluate_holder_instance(&instance, &wrong_key),
            Err(WeaponAuthenticationError::GeneratorVersion(
                WeaponRecipeKind::Holder
            ))
        ));
        instance.generator_version = HOLDER_GENERATOR_VERSION;
        assert!(matches!(
            evaluate_holder_instance(&instance, &wrong_key),
            Err(WeaponAuthenticationError::DigestLength(
                WeaponRecipeKind::Holder
            ))
        ));
    }
    #[test]
    fn weapon_evaluation_authenticates_every_persisted_projection() {
        let design = default_design("longsword").unwrap();
        let valid = WeaponInstance::from_design(42, &design).unwrap();
        assert!(evaluate_instance(&valid, &"longsword".into()).is_ok());
        assert!(evaluate_instance(&valid, &"utility_knife".into()).is_err());
        let mutations: [fn(&mut WeaponInstance); 7] = [
            |row| row.generator_version += 1,
            |row| row.design_hash[0] ^= 1,
            |row| row.mass_grams += 1,
            |row| row.length_mm += 1,
            |row| row.grip_to_tip_mm += 1,
            |row| row.recipe.insert(0, b' '),
            |row| row.recipe.truncate(row.recipe.len() - 1),
        ];
        for mutation in mutations {
            let mut changed = valid.clone();
            mutation(&mut changed);
            assert!(evaluate_instance(&changed, &"longsword".into()).is_err());
        }
    }
    #[test]
    fn holder_evaluation_authenticates_every_persisted_projection() {
        let design = default_holder_design(&default_design("longsword").unwrap()).unwrap();
        let valid = WeaponHolderInstance::from_design(42, &design).unwrap();
        assert!(evaluate_holder_instance(&valid, &"scabbard".into()).is_ok());
        assert!(evaluate_holder_instance(&valid, &"weapon_loop".into()).is_err());
        let mutations: [fn(&mut WeaponHolderInstance); 7] = [
            |row| row.generator_version += 1,
            |row| row.design_hash[0] ^= 1,
            |row| row.mass_grams += 1,
            |row| row.length_mm += 1,
            |row| row.grip_to_tip_mm += 1,
            |row| row.recipe.insert(0, b' '),
            |row| row.recipe.truncate(row.recipe.len() - 1),
        ];
        for mutation in mutations {
            let mut changed = valid.clone();
            mutation(&mut changed);
            assert!(evaluate_holder_instance(&changed, &"scabbard".into()).is_err());
        }
    }
}
