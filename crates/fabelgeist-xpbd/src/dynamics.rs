//! Particle units and the scalar operations that preserve their physical roles.
mod constraint;
mod integration;
mod mass;
mod projection;
mod scheduling;
mod time;
pub use constraint::{
    CompliancePerSubstep, ConstraintCompliance, ConstraintMultiplier, ConstraintViolation,
};
pub use integration::{DampingRate, GravityAcceleration, ParticleSpeedLimit};
pub use mass::{
    ArealDensityValidity, MassValidity, ParticleArealDensity, ParticleInverseMass, ParticleMass,
    ParticleMobility,
};
pub use projection::{
    BarycentricWeight, ClippingFraction, ConstraintGradient, EffectiveInverseMass,
    IncomingNormalSpeed, MassResponseShare, NormalSpeedCorrection, PositionCorrection,
    ProjectionActivity, ProjectionDepth, RelativeNormalSpeed,
};
pub use scheduling::{
    ConstraintSweepCount, ConstraintSweeps, SubstepCadence, SubstepCount, SubstepCycleBoundary,
    SubstepIndex, Substeps,
};
pub use time::{StepActivity, StepDuration, SubstepDuration};
