//! A rejected native shader workgroup declaration, with its entry-point context.

use fabelgeist_gpu::prelude::WorkgroupShapeError;

#[derive(Debug, thiserror::Error)]
#[error("Kernel `{entry_point}`: {source}")]
pub struct WorkgroupDeclarationError {
    entry_point: String,
    source: WorkgroupShapeError,
}

impl WorkgroupDeclarationError {
    // The label comes directly from the native shader parser, for diagnostics.
    pub(crate) fn at_entry_point(entry_point: &str, source: WorkgroupShapeError) -> Self {
        Self {
            entry_point: entry_point.to_owned(),
            source,
        }
    }
}

