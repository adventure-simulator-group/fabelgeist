//! Continuous forged exterior, blind receiving cavity, and integral basal stops.
use super::*;
use std::f64::consts::{FRAC_PI_2, PI, TAU};

pub(super) const BLADE_CREASE_COSINE: f64 = 0.98;

#[derive(Clone, Copy)]
struct Station {
    y: f64,
    stops: bool,
}

pub(super) fn construct(p: &SpearParameters, detail: Detail) -> Result<Solid, String> {
    let socket = p
        .socket
        .as_ref()
        .ok_or("socketed blade needs socket dimensions")?;
    let rows = stations(p, socket, detail);
    let angles = angles(socket, p, &rows, detail)?;
    construction_budget((rows.len() + 4) as f64 * angles.len() as f64 * 2.0)?;
    let rings: Vec<Vec<Point>> = rows
        .iter()
        .map(|row| {
            angles
                .iter()
                .map(|&angle| outer(p, socket, *row, angle))
                .collect()
        })
        .collect();
    let mut solid = Solid::default();
    for (index, pair) in rings.windows(2).enumerate() {
        connect(&mut solid, &pair[0], &pair[1], false, |_| {
            if rows[index].y >= 0.0 { 2 } else { 1 }
        });
    }
    let last = rings.last().ok_or("socketed blade needs stations")?;
    for side in 0..angles.len() {
        triangle(
            &mut solid,
            last[side],
            [0.0, p.length.get(), 0.0],
            last[(side + 1) % angles.len()],
            2,
        );
    }
    let inner = |height: f64| -> Vec<Point> {
        angles
            .iter()
            .map(|&angle| {
                let radius = socket.bore_radius(height);
                [
                    radius * angle.cos(),
                    height - socket.length.get(),
                    radius * angle.sin(),
                ]
            })
            .collect()
    };
    let bottom = inner(0.0);
    let cap = inner(socket.cavity_depth.get());
    connect(&mut solid, &bottom, &cap, true, |_| 6);
    connect(&mut solid, &bottom, &rings[0], false, |_| 7);
    for side in 0..angles.len() {
        triangle(
            &mut solid,
            [0.0, socket.cavity_depth.get() - socket.length.get(), 0.0],
            cap[side],
            cap[(side + 1) % angles.len()],
            8,
        );
    }
    Ok(solid.positive())
}

fn stations(p: &SpearParameters, socket: &SpearSocket, detail: Detail) -> Vec<Station> {
    let samples = detail.samples(p.samples.map_or(24, |n| n.0 as usize), 12);
    let belly = p.belly_position.or(p.shoulder).map_or(0.18, Ratio::get);
    let mut heights = vec![
        -socket.length.get(),
        -socket.neck_length.get(),
        0.0,
        belly * p.length.get(),
    ];
    heights.extend(
        spears::outline_stations(p, detail)
            .into_iter()
            .map(|t| t * p.length.get())
            .filter(|y| *y < p.length.get()),
    );
    heights.extend((1..samples).map(|n| -socket.neck_length.get() * n as f64 / samples as f64));
    if let Some(stops) = &socket.stops {
        let bottom = stops.center_height.get() - stops.root_height.get() - socket.length.get();
        let end_bottom =
            stops.center_height.get() - stops.end_height.get() / 2.0 - socket.length.get();
        let top = stops.center_height.get() + stops.end_height.get() / 2.0 - socket.length.get();
        heights.extend(
            adaptive_curve(
                |t| {
                    let y = bottom + (end_bottom - bottom) * (2.0 * t - t * t);
                    let radius = circular_radius(p, socket, y);
                    let root = (radius * radius - (stops.thickness.get() / 2.0).powi(2)).sqrt();
                    [root + (stops.span.get() / 2.0 - root) * t * t, y]
                },
                CurveQuality {
                    minimum_segments: 12,
                    max_chord: stops.span.get() / 16.0,
                    max_deviation: stops.root_height.get() / 800.0,
                },
                detail,
            )
            .into_iter()
            .map(|point| point[1]),
        );
        heights.extend([end_bottom, top]);
    }
    heights.sort_by(f64::total_cmp);
    heights.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
    let mut rows = Vec::new();
    for y in heights {
        rows.push(Station { y, stops: true });
        if socket.stops.as_ref().is_some_and(|s| {
            (y - (s.center_height.get() + s.end_height.get() / 2.0 - socket.length.get())).abs()
                < 1e-12
        }) {
            rows.push(Station { y, stops: false });
        }
    }
    rows
}

