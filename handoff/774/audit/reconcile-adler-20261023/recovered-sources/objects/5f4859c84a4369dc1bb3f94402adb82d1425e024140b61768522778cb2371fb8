//! Supported counter displays distinguish occupied stalls without changing their footprint.
use super::*;
use bevy::math::Vec2;

pub(super) fn display(builder: &mut Builder, variant: FurnitureVariant) {
    let (bins, sack_x, sack_height): (&[f32], f32, f32) = match variant {
        FurnitureVariant::Compact => (&[-0.40], 0.60, 0.36),
        FurnitureVariant::Broad => (&[-0.95, 0.0], 1.10, 0.46),
    };
    for (index, x) in bins.iter().copied().enumerate() {
        grain_bin(builder, x, 0.07 + index as f32 * 0.025);
    }
    super::super::containers::sack(
        builder,
        Vec2::new(sack_x, COUNTER_CENTRE_Z_METRES),
        COUNTER_HEIGHT_METRES,
        0.20,
        sack_height,
    );
}

fn grain_bin(builder: &mut Builder, x: f32, heap_height: f32) {
    let origin = Vec3::new(x, COUNTER_HEIGHT_METRES, COUNTER_CENTRE_Z_METRES);
    let width = 0.80;
    let depth = 0.44;
    let height = 0.18;
    let wall = 0.035;
    // Low open rims expose the stock to a customer while retaining a visible
    // wooden floor. Every piece starts on the counter or the supported floor.
    builder.timber(origin + Vec3::Y * wall * 0.5, Vec3::new(width, wall, depth));
    for z in [-depth * 0.5 + wall * 0.5, depth * 0.5 - wall * 0.5] {
        builder.timber(
            origin + Vec3::new(0.0, height * 0.5, z),
            Vec3::new(width, height, wall),
        );
    }
    for side in [-1.0, 1.0] {
        builder.timber(
            origin + Vec3::new(side * (width - wall) * 0.5, height * 0.5, 0.0),
            Vec3::new(wall, height, depth - wall * 2.0),
        );
    }
    let fill_height = height - wall;
    builder.cuboid(
        BuildingLodMaterial::Grain,
        origin + Vec3::Y * (wall + fill_height * 0.5),
        Vec3::new(width - wall * 2.0, fill_height, depth - wall * 2.0),
        Quat::IDENTITY,
        CollisionPolicy::Solid,
    );
    let half = Vec2::new(width, depth) * 0.5 - Vec2::splat(wall);
    let corners = [
        Vec2::new(-half.x, -half.y),
        Vec2::new(half.x, -half.y),
        Vec2::new(half.x, half.y),
        Vec2::new(-half.x, half.y),
    ];
    let crest = origin + Vec3::new(-0.08, height + heap_height, 0.025);
    for side in 0..4 {
        let edge = [corners[side], corners[(side + 1) % 4]]
            .map(|point| origin + Vec3::new(point.x, height, point.y));
        let normal = (edge[1] - edge[0]).cross(crest - edge[0]).normalize();
        builder.triangle(
            BuildingLodMaterial::Grain,
            [edge[0], edge[1], crest],
            -normal,
        );
    }
    builder.collider(
        origin + Vec3::Y * (height + heap_height * 0.5),
        Vec3::new(width - wall * 2.0, heap_height, depth - wall * 2.0),
    );
}
