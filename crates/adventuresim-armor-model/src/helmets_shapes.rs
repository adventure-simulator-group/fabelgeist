use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, PI, TAU};

use super::{
    HelmetDesign, HelmetFit,
    geometry::{AROUND, Surface, comb},
};
use crate::{GenerateError, parametric::PartMesh};

// The bare-head frame extends from chin to crown. The sight gap is below this edge.
pub(super) const BROW_HEIGHT: f32 = 0.09;
const SKIRT_ROWS: usize = 8;
const PATCH_COLUMNS: usize = 24;
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
        HelmetDesign::Morion(d) => {
            let mut shell = brimmed(
                radii,
                brow,
                d.brim_width.metres(),
                d.brim_sweep.metres(),
                0.0,
                fit,
            )?;
            shell.append(comb(
                radii,
                brow,
                d.comb_height.metres(),
                fit.wall_thickness.metres(),
            )?);
            Ok(shell)
        }
        HelmetDesign::KettleHat(d) => brimmed(
            radii,
            brow + head[1] * KETTLE_FOREHEAD_SEATING_RISE,
            d.brim_width.metres(),
            0.0,
            d.brim_drop.metres(),
            fit,
        ),
        HelmetDesign::Barbute(d) => barbute(radii, brow, head[1], d),
        HelmetDesign::Burgonet(d) => burgonet(radii, brow, head[1], d),
        HelmetDesign::Sallet(d) => sallet(radii, brow, head[1], d),
        HelmetDesign::VisoredSallet(d) => {
            let mut shell = sallet(radii, brow, head[1], &d.skull)?;
            shell.append(visor(
                radii,
                brow - d.sight_gap.metres(),
                -head[1] * 0.38,
                d.visor_projection.metres(),
                fit,
            )?);
            Ok(shell)
        }
        HelmetDesign::CloseHelmet(d) => super::close::generate(radii, brow, head[1], d),
        HelmetDesign::ArmingCap(_) => {
            let mut surface = Surface::default();
            surface.full_dome(radii, brow, 0.72);
            surface.shell(fit.wall_thickness.metres())
        }
        HelmetDesign::MailCoif(d) => super::coif::generate(radii, brow, head[1], d),
    }
}

fn brimmed(
    radii: [f32; 3],
    brow: f32,
    width: f32,
    sweep: f32,
    drop: f32,
    fit: HelmetFit,
) -> Result<PartMesh, GenerateError> {
    const BRIM_RINGS: usize = 5;
    let mut surface = Surface::default();
    let mut ring = surface.dome(radii, brow);
    for row in 1..=BRIM_RINGS {
        let t = row as f32 / BRIM_RINGS as f32;
        let next = (0..AROUND)
            .map(|i| {
                let angle = i as f32 / AROUND as f32 * TAU;
                let end_sweep = angle.cos().abs().powi(4);
                let extension = width * t * (1.0 + 0.25 * end_sweep);
                surface.vertex([
                    (radii[0] + extension) * angle.sin(),
                    brow + sweep * end_sweep * t.powi(2) - drop * t,
                    (radii[2] + extension) * angle.cos(),
                ])
            })
            .collect::<Vec<_>>();
        surface.connect(&ring, &next, true);
        ring = next;
    }
    surface.shell(fit.wall_thickness.metres())
}