fn circular_radius(p: &SpearParameters, socket: &SpearSocket, y: f64) -> f64 {
    let top = p.root_width.map_or(p.width.get() * 0.4, Metres::get) / 2.0;
    socket.outer_barrel_radius(top, y + socket.length.get())
}

fn wing_width(p: &SpearParameters, socket: &SpearSocket, row: Station) -> Option<f64> {
    let stops = socket.stops.as_ref()?;
    let height = row.y + socket.length.get();
    let bottom = stops.center_height.get() - stops.root_height.get();
    let end_bottom = stops.center_height.get() - stops.end_height.get() / 2.0;
    let top = stops.center_height.get() + stops.end_height.get() / 2.0;
    if !row.stops || height < bottom || height > top + 1e-12 {
        return None;
    }
    let radius = circular_radius(p, socket, row.y);
    let root = (radius * radius - (stops.thickness.get() / 2.0).powi(2)).sqrt();
    let progress = ((height - bottom) / (end_bottom - bottom)).clamp(0.0, 1.0);
    let t = 1.0 - (1.0 - progress).sqrt();
    Some(root + (stops.span.get() / 2.0 - root) * t * t)
}

fn angles(
    socket: &SpearSocket,
    p: &SpearParameters,
    rows: &[Station],
    detail: Detail,
) -> Result<Vec<f64>, String> {
    let radial = detail
        .radial(socket.base_radius.get(), 24)
        .max((TAU / BLADE_CREASE_COSINE.acos()).ceil() as usize)
        .div_ceil(4)
        * 4;
    let mut result: Vec<f64> = (0..radial)
        .map(|i| TAU * i as f64 / radial as f64)
        .collect();
    if let Some(stops) = &socket.stops {
        let rotation = stops.orientation.get().to_radians();
        for row in rows {
            if let Some(width) = wing_width(p, socket, *row) {
                // Retain the square end and its circle intersections exactly.
                // Intermediate underside stations use the same angular grid;
                // adding each almost-tangent intersection would create slivers.
                if (width - stops.span.get() / 2.0).abs() > 1e-12 {
                    continue;
                }
                let radius = circular_radius(p, socket, row.y);
                let half = stops.thickness.get() / 2.0;
                for angle in [
                    half.atan2(width),
                    (half / radius).asin(),
                    (width / radius).min(1.0).acos(),
                ] {
                    for a in [angle, PI - angle, PI + angle, TAU - angle] {
                        result.push((a + rotation).rem_euclid(TAU));
                    }
                }
            }
        }
    }
    result.sort_by(f64::total_cmp);
    result.dedup();
    result = refine_angles(p, socket, rows, detail, result)?;
    // Near-coincident curvature samples become collinear after float32 export:
    // the circular displacement is quadratic in angle. Coalesce below that
    // representable angular scale, retaining the ordinary grid's exact axes.
    let minimum_angle = 4.0 * f64::from(f32::EPSILON).sqrt();
    let mut compact = vec![result[0]];
    for angle in result.into_iter().skip(1) {
        if angle - compact.last().unwrap() >= minimum_angle {
            compact.push(angle);
        } else if (angle / FRAC_PI_2 - (angle / FRAC_PI_2).round()).abs() < 1e-12 {
            *compact.last_mut().unwrap() = angle;
        }
    }
    if TAU - compact.last().unwrap() + compact[0] < minimum_angle {
        compact.pop();
    }
    Ok(compact)
}

/// Refine the actual section only where its chord misses the curved surface.
/// A smaller fillet occupies a smaller angular neighborhood; it must not demand
/// a denser grid around the whole socket. Sub-resolution features converge to
/// the sharp union within this detail level's dimensional error budget.
fn refine_angles(
    p: &SpearParameters,
    socket: &SpearSocket,
    rows: &[Station],
    detail: Detail,
    angles: Vec<f64>,
) -> Result<Vec<f64>, String> {
    let error = detail.error(socket.base_radius.get() / 1000.0);
    let minimum_angle = 4.0 * f64::from(f32::EPSILON).sqrt();
    let mut pending: Vec<_> = angles
        .iter()
        .copied()
        .zip(angles.iter().copied().skip(1).chain([TAU]))
        .collect();
    pending.reverse();
    let mut result = Vec::new();
    while let Some((start, end)) = pending.pop() {
        let refine = end - start > 2.0 * minimum_angle
            && rows.iter().any(|&row| {
                if wing_width(p, socket, row).is_none() {
                    return false;
                }
                let a = outer(p, socket, row, start);
                let b = outer(p, socket, row, end);
                let edge = sub(b, a);
                [0.25, 0.5, 0.75].into_iter().any(|t| {
                    let point = outer(p, socket, row, start + (end - start) * t);
                    let progress = (dot(sub(point, a), edge) / dot(edge, edge)).clamp(0.0, 1.0);
                    magnitude(sub(point, add(a, mul(edge, progress)))) > error
                })
            });
        if refine {
            let middle = (start + end) / 2.0;
            pending.extend([(middle, end), (start, middle)]);
        } else {
            result.push(start);
        }
        construction_budget((rows.len() + 4) as f64 * (pending.len() + result.len()) as f64 * 2.0)?;
    }
    Ok(result)
}

