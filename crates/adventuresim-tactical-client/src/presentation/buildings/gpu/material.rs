use bevy::{prelude::*, render::render_resource::*, shader::ShaderRef};

const CITY_SHADER: &str = "shaders/tactical_city.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub(super) struct CityMaterial {
    #[uniform(0)]
    pub color: Vec4,
    #[uniform(0)]
    pub surface: Vec4,
    #[uniform(0)]
    pub uv_scale_offset: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub albedo: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    pub normal: Option<Handle<Image>>,
}

impl CityMaterial {
    pub(super) fn from_source(source: StandardMaterial) -> Self {
        let uv = source.uv_transform;
        Self {
            color: Vec4::from_array(source.base_color.to_linear().to_f32_array()),
            surface: Vec4::new(
                source.perceptual_roughness,
                source.metallic,
                match source.alpha_mode {
                    AlphaMode::Mask(cutoff) => cutoff,
                    _ => 0.0,
                },
                if source.normal_map_texture.is_some() {
                    1.0
                } else {
                    0.0
                },
            ),
            uv_scale_offset: Vec4::new(
                uv.matrix2.x_axis.x,
                uv.matrix2.y_axis.y,
                uv.translation.x,
                uv.translation.y,
            ),
            albedo: source.base_color_texture,
            normal: source.normal_map_texture,
        }
    }
}

impl Material for CityMaterial {
    fn vertex_shader() -> ShaderRef {
        CITY_SHADER.into()
    }
    fn fragment_shader() -> ShaderRef {
        CITY_SHADER.into()
    }
    fn prepass_vertex_shader() -> ShaderRef {
        CITY_SHADER.into()
    }
    fn prepass_fragment_shader() -> ShaderRef {
        CITY_SHADER.into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        if self.surface.z > 0.0 {
            AlphaMode::Mask(self.surface.z)
        } else {
            AlphaMode::Opaque
        }
    }
    fn specialize(
        _: &bevy::pbr::MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _: &bevy::mesh::MeshVertexBufferLayoutRef,
        _: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Geometry comes from storage; the small Mesh3d is only a phase anchor.
        // Bevy's mesh specializer caches by slot zero, even for vertex pulling.
        descriptor.layout[2] = super::scratch::geometry_layout();
        descriptor.vertex.buffers[0].array_stride = 0;
        descriptor.vertex.buffers[0].attributes.clear();
        descriptor.label = Some("gpu_city".into());
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_city_surfaces_use_depth_only_shadows() {
        let solid = CityMaterial::from_source(StandardMaterial::default());
        assert_eq!(solid.alpha_mode(), AlphaMode::Opaque);
        let cutout = CityMaterial::from_source(StandardMaterial {
            alpha_mode: AlphaMode::Mask(0.4),
            ..default()
        });
        assert_eq!(cutout.alpha_mode(), AlphaMode::Mask(0.4));
    }
}
