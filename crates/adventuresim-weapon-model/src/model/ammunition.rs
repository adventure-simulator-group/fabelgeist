//! Cast round balls and hinged leather ammunition pouches.
use super::*;
use std::f64::consts::{PI, TAU};

pub(super) fn ball(
    r: &ResolvedComponent,
    p: &LeadBallParameters,
    detail: Detail,
) -> Vec<PartSource> {
    let radius = p.radius.get();
    let longitude = detail.samples(p.segments.map_or(16, |n| n.0 as usize), 15);
    let latitude = (longitude / 2).max(8);
    let point = |ring: usize, segment: usize| {
        let phi = PI * ring as f64 / latitude as f64;
        let theta = TAU * segment as f64 / longitude as f64;
        [
            radius * phi.sin() * theta.cos(),
            radius * (1.0 + phi.cos()),
            radius * phi.sin() * theta.sin(),
        ]
    };
    let mut solid = Solid::default();
    for segment in 0..longitude {
        let next = (segment + 1) % longitude;
        solid.triangle(
            [0.0, radius * 2.0, 0.0],
            point(1, next),
            point(1, segment),
            0,
        );
    }
    for ring in 1..latitude - 1 {
        for segment in 0..longitude {
            let next = (segment + 1) % longitude;
            solid.quad(
                point(ring, segment),
                point(ring, next),
                point(ring + 1, next),
                point(ring + 1, segment),
                0,
            );
        }
    }
    for segment in 0..longitude {
        solid.triangle(
            point(latitude - 1, segment),
            point(latitude - 1, (segment + 1) % longitude),
            [0.0; 3],
            0,
        );
    }
    vec![PartSource::new(
        solid.positive(),
        r.component.material.unwrap_or(Material::Lead),
        "lead round ball",
        &r.id,
    )]
}

pub(super) fn pouch(
    r: &ResolvedComponent,
    p: &BallPouchParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let width = p.width.get();
    let height = p.height.get();
    let depth = p.depth.get();
    let half_w = width / 2.0;
    let half_d = depth / 2.0;
    let wall = p.wall.get();
    let material = r.component.material.unwrap_or(Material::Leather);
    let mut a = firearms::Assembly {
        parts: Vec::new(),
        resolved: r,
        detail,
    };
    for (size, offset, label) in [
        (
            [width, height, wall],
            [0.0, height / 2.0, half_d - wall / 2.0],
            "ball pouch front",
        ),
        (
            [width, height, wall],
            [0.0, height / 2.0, -half_d + wall / 2.0],
            "ball pouch back",
        ),
        (
            [wall, height, depth],
            [-half_w + wall / 2.0, height / 2.0, 0.0],
            "ball pouch left gusset",
        ),
        (
            [wall, height, depth],
            [half_w - wall / 2.0, height / 2.0, 0.0],
            "ball pouch right gusset",
        ),
        (
            [width, wall, depth],
            [0.0, wall / 2.0, 0.0],
            "sealed ball pouch bottom",
        ),
    ] {
        a.cuboid(size, offset, material, label)?;
    }
    pouch_flap(&mut a, p)?;
    for (side, name) in [(-1.0, "left"), (1.0, "right")] {
        let gap = p.belt_loop_gap.get();
        let width = p.belt_loop_width.get();
        let points = [
            [side * gap / 2.0 - width / 2.0, height * 0.72, -half_d],
            [side * gap / 2.0, height + 0.045, -half_d - 0.012],
            [side * gap / 2.0 + width / 2.0, height * 0.72, -half_d],
        ];
        a.add(
            Solid::sweep(
                &points,
                &Sweep {
                    section: Section::Flat,
                    width: wall,
                    depth: width,
                    fit_bends: true,
                    ..Sweep::default()
                },
                detail,
            )?,
            material,
            &format!("belt attachment loop {name}"),
        );
    }
    let hardware = p.hardware_material.unwrap_or(Material::Horn);
    match p.closure_style {
        BallPouchClosureStyle::Toggle => {
            a.add(
                Solid::lathe(&[[0.0, 0.004], [0.024, 0.004]], 8, 1.0, false, detail)?
                    .transform([0.0; 3], [0.0, height * 0.58, half_d]),
                hardware,
                "horn pouch toggle",
            );
        }
        BallPouchClosureStyle::Buckle => {
            a.cuboid(
                [0.026, 0.018, 0.004],
                [0.0, height * 0.58, half_d],
                hardware,
                "pouch buckle tongue",
            )?;
        }
    }
    Ok(a.parts)
}

fn pouch_flap(a: &mut firearms::Assembly<'_>, p: &BallPouchParameters) -> Result<(), String> {
    let detail = a.detail;
    let material = a.resolved.component.material.unwrap_or(Material::Leather);
    let width = p.width.get();
    let height = p.height.get();
    let depth = p.depth.get();
    let half_w = width / 2.0;
    let half_d = depth / 2.0;
    let wall = p.wall.get();
    let hinge = [0.0, height, -half_d];
    let mut flap = Solid::cuboid([width, wall, depth], detail)?
        .transform([0.0; 3], [0.0, -wall / 2.0, half_d]);
    let length = p.flap_length.get();
    flap.append(
        Solid::prism(
            &[
                [-half_w, 0.0],
                [-half_w * 0.92, -length],
                [0.0, -length - p.flap_overlap.get()],
                [half_w * 0.92, -length],
                [half_w, 0.0],
            ],
            wall,
            detail,
        )?
        .transform([0.0; 3], [0.0, 0.0, depth]),
    );
    a.add(
        flap.transform([p.flap_angle.get(), 0.0, 0.0], hinge),
        material,
        "ball pouch hinged flap",
    )
    .animate(hinge, output::AnimationChannel::PouchFlap);
    a.add(
        guards::tube(
            &[[-half_w * 0.85, height], [half_w * 0.85, height]],
            wall * 0.8,
            8,
            detail,
        )?
        .transform([0.0; 3], [0.0, 0.0, -half_d]),
        material,
        "pouch flap hinge",
    );

    Ok(())
}
