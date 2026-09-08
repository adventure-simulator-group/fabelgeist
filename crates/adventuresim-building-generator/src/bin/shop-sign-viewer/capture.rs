use super::*;
pub(super) fn mesh(batch: &LodMesh) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        batch
            .vertices
            .iter()
            .map(|v| v.position.to_array())
            .collect::<Vec<_>>(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        batch
            .vertices
            .iter()
            .map(|v| v.normal.to_array())
            .collect::<Vec<_>>(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        batch
            .vertices
            .iter()
            .map(|v| v.uv.to_array())
            .collect::<Vec<_>>(),
    );
    mesh.insert_indices(Indices::U32(batch.indices.clone()));
    mesh.generate_tangents()
        .expect("generated building triangles have metric UVs");
    mesh
}

pub(super) fn capture(mut commands: Commands, mut state: ResMut<Capture>) {
    if state.in_flight {
        return;
    }
    state.frames += 1;
    if state.frames < SETTLE_FRAMES {
        return;
    }
    state.in_flight = true;
    commands.spawn(Screenshot::primary_window()).observe(
        |captured: On<ScreenshotCaptured>,
         mut state: ResMut<Capture>,
         mut exit: MessageWriter<AppExit>| {
            if !state.primed {
                state.primed = true;
                state.in_flight = false;
                state.frames = 0;
                return;
            }
            save_to_disk(&state.output)(captured);
            exit.write(AppExit::Success);
        },
    );
}
