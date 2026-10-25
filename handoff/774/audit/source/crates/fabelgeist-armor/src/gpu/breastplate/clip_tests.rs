//! Cuts preserve the actual carrier and its source correspondence.
use super::{
    clip_carrier::ClippedCarrier,
    course_clip::CourseClip,
    cut_frame::CourseFrame,
    shape_wgsl,
    topology::{MidTopology, SolidTopology, V_SAMPLES},
};
use crate::{BreastplateDesign, Millimeters};

fn carrier() -> (MidTopology, ClippedCarrier) {
    let design = BreastplateDesign::default();
    let mut mid = MidTopology::new(false, vec![-1.0, 0.0, 1.0], &design);
    let width = mid.width();
    let points = (0..mid.vertex_count())
        .map(|i| {
            let row = i / width;
            let column = i % width;
            let y = if row == V_SAMPLES - 1 {
                [0.42, 0.68, 0.42][column]
            } else if row < V_SAMPLES - 1 {
                row as f32 / (V_SAMPLES - 2) as f32
            } else {
                -((row - V_SAMPLES + 1) as f32) * 0.02
            };
            [(column as f32 - 1.0) * 0.1, y, 0.2]
        })
        .collect::<Vec<_>>();
    let clipped = ClippedCarrier::new(&mid, &points, &vec![[0.0, 0.0, 1.0]; points.len()]).unwrap();
    clipped.validate_outer(Millimeters(2)).unwrap();
    clipped.install(&mut mid);
    (mid, clipped)
}

#[test]
fn the_neckline_cuts_existing_facets_and_keeps_closed_thickness() {
    let (mid, clipped) = carrier();
    let shell = SolidTopology::new(&mid, &mid).unwrap();
    assert!(!mid.cut_columns.is_empty());
    for (vertex, &point) in clipped.positions.iter().enumerate() {
        let (a, b, blend) = mid
            .surface_column(vertex)
            .bracket(mid.width() as u32)
            .unwrap();
        let x = (a as f32 + blend * (b - a) as f32 - 1.0) * 0.1;
        assert!(
            (x - point[0]).abs() < 1e-6,
            "cut lost its continuous construction column"
        );
    }
    assert!(shell.faces.contains(&crate::PlateFace::Edge));
    for face in &mid.faces[..mid.skirt_face_start] {
        for &vertex in face {
            let [x, y, _] = clipped.positions[vertex as usize];
            let trim = 0.68 - x.abs() * 2.6;
            assert!(y <= trim + 1e-6, "carrier survived above the cut");
        }
    }
    let translated = clipped.carrier_positions(
        clipped.positions[..mid.rows * mid.width()]
            .iter()
            .map(|p| [p[0] + 0.01, p[1] - 0.02, p[2] + 0.015])
            .collect(),
    );
    for (a, b) in clipped.positions.iter().zip(translated) {
        for (k, delta) in [0.01, -0.02, 0.015].into_iter().enumerate() {
            assert!((b[k] - a[k] - delta).abs() < 1e-6);
        }
    }
}

#[test]
fn parallel_course_cuts_keep_source_edges_and_actual_construction_boundaries() {
    let (mid, clipped) = carrier();
    let mut words = shape_wgsl::design_words(&BreastplateDesign::default());
    words[32] = 1.0;
    words[37] = 1.0;
    let frame = CourseFrame::from_words(&words).unwrap();
    let course = CourseClip::new(
        &mid,
        clipped.positions.clone(),
        &frame,
        0.3,
        0.15,
        Some(0.35),
    )
    .unwrap();
    let shell = SolidTopology::new(&course.mid, &course.mid).unwrap();
    assert!(!shell.indices.is_empty());
    assert!(course.samples.len() > clipped.positions.len());
    let grid = course.mid.grid_vertices.as_ref().unwrap();
    for (&top, &bottom) in grid[..mid.width()]
        .iter()
        .zip(&grid[grid.len() - mid.width()..])
    {
        let evaluate = |vertex: u32| {
            let sample = course.samples[vertex as usize];
            let [a, b] = sample.edge.map(|i| clipped.positions[i as usize]);
            let point = std::array::from_fn(|k| a[k] + sample.blend * (b[k] - a[k]));
            frame.course_level(point, 0.3)
        };
        assert!((evaluate(top) - 0.35).abs() < 0.00025);
        assert!((evaluate(bottom) - 0.15).abs() < 0.00025);
    }
}

#[test]
fn chevron_facets_must_be_split_at_the_medial_rail() {
    let (mut mid, clipped) = carrier();
    // An arbitrary cross-medial facet cannot use one affine chevron cut.
    mid.faces = vec![[0, 2, 4]];
    let mut words = shape_wgsl::design_words(&BreastplateDesign::default());
    words[32] = 1.0;
    words[37] = 1.0;
    let frame = CourseFrame::from_words(&words).unwrap();
    assert!(matches!(
        CourseClip::new(&mid, clipped.positions, &frame, 0.3, -0.1, None),
        Err(crate::GenerateError::InvalidSurface)
    ));
}

