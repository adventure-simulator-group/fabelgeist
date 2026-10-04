//! Exact-source cache admission and first-writer ownership.
use super::{Kernel, KernelCacheError};
use fabelgeist_gpu::prelude::{ShaderSource, WgpuContext};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Compiles each distinct source once for subsequent callers. Concurrent first
/// uses may compile independently; all successful callers share the first
/// inserted pipeline. Whitespace and comments remain part of cache identity.
#[derive(Clone, Debug, Default)]
pub struct KernelCache {
    kernels: Arc<RwLock<HashMap<ShaderSource, Arc<Kernel>>>>,
}
impl KernelCache {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get(
        &self,
        context: &WgpuContext,
        source: &ShaderSource,
    ) -> Result<Arc<Kernel>, KernelCacheError> {
        if let Some(kernel) = self.lookup(source)? {
            return Ok(kernel);
        }
        // Compilation stays outside the write lock, so unrelated first uses
        // do not wait for this shader's preparation.
        let kernel = Arc::new(Kernel::new(context, source.clone()).map_err(
            |cause: super::KernelBuildError| -> KernelCacheError {
                KernelCacheError::Compile {
                    source: source.clone(),
                    cause,
                }
            },
        )?);
        self.admit(source.clone(), kernel)
    }
    fn lookup(&self, source: &ShaderSource) -> Result<Option<Arc<Kernel>>, KernelCacheError> {
        let kernels = self.kernels.read().map_err(
            |_: std::sync::PoisonError<
                std::sync::RwLockReadGuard<'_, HashMap<ShaderSource, Arc<Kernel>>>,
            >|
             -> KernelCacheError { KernelCacheError::ReadPoisoned },
        )?;
        Ok(kernels.get(source).cloned())
    }
    fn admit(
        &self,
        source: ShaderSource,
        kernel: Arc<Kernel>,
    ) -> Result<Arc<Kernel>, KernelCacheError> {
        let mut kernels = self.kernels.write().map_err(
            |_: std::sync::PoisonError<
                std::sync::RwLockWriteGuard<'_, HashMap<ShaderSource, Arc<Kernel>>>,
            >|
             -> KernelCacheError { KernelCacheError::WritePoisoned },
        )?;
        Ok(kernels.entry(source).or_insert(kernel).clone())
    }
    pub fn count(&self) -> Result<CompiledKernelCount, KernelCacheError> {
        let kernels = self.kernels.read().map_err(
            |_: std::sync::PoisonError<
                std::sync::RwLockReadGuard<'_, HashMap<ShaderSource, Arc<Kernel>>>,
            >|
             -> KernelCacheError { KernelCacheError::ReadPoisoned },
        )?;
        Ok(CompiledKernelCount(kernels.len()))
    }
}
/// Number of distinct compiled module sources, separate from dispatch counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompiledKernelCount(usize);
impl std::fmt::Display for CompiledKernelCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fabelgeist_gpu::prelude::{ComputePipeline, ShaderEntryPoint};

    // Cache policy does not inspect GPU handles. Resource-free kernels isolate
    // identity and admission from device availability.
    fn policy_fixture(entry_point: ShaderEntryPoint) -> Arc<Kernel> {
        Arc::new(Kernel {
            pipeline: ComputePipeline::default(),
            workgroup_shape: fabelgeist_gpu::prelude::WorkgroupShape::try_from([1, 1, 1]).unwrap(),
            entry_point,
            fast: None,
        })
    }
    #[test]
    fn first_admitted_kernel_wins_and_source_bytes_distinguish_entries() {
        let cache = KernelCache::new();
        let first = policy_fixture(ShaderEntryPoint::from("first"));
        let racing = policy_fixture(ShaderEntryPoint::from("racing"));
        let source = ShaderSource::from("fn main() {}");
        cache.admit(source.clone(), first.clone()).unwrap();
        assert!(Arc::ptr_eq(
            &cache.admit(source.clone(), racing.clone()).unwrap(),
            &first
        ));
        assert!(Arc::ptr_eq(
            &cache.lookup(&source).unwrap().unwrap(),
            &first
        ));
        let spaced = ShaderSource::from("fn main() {} ");
        assert!(cache.lookup(&spaced).unwrap().is_none());
        cache.admit(spaced.clone(), racing.clone()).unwrap();
        assert!(Arc::ptr_eq(
            &cache.lookup(&spaced).unwrap().unwrap(),
            &racing
        ));
        assert_eq!(cache.count().unwrap(), CompiledKernelCount(2));
    }
    #[test]
    fn poisoned_cache_reports_read_and_write_operations_without_panicking() {
        let cache = KernelCache::new();
        let poisoned = cache.clone();
        let result = std::panic::catch_unwind(move || -> ! {
            let _held = poisoned.kernels.write().unwrap();
            panic!("poison fixture");
        });
        assert!(result.is_err());
        let source = ShaderSource::from("fn main() {}");
        assert!(matches!(
            cache.lookup(&source),
            Err(KernelCacheError::ReadPoisoned)
        ));
        assert!(matches!(cache.count(), Err(KernelCacheError::ReadPoisoned)));
        assert!(matches!(
            cache.admit(source, policy_fixture(ShaderEntryPoint::from("main"))),
            Err(KernelCacheError::WritePoisoned)
        ));
    }
}
