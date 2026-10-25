use std::sync::Arc;

use crate::prelude::*;
use crate::{Map, MapDefinition, MapSignature};
use fabelgeist_gpu::data::gpu::parameters::{PassParameter, PassParameters};
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use fabelgeist_gpu::data::gpu::texture::{Texture3d, TextureViewError};

pub struct DistanceField;

impl DistanceField {
    pub fn create_texture(context: &WgpuContext, size: Vec3) -> Result<Texture3d> {
        Texture3d::new(
            context,
            size,
            fabelgeist_gpu::data::gpu::texture::TextureFormat::R32Float,
        )
    }

    pub fn generate(
        context: &WgpuContext,
        definition: &MapDefinition,
        parameters: Option<PassParameters>,
        output: &Texture3d,
    ) -> Result<()> {
        let output_res = GpuResource::Texture3d(output.clone());
        Map::execute_with_parameters(context, definition, None, &output_res, parameters)?;
        Ok(())
    }

    pub fn generate_min(
        context: &WgpuContext,
        definition: &MapDefinition,
        parameters: Option<PassParameters>,
        io: &Texture3d,
    ) -> Result<()> {
        let native_input = io.native_texture().ok_or(TextureViewError::Uninitialized)?;
        io.ensure_device(context)?;
        let sig = MapSignature::parse(&definition.code)?;
        let re = regex::Regex::new(r"\bfn\s+map\b")?;
        let user_code = re.replace(&definition.code, "fn user_map").to_string();

        let mut wrapper_args = vec!["in_val: f32".to_string()];
        wrapper_args.extend(sig.declarations().map(str::to_owned));

        let user_call_args = sig.names().collect::<Vec<_>>().join(", ");

        let code = format!(
            r#"
            {user_code}

            fn map({wrapper_args}) -> f32 {{
                let user_val = user_map({user_call_args});
                return min(in_val, user_val);
            }}
            "#,
            user_code = user_code,
            wrapper_args = wrapper_args.join(", "),
            user_call_args = user_call_args
        );

        let wrapped_def = MapDefinition::new(code)?;

        // WebGPU restriction: Cannot bind same texture as read-only and write-only in the same pass.
        // Copy the original data to a temp texture.
        let temp_tex = Texture3d::new(
            context,
            fabelgeist_gpu::data::vector::Vec3::new(
                io.extent().width as f32,
                io.extent().height as f32,
                io.extent().depth_or_array_layers as f32,
            ),
            io.format(),
        )?;

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("DistanceField Copy Encoder"),
            });

        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: native_input,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: temp_tex
                    .native_texture()
                    .ok_or(TextureViewError::Uninitialized)?,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: io.extent().width,
                height: io.extent().height,
                depth_or_array_layers: io.extent().depth_or_array_layers,
            },
        );
        context
            .submit(encoder)
            .map_err(|cause| ComputePassError::Native(Arc::new(cause)))?;

        let input_res = GpuResource::Texture3d(temp_tex);
        let io_res = GpuResource::Texture3d(io.clone());

        Map::execute_with_parameters(context, &wrapped_def, Some(&input_res), &io_res, parameters)?;
        Ok(())
    }

    pub fn generate_smooth_min(
        context: &WgpuContext,
        definition: &MapDefinition,
        parameters: Option<PassParameters>,
        k: f32,
        io: &Texture3d,
    ) -> Result<()> {
        let native_input = io.native_texture().ok_or(TextureViewError::Uninitialized)?;
        io.ensure_device(context)?;
        let sig = MapSignature::parse(&definition.code)?;
        let re = regex::Regex::new(r"\bfn\s+map\b")?;
        let user_code = re.replace(&definition.code, "fn user_map").to_string();

        let mut wrapper_args = vec!["in_val: f32".to_string()];
        wrapper_args.extend(sig.declarations().map(str::to_owned));
        wrapper_args.push("smin_k: f32".to_string());

        let user_call_args = sig.names().collect::<Vec<_>>().join(", ");

        let code = format!(
            r#"
            {user_code}

            fn smin(a: f32, b: f32, k: f32) -> f32 {{
                if (k <= 0.0) {{
                    return min(a, b);
                }}
                let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
                return mix(b, a, h) - k * h * (1.0 - h);
            }}

            fn map({wrapper_args}) -> f32 {{
                let user_val = user_map({user_call_args});
                return smin(in_val, user_val, smin_k);
            }}
            "#,
            user_code = user_code,
            wrapper_args = wrapper_args.join(", "),
            user_call_args = user_call_args
        );

        let wrapped_def = MapDefinition::new(code)?;

        // WebGPU restriction: Cannot bind same texture as read-only and write-only in the same pass.
        // Copy the original data to a temp texture.
        let temp_tex = Texture3d::new(
            context,
            fabelgeist_gpu::data::vector::Vec3::new(
                io.extent().width as f32,
                io.extent().height as f32,
                io.extent().depth_or_array_layers as f32,
            ),
            io.format(),
        )?;

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("DistanceField Copy Encoder"),
            });

        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: native_input,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: temp_tex
                    .native_texture()
                    .ok_or(TextureViewError::Uninitialized)?,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: io.extent().width,
                height: io.extent().height,
                depth_or_array_layers: io.extent().depth_or_array_layers,
            },
        );
        context
            .submit(encoder)
            .map_err(|cause| ComputePassError::Native(Arc::new(cause)))?;

        let input_res = GpuResource::Texture3d(temp_tex);
        let io_res = GpuResource::Texture3d(io.clone());

        let mut final_params = parameters.unwrap_or_default();
        final_params.insert("smin_k", k);

        Map::execute_with_parameters(
            context,
            &wrapped_def,
            Some(&input_res),
            &io_res,
            Some(final_params),
        )?;
        Ok(())
    }

    pub fn min(
        context: &WgpuContext,
        a: &Texture3d,
        b: &Texture3d,
        output: &Texture3d,
    ) -> Result<()> {
        let code = r#"
        fn map(val_a: f32, val_b: f32) -> f32 {
            return min(val_a, val_b);
        }
        "#
        .to_string();

        let definition = MapDefinition::new(code)?;

        let input_a = GpuResource::Texture3d(a.clone());
        let output_res = GpuResource::Texture3d(output.clone());

        let mut params = PassParameters::new();
        params.insert("val_b", PassParameter::Texture3d(b.clone()));

        Map::execute_with_parameters(
            context,
            &definition,
            Some(&input_a),
            &output_res,
            Some(params),
        )?;
        Ok(())
    }

    pub fn smooth_min(
        context: &WgpuContext,
        a: &Texture3d,
        b: &Texture3d,
        k: f32,
        output: &Texture3d,
    ) -> Result<()> {
        let code = r#"
        fn smin(a: f32, b: f32, k: f32) -> f32 {
            if (k <= 0.0) {
                return min(a, b);
            }
            let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
            return mix(b, a, h) - k * h * (1.0 - h);
        }

        fn map(val_a: f32, val_b: f32, k: f32) -> f32 {
            return smin(val_a, val_b, k);
        }
        "#
        .to_string();

        let definition = MapDefinition::new(code)?;

        let input_a = GpuResource::Texture3d(a.clone());
        let output_res = GpuResource::Texture3d(output.clone());

        let mut params = PassParameters::new();
        params.insert("val_b", PassParameter::Texture3d(b.clone()));
        params.insert("k", k);

        Map::execute_with_parameters(
            context,
            &definition,
            Some(&input_a),
            &output_res,
            Some(params),
        )?;
        Ok(())
    }
}
