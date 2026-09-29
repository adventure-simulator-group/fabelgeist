use super::*;
use crate::export::{RiggedMorphTarget, RiggedShell};

impl DrapedGarment {
    /// `mail` must be this garment's chainmail surface when its fabric is mail.
    pub fn rigged<'a>(
        &'a self,
        targets: &'a [RiggedMorphTarget<'a>],
        mail: Option<&'a crate::garment_material::MailSurface>,
    ) -> RiggedShell<'a> {
        let (base_color, metallic, roughness) =
            mail.map_or(crate::garment_material::CLOTH_PBR, |mail| mail.pbr());
        RiggedShell {
            plate_edges: &[],
            surface: None,
            textures: mail.map(|mail| mail.maps.textures()),
            texcoords: Some(mail.map_or(&self.texcoords, |mail| &mail.texcoords)),
            hinge: None,
            name: &self.name,
            positions: &self.positions,
            normals: &self.normals,
            faces: &self.faces,
            joint_indices: Some(&self.indices),
            joint_weights: Some(&self.weights),
            morph_targets: targets,
            base_color,
            metallic,
            roughness,
        }
    }
}
