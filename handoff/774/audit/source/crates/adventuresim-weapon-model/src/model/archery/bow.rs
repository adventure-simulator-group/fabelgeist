//! Limb laminations, tip overlays and independent working-string parts.
use super::*;
pub(super) fn bow_limbs(
    r: &ResolvedComponent,
    p: &ArcheryBowParameters,
    upper: &[Point],
    lower: &[Point],
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let composite = p.construction == ArcheryBowConstruction::Composite;
    let section = match p.limb_section.as_ref() {
        Some(ArcheryBowLimbSection::DShape) => Section::DShape,
        Some(ArcheryBowLimbSection::Oval) => Section::Oval,
        Some(ArcheryBowLimbSection::Flat) => Section::Flat,
        None => {
            if composite {
                Section::Flat
            } else {
                Section::DShape
            }
        }
    };
    let member = |points: &[Point], width| {
        Solid::sweep(
            points,
            &Sweep {
                section,
                width,
                depth: p.limb_width.get(),
                tip_scale: p.tip_scale.get(),
                ..Sweep::default()
            },
            detail,
        )
    };
    let mut parts = Vec::new();
    if composite {
        let layers = composite_layers(p)?;
        for (name, points) in [("upper", &upper), ("lower", &lower)] {
            for &(suffix, width, center, material) in &layers {
                let shifted: Vec<_> = points
                    .iter()
                    .enumerate()
                    .map(|(i, &point)| {
                        let progress = i as f64 / (points.len() - 1) as f64;
                        add(
                            point,
                            [
                                center * (1.0 + (p.tip_scale.get() - 1.0) * progress),
                                0.0,
                                0.0,
                            ],
                        )
                    })
                    .collect();
                parts.push(part(
                    member(&shifted, width)?,
                    material,
                    &format!("{name} {suffix}"),
                    r,
                ));
            }
        }
    } else {
        for (name, points) in [
            ("upper D-section bow limb", &upper),
            ("lower D-section bow limb", &lower),
        ] {
            parts.push(part(
                member(points, p.limb_depth.get())?,
                r.component.material.unwrap_or(Material::Wood),
                name,
                r,
            ));
        }
    }

    Ok(parts)
}
pub(super) fn bow_furniture(
    r: &ResolvedComponent,
    p: &ArcheryBowParameters,
    upper: &[Point],
    lower: &[Point],
    detail: Detail,
    parts: &mut Vec<PartSource>,
) -> Result<(), String> {
    parts.push(part(
        Solid::sweep(
            &[
                [0.0, -p.grip_length.get() / 2.0 - 0.004, 0.0],
                [0.0, p.grip_length.get() / 2.0 + 0.004, 0.0],
            ],
            &Sweep {
                section: Section::Oval,
                width: p.grip_depth.get(),
                depth: p.grip_width.get(),
                radial_segments: 12,
                ..Sweep::default()
            },
            detail,
        )?,
        Material::Leather,
        "bow grip",
        r,
    ));
    for (name, points) in [
        ("upper horn nock overlay", &upper),
        ("lower horn nock overlay", &lower),
    ] {
        let tip = *points.last().unwrap();
        let inset = lerp(points[points.len() - 2], tip, 0.55);
        parts.push(part(
            Solid::sweep(
                &[inset, tip],
                &Sweep {
                    section: Section::Flat,
                    width: p.limb_depth.get() * p.tip_scale.get() * 1.08,
                    depth: p.limb_width.get() * p.tip_scale.get() * 1.08,
                    ..Sweep::default()
                },
                detail,
            )?,
            p.nock_material.unwrap_or(Material::Horn),
            name,
            r,
        ));
    }

    Ok(())
}
pub(super) fn bow_strings(
    r: &ResolvedComponent,
    p: &ArcheryBowParameters,
    upper: &[Point],
    lower: &[Point],
    detail: Detail,
    parts: &mut Vec<PartSource>,
) -> Result<(), String> {
    let x = -p.brace_height.get();
    let half = p.loop_gap.get() / 2.0;
    let (upper_attachment, upper_loop) = tip_loop(p, upper, detail);
    let (lower_attachment, lower_loop) = tip_loop(p, lower, detail);
    let string_start = parts.len();
    let material = p.string_material.unwrap_or(Material::Cord);
    let radial = p.radial_segments.map_or(8, |n| n.0 as usize);
    let radius = p.string_radius.get();
    for (attachment, end, label) in [
        (upper_attachment, [x, half], "upper bowstring control span"),
        (lower_attachment, [x, -half], "lower bowstring control span"),
    ] {
        parts.push(part(
            guards::tube(
                &[[attachment[0], attachment[1]], end],
                radius,
                radial,
                detail,
            )?,
            material,
            label,
            r,
        ));
    }
    for (points, label) in [
        (&upper_loop, "upper bowstring end loop"),
        (&lower_loop, "lower bowstring end loop"),
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
            material,
            label,
            r,
        ));
    }
    parts.push(part(
        guards::tube(&[[x, -half], [x, half]], radius * 1.3, radial, detail)?,
        material,
        "served nocking control span",
        r,
    ));
    for part in &mut parts[string_start..] {
        part.animation_channel = Some(output::AnimationChannel::BowString);
    }

    Ok(())
}
