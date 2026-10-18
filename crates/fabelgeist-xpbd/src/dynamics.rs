//! Particle units and the scalar operations that preserve their physical roles.

pub use constraint::{CompliancePerSubstep, ConstraintMultiplier, ConstraintViolation};
pub use damping::DampingRate;
pub use mass::{
    ArealDensityValidity, MassValidity, ParticleArealDensity, ParticleInverseMass, ParticleMass,
    ParticleMobility,
};
pub use projection::{
    BarycentricWeight, ClippingFraction, ConstraintGradient, EffectiveInverseMass,
    IncomingNormalSpeed, MassResponseShare, NormalSpeedCorrection, PositionCorrection,
    ProjectionActivity, ProjectionDepth, RelativeNormalSpeed,
};
mod constraint;
mod damping;
mod mass;
mod projection;
