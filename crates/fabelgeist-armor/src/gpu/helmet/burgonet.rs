//! The burgonet's skull with its nape and peak, the separate neck guard,
//! and the two cheek plates.

use std::f32::consts::{FRAC_PI_3, TAU};

use super::codes::{Slot, VertexKind};
use super::parts::{DesignFloats, HelmetParts};
use super::surface::{AROUND, CoordSurface};
use crate::gpu::coord::{CoordExtrusion, CoordShell};
use crate::pierced_plate_domain::PiercedDomain;
use crate::{
    ArmorComponentRole, ArmorDetail, BuffeDesign, BurgonetDesign, GenerateError, VisorBreaths,
};

const BURGONET_SKIRT_ROWS: usize = 8;
const BURGONET_NAPE_START: usize = AROUND * 7 / 24;
const BURGONET_PEAK_HALF_COLUMNS: usize = AROUND / 6;
const BURGONET_PEAK_ROWS: usize = 4;
/// The neck guard starts this fraction of the nape above the skull's edge.
const NECK_LAP_FRACTION: f32 = 0.05;
const CHEEK_RADIAL_ROWS: usize = 16;
const CHEEK_COLUMNS: usize = 48;
/// The chin tab's falloff around the cheek's lower front.
const CHIN_TAB_SHARPNESS: i32 = 10;

/// Which shell a helmet kernel dispatch evaluates, as its first value.
pub(super) const CHEEK_SHELL: f32 = 1.0;

pub(super) fn burgonet(
    d: &BurgonetDesign,
    mut floats: DesignFloats,
    gauge: f32,
) -> Result<HelmetParts, GenerateError> {
    for (slot, value) in [
        (Slot::Crest, d.comb_height.metres()),
        (Slot::NapeDepth, d.nape_depth.unit()),
        (Slot::NapeTaper, d.nape_taper.unit()),
        (Slot::NapeRecession, d.nape_recession.metres()),
        (Slot::NeckFlare, d.neck_flare.metres()),
        (Slot::PeakLength, d.peak_length.metres()),
        (Slot::PeakDrop, d.peak_drop.metres()),
        (Slot::PeakRise, d.peak_rise.metres()),
        (Slot::ChinTab, d.chin_tab.metres()),
        (Slot::CheekWidth, d.cheek_width.unit()),
        (Slot::CheekTaper, d.cheek_taper.unit()),
        (Slot::CheekDepth, d.cheek_depth.unit()),
    ] {
        floats.set(slot, value);
    }
    let rows = BURGONET_SKIRT_ROWS;
    let nape = BURGONET_NAPE_START..=AROUND - BURGONET_NAPE_START;
    let meridian = |i: usize| i as f32 / AROUND as f32 * TAU;
    let mut skull = CoordSurface::default();
    let rim = skull.styled_dome(&d.crown, d.comb_height.metres());
    let mut previous = rim[nape.clone()].to_vec();
    let join = 1.0 - d.neck_guard_fraction.unit();
    for row in 1..=rows {
        let v = row as f32 / rows as f32;
        let ring = nape
            .clone()
            .map(|i| skull.vertex(VertexKind::Nape, [join * v, v, meridian(i)]))
            .collect::<Vec<_>>();
        skull.connect(&previous, &ring, false);
        previous = ring;
    }
    let half = BURGONET_PEAK_HALF_COLUMNS;
    let mut previous = (0..=2 * half)
        .map(|i| rim[(AROUND - half + i) % AROUND])
        .collect::<Vec<_>>();
    for row in 1..=BURGONET_PEAK_ROWS {
        let t = row as f32 / BURGONET_PEAK_ROWS as f32;
        let next = (0..=2 * half)
            .map(|i| {
                let angle = -FRAC_PI_3 + 2.0 * FRAC_PI_3 * i as f32 / (2 * half) as f32;
                skull.vertex(VertexKind::Peak, [t, angle, 0.0])
            })
            .collect::<Vec<_>>();
        skull.connect(&previous, &next, false);
        previous = next;
    }
    let mut guard = CoordSurface::default();
    let mut previous = Vec::new();
    for row in 0..=rows {
        let t =
            join - NECK_LAP_FRACTION + (1.0 - join + NECK_LAP_FRACTION) * row as f32 / rows as f32;
        let ring = nape
            .clone()
            .map(|i| guard.vertex(VertexKind::Guard, [t, meridian(i), 0.0]))
            .collect::<Vec<_>>();
        if row > 0 {
            guard.connect(&previous, &ring, false);
        }
        previous = ring;
    }
    let cheek = cheek(d, gauge);
    let mut shells = vec![
        (skull.shell(gauge, CoordExtrusion::Normal), None),
        (
            guard.shell(
                gauge,
                CoordExtrusion::Radial {
                    origin: [0.0; 3],
                    axis: [0.0, 1.0, 0.0],
                },
            ),
            None,
        ),
    ];
    // The left cheek is the right one reflected.
    for mirrored in [true, false] {
        let mut shell = cheek.clone();
        shell.mirrored = mirrored;
        shells.push((shell, None));
    }
    if let Some(b) = &d.buffe {
        // The skull, its guard and cheeks come off apart from the buffe.
        if let Some((_, component)) = shells.last_mut() {
            *component = Some((ArmorComponentRole::Skull, false));
        }
        let courses = b.courses.as_ref();
        for (slot, value) in [
            (Slot::BuffeSightGap, b.sight_gap.metres()),
            (Slot::BuffeProjection, b.face_projection.metres()),
            (Slot::BuffeChinWidth, b.chin_width.unit()),
            (Slot::BuffeThroatDepth, b.throat_depth.unit()),
            (Slot::BuffeNeckDrop, b.neck_drop.metres()),
            (Slot::BuffeSideWrap, b.side_wrap.radians()),
            (Slot::BuffeRidge, b.medial_ridge.metres()),
            (Slot::BuffeRidgeSharpness, b.ridge_sharpness.unit()),
            (Slot::BuffeChinPoint, b.chin_point.metres()),
            (
                Slot::BuffeCourses,
                courses.map_or(0.0, |c| f32::from(c.plate_count)),
            ),
            (
                Slot::BuffeLower,
                courses.map_or(0.0, |c| c.lower_boundary.unit()),
            ),
            (
                Slot::BuffeUpper,
                courses.map_or(0.0, |c| c.upper_boundary.unit()),
            ),
            (
                Slot::BuffeOverlap,
                courses.map_or(0.0, |c| c.overlap.metres()),
            ),
            (
                Slot::BuffeLapClearance,
                courses.map_or(0.0, |c| c.lap_clearance.metres()),
            ),
            (
                Slot::BuffeBoundaryDrop,
                courses.map_or(0.0, |c| c.boundary_drop.metres()),
            ),
        ] {
            floats.set(slot, value);
        }
        let face = buffe(b, gauge)?;
        let last = face.len() - 1;
        for (index, shell) in face.into_iter().enumerate() {
            shells.push((
                shell,
                (index == last).then_some((ArmorComponentRole::Buffe, false)),
            ));
        }
    }
    Ok(HelmetParts {
        shells,
        design: floats,
    })
}

