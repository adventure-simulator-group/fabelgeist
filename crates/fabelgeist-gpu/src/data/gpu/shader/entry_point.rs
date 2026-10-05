/// An exact authored or parsed shader entry-point label.
///
/// This identity is distinct from module source and binding names. Admission
/// preserves the spelling; it does not validate syntax or module membership.
/// Native shader APIs receive the label through an explicit borrowed adapter.
///
/// Raw strings cannot select a cached pipeline:
///
/// ```compile_fail
/// use fabelgeist_gpu::prelude::ComputePipeline;
/// fn select(pipeline: &ComputePipeline, device: &wgpu::Device, name: &str) {
///     let _ = pipeline.get_or_create_pipeline(device, name);
/// }
/// ```
///
/// ```no_run
/// use fabelgeist_gpu::prelude::{ComputePipeline, ShaderEntryPoint};
/// fn select(pipeline: &ComputePipeline, device: &wgpu::Device) {
///     let name = ShaderEntryPoint::from("simulation_step");
///     let _ = pipeline.get_or_create_pipeline(device, &name);
/// }
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ShaderEntryPoint(String);
impl From<String> for ShaderEntryPoint {
    fn from(name: String) -> Self {
        Self(name)
    }
}
impl From<&str> for ShaderEntryPoint {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}
impl<'a> From<&'a ShaderEntryPoint> for &'a str {
    fn from(name: &'a ShaderEntryPoint) -> Self {
        &name.0
    }
}
impl std::fmt::Display for ShaderEntryPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Debug for ShaderEntryPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}
