use crate::ComponentShape;
use crate::mesh::construction::ConstructedWeapon;
use crate::{
    DerivedProperties, ValidationError, WeaponDesign, WeaponHolderDesign, WeaponHolderKind,
    validate_holder,
};

pub fn derive_holder_properties(
    design: &WeaponHolderDesign,
) -> Result<DerivedProperties, Vec<ValidationError>> {
    validate_holder(design)?;
    let weapon = derive_properties(&design.fitted_weapon)?;
    let (mass_kg, length_m) = match design.kind {
        WeaponHolderKind::BladeSheath => {
            let blade_volume = design
                .fitted_weapon
                .components
                .iter()
                .filter(|component| matches!(&component.shape, ComponentShape::Blade(_)))
                .map(|component| {
                    crate::mass_properties::SolidMoments::new(
                        &crate::mesh::RawMesh::from_shape(&component.shape),
                        [0.0; 3],
                    )
                    .volume_m3 as f32
                })
                .sum::<f32>();
            let length = weapon.grip_to_tip_m + design.chape_length.meters() * 0.5;
            let leather = blade_volume * 0.22 * design.body_material.density_kg_m3();
            let fittings = blade_volume * 0.035 * design.fitting_material.density_kg_m3();
            ((leather + fittings).max(0.08), length)
        }
        WeaponHolderKind::HaftLoop => {
            let bar = design.loop_bar_radius.meters();
            let path = std::f32::consts::PI
                * (design.hanger_width.meters() + design.hanger_height.meters())
                + 0.16;
            let mass =
                path * std::f32::consts::PI * bar.powi(2) * design.body_material.density_kg_m3();
            (mass.max(0.04), design.hanger_height.meters())
        }
    };
    Ok(DerivedProperties {
        mass_kg,
        length_m,
        grip_to_tip_m: 0.0,
        striking_head_length_m: 0.0,
        center_of_mass_from_grip_m: 0.0,
        moment_of_inertia_kg_m2: 0.0,
        balance: 1.0,
    })
}

/// Integrates the canonical recipe solids using their material densities.
/// No rendering normals, shading buffers, or independent shape approximations are used.
pub fn derive_properties(design: &WeaponDesign) -> Result<DerivedProperties, Vec<ValidationError>> {
    Ok(ConstructedWeapon::new(design)?.properties())
}

/// Per-material construction mass, conserving the weapon's total derived mass.
pub fn derive_material_masses(
    design: &WeaponDesign,
) -> Result<Vec<crate::DerivedMaterialMass>, Vec<ValidationError>> {
    Ok(ConstructedWeapon::new(design)?.material_masses())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_material_mass_conserves_total_recipe_mass() {
        for catalog_id in crate::MELEE_CATALOG_IDS {
            let design = crate::default_design(catalog_id).unwrap();
            let total = derive_properties(&design).unwrap().mass_kg;
            let by_material = derive_material_masses(&design)
                .unwrap()
                .into_iter()
                .map(|mass| mass.mass_kg)
                .sum::<f32>();
            assert!((total - by_material).abs() < 0.000_01, "{catalog_id}");
        }
    }

    #[test]
    fn per_instance_polearm_length_changes_mesh_and_combat_geometry_together() {
        let short = crate::default_design("halberd").unwrap();
        let mut long = short.clone();
        let shaft = long
            .components
            .iter_mut()
            .find(|component| component.id == "shaft")
            .expect("halberd shaft");
        let ComponentShape::Cylinder(shaft) = &mut shaft.shape else {
            panic!("halberd shaft must be cylindrical");
        };
        shaft.length.0 += 300;

        let short_properties = derive_properties(&short).unwrap();
        let long_properties = derive_properties(&long).unwrap();
        let short_mesh = crate::generate(&short).unwrap();
        let long_mesh = crate::generate(&long).unwrap();

        assert!(
            long_properties.length_m > short_properties.length_m + 0.20,
            "short={short_properties:?} long={long_properties:?}"
        );
        assert!(
            long_properties.grip_to_tip_m > short_properties.grip_to_tip_m + 0.10,
            "short={short_properties:?} long={long_properties:?}"
        );
        assert!(long_properties.mass_kg > short_properties.mass_kg);
        assert!(long_properties.moment_of_inertia_kg_m2 > short_properties.moment_of_inertia_kg_m2);
        assert!(
            (long_properties.striking_head_length_m - short_properties.striking_head_length_m)
                .abs()
                < 1.0e-6
        );
        assert!((long_mesh.derived.length_m - long_properties.length_m).abs() < f32::EPSILON);
        assert!(long_mesh.bounds.max[1] > short_mesh.bounds.max[1] + 0.20);
    }
}