/// Rows up a buffe's face, or up each of its courses.
const BUFFE_ROWS: usize = 32;
/// Columns across a buffe's face.
const BUFFE_COLUMNS: usize = 40;
/// The pierced face's design chart, in millimetres: its half width and height.
const BUFFE_CHART_HALF_WIDTH_MM: f64 = 160.0;
const BUFFE_CHART_HEIGHT_MM: f64 = 100.0;

/// A removable face defense under the peak: one face, or overlapping
/// courses, its breaths pierced through the face or its top course. A
/// vertex's coordinates are across the face, up it and its course, which the
/// design alone decides.
fn buffe(b: &BuffeDesign, gauge: f32) -> Result<Vec<CoordShell>, GenerateError> {
    let breaths = b.breaths.filter(|breaths| breaths.count_per_row > 0);
    let courses = b.courses.map_or(1, |courses| courses.plate_count);
    (0..courses)
        .map(|course| {
            let top = course + 1 == courses;
            let surface = match breaths.filter(|_| top) {
                Some(breaths) => {
                    // The top course's chart reaches down to its lapped lower edge.
                    let height = b.courses.map_or(BUFFE_CHART_HEIGHT_MM, |courses| {
                        let low = [
                            0.0,
                            courses.lower_boundary.unit(),
                            courses.upper_boundary.unit(),
                        ][usize::from(course)];
                        BUFFE_CHART_HEIGHT_MM * f64::from(1.0 - low) + f64::from(courses.overlap.0)
                    });
                    pierced_buffe(&breaths, height, f32::from(course))?
                }
                None => buffe_grid(f32::from(course)),
            };
            Ok(surface.shell(gauge, CoordExtrusion::Normal))
        })
        .collect()
}

