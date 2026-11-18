//! Shared ring, constriction-band and open panel topology in a textile chart.
use super::coord::{CoordExtrusion, CoordKernel, CoordShell};
use super::recipe::PartRecipe;
use crate::{
    ArmorComponentRole, BoundaryNormals, GenerateError, Millimeters, PuffAndSlashPattern,
    PuffCourse, PuffEdge, PuffSlashes, SleeveCuff,
};

const RING_COLUMNS: usize = 32;
const PANEL_COLUMNS: usize = 2;
/// Broad panes need angular samples for their volume, independent of slashes.
const PANEL_ARC_INTERVALS: f32 = 24.;
/// Rows along a pane: enough that its bulge and its cut edges read as
/// curves.
const PANEL_ROWS: usize = 6;
/// The lining shows only through slashes; its bulge needs fewer samples.
const LINING_SLASH_ROWS: usize = 3;
/// Share of the authored opening that a slash keeps at its two ends.
const SLASH_END_OPENING: f32 = 0.05;
const BAND_ROWS: usize = 1;
/// Rows of the uncut cloth between a band and the ends of the slashes, where
/// the puff swells out from under the band.
const PUFF_END_ROWS: usize = 2;
const CUFF_ROWS: usize = 2;
/// Separate cuff and panel boundaries by a construction-scale seam.
const PANEL_SEAM_GAP_M: f32 = 0.0001;

#[derive(Clone, Copy)]
enum Fabric {
    Lining,
    Puff,
    Band,
    Cuff,
}

pub(super) fn lining(thickness: Millimeters, design: &PuffAndSlashPattern) -> CoordShell {
    let spans = design.spans();
    // Lining shows only through slashes. Above the last slashed course it
    // would be a hidden second cloth, so it ends under the band that closes
    // that course.
    let top = design
        .courses
        .iter()
        .rposition(|course| course.slashes.is_some())
        .and_then(|last| {
            let next = spans.get(last + 1)?;
            Some((spans[last].puff()[1] + next.puff()[0]) * 0.5)
        })
        .unwrap_or(1.);
    let mut stations = vec![0., top];
    for (course, span) in design.courses.iter().zip(&spans) {
        let puff = span.puff();
        stations.extend(puff);
        let (first, rows, open) = match &course.slashes {
            Some(slashes) => (0, LINING_SLASH_ROWS, slash_span(puff, slashes)),
            None => (1, puff_rows(course), puff),
        };
        for row in first..=rows - first {
            stations.push(open[0] + (open[1] - open[0]) * row as f32 / rows as f32);
        }
    }
    stations.retain(|&station| station <= top);
    stations.sort_by(f32::total_cmp);
    stations.dedup();
    let mut shell = patch(
        thickness,
        [0.0, 1.0],
        [0.0; 2],
        None,
        stations.len() - 1,
        Fabric::Lining,
    );
    // The lining's role word is the share of fullness it gives up.
    let setback = 1. - design.lining_fullness.unit();
    for (ring, &height) in shell.coords.chunks_mut(RING_COLUMNS).zip(&stations) {
        for point in ring {
            point[1] = height;
            point[3] = setback;
        }
    }
    shell
}

/// Released puffs form the outer fabric; the fitted bands that gather them
/// are a separate component with their own cloth.
pub(super) fn append_outer(
    recipe: &mut PartRecipe,
    kernel: &CoordKernel,
    design: &PuffAndSlashPattern,
    thickness: Millimeters,
) -> Result<(), GenerateError> {
    let spans = design.spans();
    for (course, span) in design.courses.iter().zip(&spans) {
        add_puff(recipe, kernel, design, thickness, course, span.puff())?;
    }
    // The closing ring is fitted like a band whichever cloth it is cut from.
    let closing = spans.last().map_or(0., |span| span.puff()[1]);
    let close = |recipe: &mut PartRecipe| {
        recipe.push_coord(
            patch(
                thickness,
                [closing, 1.],
                [0.0; 2],
                None,
                BAND_ROWS,
                Fabric::Band,
            ),
            kernel.clone(),
        )
    };
    if design.proximal_edge == PuffEdge::Seam {
        close(recipe)?;
    }
    recipe.component(ArmorComponentRole::OuterFabric, None);
    let mut low = 0.;
    for span in &spans {
        let band = [low, span.puff()[0]];
        recipe.push_coord(
            patch(thickness, band, [0.0; 2], None, BAND_ROWS, Fabric::Band),
            kernel.clone(),
        )?;
        low = span.puff()[1];
    }
    if design.proximal_edge == PuffEdge::Band {
        close(recipe)?;
    }
    recipe.component(ArmorComponentRole::ConstrictionBand, None);
    Ok(())
}

/// A fitted course has no bulge to sample between its band edges.
fn puff_rows(course: &PuffCourse) -> usize {
    if course.fullness.0 == 0 {
        BAND_ROWS
    } else {
        PANEL_ROWS
    }
}