fn outer(p: &SpearParameters, socket: &SpearSocket, row: Station, angle: f64) -> Point {
    let cosine = angle.cos();
    let sine = angle.sin();
    let radius = if row.y >= 0.0 {
        let progress = row.y / p.length.get();
        let half = spears::half_width(p, progress);
        let depth = p.thickness.get() / 2.0 * (1.0 - 0.9 * progress);
        1.0 / (cosine.abs() / half + sine.abs() / depth)
    } else {
        let circle = circular_radius(p, socket, row.y);
        if row.y > -socket.neck_length.get() {
            let t = 1.0 + row.y / socket.neck_length.get();
            let half = p.root_width.map_or(p.width.get() * 0.4, Metres::get) / 2.0;
            // Convex superellipses interpolate the round socket to the diamond
            // ridge. Interpolating their radial distances instead introduces
            // concave flutes between the axes even with matched end tangents.
            let depth = p.thickness.get() / 2.0;
            let t2 = t * t;
            let t3 = t2 * t;
            let width_axis = socket.neck_axis(half, half, 0.0, t);
            let depth_axis = socket.neck_axis(half, depth, -depth * 0.9 / p.length.get(), t);
            let exponent = 2.0 - (3.0 * t2 - 2.0 * t3);
            ((cosine.abs() / width_axis).powf(exponent) + (sine.abs() / depth_axis).powf(exponent))
                .powf(-1.0 / exponent)
        } else {
            circle
        }
    };
    let radius = if let (Some(stops), Some(width)) = (&socket.stops, wing_width(p, socket, row)) {
        let relative = angle - stops.orientation.get().to_radians();
        let rectangle =
            (width / relative.cos().abs()).min(stops.thickness.get() / 2.0 / relative.sin().abs());
        let height = row.y + socket.length.get();
        let bottom = stops.center_height.get() - stops.root_height.get();
        let top = stops.center_height.get() + stops.end_height.get() / 2.0;
        let blend = stops.root_blend.map_or(0.0, Metres::get);
        let blend = if blend > 0.0 {
            let ramp = |v: f64| {
                let t = (v / blend).clamp(0.0, 1.0);
                t * t * (3.0 - 2.0 * t)
            };
            blend * ramp(height - bottom) * ramp(top - height)
        } else {
            0.0
        };
        let maximum = if rectangle - radius > 1e-12 {
            rectangle
        } else {
            radius
        };
        if blend > 0.0 {
            maximum + (blend - (rectangle - radius).abs()).max(0.0).powi(2) / (4.0 * blend)
        } else {
            maximum
        }
    } else {
        radius
    };
    [radius * cosine, row.y, radius * sine]
}

fn connect(
    solid: &mut Solid,
    a: &[Point],
    b: &[Point],
    reverse: bool,
    surface: impl Fn(usize) -> u32,
) {
    for side in 0..a.len() {
        let next = (side + 1) % a.len();
        let faces = [[a[side], b[next], a[next]], [a[side], b[side], b[next]]];
        for [first, mut second, mut third] in faces {
            if reverse {
                std::mem::swap(&mut second, &mut third);
            }
            triangle(solid, first, second, third, surface(side));
        }
    }
}

fn triangle(solid: &mut Solid, a: Point, b: Point, c: Point, surface: u32) {
    // A wing closing onto the socket has coincident boundary vertices. Collapse
    // those edges, retaining the nondegenerate triangle of the adjacent quad.
    if magnitude(cross(sub(b, a), sub(c, a))) > 1e-18 {
        solid.triangle(a, b, c, surface);
    }
}
