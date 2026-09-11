use std::f32::consts::TAU;

use super::{
    HelmetDesign, HelmetFit,
    geometry::{AROUND, Surface},
};
use crate::{GenerateError, parametric::PartMesh};

// The bare-head frame extends from chin to crown. The sight gap is below this edge.
pub(super) const BROW_HEIGHT: f32 = 0.09;
const KETTLE_FOREHEAD_SEATING_RISE: f32 = 0.12;

pub(super) fn generate(design: &HelmetDesign, head: [f32; 3]) -> Result<PartMesh, GenerateError> {
    let fit = design.fit();
    let gap = fit.clearance.metres() + fit.wall_thickness.metres();
    let radii = [
        head[0] + gap,
        head[1] * fit.crown_height.unit() + gap,
        head[2] + gap,
    ];
    let brow = head[1] * BROW_HEIGHT;
    match design {
        HelmetDesign::Morion(d) => brimmed(
            radii,
            brow,
            d.brim_width.metres(),
            d.brim_sweep.metres(),
            0.0,
            fit,
            BrimShape {
                crown: d.crown,
                crest: d.comb_height.metres(),
                front_reach: d.front_reach.unit(),
                back_reach: d.back_reach.unit(),
                back_sweep: d.back_sweep.unit(),
            },
        ),
        HelmetDesign::KettleHat(d) => brimmed(
            radii,
            brow + head[1] * KETTLE_FOREHEAD_SEATING_RISE,
            d.brim_width.metres(),
            0.0,
            d.brim_drop.metres(),
            fit,
            BrimShape {
                crown: d.crown,
                crest: 0.0,
                front_reach: 1.25,
                back_reach: 1.25,
                back_sweep: 1.0,
            },
        ),
        HelmetDesign::Barbute(d) => super::barbute::generate(radii, brow, head[1], d),
        HelmetDesign::Burgonet(d) => super::burgonet::generate(radii, brow, head[1], d),
        HelmetDesign::Sallet(d) => super::sallet::skull(radii, brow, head[1], d),
        HelmetDesign::VisoredSallet(d) => super::sallet::visored(radii, brow, head[1], d),
        HelmetDesign::CloseHelmet(d) => super::close::generate(radii, brow, head[1], d),
        HelmetDesign::ArmingCap(_) => {
            let mut surface = Surface::default();
            surface.full_dome(radii, brow, 0.72);
            surface.shell(fit.wall_thickness.metres(), crate::ShellExtrusion::Normal)
        }
        HelmetDesign::MailCoif(d) => super::coif::generate(radii, brow, head[1], d),
    }
}

struct BrimShape {
    crest: f32,
    crown: crate::HelmetCrown,
    front_reach: f32,
    back_reach: f32,
    back_sweep: f32,
}

fn brimmed(
    radii: [f32; 3],
    brow: f32,
    width: f32,
    sweep: f32,
    drop: f32,
    fit: HelmetFit,
    shape: BrimShape,
) -> Result<PartMesh, GenerateError> {
    const BRIM_RINGS: usize = 5;
    let mut surface = Surface::default();
    let mut ring = surface.styled_dome(radii, brow, &shape.crown, shape.crest);
    for row in 1..=BRIM_RINGS {
        let t = row as f32 / BRIM_RINGS as f32;
        let next = (0..AROUND)
            .map(|i| {
                let angle = i as f32 / AROUND as f32 * TAU;
                let end_sweep = angle.cos().abs().powi(4);
                let reach = if angle.cos() >= 0.0 {
                    shape.front_reach
                } else {
                    shape.back_reach
                };
                let sweep_scale = if angle.cos() >= 0.0 {
                    1.0
                } else {
                    shape.back_sweep
                };
                let extension = width * t * (1.0 + (reach - 1.0) * end_sweep);
                surface.vertex([
                    (radii[0] + extension) * angle.sin(),
                    brow + sweep * sweep_scale * end_sweep * t.powi(2) - drop * t,
                    (radii[2] + extension) * angle.cos(),
                ])
            })
            .collect::<Vec<_>>();
        surface.connect(&ring, &next, true);
        ring = next;
    }
    surface.shell(
        fit.wall_thickness.metres(),
        if shape.crest > 0.0 {
            crate::ShellExtrusion::AngleWeightedNormal
        } else {
            crate::ShellExtrusion::Normal
        },
    )
}
