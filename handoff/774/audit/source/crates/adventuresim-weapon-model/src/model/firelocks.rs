//! Explicit animation pivots and physical pans, jaws and sear linkages.
use super::*;
use firearms::Assembly;

fn pan(
    a: &mut Assembly<'_>,
    p: &FirearmParameters,
    center: Point,
    pivot: Point,
    prefix: &str,
) -> Result<(), String> {
    let [x, y, z] = center;
    let half = p.pan_width.get() / 2.0;
    let material = p.lock_material.unwrap_or(Material::Steel);
    for (size, position, label) in [
        (
            [p.pan_width.get(), 0.028, 0.002],
            [x, y, z],
            "pan cavity bottom",
        ),
        (
            [0.002, 0.028, 0.008],
            [x - half + 0.001, y, z + 0.004],
            "pan cavity inner wall",
        ),
        (
            [0.002, 0.028, 0.008],
            [x + half - 0.001, y, z + 0.004],
            "pan cavity outer wall",
        ),
        (
            [p.pan_width.get(), 0.002, 0.008],
            [x, y - 0.013, z + 0.004],
            "pan cavity rear wall",
        ),
        (
            [p.pan_width.get(), 0.002, 0.008],
            [x, y + 0.013, z + 0.004],
            "pan cavity front wall",
        ),
    ] {
        a.cuboid(size, position, material, &format!("{prefix} {label}"))?;
    }
    a.cuboid(
        [p.pan_width.get(), 0.028, 0.003],
        [x, y, z + 0.010],
        material,
        &format!("{prefix} pan cover"),
    )?
    .animate(pivot, output::AnimationChannel::FirearmLock);
    Ok(())
}

fn channel(
    a: &mut Assembly<'_>,
    p: &FirearmParameters,
    pan: Point,
    bore: Point,
    label: &str,
) -> Result<(), String> {
    let target = [p.bore.get() * 0.42, bore[1], bore[2]];
    let width = (p.bore.get() * 0.16).min(0.003);
    let part = a.sweep(
        &[pan, target],
        Section::Round,
        width,
        width,
        p.lock_material.unwrap_or(Material::Steel),
        label,
    )?;
    part.bore = Some(output::BoreConnection {
        touchhole_centerline: [pan, target],
        bore_center: bore,
        bore_radius: p.bore.get() / 2.0,
    });
    Ok(())
}

