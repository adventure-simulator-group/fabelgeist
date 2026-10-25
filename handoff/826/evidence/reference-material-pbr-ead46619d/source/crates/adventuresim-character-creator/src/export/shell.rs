//! Shell primitives retain their own topology, UVs, skin and surface textures.
use super::*;
pub(super) type Views = (usize, usize, usize, Option<(usize, usize)>);
pub(super) struct Writer<'a> {
    pub buffer: &'a mut BufferBuilder,
    pub primitives: &'a mut Vec<Value>,
    pub materials: &'a mut Vec<Value>,
    pub body_skin: [usize; 2],
    pub textures: &'a mut textures::TextureImages,
}
impl Writer<'_> {
    pub fn append(
        &mut self,
        shells: &[RiggedShell<'_>],
        views: Vec<Views>,
        mut accessor: impl FnMut(usize, u32, usize, &str, Option<([f32; 3], [f32; 3])>) -> usize,
    ) -> Result<()> {
        for (shell, (position_view, normal_view, index_view, skin_views)) in
            shells.iter().zip(views)
        {
            let (minimum, maximum) = position_bounds(shell.positions);
            let shell_position_accessor = accessor(
                position_view,
                5_126,
                shell.positions.len(),
                "VEC3",
                Some((minimum, maximum)),
            );
            let shell_index_accessor =
                accessor(index_view, 5_125, shell.faces.len() * 3, "SCALAR", None);
            let shell_normal_accessor =
                accessor(normal_view, 5_126, shell.normals.len(), "VEC3", None);
            let (shell_joints_0, shell_weights_0) = if let Some((joints_0, weights_0)) = skin_views
            {
                (
                    accessor(joints_0, 5_123, shell.positions.len(), "VEC4", None),
                    accessor(weights_0, 5_126, shell.positions.len(), "VEC4", None),
                )
            } else {
                (self.body_skin[0], self.body_skin[1])
            };
            let material = self.materials.len();
            let mut primitive = json!({
                "attributes": {
                    "POSITION": shell_position_accessor,
                    "NORMAL": shell_normal_accessor,
                    "JOINTS_0": shell_joints_0,
                    "WEIGHTS_0": shell_weights_0,
                },
                "indices": shell_index_accessor,
                "material": material,
            });
            if let Some(uv) = shell.texcoords {
                let view = self
                    .buffer
                    .push(&f32_bytes(uv.iter().flatten().copied()), Some(34_962));
                primitive["attributes"]["TEXCOORD_0"] =
                    accessor(view, 5_126, uv.len(), "VEC2", None).into();
            }
            self.primitives.push(primitive);
            let appearance = shell.textures.as_ref().map_or(shell.appearance, |maps| {
                maps.scalar_appearance(shell.appearance)
            });
            let factors = crate::surface_projection::LinearSurfaceMaterial::try_from(appearance)
                .with_context(|| format!("surface factors for shell {}", shell.name))?;
            let mut material = json!({
                "name": shell.name,
                "doubleSided": true,
                "pbrMetallicRoughness": {
                    "baseColorFactor": factors.base_color(),
                    "metallicFactor": factors.metallic(),
                    "roughnessFactor": factors.roughness(),
                }
            });
            if let Some(textures) = shell.textures.clone() {
                self.textures.apply(textures, self.buffer, &mut material);
            }
            if let Some(finish) = shell.textile_finish {
                self.textures.apply_textile_finish(finish, &mut material);
            }
            if let Some(textures) = shell.surface {
                self.textures
                    .apply_shell_textures(textures, self.buffer, &mut material);
            }
            self.materials.push(material);
        }
        Ok(())
    }
}

impl BufferBuilder {
    pub(super) fn shell_views(&mut self, shells: &[RiggedShell<'_>]) -> Result<Vec<Views>> {
        shells
            .iter()
            .map(|shell| {
                let positions = self.push(
                    &f32_bytes(shell.positions.iter().flatten().copied()),
                    Some(34_962),
                );
                let normals = self.push(
                    &f32_bytes(shell.normals.iter().flatten().copied()),
                    Some(34_962),
                );
                let indices = self.push(
                    &u32_bytes(shell.faces.iter().flatten().copied()),
                    Some(34_963),
                );
                let skin = shell
                    .joint_indices
                    .zip(shell.joint_weights)
                    .map(|(joints, weights)| skinning::append(self, joints, weights))
                    .transpose()?;
                Ok((positions, normals, indices, skin))
            })
            .collect::<Result<Vec<_>>>()
    }
}