/// A ring below the distal end; its seam word is metres from the wrist.
pub(super) fn append_cuff(
    recipe: &mut PartRecipe,
    kernel: &CoordKernel,
    cuff: &SleeveCuff,
    thickness: Millimeters,
) -> Result<(), GenerateError> {
    recipe.push_coord(
        patch(
            thickness,
            [0.0; 2],
            [-cuff.length.metres(), 0.0],
            None,
            CUFF_ROWS,
            Fabric::Cuff,
        ),
        kernel.clone(),
    )?;
    recipe.component(ArmorComponentRole::Cuff, None);
    Ok(())
}

/// The slashed interval, centred between the two band edges of a puff.
fn slash_span(puff: [f32; 2], slashes: &PuffSlashes) -> [f32; 2] {
    let margin = (puff[1] - puff[0]) * (1.0 - slashes.length.unit()) * 0.5;
    [puff[0] + margin, puff[1] - margin]
}

fn patch(
    thickness: crate::Millimeters,
    span: [f32; 2],
    seams: [f32; 2],
    angles: Option<[f32; 2]>,
    rows: usize,
    fabric: Fabric,
) -> CoordShell {
    let periodic = angles.is_none();
    let columns = if periodic {
        RING_COLUMNS
    } else {
        let span = angles.map_or(0., |arc| arc[1] - arc[0]);
        ((span / std::f32::consts::TAU * PANEL_ARC_INTERVALS).ceil() as usize).max(PANEL_COLUMNS)
    };
    let stride = columns + usize::from(!periodic);
    let angles = angles.unwrap_or([0.0, std::f32::consts::TAU]);
    let mut coords = Vec::with_capacity(stride * (rows + 1));
    let mut indices = Vec::with_capacity(columns * rows * 6);
    for row in 0..=rows {
        let t = row as f32 / rows as f32;
        for column in 0..stride {
            coords.push([
                angles[0] + (angles[1] - angles[0]) * column as f32 / columns as f32,
                span[0] + (span[1] - span[0]) * t,
                seams[0] + (seams[1] - seams[0]) * t,
                match fabric {
                    Fabric::Lining => 0.0,
                    Fabric::Puff => 1.0,
                    Fabric::Band => 2.0,
                    Fabric::Cuff => 3.0,
                },
            ]);
        }
    }
    for row in 0..rows {
        for column in 0..columns {
            let next = (column + 1) % stride;
            let a = (row * stride + column) as u32;
            let b = (row * stride + next) as u32;
            let c = ((row + 1) * stride + column) as u32;
            let d = ((row + 1) * stride + next) as u32;
            indices.extend([a, b, d, a, d, c]);
        }
    }
    CoordShell {
        coords,
        indices,
        boundary_normals: BoundaryNormals::Smooth,
        thickness: thickness.metres(),
        extrusion: CoordExtrusion::Normal,
        values: [0.0; 4],
        mirrored: false,
        frame: 0,
        passes: 1,
        hinge: None,
    }
}

fn add_puff(
    recipe: &mut PartRecipe,
    kernel: &CoordKernel,
    design: &PuffAndSlashPattern,
    thickness: crate::Millimeters,
    course: &PuffCourse,
    puff: [f32; 2],
) -> Result<(), GenerateError> {
    let Some(slashes) = &course.slashes else {
        recipe.push_coord(
            patch(
                thickness,
                puff,
                [0.0; 2],
                None,
                puff_rows(course),
                Fabric::Puff,
            ),
            kernel.clone(),
        )?;
        return Ok(());
    };
    let slash = slash_span(puff, slashes);
    for (span, seams) in [
        ([puff[0], slash[0]], [0.0, -PANEL_SEAM_GAP_M]),
        ([slash[1], puff[1]], [PANEL_SEAM_GAP_M, 0.0]),
    ] {
        recipe.push_coord(
            patch(thickness, span, seams, None, PUFF_END_ROWS, Fabric::Puff),
            kernel.clone(),
        )?;
    }
    let repeat = std::f32::consts::TAU / f32::from(slashes.count);
    let half_width = repeat * (1.0 - slashes.width.unit()) * 0.5;
    let closed_half_width = repeat * 0.5;
    for panel in 0..slashes.count {
        let center = repeat * (f32::from(panel) + design.rotation.unit());
        let mut pane = patch(
            thickness,
            slash,
            [PANEL_SEAM_GAP_M, -PANEL_SEAM_GAP_M],
            Some([center - half_width, center + half_width]),
            PANEL_ROWS,
            Fabric::Puff,
        );
        // A cut gapes most at its authored widest point and closes toward
        // its ends; a slant carries each pane around the limb as it runs.
        let stride = pane.coords.len() / (PANEL_ROWS + 1);
        let widest = slashes.widest.unit();
        for (row, points) in pane.coords.chunks_mut(stride).enumerate() {
            let along = row as f32 / PANEL_ROWS as f32;
            let phase = if along < widest {
                along / widest
            } else {
                1. + (along - widest) / (1. - widest)
            };
            let gape = (std::f32::consts::FRAC_PI_2 * phase).sin();
            let opening = SLASH_END_OPENING + (1.0 - SLASH_END_OPENING) * gape;
            let lean = repeat * slashes.slant.unit() * (along - 0.5);
            for point in points {
                let widened = closed_half_width - (closed_half_width - half_width) * opening;
                point[0] = center + lean + (point[0] - center) * widened / half_width;
            }
        }
        recipe.push_coord(pane, kernel.clone())?;
    }

    Ok(())
}
