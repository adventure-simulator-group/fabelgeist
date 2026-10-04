//! Metal authoring and baking failures, including preserved transport causes.

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum MetalError {
    #[error("Invalid metal parameters")]
    InvalidMaterial,
    #[error("Invalid engraving parameters")]
    InvalidEngraving,
    #[error("Invalid ornament parameters")]
    InvalidOrnament,
    #[error("Texture size {requested} must be 32–1024")]
    TextureSize { requested: u32 },
    #[error("the relief image does not match the engraving's relief")]
    ReliefMismatch,
    #[error("Engraving {path}: {source}", path = .path.display())]
    ReliefIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Engraving {source_name:?}: {source}")]
    ReliefDecode {
        source_name: super::super::engraving::ReliefSource,
        #[source]
        source: image::ImageError,
    },
    #[error("armor metal device: {0}")]
    Device(#[source] fabelgeist_gpu::prelude::Error),
    #[error("armor metal device: {0}")]
    Kernel(#[from] fabelgeist_compute::KernelCacheError),
    #[error("armor metal device: {0}")]
    BufferCreation(#[from] fabelgeist_gpu::prelude::BufferCreationError),
    #[error("armor metal device: {0}")]
    Dispatch(#[from] fabelgeist_compute::KernelDispatchError),
    #[error("armor metal readback: {0}")]
    Readback(#[from] fabelgeist_gpu::prelude::ReadbackError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use fabelgeist_gpu::prelude::{BufferByteLength, BufferReadback, ReadbackError};
    use std::error::Error;

    #[test]
    fn uniform_dispatch_causes_survive_metal_and_generation_context() {
        use fabelgeist_compute::KernelDispatchError;
        use fabelgeist_gpu::prelude::UniformPackingError;
        let metal = MetalError::from(KernelDispatchError::Uniform(UniformPackingError::Missing {
            member: "count".into(),
        }));
        let dispatch = metal
            .source()
            .unwrap()
            .downcast_ref::<KernelDispatchError>()
            .unwrap();
        assert!(
            matches!(dispatch.source().unwrap().downcast_ref::<UniformPackingError>(), Some(UniformPackingError::Missing { member }) if *member == "count".into())
        );
        let generation = crate::GenerateError::from(KernelDispatchError::Uniform(
            UniformPackingError::Missing {
                member: "count".into(),
            },
        ));
        assert!(matches!(
            generation
                .source()
                .unwrap()
                .downcast_ref::<KernelDispatchError>(),
            Some(KernelDispatchError::Uniform(
                UniformPackingError::Missing { .. }
            ))
        ));
    }

    #[test]
    fn readback_causes_survive_metal_and_generation_context() {
        let readback = BufferReadback::<u8>::new(BufferByteLength::from(u64::MAX))
            .err()
            .unwrap();
        let metal = MetalError::from(readback);
        let readback = metal
            .source()
            .unwrap()
            .downcast_ref::<ReadbackError>()
            .unwrap();
        assert!(readback.source().is_some());

        let readback = ReadbackError::EmptyStatus;
        let generated = crate::GenerateError::from(readback);
        assert!(matches!(
            generated.source().unwrap().downcast_ref::<ReadbackError>(),
            Some(ReadbackError::EmptyStatus)
        ));
    }

    #[test]
    fn kernel_parser_causes_survive_metal_and_generation_context() {
        use fabelgeist_compute::{KernelBuildError, KernelCacheError};
        use fabelgeist_gpu::prelude::{
            ComputeShaderError, PreparedComputeShader, ShaderParseError, ShaderSource,
        };
        let source = ShaderSource::from("fn {");
        let shader = PreparedComputeShader::new(source.clone()).err().unwrap();
        let cache = KernelCacheError::Compile {
            source: source.clone(),
            cause: KernelBuildError::Shader(shader),
        };
        let metal = MetalError::from(cache);
        let cache = metal
            .source()
            .unwrap()
            .downcast_ref::<KernelCacheError>()
            .unwrap();
        let build = cache
            .source()
            .unwrap()
            .downcast_ref::<KernelBuildError>()
            .unwrap();
        let shader = build
            .source()
            .unwrap()
            .downcast_ref::<ComputeShaderError>()
            .unwrap();
        let parse = shader
            .source()
            .unwrap()
            .downcast_ref::<ShaderParseError>()
            .unwrap();
        assert!(parse.source().is_some());
        assert!(
            matches!(parse, ShaderParseError::Wgsl { source: retained, .. } if retained == &source)
        );

        let shader = PreparedComputeShader::new(source.clone()).err().unwrap();
        let generated = crate::GenerateError::from(KernelCacheError::Compile {
            source,
            cause: KernelBuildError::Shader(shader),
        });
        assert!(
            generated
                .source()
                .unwrap()
                .downcast_ref::<KernelCacheError>()
                .is_some()
        );
        let metal = MetalError::from(fabelgeist_gpu::prelude::BufferCreationError::Empty);
        assert!(matches!(
            metal
                .source()
                .unwrap()
                .downcast_ref::<fabelgeist_gpu::prelude::BufferCreationError>(),
            Some(fabelgeist_gpu::prelude::BufferCreationError::Empty)
        ));
        let generation =
            crate::GenerateError::from(fabelgeist_gpu::prelude::BufferCreationError::Empty);
        assert!(matches!(
            generation
                .source()
                .unwrap()
                .downcast_ref::<fabelgeist_gpu::prelude::BufferCreationError>(),
            Some(fabelgeist_gpu::prelude::BufferCreationError::Empty)
        ));
    }
}
