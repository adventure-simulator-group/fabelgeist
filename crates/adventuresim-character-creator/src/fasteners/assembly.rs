//! Neighboring closure support includes the plate and its fitted fastenings.
//! Dependencies are evaluated inward first for the same unposed body sample.
use super::catalog::{FastenerRecipe, FastenerRecipes};
use crate::armor_frames::Wearer;
use adventuresim_armor_model::PartMesh;
use anyhow::{Result, ensure};

impl FastenerRecipe {
    /// Fit the complete supporting assembly without transferring its attachment
    /// ownership to the outer closure. Cyclic authored dependencies are invalid.
    pub fn fitted_support(
        &self,
        recipes: &FastenerRecipes,
        wearer: &Wearer<'_>,
        placement: &str,
        fit_plate: &impl Fn(&str) -> Result<PartMesh>,
    ) -> Result<Option<PartMesh>> {
        self.support_with_ancestors(recipes, wearer, placement, fit_plate, &mut Vec::new())
    }

    fn support_with_ancestors(
        &self,
        recipes: &FastenerRecipes,
        wearer: &Wearer<'_>,
        placement: &str,
        fit_plate: &impl Fn(&str) -> Result<PartMesh>,
        ancestors: &mut Vec<&'static str>,
    ) -> Result<Option<PartMesh>> {
        let Some(id) = self.support_item() else {
            return Ok(None);
        };
        ensure!(!ancestors.contains(&id), "cyclic closure support at {id}");
        ancestors.push(id);
        let mut assembly = fit_plate(id)?;
        if assembly.components.is_empty() {
            assembly =
                assembly.with_component(adventuresim_armor_model::ArmorComponentRole::Plate, None);
        }
        if let Some(recipe) = recipes.get(id) {
            let support =
                recipe.support_with_ancestors(recipes, wearer, placement, fit_plate, ancestors)?;
            let closures = recipe.generate(&assembly, wearer, placement, support.as_ref())?;
            assembly.append(closures);
        }
        ancestors.pop();
        Ok(Some(assembly))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fasteners::{
        StrapDesign,
        catalog::{AttachmentRegion, ClosureSupport, RetentionDesign},
        mesh,
    };

    fn retention(support: ClosureSupport) -> FastenerRecipe {
        FastenerRecipe::Retention(RetentionDesign {
            region: AttachmentRegion::UpperArm,
            closure: StrapDesign::default(),
            support,
        })
    }

    #[test]
    fn outer_route_encloses_the_inner_closure_assembly() {
        let body = mesh::cuboid([0.04, 0.2, 0.035]);
        let names = ["l_uparm", "l_lowarm", "l_eye", "r_eye", "c_head"].map(String::from);
        let joints = [
            [0.0, 0.2, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0],
            [0.0, -0.2, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0],
            [0.02, 0.4, 0.1, 0.0, 0.0, 0.0, 1.0, 1.0],
            [-0.02, 0.4, 0.1, 0.0, 0.0, 0.0, 1.0, 1.0],
            [0.0, 0.4, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0],
        ];
        let indices = vec![[0; 8]; body.positions.len()];
        let weights = vec![[0.125; 8]; body.positions.len()];
        let wearer = Wearer {
            faces: body.indices.as_chunks::<3>().0,
            positions: &body.positions,
            normals: &[],
            joint_indices: &indices,
            joint_weights: &weights,
            joint_names: &names,
            joints: &joints,
        };
        let inner = mesh::cuboid([0.05, 0.15, 0.045]);
        let outer = mesh::cuboid([0.055, 0.15, 0.05]);
        let recipe = retention(ClosureSupport::Rerebrace);
        let recipes =
            FastenerRecipes::from([("rerebrace".into(), retention(ClosureSupport::Wearer))]);
        let complete = recipe
            .fitted_support(&recipes, &wearer, "left", &|_| Ok(inner.clone()))
            .unwrap()
            .unwrap();
        complete.normals().unwrap();
        assert_eq!(
            &complete.positions[..inner.positions.len()],
            &inner.positions
        );
        assert!(
            complete
                .components
                .iter()
                .any(|part| { part.role == adventuresim_armor_model::ArmorComponentRole::Buckles })
        );
        let bare = recipe
            .generate(&outer, &wearer, "left", Some(&inner))
            .unwrap();
        let fitted = recipe
            .generate(&outer, &wearer, "left", Some(&complete))
            .unwrap();
        assert_eq!(fitted.indices, bare.indices);
        fitted.normals().unwrap();
        let movement = fitted
            .positions
            .iter()
            .zip(&bare.positions)
            .map(|(a, b)| {
                bevy::math::Vec3::from_array(*a).distance(bevy::math::Vec3::from_array(*b))
            })
            .fold(0.0_f32, f32::max);
        assert!(
            movement > 0.001,
            "outer route ignored the inner buckle and strap"
        );
    }

    #[test]
    fn cyclic_support_is_rejected_before_generating_a_closure() {
        let wearer = Wearer {
            faces: &[],
            positions: &[],
            normals: &[],
            joint_indices: &[],
            joint_weights: &[],
            joint_names: &[],
            joints: &[],
        };
        let recipe = retention(ClosureSupport::Rerebrace);
        let recipes = FastenerRecipes::from([("rerebrace".into(), recipe.clone())]);
        assert!(
            recipe
                .fitted_support(&recipes, &wearer, "left", &|_| Ok(PartMesh::new()))
                .is_err()
        );
    }
}
