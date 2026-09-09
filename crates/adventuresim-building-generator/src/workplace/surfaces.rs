//! Canonical unlit surface references used by production and geometry diagnostics alike.

/// Authored workplace finish, independent of the building's wall and roof palette.
#[derive(Clone, Copy, Debug)]
pub struct WorkplaceSurface {
    pub srgb: [f32; 3],
    pub perceptual_roughness: f32,
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
            _ => return None,
        };
        Some(WorkplaceSurface {
            srgb,
            perceptual_roughness,
        })
    }
}
