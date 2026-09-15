//! Turned, forged, fluted and ornamented pommel constructions.
use super::*;
use std::f64::consts::{PI, TAU};

fn part(solid: Solid, r: &ResolvedComponent, suffix: &str, material: Material) -> PartSource {
    PartSource::new(
        solid,
        material,
        &format!(
            "{}{}{}",
            r.label,
            if suffix.is_empty() { "" } else { " " },
            suffix
        ),
        &r.id,
    )
}

pub(super) fn pommel(
    r: &ResolvedComponent,
    p: &PommelParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let material = r.component.material.unwrap_or(Material::Steel);
    match p.construction {
        PommelConstruction::Composite => composite(r, p, detail),
        PommelConstruction::Lathed => {
            let profile = p
                .profile
                .as_ref()
                .ok_or("lathed pommel needs profile")?
                .iter()
                .map(|p| p.map(Metres::get))
                .collect::<Vec<_>>();
            Ok(vec![part(
                Solid::lathe(
                    &profile,
                    p.segments.map_or(14, |n| n.0 as usize),
                    1.0,
                    false,
                    detail,
                )?,
                r,
                "",
                material,
            )])
        }
        PommelConstruction::Plate => plate(r, p, detail),
        PommelConstruction::Outline => outline(r, p, detail),
        PommelConstruction::Faceted | PommelConstruction::Writhen => {
            Ok(vec![part(writhen(p, detail), r, "", material)])
        }
    }
}

fn plate(
    r: &ResolvedComponent,
    p: &PommelParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let radius = p.diameter.map_or(0.06, Metres::get) * p.width_scale.map_or(1.0, Ratio::get) / 2.0;
    let height =
        p.height.or(p.diameter).map_or(0.06, Metres::get) * p.length_scale.map_or(1.0, Ratio::get);
    let thickness = p.thickness.map_or(0.018, Metres::get);
    let bevel = p.rim_bevel.map_or(0.15, Ratio::get);
    let count = detail.samples(24, 10);
    let material = r.component.material.unwrap_or(Material::Steel);
    let outline: Vec<_> = (0..count)
        .map(|i| {
            let a = i as f64 / count as f64 * TAU;
            [a.cos() * radius, height * 0.425 + a.sin() * height * 0.425]
        })
        .collect();
    let mut parts = vec![
        part(
            Solid::rounded_plate(&outline, thickness, bevel)?,
            r,
            "",
            material,
        ),
        part(
            Solid::lathe(
                &[[height * 0.66, 0.009], [height, 0.01]],
                14,
                thickness / 0.02,
                false,
                detail,
            )?,
            r,
            "tang seat",
            material,
        ),
    ];
    let bulge = thickness * p.face_convexity.map_or(0.15, Ratio::get);
    let face_radius = radius * (1.0 - bevel);
    if bulge > 0.0 {
        for side in [-1.0, 1.0] {
            parts.push(part(
                Solid::lathe(
                    &[
                        [0.0, face_radius],
                        [bulge * 0.65, face_radius * 0.72],
                        [bulge, 0.001],
                    ],
                    18,
                    height * 0.425 / radius,
                    false,
                    detail,
                )?
                .transform(
                    [90.0 * side, 0.0, 0.0],
                    [0.0, height * 0.425, side * thickness * 0.49],
                ),
                r,
                "convex face",
                material,
            ));
        }
    }
    Ok(parts)
}

fn outline(
    r: &ResolvedComponent,
    p: &PommelParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let width = p.diameter.map_or(0.055, Metres::get) * p.width_scale.map_or(1.0, Ratio::get);
    let height = p.height.map_or(0.06, Metres::get) * p.length_scale.map_or(1.0, Ratio::get);
    let thickness = p.thickness.map_or(0.018, Metres::get);
    let material = r.component.material.unwrap_or(Material::Steel);
    if p.outline_style == Some(PommelOutlineStyle::Fan) {
        return Ok(vec![part(fan(width, height, thickness)?, r, "", material)]);
    }
    let points = if p
        .outline_style
        .as_ref()
        .is_none_or(|style| *style == PommelOutlineStyle::Fishtail)
    {
        let notch = p.notch_depth.map_or(0.22, Ratio::get);
        let spread = p.lobe_spread.map_or(0.9, Ratio::get);
        let shoulder = p.shoulder_width.map_or(0.42, Ratio::get);
        let mut points = vec![
            [-width * shoulder / 2.0, 0.0],
            [-width / 2.0, height * 0.72],
            [-width * spread / 2.0, height],
            [0.0, height * (1.0 - notch)],
            [width * spread / 2.0, height],
            [width / 2.0, height * 0.72],
            [width * shoulder / 2.0, 0.0],
        ];
        for point in &mut points {
            point[1] = height - point[1];
        }
        points.reverse();
        points
    } else {
        vec![
            [-width * 0.22, 0.0],
            [-width / 2.0, height * 0.45],
            [-width * 0.4, height],
            [width * 0.4, height],
            [width / 2.0, height * 0.45],
            [width * 0.22, 0.0],
        ]
    };
    Ok(vec![
        part(
            Solid::rounded_plate(&points, thickness, 0.14)?,
            r,
            "",
            material,
        ),
        part(
            Solid::lathe(
                &[[height * 0.78, 0.008], [height, 0.01]],
                14,
                thickness / 0.02,
                false,
                detail,
            )?,
            r,
            "tang seat",
            material,
        ),
    ])
}

