//! Already fitted lower surfaces, in the same unposed frame as their wearer.
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::GeneratedArmor;

/// Geometry and anatomical ownership used when seating another item over it.
#[derive(Clone, Copy)]
pub struct ArmorLayerSurface<'a> {
    pub positions: &'a [[f32; 3]],
    pub faces: &'a [[u32; 3]],
    pub plate_faces: &'a [fabelgeist_armor::PlateFace],
    pub components: &'a [fabelgeist_armor::ArmorComponent],
    pub joint_indices: &'a fabelgeist_rig::PackedRigSkin<8>,
    pub joint_weights: &'a [[f32; 8]],
}

impl<'a> ArmorLayerSurface<'a> {
    /// Geometry-only canonical anatomy view. It has no garment material roles.
    pub(crate) fn anatomy(body: &crate::armor_frames::Wearer<'a>) -> Self {
        Self {
            positions: body.positions,
            faces: body.faces,
            plate_faces: &[],
            components: &[],
            joint_indices: body.joint_indices,
            joint_weights: body.joint_weights,
        }
    }

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
        Self {
            positions,
            faces,
            plate_faces: &armor.faces,
            components: &armor.components,
            joint_indices: &armor.joint_indices,
            joint_weights: &armor.joint_weights,
        }
        .validate()
    }

    /// Validate borrowed native mesh metadata before it enters numerical
    /// fitting code, where component ranges become vertex masks.
    fn validate(self) -> Result<Self> {
        ensure!(
            !self.positions.is_empty() && !self.faces.is_empty(),
            "lower equipment surface is empty"
        );
        ensure!(
            self.plate_faces.is_empty() || self.plate_faces.len() == self.faces.len(),
            "lower equipment surface has incomplete shell ownership"
        );
        ensure!(
            self.positions.len() == self.joint_indices.len()
                && self.positions.len() == self.joint_weights.len(),
            "lower equipment surface has incomplete skin ownership"
        );
        ensure!(
            self.faces
                .iter()
                .flatten()
                .all(|&index| (index as usize) < self.positions.len()),
            "lower equipment surface has an invalid vertex index"
        );
        ensure!(
            !self.components.is_empty()
                && self
                    .components
                    .iter()
                    .all(
                        |component| component.vertices.start < component.vertices.end
                            && component.vertices.end <= self.positions.len()
                    ),
            "lower equipment surface has invalid component ownership"
        );
        Ok(self)
    }
}
