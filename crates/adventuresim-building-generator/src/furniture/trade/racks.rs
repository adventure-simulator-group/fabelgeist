use super::{Builder, FurnitureKind, Vec3, beam, legs};

pub(super) fn rack(builder: &mut Builder, size: Vec3, kind: FurnitureKind) {
    if kind == FurnitureKind::HayRack {
        hay_rack(builder, size);
        return;
    }
    let post = 0.075;
    legs(builder, size, size.y, post);
    for y in [0.12, size.y - post * 0.5] {
        for sign in [-1.0, 1.0] {
            builder.timber(
                Vec3::new(0.0, y, sign * (size.z - post) * 0.5),
                Vec3::new(size.x, post, post),
            );
            builder.timber(
                Vec3::new(sign * (size.x - post) * 0.5, y, 0.0),
                Vec3::new(post, post, size.z),
            );
        }
    }
    match kind {
        FurnitureKind::ToolRack => tool_rails(builder, size),
        FurnitureKind::WeaponRack => weapon_rails(builder, size),
        FurnitureKind::DryingRack => drying_rails(builder, size),
        FurnitureKind::CaskRack => cask_cradles(builder, size),
        _ => unreachable!("unsupported rack kind"),
    }
}

fn tool_rails(builder: &mut Builder, size: Vec3) {
    let hooks = if size.x > 1.2 { 7 } else { 4 };
    for y in [size.y * 0.42, size.y * 0.76] {
        builder.timber(
            Vec3::new(0.0, y, size.z * 0.5 - 0.04),
            Vec3::new(size.x, 0.18, 0.08),
        );
        for index in 0..hooks {
            let x = ((index as f32 + 0.5) / hooks as f32 - 0.5) * (size.x - 0.14);
            builder.timber(
                Vec3::new(x, y, size.z * 0.1),
                Vec3::new(0.035, 0.035, size.z * 0.8),
            );
        }
    }
}

fn weapon_rails(builder: &mut Builder, size: Vec3) {
    let slots = if size.x > 1.5 { 7 } else { 5 };
    builder.timber(Vec3::Y * 0.18, Vec3::new(size.x, 0.075, size.z));
    for y in [0.28, size.y * 0.78] {
        builder.timber(
            Vec3::new(0.0, y, size.z * 0.5 - 0.065),
            Vec3::new(size.x, 0.1, 0.13),
        );
        for index in 0..=slots {
            let x = (index as f32 / slots as f32 - 0.5) * (size.x - 0.1);
            builder.timber(
                Vec3::new(x, y, size.z * 0.08),
                Vec3::new(0.06, 0.1, size.z * 0.78),
            );
        }
    }
}

fn drying_rails(builder: &mut Builder, size: Vec3) {
    // Open ladder rails take suspended skins, linen or herbs in a future stock
    // system; nothing is attached to the furniture in this recipe.
    for fraction in [0.35, 0.58, 0.81] {
        for sign in [-1.0, 1.0] {
            builder.timber(
                Vec3::new(0.0, size.y * fraction, sign * (size.z - 0.075) * 0.5),
                Vec3::new(size.x, 0.045, 0.045),
            );
        }
    }
}

fn cask_cradles(builder: &mut Builder, size: Vec3) {
    for fraction in [0.18, 0.58] {
        let height = size.y * fraction;
        for sign in [-1.0, 1.0] {
            builder.timber(
                Vec3::new(0.0, height, sign * (size.z - 0.075) * 0.5),
                Vec3::new(size.x, 0.1, 0.075),
            );
        }
        for x in [-size.x * 0.3, size.x * 0.3] {
            builder.timber(Vec3::new(x, height, 0.0), Vec3::new(0.12, 0.1, size.z));
            for sign in [-1.0, 1.0] {
                beam(
                    builder,
                    Vec3::new(x, height + 0.04, sign * 0.025),
                    Vec3::new(x, height + 0.22, sign * (size.z * 0.5 - 0.075)),
                    0.075,
                );
            }
        }
    }
}

fn hay_rack(builder: &mut Builder, size: Vec3) {
    let post = 0.075;
    legs(builder, size, size.y, post);
    let bottom = size.y * 0.32;
    builder.timber(
        Vec3::new(0.0, bottom, size.z * 0.15),
        Vec3::new(size.x, 0.075, size.z * 0.7),
    );
    for z in [-1.0, 1.0] {
        builder.timber(
            Vec3::new(0.0, size.y - post * 0.5, z * (size.z - post) * 0.5),
            Vec3::new(size.x, post, post),
        );
    }
    let slats = if size.x > 1.5 { 9 } else { 6 };
    for index in 0..slats {
        let x = ((index as f32 + 0.5) / slats as f32 - 0.5) * (size.x - post);
        beam(
            builder,
            Vec3::new(x, bottom, -size.z * 0.18),
            Vec3::new(x, size.y - 0.065, -size.z * 0.5 + 0.045),
            0.035,
        );
        builder.timber(
            Vec3::new(x, (bottom + size.y - post) * 0.5, size.z * 0.5 - 0.04),
            Vec3::new(0.035, size.y - post - bottom, 0.035),
        );
    }
}

pub(super) fn armour_stand(builder: &mut Builder, size: Vec3) {
    // Broad cruciform feet support a central post and empty shoulder yoke.
    builder.timber(Vec3::Y * 0.055, Vec3::new(size.x, 0.11, 0.13));
    builder.timber(Vec3::Y * 0.055, Vec3::new(0.13, 0.11, size.z));
    builder.timber(Vec3::Y * (size.y * 0.5), Vec3::new(0.11, size.y, 0.11));
    builder.timber(Vec3::Y * (size.y * 0.82), Vec3::new(size.x, 0.1, 0.18));
    builder.timber(
        Vec3::Y * (size.y * 0.48),
        Vec3::new(size.x * 0.7, 0.08, 0.13),
    );
    for sign in [-1.0, 1.0] {
        beam(
            builder,
            Vec3::new(0.0, 0.16, 0.0),
            Vec3::new(sign * size.x * 0.32, 0.085, 0.0),
            0.06,
        );
    }
}
