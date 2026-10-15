//! Reject crossings and sampled clearance loss after skirt thickening.

use anyhow::{Result, ensure};
use fabelgeist_armor::BuiltPart;
use fabelgeist_bvh::TriangleBvh;
use fabelgeist_math::Vec3;

use crate::device_textile_support::TextileSupport;

/// Allow submillimetre triangulation and floating-point roundoff. This is a
/// sampled vertex clearance check, not a continuous distance certificate.
const CLEARANCE_ROUNDOFF_M: f32 = 0.0002;

pub(crate) fn validate(part: &BuiltPart, support: &TextileSupport, clearance: f32) -> Result<()> {
    let support = TriangleBvh::new(
        support
            .positions
            .iter()
            .copied()
            .map(Vec3::from_array)
            .collect(),
        support.faces.clone(),
    );
    let (faces, remainder) = part.indices.as_chunks::<3>();
    ensure!(remainder.is_empty(), "skirt shell is not triangle aligned");
    let skirt = TriangleBvh::new(
        part.positions
            .iter()
            .copied()
            .map(Vec3::from_array)
            .collect(),
        faces.to_vec(),
    );
    crate::garment::validate_surface(&skirt, &support)?;
    let minimum = (clearance - CLEARANCE_ROUNDOFF_M).max(0.0);
    for &point in &skirt.positions {
        let (_, closest, _) = support
            .closest_point(point, f32::MAX)
            .ok_or_else(|| anyhow::anyhow!("skirt has no clearance support"))?;
        ensure!(
            (point - closest).length() >= minimum,
            "thickened skirt violates sampled inner-layer clearance"
        );
    }
    Ok(())
}
