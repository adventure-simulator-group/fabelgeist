//! Fit a fixed coarse carrier around its retained dense anatomical target.
//! Numerical boundary: canonical metres, native ancestry and fixed triangles.
use crate::armor_frames::Wearer;
use crate::armor_layer::ArmorLayerSurface;
use crate::cloth_layer_seating::{CutTopology, SupportScope};
use crate::skin_transfer::SkinWeights;
use anyhow::{Context, Result};
use fabelgeist_armor::Millimeters;

pub(super) fn fit(
    positions: &mut [[f32; 3]],
    faces: &[[u32; 3]],
    target: &Wearer<'_>,
    body: &Wearer<'_>,
    clearance: Millimeters,
    banks: &super::banks::ContactBanks,
) -> Result<()> {
    // Native outward motion belongs to the retained surface, including the
    // medial proximal arm. Torso-axis radial motion cannot represent that bank.
    let points = positions
        .iter()
        .copied()
        .map(bevy::math::Vec3::from_array)
        .collect::<Vec<_>>();
    let directions = super::normal_field::outward(&points, faces)?
        .into_iter()
        .map(|direction| direction.to_array())
        .collect::<Vec<_>>();
    let table = (0..positions.len())
        .flat_map(|vertex| {
            let index = vertex as u32;
            [
                index,
                index,
                index,
                index,
                index,
                index,
                1_f32.to_bits(),
                0_f32.to_bits(),
                0_f32.to_bits(),
            ]
        })
        .collect::<Vec<_>>();
    let layer = ArmorLayerSurface::anatomy(target);
    crate::cloth_layer_seating::seat(
        positions,
        faces,
        &directions,
        body,
        &[layer],
        CutTopology {
            table: &table,
            faces,
        },
        SupportScope::RetainedAnatomy { clearance, banks },
    )
    .context("enclosing retained anatomical cloth support")
}

/// Re-evaluate dense crop skin through both frozen native barycentric levels.
pub(super) fn target_skin(
    domain: &super::domain::PanelDomain,
    body: &Wearer<'_>,
) -> Result<SkinWeights> {
    let carrier = domain.carrier_skin(body)?;
    crate::skin_transfer::interpolate_sources(
        &domain
            .cut
            .source_table(&domain.carrier.faces, &domain.carrier.faces),
        &carrier.indices,
        &carrier.weights,
    )
}
