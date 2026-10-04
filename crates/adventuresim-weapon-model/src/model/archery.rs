//! Bow laminations, strung control spans, arrows, nocks and open quivers.
use super::*;
mod bow;
use bow::*;
use std::f64::consts::{PI, TAU};

fn part(solid: Solid, material: Material, label: &str, r: &ResolvedComponent) -> PartSource {
    PartSource::new(solid, material, label, &r.id)
}

pub(super) fn limb(p: &ArcheryBowParameters, upper: bool, detail: Detail) -> Vec<Point> {
    let half = p.length.get()
        * if upper {
            p.upper_ratio.get()
        } else {
            1.0 - p.upper_ratio.get()
        };
    let grip = p.grip_length.get() / 2.0;
    let working = half - grip;
    let samples = detail.samples(p.samples.map_or(18, |n| n.0 as usize), 10);
    (0..=samples)
        .map(|i| {
            let t = i as f64 / samples as f64;
            let y = grip + working * t;
            [
                p.reflex.get() * (PI * t).sin() + p.recurve.get() * t.powi(3),
                if upper { y } else { -y },
                0.0,
            ]
        })
        .collect()
}

pub(super) fn tip_loop(
    p: &ArcheryBowParameters,
    points: &[Point],
    detail: Detail,
) -> (Point, Vec<Point>) {
    let tip = *points.last().unwrap();
    let previous = points[points.len() - 2];
    let tangent = normalize(sub(tip, previous));
    let normal = normalize(cross([0.0, 0.0, 1.0], tangent));
    let binormal = normalize(cross(tangent, normal));
    let radial = p.limb_depth.get() * p.tip_scale.get() * 0.54 + p.loop_radius.get();
    let width = p.limb_width.get() * p.tip_scale.get() * 0.54 + p.loop_radius.get();
    let samples = detail.samples(24, 20);
    let toward = normalize([-p.brace_height.get() - tip[0], -tip[1], 0.0]);
    let sign = if dot(toward, normal) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let center = add(
        tip,
        mul(
            normal,
            (p.string_radius.get() * 0.7 - p.loop_radius.get()) * sign,
        ),
    );
    let attachment = add(center, mul(normal, radial * sign));
    let points = (0..=samples)
        .map(|i| {
            let angle = i as f64 / samples as f64 * TAU;
            add(
                add(center, mul(normal, angle.cos() * radial)),
                mul(binormal, angle.sin() * width),
            )
        })
        .collect();
    (attachment, points)
}

pub(super) fn bow(
    r: &ResolvedComponent,
    p: &ArcheryBowParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let upper = limb(p, true, detail);
    let lower = limb(p, false, detail);
    let mut parts = bow_limbs(r, p, &upper, &lower, detail)?;
    bow_furniture(r, p, &upper, &lower, detail, &mut parts)?;
    bow_strings(r, p, &upper, &lower, detail, &mut parts)?;
    Ok(parts)
}

pub(super) fn arrow(
    r: &ResolvedComponent,
    p: &ArrowParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let radius = p.shaft_radius.get();
    let length = p.length.get();
    let head_length = p.head_length.get();
    let bodkin = p.head_style == Some(ArrowHeadStyle::Bodkin);
    let head_width = if bodkin {
        (radius * 2.4).max(p.head_width.get() * 0.52)
    } else {
        p.head_width.get()
    };
    let nock_length = p.nock_length.get();
    let slot = p.nock_slot_width.get();
    let material = r.component.material.unwrap_or(Material::Wood);
    let mut parts = vec![
        part(
            Solid::lathe(
                &[[0.0, radius], [length, radius * 0.92]],
                p.segments.map_or(12, |n| n.0 as usize),
                1.0,
                true,
                detail,
            )?,
            material,
            "arrow shaft",
            r,
        ),
        part(
            Solid::prism(
                &[
                    [-head_width / 2.0, 0.0],
                    [-radius, head_length * 0.18],
                    [0.0, head_length],
                    [radius, head_length * 0.18],
                    [head_width / 2.0, 0.0],
                ],
                p.head_thickness.get(),
                detail,
            )?
            .transform([0.0; 3], [0.0, length, 0.0]),
            p.head_material.unwrap_or(Material::Steel),
            if bodkin {
                "bodkin arrowhead"
            } else {
                "broadhead arrowhead"
            },
            r,
        ),
        part(
            Solid::prism(
                &[
                    [-radius, -nock_length],
                    [-slot / 2.0, -nock_length],
                    [-slot / 2.0, -nock_length * 0.32],
                    [slot / 2.0, -nock_length * 0.32],
                    [slot / 2.0, -nock_length],
                    [radius, -nock_length],
                    [radius, 0.0],
                    [-radius, 0.0],
                ],
                radius * 1.7,
                detail,
            )?,
            if p.nock_style == Some(ArrowNockStyle::SelfWood) {
                material
            } else {
                p.nock_material.unwrap_or(Material::Horn)
            },
            "slotted arrow nock",
            r,
        ),
    ];
    let base = nock_length + 0.025;
    let h = p.fletching_height.get();
    let l = p.fletching_length.get();
    for index in 0..p.fletching_count.0 {
        let solid = Solid::prism(
            &[[0.0, 0.0], [h, l * 0.18], [h * 0.78, l], [0.0, l]],
            radius * 0.36,
            detail,
        )?
        .transform([0.0; 3], [radius * 0.9, base, 0.0])
        .transform(
            [0.0, index as f64 * 360.0 / p.fletching_count.0 as f64, 0.0],
            [0.0; 3],
        );
        parts.push(part(
            solid,
            p.fletching_material.unwrap_or(Material::Feather),
            &format!("fletching {}", index + 1),
            r,
        ));
    }
    Ok(parts)
}

