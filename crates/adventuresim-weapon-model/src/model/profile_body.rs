//! Profiled oval grips with disjoint core and cover material volumes.
use super::*;
use std::f64::consts::TAU;

pub(super) fn construct(
    r: &ResolvedComponent,
    p: &ProfileBodyParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let outer = rings(p, detail)?;
    let make = |solid, material, label: &str| PartSource::new(solid, material, label, &r.id);
    let core_material = r.component.material.unwrap_or(Material::Wood);
    let Some(cover) = &p.cover else {
        return Ok(vec![make(filled(&outer)?, core_material, &r.label)]);
    };
    let core_end = p.length.get() - cover.end_cap.map_or(0.0, Metres::get);
    let inner: Vec<_> = outer
        .iter()
        .take_while(|ring| ring[0][1] <= core_end)
        .map(|ring| inset(ring, cover.thickness.get()))
        .collect::<Result<_, _>>()?;
    Ok(vec![
        make(filled(&inner)?, core_material, &format!("{} core", r.label)),
        make(
            Solid::section_shell(
                &inner,
                &outer,
                if cover.end_cap.is_some() {
                    LoftEnd::Closed
                } else {
                    LoftEnd::Open
                },
            )?,
            cover.material,
            &format!("{} cover", r.label),
        ),
    ])
}

fn rings(p: &ProfileBodyParameters, detail: Detail) -> Result<Vec<Vec<Point>>, String> {
    let length = p.length.get();
    let width = SmoothProfile::new(
        p.profile
            .iter()
            .map(|s| [s.at.get(), s.width.get() / 2.0])
            .collect(),
    )?;
    let depth = SmoothProfile::new(
        p.profile
            .iter()
            .map(|s| [s.at.get(), s.depth.get() / 2.0])
            .collect(),
    )?;
    let quality = CurveQuality {
        minimum_segments: 2,
        max_chord: 0.015,
        max_deviation: 0.00005,
    };
    let mut stations = Vec::new();
    for pair in p.profile.windows(2) {
        for axis in [&width, &depth] {
            stations.extend(
                adaptive_curve(
                    |u| {
                        let t = pair[0].at.get() + u * (pair[1].at.get() - pair[0].at.get());
                        [length * t, axis.value(t)]
                    },
                    quality,
                    detail,
                )
                .into_iter()
                .map(|point| point[0] / length),
            );
        }
    }
    let core_end = p
        .cover
        .as_ref()
        .and_then(|c| c.end_cap)
        .map(|cap| length - cap.get());
    if let Some(end) = core_end {
        stations.push(end / length);
    }
    stations.sort_by(f64::total_cmp);
    stations.dedup_by(|a, b| (*a - *b).abs() < 1e-10);
    let radial = p
        .radial_segments
        .map_or_else(
            || detail.radial(p.maximum_width() / 2.0, 24),
            |count| detail.samples(count.0 as usize, MIN_PROFILE_RADIAL_SEGMENTS as usize),
        )
        .div_ceil(4)
        * 4;
    construction_budget((stations.len() * radial * 4) as f64)?;
    Ok(stations
        .into_iter()
        .map(|t| {
            (0..radial)
                .map(|i| {
                    let angle = TAU * i as f64 / radial as f64;
                    [
                        width.value(t) * angle.cos(),
                        if core_end.is_some_and(|end| t == end / length) {
                            core_end.unwrap()
                        } else {
                            t * length
                        },
                        depth.value(t) * angle.sin(),
                    ]
                })
                .collect()
        })
        .collect())
}

fn inset(ring: &[Point], distance: f64) -> Result<Vec<Point>, String> {
    let count = ring.len();
    let normal = |a: Point, b: Point| {
        let (x, z) = (b[0] - a[0], b[2] - a[2]);
        let length = x.hypot(z);
        [z / length, -x / length]
    };
    let inner: Vec<_> = (0..count)
        .map(|i| {
            let p = ring[i];
            let a = normal(ring[(i + count - 1) % count], p);
            let b = normal(p, ring[(i + 1) % count]);
            let denominator = 1.0 + a[0] * b[0] + a[1] * b[1];
            [
                p[0] - distance * (a[0] + b[0]) / denominator,
                p[1],
                p[2] - distance * (a[1] + b[1]) / denominator,
            ]
        })
        .collect();
    for i in 0..count {
        let [a, b, c] = [inner[i], inner[(i + 1) % count], inner[(i + 2) % count]];
        if (b[0] - a[0]) * (c[2] - b[2]) - (b[2] - a[2]) * (c[0] - b[0]) <= 0.0 {
            return Err("grip cover inset collapses its core section".into());
        }
    }
    Ok(inner)
}

fn filled(rings: &[Vec<Point>]) -> Result<Solid, String> {
    let mut solid = Solid::default();
    for pair in rings.windows(2) {
        for i in 0..pair[0].len() {
            let j = (i + 1) % pair[0].len();
            solid.quad(pair[0][i], pair[1][i], pair[1][j], pair[0][j], 1);
        }
    }
    for (ring, reverse) in [
        (rings.first().unwrap(), true),
        (rings.last().unwrap(), false),
    ] {
        let section: Vec<_> = ring.iter().map(|p| [p[0], p[2]]).collect();
        let cap = Region::triangulate(&section, false)?;
        for [a, b, c] in cap.triangles {
            let point = |i: usize| [cap.points[i][0], ring[0][1], cap.points[i][1]];
            if reverse {
                solid.triangle(point(a), point(b), point(c), 0);
            } else {
                solid.triangle(point(a), point(c), point(b), 0);
            }
        }
    }
    Ok(solid.positive())
}
