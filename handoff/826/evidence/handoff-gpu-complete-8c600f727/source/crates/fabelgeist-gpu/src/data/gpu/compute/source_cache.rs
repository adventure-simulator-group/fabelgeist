//! Shared source/device admission for compiled compute recipe caches.

use super::ComputePipelineError;
use crate::globals::WgpuContext;
use anyhow::Result;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

/// Cached successful builds of one recipe family.
///
/// The authored source and recipe parameters identify the program. Native device
/// identity and validation policy keep resources and observed outcomes local to
/// their provider. An empty source is valid only for an immutable built-in recipe
/// whose complete variable inputs are carried by its parameters.
#[derive(Clone, Debug)]
pub struct ComputeSourceCache<Parameters, Value> {
    entries: Arc<RwLock<HashMap<ComputeSourceKey<Parameters>, Value>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ComputeSourceKey<Parameters> {
    device: wgpu::Device,
    observes_validation: bool,
    source: String,
    parameters: Parameters,
}

// The map needs no default recipe parameters or cached value.
impl<Parameters, Value> Default for ComputeSourceCache<Parameters, Value> {
    fn default() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<Parameters: Eq + Hash, Value: Clone> ComputeSourceCache<Parameters, Value> {
    /// Reuse a successful build, or construct it without holding the cache lock.
    /// Failed construction never publishes a value.
    pub fn get_or_try_insert(
        &self,
        context: &WgpuContext,
        source: &str,
        parameters: Parameters,
        build: impl FnOnce() -> Result<Value>,
    ) -> Result<Value> {
        let key = ComputeSourceKey {
            device: context.device.as_ref().clone(),
            observes_validation: cfg!(not(target_arch = "wasm32")) && context.blocking_validation,
            source: source.to_owned(),
            parameters,
        };
        if let Some(value) = self
            .entries
            .read()
            .map_err(|_| ComputePipelineError::CachePoisoned)?
            .get(&key)
        {
            return Ok(value.clone());
        }

        let value = build()?;
        let mut entries = self
            .entries
            .write()
            .map_err(|_| ComputePipelineError::CachePoisoned)?;
        // Racing successful builders share the first published native outcome.
        Ok(entries.entry(key).or_insert(value).clone())
    }

    pub fn len(&self) -> usize {
        self.entries
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
