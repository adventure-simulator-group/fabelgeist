//! Canonical unlit surface references used by production and geometry diagnostics alike.

/// Authored workplace finish, independent of the building's wall and roof palette.
#[derive(Clone, Copy, Debug)]
pub struct WorkplaceSurface {
    pub srgb: [f32; 3],
    pub perceptual_roughness: f32,
    pub metallic: f32,
}

impl crate::BuildingLodMaterial {
    pub const fn workplace_surface(self) -> Option<WorkplaceSurface> {
        let (srgb, perceptual_roughness) = match self {
            Self::DyedCloth => ([0.22, 0.30, 0.37], 0.95),
            Self::UndyedCloth => ([0.66, 0.62, 0.52], 0.95),
            Self::Hide => ([0.29, 0.19, 0.115], 0.86),
            Self::ProcessLiquid => ([0.13, 0.16, 0.12], 0.3),
            Self::Grain => ([0.72, 0.58, 0.34], 1.0),
            Self::HempRope => ([0.45, 0.34, 0.20], 0.95),
            Self::CarvedSandstone => ([0.51, 0.38, 0.29], 0.94),
            Self::LeadAlloy => ([0.43, 0.45, 0.47], 0.48),
            Self::Bronze => ([0.48, 0.32, 0.13], 0.36),
            Self::CandleWax => ([0.79, 0.71, 0.48], 0.7),
            Self::Earthenware => ([0.40, 0.25, 0.16], 0.72),
            Self::GlazedTile => ([0.12, 0.24, 0.11], 0.28),
            Self::Millstone => ([0.43, 0.41, 0.36], 0.96),
            Self::FurnitureWood(surface) => match surface {
                crate::furniture::FurnitureWoodSurface::Handled => ([0.39, 0.28, 0.17], 0.58),
                crate::furniture::FurnitureWoodSurface::Replacement => ([0.46, 0.34, 0.20], 0.94),
                crate::furniture::FurnitureWoodSurface::ReplacementEndGrain => {
                    ([0.40, 0.29, 0.16], 0.94)
                }
                crate::furniture::FurnitureWoodSurface::Painted => ([0.28, 0.11, 0.075], 0.84),
            },
            Self::TimberEndGrain => ([0.34, 0.24, 0.14], 0.91),
            _ => return None,
        };
        Some(WorkplaceSurface {
            srgb,
            perceptual_roughness,
            metallic: if matches!(self, Self::Bronze | Self::LeadAlloy) {
                0.85
            } else {
                0.0
            },
        })
    }
}
