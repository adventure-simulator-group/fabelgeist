//! Already fitted lower surfaces, in the same unposed frame as their wearer.
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::GeneratedArmor;

/// Geometry and anatomical ownership used when seating another item over it.
#[derive(Clone, Copy)]
pub struct ArmorLayerSurface<'a> {
    pub positions: &'a [[f32; 3]],
    pub faces: &'a [[u32; 3]],
    pub joint_indices: &'a [[u32; 8]],
    pub joint_weights: &'a [[f32; 8]],
}

impl<'a> ArmorLayerSurface<'a> {
    /// A runtime fit uses its sole wearer surface. The studio can ask for the
    /// matching morph realization; a missing target is an error, never a base
    /// surface substituted under a differently shaped outer garment.
    pub fn from_generated(armor: &'a GeneratedArmor, target: Option<&str>) -> Result<Self> {
        let positions = match target {
            None => &armor.positions,
            Some(name) => {
                &armor
                    .morphs
                    .iter()
                    .find(|morph| morph.name == name)
                    .with_context(|| format!("lower equipment surface has no morph {name}"))?
                    .direct_positions
            }
        };
        let (faces, remainder) = armor.indices.as_chunks::<3>();
        ensure!(
            remainder.is_empty() && !faces.is_empty(),
            "lower equipment surface has invalid triangles"
        );
        ensure!(
            positions.len() == armor.joint_indices.len()
                && positions.len() == armor.joint_weights.len(),
            "lower equipment surface has incomplete skin ownership"
        );
        ensure!(
            armor
                .indices
                .iter()
                .all(|&index| (index as usize) < positions.len()),
            "lower equipment surface has an invalid vertex index"
        );
        Ok(Self {
            positions,
            faces,
            joint_indices: &armor.joint_indices,
            joint_weights: &armor.joint_weights,
        })
    }
}
