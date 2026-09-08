//! A reference grid behind transmitting materials makes distortion and tint legible.
use super::*;

pub(super) fn spawn(world: &mut World, assets: &mut SceneAssets, x: f32) {
    let mesh = world
        .resource_mut::<Assets<Mesh>>()
        .add(Rectangle::new(0.30, 0.30));
    assets.meshes.push(mesh.clone());
    let materials = [[0.68, 0.65, 0.58], [0.10, 0.18, 0.23]].map(|rgb| {
        world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::srgb_from_array(rgb),
                unlit: true,
                cull_mode: None,
                ..default()
            })
    });
    assets.materials.extend(materials.iter().cloned());
    for row in 0..7 {
        for column in 0..7 {
            assets.entities.push(
                world
                    .spawn((
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(materials[(row + column) % 2].clone()),
                        Transform::from_xyz(
                            x + (column as f32 - 3.0) * 0.30,
                            (row as f32 - 3.0) * 0.30,
                            -0.95,
                        ),
                    ))
                    .id(),
            );
        }
    }
}
