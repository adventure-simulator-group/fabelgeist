//! A rejected native shader workgroup declaration, with its entry-point context.

use fabelgeist_gpu::prelude::WorkgroupShapeError;

#[derive(Debug)]
pub struct WorkgroupDeclarationError {
    entry_point: String,
    source: WorkgroupShapeError,
}

impl WorkgroupDeclarationError {
    // The label comes directly from the native shader parser, for diagnostics.
    pub(super) fn at_entry_point(entry_point: &str, source: WorkgroupShapeError) -> Self {
        Self {
            entry_point: entry_point.to_owned(),
            source,
        }
    }
}

impl std::fmt::Display for WorkgroupDeclarationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Kernel `{}`: {}", self.entry_point, self.source)
    }
}

impl std::error::Error for WorkgroupDeclarationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}