fn fan(width: f64, height: f64, thickness: f64) -> Result<Solid, String> {
    let mut points = vec![[-width * 0.18, 0.0]];
    for i in 0..12 {
        let t = i as f64 / 12.0;
        points.push([
            -width * (0.18 + 0.32 * (t * PI / 2.0).sin()),
            height * (0.04 + 0.36 * t),
        ]);
    }
    for i in 0..=24 {
        let angle = PI - i as f64 / 24.0 * PI;
        points.push([
            angle.cos() * width / 2.0,
            height * 0.4 + angle.sin() * height * 0.6,
        ]);
    }
    for i in (0..12).rev() {
        let t = i as f64 / 12.0;
        points.push([
            width * (0.18 + 0.32 * (t * PI / 2.0).sin()),
            height * (0.04 + 0.36 * t),
        ]);
    }
    points.push([width * 0.18, 0.0]);
    for p in &mut points {
        p[1] = height - p[1];
    }
    points.reverse();
    Solid::rounded_plate(&points, thickness, 0.14)
}

fn writhen(p: &PommelParameters, detail: Detail) -> Solid {
    let faceted = p.construction == PommelConstruction::Faceted;
    let flutes = p.flute_count.map_or(8, |n| n.0 as usize);
    let h = p.height.map_or(0.06, Metres::get) * p.length_scale.map_or(1.0, Ratio::get);
    let radius = p.diameter.map_or(0.06, Metres::get) * p.width_scale.map_or(1.0, Ratio::get) / 2.0;
    let depth = if faceted {
        0.0
    } else {
        p.flute_depth.map_or(0.12, Ratio::get)
    };
    let twist = p.twist.map_or(75.0, Degrees::get).to_radians();
    let axial = detail
        .samples(10, 12)
        .max((h.hypot(2.0 * radius).hypot(radius * twist) / detail.error(0.015)).ceil() as usize);
    let profile = if faceted {
        vec![
            [0.0, radius * 0.28],
            [h * 0.12, radius * 0.72],
            [h * 0.32, radius],
            [h * 0.68, radius * 0.94],
            [h * 0.9, radius * 0.56],
            [h, radius * 0.3],
        ]
    } else {
        (0..=axial)
            .map(|i| {
                let t = i as f64 / axial as f64;
                [
                    h * t,
                    radius * (0.28 + 0.06 * t + 0.82 * (PI * t).sin() * (1.0 - 0.35 * t)),
                ]
            })
            .collect()
    };
    let flute_samples = 6.max(
        (PI / (2.0 * detail.error(0.0006) / (radius * depth).max(0.0001)).sqrt()).ceil() as usize,
    );
    let segments = if faceted {
        p.facets.map_or(8, |n| n.0 as usize)
    } else {
        detail
            .radial(radius, p.segments.map_or(18, |n| n.0 as usize))
            .max(flutes * flute_samples)
    };
    let vertex = |row: usize, segment: usize| {
        let [y, base] = profile[row];
        let base_angle = segment as f64 / segments as f64 * TAU;
        let t = (y - profile[0][0]) / (profile.last().unwrap()[0] - profile[0][0]).max(1e-6);
        let angle = base_angle - if faceted { 0.0 } else { twist * t };
        let radius = base * (1.0 + depth * (base_angle * flutes as f64).cos() * (PI * t).sin());
        [angle.cos() * radius, y, angle.sin() * radius]
    };
    let mut solid = Solid::default();
    for row in 0..profile.len() - 1 {
        for segment in 0..segments {
            let next = (segment + 1) % segments;
            let group = if faceted { segment as u32 + 1 } else { 1 };
            solid.triangle(
                vertex(row, segment),
                vertex(row + 1, next),
                vertex(row, next),
                group,
            );
            solid.triangle(
                vertex(row, segment),
                vertex(row + 1, segment),
                vertex(row + 1, next),
                group,
            );
        }
    }
    for (row, reverse) in [(0, true), (profile.len() - 1, false)] {
        for segment in 0..segments {
            let center = [0.0, profile[row][0], 0.0];
            let a = vertex(row, segment);
            let b = vertex(row, (segment + 1) % segments);
            if reverse {
                solid.triangle(center, a, b, 0);
            } else {
                solid.triangle(center, b, a, 0);
            }
        }
    }
    solid.positive()
}

