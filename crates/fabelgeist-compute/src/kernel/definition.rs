//! CPU admission of a kernel's compute entry and declared workgroup shape.
use super::KernelBuildError;
use fabelgeist_gpu::prelude::{
    ShaderEntryPoint, ShaderSource, WorkgroupShape, WorkgroupShapeError,
};

pub(super) struct KernelDefinition {
    pub entry_point: ShaderEntryPoint,
    pub workgroup_shape: WorkgroupShape,
}

impl KernelDefinition {
    pub fn new(source: &ShaderSource) -> Result<Self, KernelBuildError> {
        let module = source
            .parse(wgpu::naga::ShaderStage::Compute)
            .map_err(KernelBuildError::Parse)?;
        let entry = module
            .entry_points
            .iter()
            .find(|ep: &&wgpu::naga::EntryPoint| -> bool {
                ep.stage == wgpu::naga::ShaderStage::Compute
            })
            .ok_or(KernelBuildError::MissingComputeEntryPoint)?;
        let entry_point = ShaderEntryPoint::from(entry.name.clone());
        let workgroup_shape = WorkgroupShape::try_from(entry.workgroup_size).map_err(
            |cause: WorkgroupShapeError| -> KernelBuildError {
                KernelBuildError::ZeroWorkgroupDimension {
                    entry: entry_point.clone(),
                    cause,
                }
            },
        )?;
        Ok(Self {
            entry_point,
            workgroup_shape,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    #[test]
    fn parsing_missing_entry_and_zero_dimensions_are_distinct_build_failures() {
        let malformed = KernelDefinition::new(&ShaderSource::from("fn {"))
            .err()
            .unwrap();
        assert!(matches!(malformed, KernelBuildError::Parse(_)));
        assert!(malformed.source().unwrap().source().is_some());
        let helper = KernelDefinition::new(&ShaderSource::from("fn helper() {}"));
        assert!(matches!(
            helper,
            Err(KernelBuildError::MissingComputeEntryPoint)
        ));
        let zero = KernelDefinition::new(&ShaderSource::from(
            "@compute @workgroup_size(1, 0, 1) fn main() {}",
        ));
        assert!(matches!(
            zero,
            Err(KernelBuildError::ZeroWorkgroupDimension { .. })
        ));
        let zero = zero.err().unwrap();
        assert!(zero.source().unwrap().is::<WorkgroupShapeError>());
        assert_eq!(
            zero.to_string(),
            "Kernel `main`: workgroup size [1, 0, 1] has a zero dimension"
        );
    }
    #[test]
    fn admitted_definition_keeps_declared_entry_and_workgroup_dimensions() {
        let definition = KernelDefinition::new(&ShaderSource::from(
            "@compute @workgroup_size(4, 2, 1) fn exact_entry() {}",
        ))
        .unwrap();
        assert_eq!(
            definition.entry_point,
            ShaderEntryPoint::from("exact_entry")
        );
        assert_eq!(
            definition.workgroup_shape,
            WorkgroupShape::try_from([4, 2, 1]).unwrap()
        );
    }
}
