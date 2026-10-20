//! Actual required tree presentation, separate from generic capture metadata.
use serde::Serialize;

/// The focused native specimen makes no adaptive five-tier claim.
#[derive(Serialize)]
#[serde(tag = "required_capability", rename_all = "snake_case")]
pub(super) enum TreePresentationCapability {
    #[cfg(feature = "grown-review")]
    GeneratedGrownSelection {
        bank_id: adventuresim_environment::prepared::LibraryId,
        sources: [crate::presentation::GrownSource; 2],
        selection: adventuresim_environment::render::LevelSelection,
        expected_roots: usize,
        presented_roots: usize,
        owned_inventory_verified: bool,
    },
    FiveLodProduction {
        passed: bool,
    },
    #[cfg(feature = "grown-review")]
    FixedGrownLevel {
        role: super::grown_capture::Role,
        level: adventuresim_environment::render::Level,
        source_id: u64,
        owned_inventory_verified: bool,
    },
}

impl TreePresentationCapability {
    pub(super) fn passed(&self) -> bool {
        match self {
            Self::FiveLodProduction { passed } => *passed,
            #[cfg(feature = "grown-review")]
            Self::GeneratedGrownSelection {
                owned_inventory_verified,
                ..
            }
            | Self::FixedGrownLevel {
                owned_inventory_verified,
                ..
            } => *owned_inventory_verified,
        }
    }
}