fn barbute(
    radii: [f32; 3],
    brow: f32,
    half_height: f32,
    d: &super::BarbuteDesign,
) -> Result<PartMesh, GenerateError> {
    const OPENING_START: usize = AROUND / 8;
    const CHEEK_COLUMNS: usize = 4;
    let mut surface = Surface::default();
    let eye = d.eye_opening.unit();
    let mouth = d.mouth_opening.unit();
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
    let rim = surface.dome_angles(radii, brow, 1.0, &angles);
    // Extra cheek columns start at the eye shelf. Skull/rear columns never slide
    // around the circumference when the mouth opening changes.
    let mut lower_angles = (0..CHEEK_COLUMNS)
        .map(|i| mouth + (eye - mouth) * i as f32 / CHEEK_COLUMNS as f32)
        .collect::<Vec<_>>();
    lower_angles.extend(&angles[OPENING_START..=AROUND - OPENING_START]);
    lower_angles.extend(
        (1..=CHEEK_COLUMNS).map(|i| TAU - eye + (eye - mouth) * i as f32 / CHEEK_COLUMNS as f32),
    );
    let mut previous = Vec::new();
    for height in [-0.04, -0.22, -0.45, -0.68, -d.cheek_depth.unit()] {
        let ring = lower_angles
            .iter()
            .map(|&angle| {
                let jaw = (-height).max(0.0);
                let basal_return = ((jaw - 0.7) / 0.3).max(0.0) * 0.035;
                let taper = 1.0 - 0.16 * jaw.powi(2) + basal_return;
                surface.vertex([
                    radii[0] * taper * angle.sin(),
                    half_height * height,
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
    surface.shell(d.fit.wall_thickness.metres())
}

fn rear_skirt(
    surface: &mut Surface,
    rim: &[u32],
    radii: [f32; 3],
    brow: f32,
    depth: f32,
    flare: f32,
) {
    const OPENING_START: usize = AROUND / 6;
    let mut previous = rim[OPENING_START..=AROUND - OPENING_START].to_vec();
    for row in 1..=SKIRT_ROWS {
        let t = row as f32 / SKIRT_ROWS as f32;
        let ring = (0..previous.len())
            .map(|i| {
                let angle =
                    FRAC_PI_3 + (TAU - 2.0 * FRAC_PI_3) * i as f32 / (previous.len() - 1) as f32;
                let rear = (-angle.cos()).max(0.0);
                let radial = flare * t.powi(3);
                surface.vertex([
                    (radii[0] + radial * 0.35) * angle.sin(),
                    brow - depth * t * (0.7 + 0.3 * rear),
                    (radii[2] + radial * rear) * angle.cos(),
                ])
            })
            .collect::<Vec<_>>();
        surface.connect(&previous, &ring, false);
        previous = ring;
    }
}

fn sallet(
    radii: [f32; 3],
    brow: f32,
    half_height: f32,
    d: &super::SalletDesign,
) -> Result<PartMesh, GenerateError> {
    let mut surface = Surface::default();
    let mut bowl_radii = radii;
    bowl_radii[2] += d.brow_projection.metres();
    let rim = surface.dome(bowl_radii, brow);
    rear_skirt(
        &mut surface,
        &rim,
        bowl_radii,
        brow,
        half_height * 0.35 + d.tail_drop.metres(),
        d.tail_length.metres(),
    );
    surface.shell(d.fit.wall_thickness.metres())
}

fn burgonet(
    radii: [f32; 3],
    brow: f32,
    half_height: f32,
    d: &super::BurgonetDesign,
) -> Result<PartMesh, GenerateError> {
    let mut skull = Surface::default();
    let rim = skull.dome(radii, brow);
    const NAPE_START: usize = AROUND * 7 / 24;
    let mut previous = rim[NAPE_START..=AROUND - NAPE_START].to_vec();
    for row in 1..=SKIRT_ROWS {
        let t = row as f32 / SKIRT_ROWS as f32;
        let ring = (NAPE_START..=AROUND - NAPE_START)
            .map(|i| {
                let angle = i as f32 / AROUND as f32 * TAU;
                let taper = 1.0 - 0.12 * t.powi(2);
                let flare = d.neck_flare.metres() * ((t - 0.75) / 0.25).max(0.0);
                skull.vertex([
                    (radii[0] * taper + flare * 0.4) * angle.sin(),
                    brow - half_height * 0.92 * t,
                    (radii[2] * taper + flare) * angle.cos(),
                ])
            })
            .collect::<Vec<_>>();
        skull.connect(&previous, &ring, false);
        previous = ring;
    }
    let mut mesh = skull.shell(d.fit.wall_thickness.metres())?;
    let mut peak = Surface::default();
    let inner = (0..=PATCH_COLUMNS)
        .map(|i| {
            let angle = -FRAC_PI_3 + 2.0 * FRAC_PI_3 * i as f32 / PATCH_COLUMNS as f32;
            peak.vertex([radii[0] * angle.sin(), brow, radii[2] * angle.cos()])
        })
        .collect::<Vec<_>>();
    let outer = (0..=PATCH_COLUMNS)
        .map(|i| {
            let angle = -FRAC_PI_3 + 2.0 * FRAC_PI_3 * i as f32 / PATCH_COLUMNS as f32;
            let reach = d.peak_length.metres() * angle.cos();
            peak.vertex([
                (radii[0] + reach) * angle.sin(),
                brow - reach * 0.15,
                (radii[2] + reach) * angle.cos(),
            ])
        })
        .collect::<Vec<_>>();
    peak.connect(&inner, &outer, false);
    mesh.append(peak.shell(d.fit.wall_thickness.metres())?);
    for side in [-1.0, 1.0] {
        let cheek = curved_patch(
            FRAC_PI_4 * 0.85,
            PI * 0.66,
            |t, angle| {
                let across = (angle - FRAC_PI_4 * 0.85) / (PI * 0.66 - FRAC_PI_4 * 0.85);
                let rounded_edge = (PI * across).sin().max(0.0);
                let shaped_angle = angle + (0.10 - 0.15 * across) * t.powi(2);
                let taper = 1.0 - 0.10 * t;
                [
                    radii[0] * taper * shaped_angle.sin(),
                    brow - half_height * d.cheek_depth.unit() * t * (0.88 + 0.12 * rounded_edge),
                    radii[2] * shaped_angle.cos(),
                ]
            },
            d.fit.wall_thickness.metres(),
        )?;
        let cheek = if side < 0.0 { mirror(cheek) } else { cheek };
        mesh.append(cheek);
    }
    if d.comb_height.0 > 0 {
        mesh.append(comb(
            radii,
            brow,
            d.comb_height.metres(),
            d.fit.wall_thickness.metres(),
        )?);
    }
    Ok(mesh)
}

fn mirror(mut mesh: PartMesh) -> PartMesh {
    for point in &mut mesh.positions {
        point[0] = -point[0];
    }
    for triangle in mesh.indices.as_chunks_mut::<3>().0 {
        triangle.swap(1, 2);
    }
    mesh
}

pub(super) fn curved_patch(
    start: f32,
    end: f32,
    point: impl Fn(f32, f32) -> [f32; 3],
    thickness: f32,
) -> Result<PartMesh, GenerateError> {
    let mut surface = Surface::default();
    let mut previous = Vec::new();
    for row in 0..=SKIRT_ROWS {
        let t = row as f32 / SKIRT_ROWS as f32;
        let ring = (0..=PATCH_COLUMNS)
            .map(|i| {
                surface.vertex(point(
                    t,
                    start + (end - start) * i as f32 / PATCH_COLUMNS as f32,
                ))
            })
            .collect::<Vec<_>>();
        if !previous.is_empty() {
            surface.connect(&previous, &ring, false);
        }
        previous = ring;
    }
    surface.shell(thickness)
}

fn visor(
    radii: [f32; 3],
    top: f32,
    bottom: f32,
    projection: f32,
    fit: HelmetFit,
) -> Result<PartMesh, GenerateError> {
    curved_patch(
        -FRAC_PI_2,
        FRAC_PI_2,
        |t, angle| {
            let ridge = (1.0 - t).powi(2) * projection;
            [
                radii[0] * angle.sin(),
                top + (bottom - top) * t,
                radii[2] * angle.cos() + ridge * angle.cos().powi(2),
            ]
        },
        fit.wall_thickness.metres(),
    )
}
