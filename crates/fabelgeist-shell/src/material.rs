//! Material admission precedes GPU allocation; diagnostic roles remain stable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShellMaterial {
    pub stretch_compliance: f32,
    pub bend_compliance: f32,
    /// Compliance of zero-length attachment constraints (including cloth seams).
    pub seam_compliance: f32,
    pub thickness: f32,
    /// Suggested world-contact coefficient; configure it on physics colliders.
    pub friction: f32,
    pub damping: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellMaterialParameter {
    StretchCompliance,
    BendCompliance,
    SeamCompliance,
    Thickness,
    Friction,
    Damping,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellMaterialError {
    InvalidParameter { parameter: ShellMaterialParameter },
    NonPositiveThickness,
}

impl std::fmt::Display for ShellMaterialError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidParameter { .. } => "invalid shell material",
            Self::NonPositiveThickness => "shell thickness must be positive",
        })
    }
}
impl std::error::Error for ShellMaterialError {}

impl ShellMaterial {
    pub fn particle_radius(&self) -> f32 {
        self.thickness * 0.5
    }
    pub fn validate(&self) -> Result<(), ShellMaterialError> {
        use ShellMaterialParameter::*;
        for (parameter, value) in [
            (StretchCompliance, self.stretch_compliance),
            (BendCompliance, self.bend_compliance),
            (SeamCompliance, self.seam_compliance),
            (Thickness, self.thickness),
            (Friction, self.friction),
            (Damping, self.damping),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(ShellMaterialError::InvalidParameter { parameter });
            }
        }
        if self.thickness <= 0.0 {
            return Err(ShellMaterialError::NonPositiveThickness);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
