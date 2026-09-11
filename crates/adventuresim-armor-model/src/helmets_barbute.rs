//! Integrated barbute bowl with independently sized, rounded T opening.
use super::geometry::{AROUND, Surface};
use crate::{GenerateError, PartMesh};
use std::f32::consts::{PI, TAU};

const OPENING_START: usize = AROUND / 8;
fn bowl(
    surface: &mut Surface,
    radii: [f32; 3],
    brow: f32,
    d: &super::BarbuteDesign,
) -> (Vec<u32>, Vec<f32>) {
    let eye = d.eye_opening.radians();
    let angles = (0..AROUND)
        .map(|i| {
            let half = i.min(AROUND - i);
            let angle = if half <= OPENING_START {
                eye * half as f32 / OPENING_START as f32
            } else {
                eye + (PI - eye) * (half - OPENING_START) as f32
                    / (AROUND / 2 - OPENING_START) as f32
            };
            if i > AROUND / 2 { TAU - angle } else { angle }
        })
        .collect::<Vec<_>>();
    let rim = surface.styled_dome(radii, brow, &d.crown, 0.0);
    for point in &mut surface.positions {
        let x = point[0] / radii[0];
        let z = point[2] / radii[2];
        let radius = x.hypot(z);
        let original = x.atan2(z).rem_euclid(TAU);
        let coordinate = original / TAU * AROUND as f32;
        let index = coordinate.floor() as usize % AROUND;
        let next = if index + 1 == AROUND {
            angles[0] + TAU
        } else {
            angles[index + 1]
        };
        let angle = angles[index] + (next - angles[index]) * coordinate.fract();
        point[0] = radii[0] * radius * angle.sin();
        point[2] = radii[2] * radius * angle.cos();
    }
    (rim, angles)
}

pub(super) fn generate(
    radii: [f32; 3],
    brow: f32,
    half_height: f32,
    d: &super::BarbuteDesign,
) -> Result<PartMesh, GenerateError> {
    const CHEEK_COLUMNS: usize = 12;
    let mut surface = Surface::default();
    let eye = d.eye_opening.radians();
    let mouth = d.mouth_opening.radians();
    let (rim, angles) = bowl(&mut surface, radii, brow, d);
    // Extra cheek columns start at the eye shelf. Skull/rear columns never slide
    // around the circumference when the mouth opening changes.
    let mut lower_angles = (0..CHEEK_COLUMNS)
        .map(|i| mouth + (eye - mouth) * i as f32 / CHEEK_COLUMNS as f32)
        .collect::<Vec<_>>();
    lower_angles.extend(&angles[OPENING_START..=AROUND - OPENING_START]);
    lower_angles.extend(
        (1..=CHEEK_COLUMNS).map(|i| TAU - eye + (eye - mouth) * i as f32 / CHEEK_COLUMNS as f32),
    );
    let shelf = brow - d.eye_height.metres();
    let corner = d.eye_height.metres() * 0.45 * d.opening_roundness.unit();
    let corner_angle = (corner / radii[0]).min((eye - mouth) * 0.35);
    let lower = -half_height * d.cheek_depth.unit();
    let mut heights = vec![shelf];
    if corner > 1e-6 {
        heights.extend((1..=4).map(|i| shelf - corner * i as f32 / 4.0));
    }
    let start = shelf - corner;
    heights.extend((1..=8).map(|i| start + (lower - start) * i as f32 / 8.0));
    let mut previous = Vec::new();
    for (row, y) in heights.into_iter().enumerate() {
        let ring = lower_angles
            .iter()
            .map(|&angle| {
                let lower_blend = if corner > 1e-6 {
                    ((shelf - y) / corner).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                let inner_round = corner_angle * (1.0 - (1.0 - (1.0 - lower_blend).powi(2)).sqrt());
                let absolute = angle.min(TAU - angle);
                let shift =
                    inner_round * (1.0 - ((absolute - mouth) / (eye - mouth)).clamp(0.0, 1.0));
                let angle = angle + if angle < PI { shift } else { -shift };
                let outer_round = if row == 0 && corner_angle > 1e-6 {
                    let t = ((absolute - (eye - corner_angle)) / corner_angle).clamp(0.0, 1.0);
                    corner * (1.0 - (1.0 - t * t).sqrt())
                } else {
                    0.0
                };
                let descent = ((shelf - y) / (shelf - lower)).clamp(0.0, 1.0);
                let y = y
                    + outer_round
                    + d.rear_edge_lift.metres()
                        * descent.powi(2)
                        * ((1.0 - angle.cos()) * 0.5).powi(2);
                let jaw = (-y / half_height).max(0.0);
                let basal_return = ((jaw - 0.7) / 0.3).max(0.0) * d.nape_flare.metres() / radii[0];
                let taper = 1.0 - (1.0 - d.chin_taper.unit()) * jaw.powi(2) + basal_return;
                surface.vertex([
                    radii[0] * taper * angle.sin(),
                    y,
                    radii[2]
                        * (1.0 - 0.12 * jaw.powi(2) * (-angle.cos()).max(0.0) + basal_return)
                        * angle.cos(),
                ])
            })
            .collect::<Vec<_>>();
        if previous.is_empty() {
            surface.connect(
                &rim[OPENING_START..=AROUND - OPENING_START],
                &ring[CHEEK_COLUMNS..ring.len() - CHEEK_COLUMNS],
                false,
            );
        } else {
            surface.connect(&previous, &ring, false);
        }
        previous = ring;
    }
    surface.shell(d.fit.wall_thickness.metres(), crate::ShellExtrusion::Normal)
}
