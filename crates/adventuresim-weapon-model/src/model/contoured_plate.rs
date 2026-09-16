//! Conforming surfaces of a single forged blank, including its closed apex.
use super::*;

const PLATE_CHORD: f64 = 0.008;
const PLATE_CURVE_DEVIATION: f64 = 0.00015;
const PLATE_SURFACE_EDGE: f64 = 0.012;
const PLATE_SURFACE_DEVIATION: f64 = 0.00005;
const MAX_PLATE_OUTLINE_POINTS: usize = 4096;

pub(super) fn construct(p: &ContouredPlateParameters, detail: Detail) -> Result<Solid, String> {
    let surface_edge = detail.error(PLATE_SURFACE_EDGE);
    construction_budget(8.0 * p.width.get() * p.length.get() / surface_edge.powi(2))?;
    let cuts = cuts(p);
    let outline = outline(p, detail, &cuts)?;
    let mut region = Region::triangulate(&outline, false)?;
    for cut in cuts {
        region.partition(cut)?;
    }
    region.remesh_cells(|point| cell(p, point))?;
    region.refine_rational_surface(
        |point| cell(p, point),
        |[x, y]| thickness(p, x, y) / 2.0,
        detail.error(PLATE_SURFACE_DEVIATION),
        surface_edge,
    )?;
    region.improve_surface_cells(
        |point| cell(p, point),
        |[x, y]| thickness(p, x, y) / 2.0,
        detail.error(PLATE_SURFACE_DEVIATION),
        surface_edge,
    );
    lift(p, region)
}

fn cuts(p: &ContouredPlateParameters) -> Vec<PlanarCut> {
    let mut cuts = Vec::new();
    for station in &p.thickness {
        cuts.push(PlanarCut::Axial(station.at.get() * p.length.get()));
    }
    for pair in p.thickness.windows(2) {
        if pair.iter().all(|s| s.edge == s.ridge) {
            continue;
        }
        for side in [-1.0, 1.0] {
            let mut fractions = vec![1.0, 0.0];
            if pair.iter().any(|s| s.hollow_depth() > 0.0) {
                fractions.push(0.5);
            }
            for fraction in fractions {
                let point = |s: &PlateThicknessStation| {
                    [
                        side * if fraction == 1.0 {
                            s.ridge_half_width.get()
                        } else if fraction == 0.0 {
                            s.flat_half_width.get()
                        } else {
                            (s.flat_half_width.get() + s.ridge_half_width.get()) / 2.0
                        },
                        s.at.get() * p.length.get(),
                    ]
                };
                cuts.push(PlanarCut::Transverse {
                    start: point(&pair[0]),
                    end: point(&pair[1]),
                });
            }
        }
    }
    cuts
}

fn lift(p: &ContouredPlateParameters, region: Region) -> Result<Solid, String> {
    construction_budget((region.triangles.len() * 2 + region.boundary.len() * 2) as f64)?;
    let vertex = |i: usize, side: f64| {
        let [x, y] = region.points[i];
        [x, y, side * thickness(p, x, y) / 2.0]
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum PlateBand {
    Flat,
    LeftSlope,
    RightSlope,
    LeftOuterSlope,
    RightOuterSlope,
    LeftEdge,
    RightEdge,
}

fn cell(p: &ContouredPlateParameters, [x, y]: PlanarPoint) -> (usize, PlateBand) {
    let t = (y / p.length.get()).clamp(0.0, 1.0);
    let index = p
        .thickness
        .windows(2)
        .position(|pair| t <= pair[1].at.get())
        .unwrap();
    let a = &p.thickness[index];
    let b = &p.thickness[index + 1];
    if a.edge == a.ridge && b.edge == b.ridge {
        return (index, PlateBand::Flat);
    }
    let u = (t - a.at.get()) / (b.at.get() - a.at.get());
    let half = a.ridge_half_width.get() + (b.ridge_half_width.get() - a.ridge_half_width.get()) * u;
    let flat = a.flat_half_width.get() + (b.flat_half_width.get() - a.flat_half_width.get()) * u;
    let band = if x.abs() <= flat {
        PlateBand::Flat
    } else if x.abs() <= half {
        let outer =
            (a.hollow_depth() > 0.0 || b.hollow_depth() > 0.0) && x.abs() > (flat + half) / 2.0;
        if outer && x < 0.0 {
            PlateBand::LeftOuterSlope
        } else if outer {
            PlateBand::RightOuterSlope
        } else if x < 0.0 {
            PlateBand::LeftSlope
        } else {
            PlateBand::RightSlope
        }
    } else if x < 0.0 {
        PlateBand::LeftEdge
    } else {
        PlateBand::RightEdge
    };
    (index, band)
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

fn thickness(p: &ContouredPlateParameters, x: f64, y: f64) -> f64 {
    let t = (y / p.length.get()).clamp(0.0, 1.0);
    let stations = p.thickness.windows(2).find(|s| t <= s[1].at.get()).unwrap();
    let u = (t - stations[0].at.get()) / (stations[1].at.get() - stations[0].at.get());
    let edge = stations[0].edge.get() + (stations[1].edge.get() - stations[0].edge.get()) * u;
    let ridge = stations[0].ridge.get() + (stations[1].ridge.get() - stations[0].ridge.get()) * u;
    if edge == 0.0 && ridge == 0.0 {
        return 0.0;
    }
    let half = stations[0].ridge_half_width.get()
        + (stations[1].ridge_half_width.get() - stations[0].ridge_half_width.get()) * u;
    let flat = stations[0].flat_half_width.get()
        + (stations[1].flat_half_width.get() - stations[0].flat_half_width.get()) * u;
    let q = ((half - x.abs()) / (half - flat)).clamp(0.0, 1.0);
    let hollow =
        stations[0].hollow_depth() + (stations[1].hollow_depth() - stations[0].hollow_depth()) * u;
    edge + (ridge - edge) * q - 4.0 * hollow * q.min(1.0 - q)
}
