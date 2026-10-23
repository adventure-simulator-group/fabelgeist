//! Physical admission and common face retention for sewn material interfaces.
use crate::device_fit::DevicePiece;
use anyhow::{Result, ensure};
use fabelgeist_armor::{BuiltPart, Micrometers};

// A joined interface may separate by at most one micrometre. This physical
// admission margin is much smaller than the minimum 200-micrometre tape gauge;
// it is not a floating-point error bound or a spatial matching heuristic.
pub(crate) const MAXIMUM_SEWN_INTERFACE_GAP: Micrometers = Micrometers(1);

pub(crate) fn validate_endpoints(piece: &DevicePiece) -> Result<()> {
    for endpoint in &piece.endpoints {
        ensure!(
            endpoint.positions.len() == piece.base.positions.len()
                && endpoint.indices == piece.base.indices
                && endpoint.faces == piece.base.faces
                && endpoint.components.len() == piece.base.components.len()
                && endpoint
                    .components
                    .iter()
                    .zip(&piece.base.components)
                    .all(|(a, b)| a.vertices == b.vertices && a.indices == b.indices),
            "sewn interface layout changed across realizations"
        );
    }
    Ok(())
}

pub(crate) fn retain_faces(part: &mut BuiltPart, retained: &[bool]) {
    let indices = std::mem::take(&mut part.indices);
    let faces = std::mem::take(&mut part.faces);
    for component in &mut part.components {
        let first = part.indices.len();
        for face in component.indices.start / 3..component.indices.end / 3 {
            if retained[face] {
                part.indices
                    .extend_from_slice(&indices[face * 3..face * 3 + 3]);
                part.faces.push(faces[face]);
            }
        }
        component.indices = first..part.indices.len();
    }
}
