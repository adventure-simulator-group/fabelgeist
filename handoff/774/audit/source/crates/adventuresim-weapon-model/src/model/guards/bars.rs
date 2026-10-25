//! Continuous swept rings, bows and figure-eight guards.
use super::*;
pub(in crate::model) fn tube(
    points: &[PlanarPoint],
    radius: f64,
    segments: usize,
    detail: Detail,
) -> Result<Solid, String> {
    Solid::sweep(
        &points.iter().map(|&[x, y]| [x, y, 0.0]).collect::<Vec<_>>(),
        &Sweep {
            width: radius * 2.0,
            depth: radius * 2.0,
            radial_segments: segments,
            ..Sweep::default()
        },
        detail,
    )
}

pub(in crate::model) fn ring(p: &RingGuardParameters, detail: Detail) -> Result<Solid, String> {
    let radius = p.radius.get();
    let start = p.arc_start.map_or(0.0, Radians::get);
    let end = p.arc_end.map_or(TAU, Radians::get);
    let points = adaptive_curve(
        |t| {
            let a = start + t * (end - start);
            [a.cos() * radius, a.sin() * radius]
        },
        CurveQuality {
            minimum_segments: p.samples.map_or(24, |n| n.0 as usize),
            max_chord: 0.004_f64.max(radius * 0.24),
            max_deviation: 0.00025_f64.max(radius / 220.0),
        },
        detail,
    );
    tube(
        &points,
        p.bar.map_or(0.007, Metres::get),
        p.radial_segments.map_or(8, |n| n.0 as usize),
        detail,
    )
}

pub(in crate::model) fn knuckle(
    r: &ResolvedComponent,
    p: &KnuckleBowParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let width = p.width.get();
    let length = p.length.get();
    let bar = p.bar.map_or(0.012, Metres::get);
    let thickness = p.thickness.map_or(0.012, Metres::get);
    let side = p.side.map_or(1.0, Direction::sign);
    let bulge = p.bulge.map_or(0.035, Metres::get);
    let samples = p.samples.map_or(18, |n| n.0 as usize);
    let radius = bar.min(thickness) / 2.0;
    let points = adaptive_curve(
        |t| {
            let u = 1.0 - t;
            [
                2.0 * u * t * width * side + t * t * bar * side,
                2.0 * u * t * (length * 0.48 + bulge) + t * t * length,
            ]
        },
        CurveQuality {
            minimum_segments: samples,
            max_chord: width.max(length) / samples as f64,
            max_deviation: bar.min(thickness) / 10.0,
        },
        detail,
    );
    let mut parts = vec![part(
        tube(
            &points,
            radius,
            p.radial_segments.map_or(8, |n| n.0 as usize),
            detail,
        )?,
        r,
        "",
        r.component.material.unwrap_or(Material::Steel),
    )];
    for point in [points[0], *points.last().unwrap()] {
        parts.push(part(
            Solid::lathe(
                &[
                    [-radius, radius * 0.8],
                    [0.0, radius * 1.35],
                    [radius, radius * 0.8],
                ],
                10,
                1.0,
                false,
                detail,
            )?
            .transform([0.0; 3], [point[0], point[1], 0.0]),
            r,
            "anchor",
            Material::DarkSteel,
        ));
    }
    Ok(parts)
}

pub(in crate::model) fn figure_eight(
    p: &FigureEightParameters,
    detail: Detail,
) -> Result<Solid, String> {
    let half_width = p.width.get() / 2.0;
    let height = p.height.map_or(p.width.get() * 0.28, Metres::get);
    let bar = p.bar.map_or(0.009, Metres::get);
    if p.construction == Some(FigureEightConstruction::RoundTube) {
        return round_figure_eight(p, detail);
    }
    let bridge = bar.min(half_width * 0.2).min(height * 0.2);
    let cx = (half_width - bridge) / 2.0;
    let rx = (half_width + bridge) / 2.0;
    let ry = height / 2.0 + bridge;
    let inset = (bar * 2.0).min(rx * 0.45).min(ry * 0.6);
    let angle = (cx / rx).acos();
    let count = detail.samples(p.samples.map_or(48, |n| n.0 as usize), 20);
    let mut solid = Solid::default();
    for side in [-1.0, 1.0] {
        let mut outer = Vec::new();
        let mut inner = Vec::new();
        for i in 0..=count {
            let a = angle + (TAU - angle * 2.0) * i as f64 / count as f64;
            outer.push([
                if i == 0 || i == count {
                    0.0
                } else {
                    side * (cx - rx * a.cos())
                },
                ry * a.sin(),
            ]);
            inner.push([side * (cx - (rx - inset) * a.cos()), (ry - inset) * a.sin()]);
        }
        outer[count][1] = -outer[0][1];
        let point = |p: PlanarPoint, z: f64| [p[0], p[1], z];
        let mut emit = |a, b, c, group| {
            if side < 0.0 {
                solid.triangle(a, b, c, group)
            } else {
                solid.triangle(a, c, b, group)
            }
        };
        for i in 0..=count {
            let j = (i + 1) % (count + 1);
            for z in [-bar, bar] {
                let [a, b, c, d] = [
                    point(outer[i], z),
                    point(outer[j], z),
                    point(inner[j], z),
                    point(inner[i], z),
                ];
                if z > 0.0 {
                    emit(a, b, c, 1);
                    emit(a, c, d, 1);
                } else {
                    emit(a, c, b, 2);
                    emit(a, d, c, 2);
                }
            }
            if i < count {
                let [a, b, c, d] = [
                    point(outer[i], -bar),
                    point(outer[j], -bar),
                    point(outer[j], bar),
                    point(outer[i], bar),
                ];
                emit(a, b, c, 3);
                emit(a, c, d, 3);
            }
            let [a, b, c, d] = [
                point(inner[i], -bar),
                point(inner[j], -bar),
                point(inner[j], bar),
                point(inner[i], bar),
            ];
            emit(a, c, b, 4);
            emit(a, d, c, 4);
        }
    }
    Ok(solid.positive())
}

fn round_figure_eight(p: &FigureEightParameters, detail: Detail) -> Result<Solid, String> {
    let half_width = p.width.get() / 2.0;
    let height = p.height.map_or(p.width.get() * 0.28, Metres::get);
    let bar = p.bar.map_or(0.009, Metres::get);
    let count = detail.samples(p.samples.map_or(48, |n| n.0 as usize), 20);
    let points: Vec<_> = (0..=count)
        .map(|i| {
            let angle = i as f64 / count as f64 * TAU;
            [
                (half_width - bar) * angle.sin(),
                height * angle.sin() * angle.cos(),
                0.0,
            ]
        })
        .collect();
    Solid::sweep(
        &points,
        &Sweep {
            width: bar * 2.0,
            depth: bar * 2.0,
            radial_segments: p.radial_segments.map_or(14, |n| n.0 as usize),
            ..Sweep::default()
        },
        detail,
    )
}
