//! Conforming surfaces of a single forged blank, including its closed apex.
use super::*;

const PLATE_CHORD: f64 = 0.008;
const PLATE_CURVE_DEVIATION: f64 = 0.00015;
const PLATE_SURFACE_EDGE: f64 = 0.012;
const PLATE_SURFACE_DEVIATION: f64 = 0.00005;
const MAX_PLATE_OUTLINE_POINTS: usize = 4096;

pub(super) fn construct(p: &ContouredPlateParameters, detail: Detail) -> Result<Solid, String> {
    match &p.surface {
        PlateSurface::Ridge { stations } => construct_surface(
            p,
            detail,
            plate_ridge::RidgeField {
                length: p.length.get(),
                stations,
            },
        ),
        PlateSurface::Profile { stations } => construct_surface(
            p,
            detail,
            plate_profile::ProfileField {
                length: p.length.get(),
                width: p.width.get(),
                stations,
            },
        ),
    }
}

pub(super) trait PlateField {
    type Cell: Ord;
    fn cuts(&self) -> Vec<PlanarCut>;
    fn cell(&self, point: PlanarPoint) -> Self::Cell;
    fn thickness(&self, x: f64, y: f64) -> f64;
}

fn construct_surface(
    p: &ContouredPlateParameters,
    detail: Detail,
    field: impl PlateField,
) -> Result<Solid, String> {
    let surface_edge = detail.error(PLATE_SURFACE_EDGE);
    construction_budget(8.0 * p.width.get() * p.length.get() / surface_edge.powi(2))?;
    let cuts = field.cuts();
    let outline = outline(p, detail, &cuts)?;
    let mut region = Region::triangulate(&outline, false)?;
    for cut in cuts {
        region.partition(cut)?;
    }
    region.remesh_cells(|point| field.cell(point))?;
    validate_apices(p, &field, &region)?;
    region.refine_rational_surface(
        |point| field.cell(point),
        |[x, y]| field.thickness(x, y) / 2.0,
        detail.error(PLATE_SURFACE_DEVIATION),
        surface_edge,
    )?;
    region.improve_surface_cells(
        |point| field.cell(point),
        |[x, y]| field.thickness(x, y) / 2.0,
        detail.error(PLATE_SURFACE_DEVIATION),
        surface_edge,
    );
    lift(p, &field, region)
}

fn lift(
    p: &ContouredPlateParameters,
    field: &impl PlateField,
    region: Region,
) -> Result<Solid, String> {
    construction_budget((region.triangles.len() * 2 + region.boundary.len() * 2) as f64)?;
    validate_apices(p, field, &region)?;
    let vertex = |i: usize, side: f64| {
        let [x, y] = region.points[i];
        [x, y, side * field.thickness(x, y) / 2.0]
    };
    let mut solid = Solid::default();
    for &[a, b, c] in &region.triangles {
        solid.triangle(vertex(a, 1.0), vertex(b, 1.0), vertex(c, 1.0), 0);
        solid.triangle(vertex(a, -1.0), vertex(c, -1.0), vertex(b, -1.0), 0);
    }
    for i in 0..region.boundary.len() {
        let a = region.boundary[i];
        let b = region.boundary[(i + 1) % region.boundary.len()];
        let points = [
            vertex(a, -1.0),
            vertex(b, -1.0),
            vertex(b, 1.0),
            vertex(a, 1.0),
        ];
        let mut unique = Vec::new();
        for point in points {
            if !unique.contains(&point) {
                unique.push(point);
            }
        }
        for j in 1..unique.len().saturating_sub(1) {
            solid.triangle(unique[0], unique[j], unique[j + 1], 0);
        }
    }
    Ok(solid.positive())
}

fn validate_apices(
    p: &ContouredPlateParameters,
    field: &impl PlateField,
    region: &Region,
) -> Result<(), String> {
    if !matches!(&p.surface, PlateSurface::Profile { .. }) {
        return Ok(());
    }
    let height: Vec<_> = region
        .points
        .iter()
        .map(|&[x, y]| field.thickness(x, y))
        .collect();
    let endpoints: Vec<_> = std::iter::once(p.start)
        .chain(p.boundary.iter().map(PlateBoundarySpan::end))
        .map(|point| {
            [
                point[0].get() * p.width.get(),
                point[1].get() * p.length.get(),
            ]
        })
        .collect();
    for (index, &value) in height.iter().enumerate() {
        if value > 0.0 {
            continue;
        }
        let boundary = region.boundary.iter().position(|&i| i == index);
        let isolated = boundary.is_some_and(|i| {
            let count = region.boundary.len();
            height[region.boundary[(i + count - 1) % count]] > 0.0
                && height[region.boundary[(i + 1) % count]] > 0.0
        });
        if value != 0.0 || !isolated || !endpoints.contains(&region.points[index]) {
            return Err(
                "plate profile permits zero thickness only at isolated authored boundary apices"
                    .into(),
            );
        }
    }
    if region
        .triangles
        .iter()
        .any(|face| face.iter().all(|&i| height[i] == 0.0))
    {
        return Err("plate profile leaves a zero-thickness surface".into());
    }
    Ok(())
}

fn outline(
    p: &ContouredPlateParameters,
    detail: Detail,
    cuts: &[PlanarCut],
) -> Result<Vec<PlanarPoint>, String> {
    let scale = |point: [Ratio; 2]| {
        [
            point[0].get() * p.width.get(),
            point[1].get() * p.length.get(),
        ]
    };
    let mut previous = scale(p.start);
    let mut result = vec![previous];
    for span in &p.boundary {
        let end = scale(span.end());
        match span {
            PlateBoundarySpan::Line { .. } => result.push(end),
            PlateBoundarySpan::Cubic { controls, .. } => append_curve(
                &mut result,
                partitioned_cubic(
                    [previous, scale(controls[0]), scale(controls[1]), end],
                    CurveQuality {
                        minimum_segments: 4,
                        max_chord: PLATE_CHORD,
                        max_deviation: PLATE_CURVE_DEVIATION,
                    },
                    detail,
                    cuts,
                ),
            ),
        }
        if result.len() > MAX_PLATE_OUTLINE_POINTS {
            return Err("plate outline exceeds its bounded sampling budget".into());
        }
        previous = end;
    }
    result.pop();
    Ok(result)
}