pub(super) fn profile_radius(profile: &[PlanarPoint], y: f64) -> f64 {
    let upper = profile
        .iter()
        .position(|p| p[0] >= y)
        .unwrap_or(profile.len() - 1);
    let [a, b] = [profile[upper.saturating_sub(1)], profile[upper]];
    let t = if b[0] == a[0] {
        0.0
    } else {
        (y - a[0]) / (b[0] - a[0])
    };
    a[1] + (b[1] - a[1]) * t
}

pub(super) fn quiver(
    r: &ResolvedComponent,
    p: &ArrowQuiverParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let bag = p.carrier_style == Some(ArrowQuiverCarrierStyle::Bag);
    let radius = p.mouth_radius.get();
    let length = p.length.get();
    let wall = p.wall.get();
    let rim = p.rim_radius.get();
    let profile = quiver_profile(p);
    let bottom = profile[0][1];
    let inner: Vec<_> = profile.iter().map(|p| p[1] - wall).collect();
    if inner.iter().any(|&r| r <= 0.0) {
        return Err("quiver wall must leave an open interior".into());
    }
    let material = r.component.material.unwrap_or(Material::Leather);
    let mut parts = vec![
        part(
            Solid::hollow_socket(&profile, &inner, None, detail),
            material,
            "open arrow quiver body",
            r,
        ),
        part(
            Solid::lathe(
                &[[0.0, bottom], [wall, bottom]],
                p.segments.map_or(16, |n| n.0 as usize),
                1.0,
                true,
                detail,
            )?,
            material,
            "sealed quiver bottom",
            r,
        ),
    ];
    if !bag {
        parts.push(part(
            Solid::hollow_socket(
                &[[length - rim, radius + rim], [length + rim, radius + rim]],
                &[radius - wall; 2],
                None,
                detail,
            ),
            p.rim_material.unwrap_or(Material::DarkLeather),
            "quiver mouth binding",
            r,
        ));
    }
    let path = quiver_strap(p, &profile);
    parts.push(part(
        Solid::sweep(
            &path,
            &Sweep {
                section: Section::Flat,
                width: p.strap_thickness.get(),
                depth: p.strap_width.get(),
                ..Sweep::default()
            },
            detail,
        )?,
        p.strap_material.unwrap_or(Material::DarkLeather),
        "quiver shoulder strap",
        r,
    ));
    Ok(parts)
}

pub(super) fn composite_layers(
    p: &ArcheryBowParameters,
) -> Result<[(&'static str, f64, f64, Material); 3], String> {
    let horn = p
        .horn_thickness
        .ok_or("composite bow needs horn thickness")?
        .get();
    let backing = p
        .backing_thickness
        .ok_or("composite bow needs backing thickness")?
        .get();
    let depth = p.limb_depth.get();
    let core = depth - horn - backing;
    if core <= 0.0 {
        return Err("bow core must retain positive depth between laminations".into());
    }
    Ok([
        (
            "wood core",
            core,
            (horn - backing) / 2.0,
            p.core_material.unwrap_or(Material::Wood),
        ),
        (
            "horn belly",
            horn,
            -depth / 2.0 + horn / 2.0,
            p.horn_material.unwrap_or(Material::Horn),
        ),
        (
            "sinew backing",
            backing,
            depth / 2.0 - backing / 2.0,
            p.back_material.unwrap_or(Material::Sinew),
        ),
    ])
}

pub(super) fn quiver_profile(p: &ArrowQuiverParameters) -> Vec<PlanarPoint> {
    let bag = p.carrier_style == Some(ArrowQuiverCarrierStyle::Bag);
    let radius = p.mouth_radius.get();
    let length = p.length.get();
    let bottom = radius
        * if bag {
            p.bottom_scale.get().max(0.68)
        } else {
            p.bottom_scale.get()
        };

    if bag {
        vec![
            [0.0, bottom],
            [length * 0.12, bottom * 1.08],
            [length * 0.82, radius],
            [length, radius * 0.9],
        ]
    } else {
        vec![
            [0.0, bottom],
            [length * 0.08, bottom * 1.05],
            [length, radius],
        ]
    }
}

pub(super) fn quiver_strap(p: &ArrowQuiverParameters, profile: &[PlanarPoint]) -> [Point; 3] {
    let length = p.length.get();
    let lower = length * 0.18;
    let upper = length * 0.82;
    let lower_radius = profile_radius(profile, lower);
    let upper_radius = profile_radius(profile, upper);
    let overlap = p.strap_thickness.get() * 0.35;
    let outer = lower_radius.max(upper_radius) + p.strap_drop.get();

    [
        [lower_radius - overlap, lower, 0.0],
        [outer, length * 0.52, 0.0],
        [upper_radius - overlap, upper, 0.0],
    ]
}
