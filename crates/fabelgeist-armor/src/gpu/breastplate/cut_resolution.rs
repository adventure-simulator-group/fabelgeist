//! Precision of metal cuts through the evaluated, densely sampled carrier.
/// Resolve boundary levels to a quarter millimetre at an existing vertex.
/// Smaller corner fragments can have unusable f32 shading normals, while this
/// tolerance retains the carrier surface and remains well below a plate gauge.
pub(super) const CUT_RESOLUTION_METERS: f64 = 0.00025;
