//! Sewn eyelet apertures are Boolean cuts in original body-source cloth.
use crate::armor_frames::Wearer;
use crate::bodice_opening_edges::BodiceOpeningEdges;
use crate::cloth_torso_frame::ClothTorsoFrame;
use crate::surface_cut::{ClothAperture, Plane, SurfaceCut, dot, interpolate};
use anyhow::{Result, ensure};
use fabelgeist_armor::{BodiceOpening, ClothLacing, GownDesign, LacingEyelet, PartFrame};

pub(crate) fn regions(
    cut: &SurfaceCut,
    body: &Wearer<'_>,
    head: &PartFrame,
    gown: &GownDesign,
) -> Result<Vec<ClothAperture>> {
    let Some(lacing) = &gown.lacing else {
        return Ok(Vec::new());
    };
    if !matches!(lacing.eyelet, LacingEyelet::Sewn { .. }) {
        return Ok(Vec::new());
    }
    let frame = ClothTorsoFrame::new(body, head)?;
    let edges = BodiceOpeningEdges::new(cut, body, head, gown)?;
    let normals = cut
        .points
        .iter()
        .map(|p| {
            interpolate(
                body.faces[p.triangle].map(|v| body.normals[v as usize]),
                p.weights,
            )
        })
        .collect::<Vec<_>>();
    let radius = lacing.ring_radius.metres() - lacing.eyelet.gauge().metres() * 0.5;
    ensure!(radius > 0., "sewn eyelet thread consumes its aperture");
    let side = match gown.bodice.opening {
        BodiceOpening::Front { .. } => 1.,
        BodiceOpening::Back { .. } => -1.,
        BodiceOpening::Closed => anyhow::bail!("sewn eyelets require a cloth opening"),
    };
    let mut regions = Vec::new();
    for &at in &lacing.positions {
        let anchors = edges.opening.sample(at, 0)?;
        let left = anchors.left.interpolate(&edges.positions)?;
        let right = anchors.right.interpolate(&edges.positions)?;
        let across = (right - left).normalize();
        let normal = (anchors.left.interpolate(&normals)? + anchors.right.interpolate(&normals)?)
            .normalize();
        let up = normal.cross(across).normalize();
        let across = [across.x, across.y, across.z];
        let up = [up.x, up.y, up.z];
        let lateral = fabelgeist_math::Vec3::new(across[0], across[1], across[2]);
        for center in [
            left - lateral * lacing.inset.metres(),
            right + lateral * lacing.inset.metres(),
        ] {
            let center = [center.x, center.y, center.z];
            let mut planes = vec![frame.depth_half(side)];
            // The same six-facet construction as the thread-ring centerline;
            // inward gauge leaves the real cloth aperture inside its outline.
            let sides = ClothLacing::RING_CURVE_SEGMENTS;
            let apothem = radius * (std::f32::consts::PI / sides as f32).cos();
            for facet in 0..sides {
                let angle = std::f32::consts::TAU * (facet as f32 + 0.5) / sides as f32;
                let normal = std::array::from_fn(|i| across[i] * angle.cos() + up[i] * angle.sin());
                planes.push(Plane {
                    normal,
                    offset: dot(center, normal) + apothem,
                });
            }
            regions.push(ClothAperture {
                region: planes,
                center,
                across,
                up,
            });
        }
    }
    Ok(regions)
}
