//! Quarrels and layered wooden bolt carriers.
use super::*;

fn part(solid: Solid, material: Material, label: &str, r: &ResolvedComponent) -> PartSource {
    PartSource::new(solid, material, label, &r.id)
}

pub(super) fn bolt(
    r: &ResolvedComponent,
    p: &CrossbowBoltParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let radius = p.shaft_radius.get();
    let length = p.length.get();
    let h = p.head_length.get();
    let width = p.head_width.get()
        * if p.head_style == CrossbowBoltHeadStyle::Bodkin {
            0.5
        } else {
            1.0
        };
    let outline = if p.head_style == CrossbowBoltHeadStyle::Hunting {
        vec![
            [-width / 2.0, 0.0],
            [-width * 0.34, h * 0.30],
            [-radius, h * 0.22],
            [0.0, h],
            [radius, h * 0.22],
            [width * 0.34, h * 0.30],
            [width / 2.0, 0.0],
        ]
    } else {
        vec![
            [-width / 2.0, 0.0],
            [-radius, h * 0.2],
            [0.0, h],
            [radius, h * 0.2],
            [width / 2.0, 0.0],
        ]
    };
    let head_label = match p.head_style {
        CrossbowBoltHeadStyle::Bodkin => "bodkin bolt head",
        CrossbowBoltHeadStyle::Broadhead => "broadhead bolt head",
        CrossbowBoltHeadStyle::Hunting => "hunting bolt head",
    };
    let mut parts = vec![
        part(
            Solid::lathe(
                &[[0.0, radius], [length, radius * 0.95]],
                p.segments.map_or(12, |n| n.0 as usize),
                1.0,
                true,
                detail,
            )?,
            r.component.material.unwrap_or(Material::Wood),
            "bolt shaft",
            r,
        ),
        part(
            Solid::prism(&outline, p.head_thickness.get(), detail)?
                .transform([0.0; 3], [0.0, length, 0.0]),
            p.head_material.unwrap_or(Material::Steel),
            head_label,
            r,
        ),
        part(
            Solid::cuboid(
                [p.butt_width.get(), p.butt_length.get(), p.butt_height.get()],
                detail,
            )?
            .transform([0.0; 3], [0.0, p.butt_length.get() / 2.0, 0.0]),
            p.butt_material.unwrap_or(Material::Horn),
            "flattened reinforced quarrel butt",
            r,
        ),
        part(
            Solid::cuboid([p.butt_width.get(), 0.0025, p.butt_height.get()], detail)?,
            p.butt_material.unwrap_or(Material::Horn),
            "flat bolt butt bearing face",
            r,
        ),
    ];
    parts.extend(vanes(r, p, detail)?);
    Ok(parts)
}

pub(super) fn quiver(
    r: &ResolvedComponent,
    p: &BoltQuiverParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let bottom = p.bottom_width.get() / 2.0;
    let mouth = p.mouth_width.get() / 2.0;
    let depth = p.depth.get() / 2.0;
    let length = p.length.get();
    let wall = p.wall.get();
    let outline = [
        [-bottom, 0.0],
        [-mouth, length],
        [mouth, length],
        [bottom, 0.0],
    ];
    let wood = r.component.material.unwrap_or(Material::Wood);
    let mut parts = vec![
        part(
            Solid::prism(&outline, wall, detail)?
                .transform([0.0; 3], [0.0, 0.0, depth - wall / 2.0]),
            wood,
            "bolt quiver wood front shell",
            r,
        ),
        part(
            Solid::prism(&outline, wall, detail)?
                .transform([0.0; 3], [0.0, 0.0, -depth + wall / 2.0]),
            wood,
            "bolt quiver wood back shell",
            r,
        ),
        part(
            Solid::prism(
                &[
                    [-bottom, 0.0],
                    [-mouth, length],
                    [-mouth + wall, length],
                    [-bottom + wall, 0.0],
                ],
                p.depth.get(),
                detail,
            )?,
            wood,
            "bolt quiver left wood side",
            r,
        ),
        part(
            Solid::prism(
                &[
                    [bottom - wall, 0.0],
                    [mouth - wall, length],
                    [mouth, length],
                    [bottom, 0.0],
                ],
                p.depth.get(),
                detail,
            )?,
            wood,
            "bolt quiver right wood side",
            r,
        ),
        part(
            Solid::cuboid([p.bottom_width.get(), wall, p.depth.get()], detail)?
                .transform([0.0; 3], [0.0, wall / 2.0, 0.0]),
            wood,
            "sealed broad bolt quiver bottom",
            r,
        ),
        part(
            Solid::prism(&outline, p.lining.get(), detail)?
                .transform([0.0; 3], [0.0, 0.0, depth - wall - p.lining.get() / 2.0]),
            Material::Feather,
            "paper lining layer",
            r,
        ),
        part(
            Solid::prism(&outline, p.hide_cover.get(), detail)?
                .transform([0.0; 3], [0.0, 0.0, depth + p.hide_cover.get() / 2.0]),
            Material::Leather,
            "hide outer cover layer",
            r,
        ),
    ];
    parts.extend(carrier_fittings(r, p, detail)?);
    Ok(parts)
}

