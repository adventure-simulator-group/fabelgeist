//! Closed asymmetric wedge sections, with capped or continuously pointed ends.
use super::*;

pub(super) fn blade(p: &BladeParameters, detail: Detail) -> Result<Solid, String> {
    p.validate_form().map_err(|e| e.to_string())?;
    let point = p.point_curve()?;
    let ring = |y| section(p, y, point.as_ref());
    let mut boundaries = vec![0.0];
    if let Some(start) = p.point_start() {
        boundaries.push(start);
    }
    boundaries.push(p.length.get());
    let mut stations = vec![0.0];
    for pair in boundaries.windows(2) {
        blade_sections::refine_sections(pair[0], pair[1], &ring, detail, 0, &mut stations)?;
    }
    let rings: Vec<_> = stations.into_iter().map(ring).collect();
    let mut solid = Solid::default();
    for pair in rings.windows(2) {
        for side in 0..pair[0].len() {
            let next = (side + 1) % pair[0].len();
            blade_sections::face(
                &mut solid,
                [pair[0][side], pair[0][next], pair[1][next], pair[1][side]],
                side as u32 + 1,
            )?;
        }
    }
    cap(&mut solid, &rings[0], true)?;
    if point.is_none() {
        cap(&mut solid, rings.last().unwrap(), false)?;
    }
    Ok(solid.positive())
}

fn section(p: &BladeParameters, y: f64, point: Option<&PointCurve>) -> Vec<Point> {
    let [w, thickness, edge] = p.section_dimensions(y, point);
    let single = p.single_edge.map_or(0.0, Ratio::get);
    let center = p.curvature.map_or(0.0, Metres::get) * (y / p.length.get()).powi(2);
    if p.section == Some(ForgedBladeSection::Diamond) {
        return vec![
            [center - w, y, 0.0],
            [center, y, thickness / 2.0],
            [center + w, y, 0.0],
            [center, y, -thickness / 2.0],
        ];
    }
    let left = center - w * (1.0 - single);
    let right = center + w * (1.0 + single);
    let ridge = left + w * (1.0 - single);
    // An extreme single-edge section has its full-thickness spine on one side.
    // Omit the collinear edge-floor landmarks there before constructing faces.
    let mut ring = Vec::with_capacity(6);
    if single != 1.0 {
        ring.push([left, y, -edge / 2.0]);
    }
    ring.push([ridge, y, -thickness / 2.0]);
    if single != -1.0 {
        ring.push([right, y, -edge / 2.0]);
        ring.push([right, y, edge / 2.0]);
    }
    ring.push([ridge, y, thickness / 2.0]);
    if single != 1.0 {
        ring.push([left, y, edge / 2.0]);
    }
    ring.reverse();
    ring
}

fn cap(solid: &mut Solid, ring: &[Point], reverse: bool) -> Result<(), String> {
    let outline: Vec<_> = ring.iter().map(|p| [p[0], p[2]]).collect();
    let cap = Region::triangulate(&outline, true)?;
    for [a, b, c] in cap.triangles {
        let vertex = |i: usize| [cap.points[i][0], ring[0][1], cap.points[i][1]];
        if reverse {
            solid.triangle(vertex(a), vertex(b), vertex(c), 0);
        } else {
            solid.triangle(vertex(a), vertex(c), vertex(b), 0);
        }
    }
    Ok(())
}