fn composite(
    r: &ResolvedComponent,
    p: &PommelParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let mut base = p.clone();
    let construction = p
        .base_construction
        .as_ref()
        .ok_or("composite pommel needs baseConstruction")?;
    base.construction = match construction {
        PommelBaseConstruction::Lathed => PommelConstruction::Lathed,
        PommelBaseConstruction::Plate => PommelConstruction::Plate,
        PommelBaseConstruction::Faceted => PommelConstruction::Faceted,
        PommelBaseConstruction::Writhen => PommelConstruction::Writhen,
        PommelBaseConstruction::Outline => PommelConstruction::Outline,
    };
    let mut parts = pommel(r, &base, detail)?;
    let sockets = p.sockets.as_ref().ok_or("composite pommel needs sockets")?;
    for ornament in p
        .ornaments
        .as_ref()
        .ok_or("composite pommel needs ornaments")?
    {
        let mut socket = sockets
            .get(&ornament.socket)
            .ok_or("missing ornament socket")?
            .map(Metres::get);
        seat_ornament(&parts, &mut socket);
        let material = ornament
            .material
            .or(r.component.material)
            .unwrap_or(Material::Steel);
        let rotation = ornament.rotation.map_or([0.0; 3], |p| p.map(Degrees::get));
        let additions = ornament_solids(ornament, detail)?;
        for (solid, suffix) in additions {
            parts.push(part(solid.transform(rotation, socket), r, suffix, material));
        }
    }
    Ok(parts)
}

fn seat_ornament(parts: &[PartSource], socket: &mut Point) {
    let sign = socket[2].signum();
    if sign != 0.0 {
        for part in parts {
            for &[a, b, c] in &part.solid.faces {
                let [a, b, c] = [
                    part.solid.positions[a],
                    part.solid.positions[b],
                    part.solid.positions[c],
                ];
                let denominator = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
                if denominator.abs() < 1e-12 {
                    continue;
                }
                let u = ((b[1] - c[1]) * (socket[0] - c[0]) + (c[0] - b[0]) * (socket[1] - c[1]))
                    / denominator;
                let v = ((c[1] - a[1]) * (socket[0] - c[0]) + (a[0] - c[0]) * (socket[1] - c[1]))
                    / denominator;
                if u < 0.0 || v < 0.0 || u + v > 1.0 {
                    continue;
                }
                let z = u * a[2] + v * b[2] + (1.0 - u - v) * c[2];
                socket[2] = if sign > 0.0 {
                    socket[2].max(z)
                } else {
                    socket[2].min(z)
                };
            }
        }
    }
}

fn ornament_solids(
    ornament: &Ornament,
    detail: Detail,
) -> Result<Vec<(Solid, &'static str)>, String> {
    let scale = ornament.scale.get();
    let mut additions = Vec::new();
    match &ornament.geometry {
        OrnamentGeometry::Crown {} => {
            additions.push((
                Solid::hollow_socket(
                    &[[0.0, scale * 0.58], [scale * 0.34, scale * 0.62]],
                    &[scale * 0.42; 2],
                    None,
                    detail,
                ),
                "crown band",
            ));
            for index in 0..5 {
                let angle = index as f64 / 5.0 * TAU;
                let point = [
                    angle.cos() * scale * 0.52,
                    scale * 0.25,
                    angle.sin() * scale * 0.52,
                ];
                additions.push((
                    Solid::prism(
                        &[
                            [-scale * 0.12, 0.0],
                            [scale * 0.12, 0.0],
                            [0.0, scale * 0.65],
                        ],
                        scale * 0.16,
                        detail,
                    )?
                    .transform([0.0, -angle.to_degrees(), 0.0], point),
                    "crown point",
                ));
            }
        }
        OrnamentGeometry::Escutcheon {} => additions.push((
            Solid::rounded_plate(
                &[
                    [-scale * 0.55, 0.0],
                    [-scale * 0.48, scale * 0.72],
                    [0.0, scale],
                    [scale * 0.48, scale * 0.72],
                    [scale * 0.55, 0.0],
                    [0.0, -scale * 0.36],
                ],
                scale * 0.13,
                0.14,
            )?,
            "escutcheon",
        )),
        OrnamentGeometry::Authored {
            positions,
            indices,
            smooth,
        } => {
            if positions.len() % 3 != 0 || indices.len() % 3 != 0 {
                return Err("ornament requires complete vertices and triangles".into());
            }
            let points: Vec<Point> = positions
                .as_chunks::<3>()
                .0
                .iter()
                .map(|p| [p[0].get() * scale, p[1].get() * scale, p[2].get() * scale])
                .collect();
            let mut solid = Solid::default();
            for face in indices.as_chunks::<3>().0 {
                let points = face
                    .iter()
                    .map(|&i| {
                        points
                            .get(i as usize)
                            .copied()
                            .ok_or("ornament vertex index out of range")
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                solid.triangle(
                    points[0],
                    points[1],
                    points[2],
                    if smooth == &Some(true) { 1 } else { 0 },
                );
            }
            additions.push((solid.positive(), "authored ornament"));
        }
    }

    Ok(additions)
}
