//! Stock, prod, string, bridle and nut constructions.
use super::*;
pub(super) fn stock(
    r: &ResolvedComponent,
    p: &CrossbowParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let StockLayout {
        rear,
        fore,
        cavity_start,
        cavity_end,
        gap,
        cheek,
    } = StockLayout::new(p);
    let table = p.lock_table_height.get();
    let wood = r.component.material.unwrap_or(Material::Wood);
    let mut parts = vec![
        part(
            Solid::stock(&rear)?,
            wood,
            "profiled rear crossbow tiller stock",
            r,
        ),
        part(
            Solid::stock(&fore)?,
            wood,
            "profiled fore-end crossbow tiller stock",
            r,
        ),
    ];
    for (side, label) in [
        (-1.0, "left open nut-cavity stock cheek"),
        (1.0, "right open nut-cavity stock cheek"),
    ] {
        parts.push(part(
            Solid::cuboid([cheek, cavity_end - cavity_start, table], detail)?.transform(
                [0.0; 3],
                [
                    side * (gap + cheek) / 2.0,
                    (cavity_start + cavity_end) / 2.0,
                    -table / 2.0,
                ],
            ),
            wood,
            label,
            r,
        ));
    }

    Ok(parts)
}
pub(super) fn limbs(
    r: &ResolvedComponent,
    p: &CrossbowParameters,
    detail: Detail,
    parts: &mut Vec<PartSource>,
) -> Result<(), String> {
    let points = prod(p, detail);
    let member = |points: &[Point], depth| {
        Solid::sweep(
            points,
            &Sweep {
                section: Section::Flat,
                width: depth,
                depth: p.prod_thickness.get(),
                centered_taper: true,
                tip_scale: p.prod_tip_scale.get(),
                ..Sweep::default()
            },
            detail,
        )
    };
    if p.prod_construction == CrossbowProdConstruction::Steel {
        parts.push(part(
            member(&points, p.prod_depth.get())?,
            Material::DarkSteel,
            "steel crossbow prod",
            r,
        ));
    } else {
        let horn = p
            .horn_thickness
            .ok_or("composite prod needs horn thickness")?
            .get();
        let sinew = p
            .sinew_thickness
            .ok_or("composite prod needs sinew thickness")?
            .get();
        let depth = p.prod_depth.get();
        let core = depth - horn - sinew;
        if core <= 0.0 {
            return Err("crossbow core must retain positive depth".into());
        }
        for (width, offset, material, label) in [
            (
                core,
                (horn - sinew) / 2.0,
                p.core_material.unwrap_or(Material::Wood),
                "composite prod wood core",
            ),
            (
                horn,
                -depth / 2.0 + horn / 2.0,
                p.horn_material.unwrap_or(Material::Horn),
                "composite prod horn belly",
            ),
            (
                sinew,
                depth / 2.0 - sinew / 2.0,
                p.back_material.unwrap_or(Material::Sinew),
                "composite prod sinew back",
            ),
        ] {
            let shifted: Vec<_> = points.iter().map(|&p| add(p, [0.0, offset, 0.0])).collect();
            parts.push(part(member(&shifted, width)?, material, label, r));
        }
    }

    Ok(())
}
pub(super) fn strings(
    r: &ResolvedComponent,
    p: &CrossbowParameters,
    detail: Detail,
    parts: &mut Vec<PartSource>,
) -> Result<(), String> {
    let nut = p.nut_position.get();
    let points = prod(p, detail);
    let (left, left_loop) = tip_loop(p, &points, false, detail);
    let (right, right_loop) = tip_loop(p, &points, true, detail);
    let string_start = parts.len();
    let string = p.string_material.unwrap_or(Material::Cord);
    let serving = p.serving_width.get() / 2.0;
    let radius = p.string_radius.get();
    let radial = p.radial_segments.map_or(8, |n| n.0 as usize);
    for (start, end, scale, label) in [
        (
            [left[0], left[1]],
            [-serving, nut],
            1.0,
            "left crossbow string control span",
        ),
        (
            [right[0], right[1]],
            [serving, nut],
            1.0,
            "right crossbow string control span",
        ),
        (
            [-serving, nut],
            [serving, nut],
            1.3,
            "served crossbow nocking span",
        ),
    ] {
        parts.push(part(
            guards::tube(&[start, end], radius * scale, radial, detail)?,
            string,
            label,
            r,
        ));
    }
    for (points, label) in [
        (&left_loop, "left crossbow string end loop"),
        (&right_loop, "right crossbow string end loop"),
    ] {
        parts.push(part(
            Solid::sweep(
                points,
                &Sweep {
                    width: radius * 2.0,
                    depth: radius * 2.0,
                    radial_segments: radial,
                    ..Sweep::default()
                },
                detail,
            )?,
            string,
            label,
            r,
        ));
    }
    for part in &mut parts[string_start..] {
        part.animation_channel = Some(output::AnimationChannel::CrossbowString);
    }

    Ok(())
}
pub(super) fn bridles(
    r: &ResolvedComponent,
    p: &CrossbowParameters,
    detail: Detail,
    parts: &mut Vec<PartSource>,
) -> Result<(), String> {
    let prod_position = p.prod_position.get();
    for (side, name) in [(-1.0, "left"), (1.0, "right")] {
        let x = side * p.bridle_spacing.get();
        let distance = x.abs() / (p.prod_span.get() / 2.0);
        let y = prod_position + p.prod_sweep.get() * distance.powf(1.7);
        let scale = 1.0 + (p.prod_tip_scale.get() - 1.0) * distance;
        let samples = detail.samples(18, 12);
        let points: Vec<_> = (0..=samples)
            .map(|i| {
                let a = i as f64 / samples as f64 * TAU;
                [
                    x,
                    y + a.cos() * (p.prod_depth.get() * scale / 2.0),
                    a.sin() * (p.prod_thickness.get() * scale / 2.0),
                ]
            })
            .collect();
        let br = p.bridle_radius.get();
        parts.push(part(
            Solid::sweep(
                &points,
                &Sweep {
                    width: br * 2.0,
                    depth: br * 2.0,
                    fit_bends: true,
                    ..Sweep::default()
                },
                detail,
            )?,
            p.binding_material.unwrap_or(Material::Cord),
            &format!("{name} prod bridle binding"),
            r,
        ));
    }

    Ok(())
}
pub(super) fn nut(
    r: &ResolvedComponent,
    p: &CrossbowParameters,
    detail: Detail,
    parts: &mut Vec<PartSource>,
) -> Result<(), String> {
    let nut = p.nut_position.get();
    let nr = p.nut_radius.get();
    let radius = p.string_radius.get();
    let notch = (radius * 3.0).max(0.006);
    let cheek = (p.nut_width.get() - notch) / 2.0;
    for (side, name) in [(-1.0, "left"), (1.0, "right")] {
        parts.push(part(
            Solid::lathe(
                &[[-cheek / 2.0, nr], [cheek / 2.0, nr]],
                14,
                1.0,
                false,
                detail,
            )?
            .transform([0.0, 0.0, 90.0], [side * (notch + cheek) / 2.0, nut, 0.0]),
            Material::Horn,
            &format!("{name} rotating nut cheek"),
            r,
        ));
    }
    parts.push(part(
        Solid::lathe(
            &[
                [-p.nut_width.get() * 0.62, 0.003],
                [p.nut_width.get() * 0.62, 0.003],
            ],
            10,
            1.0,
            false,
            detail,
        )?
        .transform([0.0, 0.0, 90.0], [0.0, nut, 0.0]),
        Material::Steel,
        "nut axle and bearing",
        r,
    ));

    Ok(())
}
