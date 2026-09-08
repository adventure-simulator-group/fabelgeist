/// Combat-facing geometry derived from one physical weapon recipe.
///
/// Mass and rotational inertia come from the same component solids as the mesh.
/// Melee reach is the generated controlling-grip-to-tip distance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParametricWeaponCombatGeometry {
    pub mass_kg: f32,
    pub total_length_m: f32,
    pub grip_to_tip_m: f32,
    pub striking_head_length_m: f32,
    pub moment_of_inertia_kg_m2: f32,
    pub balance: f32,
    pub precision: f32,
}

impl ParametricWeaponCombatGeometry {
    pub fn new(
        mass_kg: f32,
        total_length_m: f32,
        grip_to_tip_m: f32,
        striking_head_length_m: f32,
        moment_of_inertia_kg_m2: f32,
        balance: f32,
        precision: f32,
    ) -> Option<Self> {
        let values = [
            mass_kg,
            total_length_m,
            grip_to_tip_m,
            striking_head_length_m,
            moment_of_inertia_kg_m2,
            balance,
            precision,
        ];
        (values.into_iter().all(f32::is_finite)
            && mass_kg > 0.0
            && total_length_m > 0.0
            && grip_to_tip_m > 0.0
            && grip_to_tip_m <= total_length_m
            && (0.0..=total_length_m).contains(&striking_head_length_m)
            && moment_of_inertia_kg_m2 >= 0.0
            && balance >= 0.0
            && precision >= 0.0)
            .then_some(Self {
                mass_kg,
                total_length_m,
                grip_to_tip_m,
                striking_head_length_m,
                moment_of_inertia_kg_m2,
                balance,
                precision,
            })
    }

    pub const fn melee_reach_m(self) -> f32 {
        self.grip_to_tip_m
    }

    /// Apply the existing edge-condition factor once to this instance's geometry.
    pub fn conditioned_precision(self, catalog_id: &str, effective_catalog_precision: f32) -> f32 {
        let base = crate::item_catalog::weapon_precision(catalog_id)
            .filter(|value| *value > 0.0)
            .expect("generated weapon has positive catalog precision");
        self.precision * effective_catalog_precision / base
    }
}