/// A plain face or course: rows up it, columns across it.
fn buffe_grid(course: f32) -> CoordSurface {
    let mut surface = CoordSurface::default();
    let mut previous: Vec<u32> = Vec::new();
    for row in 0..=BUFFE_ROWS {
        let v = row as f32 / BUFFE_ROWS as f32;
        let ring = (0..=BUFFE_COLUMNS)
            .map(|column| {
                let u = column as f32 / BUFFE_COLUMNS as f32;
                surface.vertex(VertexKind::Buffe, [u, v, course])
            })
            .collect::<Vec<_>>();
        if !previous.is_empty() {
            surface.connect(&ring, &previous, false);
        }
        previous = ring;
    }
    surface
}

/// A face or top course with its breaths cut through, triangulated in the
/// design chart `height` millimetres tall.
fn pierced_buffe(
    breaths: &VisorBreaths,
    height: f64,
    course: f32,
) -> Result<CoordSurface, GenerateError> {
    let outer = [
        [-BUFFE_CHART_HALF_WIDTH_MM, 0.0],
        [BUFFE_CHART_HALF_WIDTH_MM, 0.0],
        [BUFFE_CHART_HALF_WIDTH_MM, height],
        [-BUFFE_CHART_HALF_WIDTH_MM, height],
    ];
    let half_columns = BUFFE_COLUMNS as i32;
    let interior = (1..BUFFE_ROWS).flat_map(|row| {
        (-half_columns + 1..half_columns).map(move |column| {
            [
                f64::from(column) * BUFFE_CHART_HALF_WIDTH_MM / f64::from(half_columns),
                row as f64 * height / BUFFE_ROWS as f64,
            ]
        })
    });
    let holes = breaths.openings(BUFFE_CHART_HEIGHT_MM, ArmorDetail::BakeSource);
    let domain = PiercedDomain::new(&outer, &holes, interior, ArmorDetail::BakeSource)?;
    let mut surface = CoordSurface::default();
    for p in &domain.points {
        surface.vertex(
            VertexKind::Buffe,
            [
                ((p[0] / BUFFE_CHART_HALF_WIDTH_MM + 1.0) * 0.5) as f32,
                (1.0 - p[1] / height) as f32,
                course,
            ],
        );
    }
    surface.indices = domain.indices;
    Ok(surface)
}

/// A cheek plate fanned from its root: the pole, then rings of columns.
/// Its coordinates are the ring fraction, the fan angle, the chin tab and
/// the relief, all of which the design alone decides.
fn cheek(d: &BurgonetDesign, gauge: f32) -> CoordShell {
    let pattern = d.cheek_fluting.as_ref();
    let mut columns = pattern.map_or_else(
        || {
            (0..=CHEEK_COLUMNS)
                .map(|i| i as f32 / CHEEK_COLUMNS as f32)
                .collect()
        },
        |p| p.columns(CHEEK_COLUMNS),
    );
    columns.pop();
    // The cheek laps outside the rear skull sheet by one gauge of air.
    let separation = gauge * 2.0;
    let mut surface = CoordSurface::default();
    let pole = surface.coord([0.0, 0.0, 0.0, separation]);
    let mut previous: Vec<u32> = Vec::new();
    for row in 1..=CHEEK_RADIAL_ROWS {
        let v = row as f32 / CHEEK_RADIAL_ROWS as f32;
        let mut ring = Vec::new();
        for u in &columns {
            let mapped = pattern.map_or(*u, |p| p.fan_coordinate(*u, v));
            let angle = (mapped - 0.5) * TAU;
            let tab = d.chin_tab.metres()
                * (angle + 3.0 * TAU / 8.0)
                    .cos()
                    .max(0.0)
                    .powi(CHIN_TAB_SHARPNESS);
            let relief = separation + pattern.map_or(0.0, |p| p.relief(*u, v));
            ring.push(surface.coord([v, angle, tab, relief]));
        }
        if previous.is_empty() {
            for i in 0..ring.len() {
                surface
                    .indices
                    .extend([pole, ring[(i + 1) % ring.len()], ring[i]]);
            }
        } else {
            for i in 0..ring.len() {
                let next = (i + 1) % ring.len();
                surface.indices.extend([
                    previous[i],
                    previous[next],
                    ring[next],
                    previous[i],
                    ring[next],
                    ring[i],
                ]);
            }
        }
        previous = ring;
    }
    let mut shell = surface.shell(gauge, CoordExtrusion::Normal);
    shell.values[0] = CHEEK_SHELL;
    shell
}