#[test]
fn nested_trims_preserve_the_carrier_gauge_and_cut_columns() {
    let (source, original) = carrier();
    let mut mid = MidTopology::new(false, source.columns, &BreastplateDesign::default());
    let count = mid.rows * mid.width();
    // An affine, nonconstant extrusion makes lost interpolation observable.
    let direction = |p: [f32; 3]| [0.1 + p[0] * 0.2, p[1] * 0.3, 1.0 + p[0] * 0.5];
    let points = &original.positions[..count];
    let directions = points.iter().copied().map(direction).collect::<Vec<_>>();
    let mut clipped = ClippedCarrier::new(&mid, points, &directions).unwrap();
    let distances = points
        .iter()
        .map(|p| p[0].abs() * 10.0 - (1.0 - 0.25 * (p[1] / 0.2).clamp(0.0, 1.0)))
        .collect::<Vec<_>>();
    clipped.trim_arms(&distances).unwrap();
    clipped.install(&mut mid);
    let translated =
        clipped.carrier_positions(points.iter().map(|p| [p[0], p[1] + 0.02, p[2]]).collect());
    for (vertex, &point) in clipped.positions.iter().enumerate() {
        assert!((translated[vertex][1] - point[1] - 0.02).abs() < 1e-6);
        for (actual, expected) in clipped.directions[vertex].into_iter().zip(direction(point)) {
            assert!((actual - expected).abs() < 1e-6);
        }
        let (a, b, blend) = mid
            .surface_column(vertex)
            .bracket(mid.width() as u32)
            .unwrap();
        let x = (a as f32 + blend * (b - a) as f32 - 1.0) * 0.1;
        assert!((x - point[0]).abs() < 1e-6);
    }
    let mut shell = SolidTopology::new(&mid, &mid).unwrap();
    shell.compact();
    assert!(
        shell.grids[0]
            .samples
            .iter()
            .all(|s| s.column.bracket(shell.grids[0].columns).is_some())
    );
}

#[test]
fn vertical_arm_boundary_moves_continuously_across_sampled_rails() {
    let (source, original) = carrier();
    let mid = MidTopology::new(false, source.columns, &BreastplateDesign::default());
    let count = mid.rows * mid.width();
    let points = &original.positions[..count];
    let mut extents = Vec::new();
    for width in [499, 500, 501, 999, 1000] {
        let limit = 0.5 + 0.5 * width as f32 / 1000.0;
        let mut clipped = ClippedCarrier::new(&mid, points, &vec![[0.0, 0.0, 1.0]; count]).unwrap();
        clipped
            .trim_arms(
                &points
                    .iter()
                    .map(|p| {
                        p[0].abs() * 10.0 - (1.0 - (1.0 - limit) * (p[1] / 0.2).clamp(0.0, 1.0))
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        let mut trimmed = mid.clone();
        clipped.install(&mut trimmed);
        let extent = clipped
            .positions
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                clipped.positions[*i][1] > 0.25
                    && clipped.rim_vertices.contains(&(*i as u32))
                    && trimmed.faces.iter().flatten().any(|&v| v as usize == *i)
            })
            .map(|(_, p)| p[0].abs())
            .fold(0.0, f32::max);
        assert!((extent - limit * 0.1).abs() < 1e-6);
        extents.push(extent);
    }
    assert!((extents[2] - extents[0]).abs() < 0.00011);
    assert!((extents[4] - extents[3]).abs() < 0.00006);
}

#[test]
fn courses_with_missing_flank_rails_export_only_retained_construction_vertices() {
    let mid = MidTopology::course(false, vec![-1.0, -0.5, 0.0, 0.5, 1.0], 3, false);
    let points = (0..mid.vertex_count())
        .map(|i| {
            let column = i % mid.width();
            [
                (column as f32 - 2.0) * 0.05,
                (i / mid.width()) as f32 * 0.5 * [0.4, 0.8, 0.9, 0.8, 0.4][column],
                0.2,
            ]
        })
        .collect();
    let mut words = shape_wgsl::design_words(&BreastplateDesign::default());
    words[32] = 1.0;
    words[37] = 1.0;
    let frame = CourseFrame::from_words(&words).unwrap();
    let course = CourseClip::new(&mid, points, &frame, 0.0, 0.6, Some(0.7)).unwrap();
    let mut shell = SolidTopology::new(&course.mid, &course.mid).unwrap();
    shell.compact();
    let grid = &shell.grids[0];
    assert_eq!(grid.columns, 3);
    assert_eq!(
        grid.vertices.len(),
        grid.rows as usize * grid.columns as usize
    );
    assert!(
        grid.vertices
            .iter()
            .all(|vertex| shell.indices.contains(vertex))
    );
    assert!(
        grid.samples
            .iter()
            .all(|s| s.column.bracket(grid.columns).is_some())
    );
    assert!(grid.samples.len() > grid.columns as usize);
}