fn vanes(
    r: &ResolvedComponent,
    p: &CrossbowBoltParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let mut parts = Vec::new();
    let radius = p.shaft_radius.get();
    let length = p.length.get();
    let hunting = p.bolt_use == CrossbowBoltBoltUse::Hunting;
    let count = if hunting { 3 } else { 2 };
    let vane_material = if hunting {
        Material::Feather
    } else {
        p.fletching_material.unwrap_or(Material::Leather)
    };
    let height = p.fletching_height.get();
    let fin_length = p.fletching_length.get();
    for index in 0..count {
        let solid = Solid::prism(
            &[
                [0.0, 0.0],
                [height, fin_length * 0.18],
                [height * 0.72, fin_length],
                [0.0, fin_length],
            ],
            radius * if hunting { 0.28 } else { 0.6 },
            detail,
        )?
        .transform(
            [0.0; 3],
            [
                radius * (1.0 - 0.05 * (p.butt_length.get() + 0.018 + fin_length) / length)
                    - radius * 0.03,
                p.butt_length.get() + 0.018,
                0.0,
            ],
        )
        .transform(
            [
                0.0,
                index as f64 * 360.0 / count as f64 + if hunting { 0.0 } else { 45.0 },
                0.0,
            ],
            [0.0; 3],
        );
        parts.push(part(
            solid,
            vane_material,
            &format!(
                "{} bolt vane {}",
                if hunting { "hunting" } else { "war" },
                index + 1
            ),
            r,
        ));
    }

    Ok(parts)
}

fn carrier_fittings(
    r: &ResolvedComponent,
    p: &BoltQuiverParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let mut parts = Vec::new();
    let bottom = p.bottom_width.get() / 2.0;
    let mouth = p.mouth_width.get() / 2.0;
    let length = p.length.get();
    let depth = p.depth.get() / 2.0;
    parts.push(part(
        Solid::sweep(
            &[
                [-mouth, length, -depth],
                [mouth, length, -depth],
                [mouth, length, depth],
                [-mouth, length, depth],
                [-mouth, length, -depth],
            ],
            &Sweep {
                width: 0.006,
                depth: 0.006,
                ..Sweep::default()
            },
            detail,
        )?,
        p.rim_material.unwrap_or(Material::Leather),
        "open bolt quiver leather mouth binding",
        r,
    ));
    let lower_y = length * 0.18;
    let upper_y = length * 0.78;
    let lower_x = bottom + (mouth - bottom) * 0.18 + p.strap_thickness.get() * 0.25;
    let upper_x = bottom + (mouth - bottom) * 0.78 + p.strap_thickness.get() * 0.25;
    parts.push(part(
        Solid::sweep(
            &[
                [lower_x, lower_y, 0.0],
                [lower_x.max(upper_x) + p.strap_drop.get(), length * 0.5, 0.0],
                [upper_x, upper_y, 0.0],
            ],
            &Sweep {
                section: Section::Flat,
                width: p.strap_thickness.get(),
                depth: p.strap_width.get(),
                ..Sweep::default()
            },
            detail,
        )?,
        p.strap_material.unwrap_or(Material::DarkLeather),
        "attached bolt quiver shoulder strap",
        r,
    ));

    Ok(parts)
}