pub(super) fn locks(
    a: &mut Assembly<'_>,
    p: &FirearmParameters,
    centers: &[f64],
    outer: f64,
    start: f64,
) -> Result<(), String> {
    let side = p.waist_width.get().max(p.fore_width.get()) / 2.0 + 0.003;
    let z = -p.stock_depth.get() * 0.22;
    let lock = p.lock_position.get();
    let depth = p.stock_depth.get();
    let plate_depth = depth
        * if p.stock_style == FirearmStockStyle::Pistol {
            1.18
        } else {
            0.72
        };
    let material = p.lock_material.unwrap_or(Material::Steel);

    a.cuboid(
        [0.004, 0.145, plate_depth],
        [side, lock, z],
        material,
        "combined firearm lock plate",
    )?;

    if p.lock_type == FirearmLockType::Wheellock {
        for (index, &barrel_z) in centers.iter().enumerate() {
            wheel(a, p, index)?;
            wheel_ignition(a, p, start, index, barrel_z, outer)?;
        }
        wheel_safety(a, p)?;
    } else {
        match_ignition(a, p, start, centers, outer)?;
        match_trigger(a, p)?;
    }
    Ok(())
}
fn wheel(a: &mut Assembly<'_>, p: &FirearmParameters, index: usize) -> Result<(), String> {
    let side = p.waist_width.get().max(p.fore_width.get()) / 2.0 + 0.003;
    let z = -p.stock_depth.get() * 0.22;
    let lock = p.lock_position.get();
    let material = p.lock_material.unwrap_or(Material::Steel);
    let name = if index == 0 { "upper" } else { "lower" };
    let train_z = z + if index == 0 { 0.018 } else { -0.025 };
    let wheel = [
        side + 0.005,
        lock + if index == 0 { 0.025 } else { -0.028 },
        train_z,
    ];
    a.add(
        Solid::lathe(
            &[
                [-0.004, p.lock_wheel_radius.get()],
                [0.004, p.lock_wheel_radius.get()],
            ],
            20,
            1.0,
            false,
            a.detail,
        )?
        .transform([0.0, 0.0, 90.0], wheel),
        material,
        &format!("wheellock {name} wheel"),
    )
    .animate(wheel, output::AnimationChannel::FirearmLock);
    a.sweep(
        &[
            [side - 0.006, wheel[1], wheel[2]],
            [side + 0.014, wheel[1], wheel[2]],
        ],
        Section::Round,
        0.006,
        0.006,
        material,
        &format!("wheellock {name} axle and bearing"),
    )?;
    a.sweep(
        &[
            [side + 0.002, wheel[1] - 0.045, train_z - 0.010],
            [side + 0.002, wheel[1], train_z - p.lock_wheel_radius.get()],
            [side + 0.002, wheel[1] + 0.030, train_z - 0.010],
        ],
        Section::Flat,
        0.005,
        0.010,
        material,
        &format!("wheellock {name} mainspring"),
    )?;

    Ok(())
}
fn wheel_ignition(
    a: &mut Assembly<'_>,
    p: &FirearmParameters,
    start: f64,
    index: usize,
    barrel_z: f64,
    outer: f64,
) -> Result<(), String> {
    let side = p.waist_width.get().max(p.fore_width.get()) / 2.0 + 0.003;
    let z = -p.stock_depth.get() * 0.22;
    let lock = p.lock_position.get();
    let material = p.lock_material.unwrap_or(Material::Steel);
    let pan_y = start + 0.012;
    let name = if index == 0 { "upper" } else { "lower" };
    let train_z = z + if index == 0 { 0.018 } else { -0.025 };
    let pan_z = barrel_z - outer * 0.45;
    let cock = [side + 0.010, lock + 0.055, train_z];
    let pan_pivot = [side + p.pan_width.get() / 2.0, pan_y - 0.014, pan_z + 0.010];
    pan(
        a,
        p,
        [side + p.pan_width.get() / 2.0 - 0.003, pan_y, pan_z],
        pan_pivot,
        &format!("wheellock {name}"),
    )?;
    channel(
        a,
        p,
        [side, pan_y, pan_z + 0.003],
        [0.0, pan_y, barrel_z],
        &format!("{name} touchhole channel to bore"),
    )?;
    let jaw_y = pan_y - 0.002;
    let jaw_z = pan_z + 0.016;
    a.sweep(
        &[cock, [side + 0.010, jaw_y, jaw_z]],
        Section::Flat,
        0.006,
        0.010,
        material,
        &format!("wheellock {name} cock arm"),
    )?
    .animate(cock, output::AnimationChannel::FirearmLock);
    for (size, position, material, suffix) in [
        (
            [0.005, 0.016, 0.005],
            [side + 0.010, jaw_y - 0.005, jaw_z + 0.005],
            material,
            "cock upper jaw",
        ),
        (
            [0.005, 0.016, 0.005],
            [side + 0.010, jaw_y - 0.005, jaw_z - 0.005],
            material,
            "cock lower jaw",
        ),
        (
            [0.004, 0.009, 0.006],
            [side + 0.010, jaw_y + 0.002, jaw_z],
            Material::Pyrite,
            "visible pyrite",
        ),
    ] {
        a.cuboid(
            size,
            position,
            material,
            &format!("wheellock {name} {suffix}"),
        )?
        .animate(cock, output::AnimationChannel::FirearmLock);
    }
    Ok(())
}
fn wheel_safety(a: &mut Assembly<'_>, p: &FirearmParameters) -> Result<(), String> {
    let side = p.waist_width.get().max(p.fore_width.get()) / 2.0 + 0.003;
    let z = -p.stock_depth.get() * 0.22;
    let lock = p.lock_position.get();
    let depth = p.stock_depth.get();
    let material = p.lock_material.unwrap_or(Material::Steel);
    let furniture = p
        .furniture_material
        .or(p.lock_material)
        .unwrap_or(Material::Steel);
    let safety = [side + 0.008, lock - 0.055, z + 0.012];
    let trigger = [0.0, lock - 0.035, -depth * 0.54];
    a.add(
        Solid::lathe(&[[-0.001, 0.003], [0.009, 0.003]], 10, 1.0, false, a.detail)?
            .transform([0.0, 0.0, -90.0], [side, safety[1], safety[2]]),
        material,
        "safety lever bearing",
    );
    a.sweep(
        &[safety, [side + 0.008, lock - 0.020, z + 0.012]],
        Section::Flat,
        0.005,
        0.010,
        material,
        "wheellock safety lever",
    )?
    .animate(safety, output::AnimationChannel::FirearmLock);
    a.sweep(
        &[
            trigger,
            [
                0.0,
                trigger[1] - p.trigger_length.get() * 0.58,
                trigger[2] - 0.018,
            ],
        ],
        Section::Flat,
        0.006,
        0.010,
        furniture,
        "firearm trigger blade",
    )?
    .animate(trigger, output::AnimationChannel::FirearmLock);
    a.sweep(
        &[
            [side, trigger[1], trigger[2]],
            [side, lock - 0.010, z - 0.010],
            [side, lock + 0.020, z],
        ],
        Section::Round,
        0.004,
        0.004,
        material,
        "wheellock sear linkage",
    )?
    .animate(trigger, output::AnimationChannel::FirearmLock);

    Ok(())
}
fn match_ignition(
    a: &mut Assembly<'_>,
    p: &FirearmParameters,
    start: f64,
    centers: &[f64],
    outer: f64,
) -> Result<(), String> {
    let side = p.waist_width.get().max(p.fore_width.get()) / 2.0 + 0.003;
    let z = -p.stock_depth.get() * 0.22;
    let lock = p.lock_position.get();
    let material = p.lock_material.unwrap_or(Material::Steel);
    let pan_y = start + 0.012;
    let pan_z = centers[0] - outer * 0.45;
    let pan_pivot = [side + p.pan_width.get() / 2.0, pan_y - 0.014, pan_z + 0.010];
    let serpentine = [side + 0.010, lock, z - 0.018];
    pan(
        a,
        p,
        [side + p.pan_width.get() / 2.0 - 0.003, pan_y, pan_z],
        pan_pivot,
        "matchlock",
    )?;
    channel(
        a,
        p,
        [side, pan_y, pan_z + 0.003],
        [0.0, pan_y, centers[0]],
        "matchlock touchhole channel to bore",
    )?;
    let jaw_y = pan_y;
    let jaw_z = pan_z + 0.016;
    a.sweep(
        &[serpentine, [side + 0.010, jaw_y, jaw_z]],
        Section::Round,
        0.009,
        0.009,
        material,
        "matchlock serpentine arm",
    )?
    .animate(serpentine, output::AnimationChannel::FirearmLock);
    for (offset, label) in [
        (0.005, "matchlock serpentine upper jaw"),
        (-0.005, "matchlock serpentine lower jaw"),
    ] {
        a.cuboid(
            [0.005, 0.018, 0.005],
            [side + 0.010, jaw_y - 0.004, jaw_z + offset],
            material,
            label,
        )?
        .animate(serpentine, output::AnimationChannel::FirearmLock);
    }
    a.sweep(
        &[
            [side + 0.010, jaw_y - 0.010, jaw_z],
            [side + 0.010, jaw_y + 0.018, jaw_z],
        ],
        Section::Round,
        0.0044,
        0.0044,
        Material::MatchCord,
        "visible match cord",
    )?
    .animate(serpentine, output::AnimationChannel::FirearmLock);

    Ok(())
}
fn match_trigger(a: &mut Assembly<'_>, p: &FirearmParameters) -> Result<(), String> {
    let side = p.waist_width.get().max(p.fore_width.get()) / 2.0 + 0.003;
    let z = -p.stock_depth.get() * 0.22;
    let lock = p.lock_position.get();
    let depth = p.stock_depth.get();
    let material = p.lock_material.unwrap_or(Material::Steel);
    let furniture = p
        .furniture_material
        .or(p.lock_material)
        .unwrap_or(Material::Steel);
    let trigger = [0.0, lock - 0.040, -depth * 0.54];
    let serpentine = [side + 0.010, lock, z - 0.018];
    a.sweep(
        &[
            trigger,
            [
                0.0,
                trigger[1] - p.trigger_length.get() * 0.62,
                trigger[2] - 0.020,
            ],
        ],
        Section::Flat,
        0.006,
        0.010,
        furniture,
        "firearm trigger blade",
    )?
    .animate(trigger, output::AnimationChannel::FirearmLock);
    a.sweep(
        &[
            [side, trigger[1], trigger[2]],
            [side, lock - 0.025, z - 0.030],
            serpentine,
        ],
        Section::Flat,
        0.005,
        0.008,
        material,
        "matchlock trigger linkage",
    )?
    .animate(trigger, output::AnimationChannel::FirearmLock);
    Ok(())
}
