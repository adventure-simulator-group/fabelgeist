//! Authenticate persisted projections with one canonical construction evaluation.
use super::*;
use adventuresim_weapon_model::{EvaluatedHolder, EvaluatedWeapon};
impl WeaponInstance {
    pub(super) fn from_design(
        physical_object_id: u64,
        design: &WeaponDesign,
    ) -> Result<Self, String> {
        let evaluated = EvaluatedWeapon::new(design.clone()).map_err(|error| error.to_string())?;
        Self::from_evaluation(physical_object_id, &evaluated)
    }
    fn from_evaluation(
        physical_object_id: u64,
        evaluated: &EvaluatedWeapon,
    ) -> Result<Self, String> {
        let design = evaluated.design();
        let derived = evaluated.derived();
        let recipe = evaluated.encode().map_err(|error| error.to_string())?;
        if recipe.len() > MAX_WEAPON_RECIPE_BYTES {
            return Err("Weapon recipe exceeds the tactical transport limit".into());
        }
        let mass_grams = checked_scaled_u32(derived.mass_kg, 1_000.0, "mass")?.max(1);
        let length_mm = checked_scaled_u32(derived.length_m, 1_000.0, "length")?.max(1);
        let grip_to_tip_mm =
            checked_scaled_u32(derived.grip_to_tip_m, 1_000.0, "grip-to-tip distance")?;
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
    ) -> Result<Self, String> {
        let evaluated = EvaluatedHolder::new(design.clone()).map_err(|error| error.to_string())?;
        Self::from_evaluation(physical_object_id, &evaluated)
    }
    fn from_evaluation(
        physical_object_id: u64,
        evaluated: &EvaluatedHolder,
    ) -> Result<Self, String> {
        let design = evaluated.design();
        let recipe = evaluated.encode().map_err(|error| error.to_string())?;
        if recipe.len() > MAX_WEAPON_RECIPE_BYTES {
            return Err("Weapon holder recipe exceeds the tactical transport limit".into());
        }
        let derived = evaluated.derived();
        Ok(WeaponHolderInstance {
            physical_object_id,
            generator_version: HOLDER_GENERATOR_VERSION,
            design_hash: holder_design_hash(design).0.to_vec(),
            recipe,
            mass_grams: checked_scaled_u32(derived.mass_kg, 1_000.0, "holder mass")?.max(1),
            length_mm: checked_scaled_u32(derived.length_m, 1_000.0, "holder length")?.max(1),
            grip_to_tip_mm: checked_scaled_u32(
                derived.grip_to_tip_m,
                1_000.0,
                "holder anchor-to-tip distance",
            )?,
        })
    }
}

pub(super) fn evaluate_instance(
    instance: &WeaponInstance,
    expected_catalog_id: &str,
) -> Option<EvaluatedWeapon> {
    if instance.generator_version != GENERATOR_VERSION
        || instance.design_hash.len() != 32
        || instance.recipe.len() > MAX_WEAPON_RECIPE_BYTES
    {
        return None;
    }
    let evaluated = EvaluatedWeapon::decode(&instance.recipe).ok()?;
    if evaluated.design().catalog_id != expected_catalog_id {
        return None;
    }
    (WeaponInstance::from_evaluation(instance.physical_object_id, &evaluated).ok()? == *instance)
        .then_some(evaluated)
}
pub(super) fn evaluate_holder_instance(
    instance: &WeaponHolderInstance,
    expected_catalog_id: &str,
) -> Option<EvaluatedHolder> {
    if instance.generator_version != HOLDER_GENERATOR_VERSION
        || instance.design_hash.len() != 32
        || instance.recipe.len() > MAX_WEAPON_RECIPE_BYTES
    {
        return None;
    }
    let evaluated = EvaluatedHolder::decode(&instance.recipe).ok()?;
    if evaluated.design().catalog_id != expected_catalog_id {
        return None;
    }
    (WeaponHolderInstance::from_evaluation(instance.physical_object_id, &evaluated).ok()?
        == *instance)
        .then_some(evaluated)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn weapon_evaluation_authenticates_every_persisted_projection() {
        let design = default_design("longsword").unwrap();
        let valid = WeaponInstance::from_design(42, &design).unwrap();
        assert!(evaluate_instance(&valid, "longsword").is_some());
        assert!(evaluate_instance(&valid, "utility_knife").is_none());
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
            assert!(evaluate_instance(&changed, "longsword").is_none());
        }
    }
    #[test]
    fn holder_evaluation_authenticates_every_persisted_projection() {
        let design = default_holder_design(&default_design("longsword").unwrap()).unwrap();
        let valid = WeaponHolderInstance::from_design(42, &design).unwrap();
        assert!(evaluate_holder_instance(&valid, "scabbard").is_some());
        assert!(evaluate_holder_instance(&valid, "weapon_loop").is_none());
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
            assert!(evaluate_holder_instance(&changed, "scabbard").is_none());
        }
    }
}
