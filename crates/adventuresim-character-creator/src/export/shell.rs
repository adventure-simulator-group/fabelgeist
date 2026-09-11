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
    ) {
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
            let mut material = json!({
                "name": shell.name,
                "pbrMetallicRoughness": {
                    "baseColorFactor": linear_base_color(shell.base_color),
                    "metallicFactor": shell.metallic,
                    "roughnessFactor": shell.roughness,
                }
            });
            if let Some(textures) = shell.textures {
                self.textures.apply(textures, self.buffer, &mut material);
            }
            self.materials.push(material);
        }
    }
}
