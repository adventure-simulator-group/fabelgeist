//! A camera and its cascades share scratch storage reused by the next camera.
use super::*;
use bevy::render::{
    render_asset::RenderAssets,
    render_resource::{binding_types::*, *},
    renderer::{RenderDevice, RenderQueue},
    storage::GpuShaderBuffer,
};

pub(super) fn geometry_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "city indexed geometry",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX,
            (
                storage_buffer_read_only::<Vec4>(false),
                storage_buffer_read_only::<assembly::Building>(false),
                storage_buffer_read_only::<UVec2>(false),
                storage_buffer_read_only::<assembly::DrawRange>(false),
                storage_buffer_read_only::<u32>(false),
                storage_buffer_read_only::<u32>(false),
                storage_buffer_read_only::<CityFrame>(false),
            ),
        ),
    )
}

pub(super) struct Batch {
    pub geometry: BindGroup,
    pub visible: Buffer,
    pub indirect: Buffer,
}

#[derive(Resource, Default)]
pub(super) struct Scratch {
    pub owners: PresentationOwners<CityScratch>,
}

#[derive(Default)]
pub(super) struct CityScratch {
    pub batches: Vec<Batch>,
    source: Option<AssetId<ShaderBuffer>>,
    pub slots: usize,
}

impl CityScratch {
    pub fn prepare(
        &mut self,
        scene: &CityGpuScene,
        slots: usize,
        buffers: &RenderAssets<GpuShaderBuffer>,
        device: &RenderDevice,
        queue: &RenderQueue,
        cache: &PipelineCache,
    ) -> bool {
        if scene.count == 0 {
            *self = Self::default();
            return false;
        }
        let source = Some(scene.buildings.id());
        if self.source == source && self.slots >= slots {
            return true;
        }
        let (Some(buildings), Some(frame)) = (
            buffers.get(&scene.buildings),
            buffers.get(&scene.frame_buffer),
        ) else {
            return false;
        };
        let layout = cache.get_bind_group_layout(&geometry_layout());
        let mut prepared = Vec::new();
        let mut bytes = 0;
        for batch in scene.batches.iter() {
            let (Some(vertices), Some(indices), Some(ranges), Some(owners)) = (
                buffers.get(&batch.vertices),
                buffers.get(&batch.indices),
                buffers.get(&batch.source),
                buffers.get(&batch.owners),
            ) else {
                return false;
            };
            let size = u64::from(batch.capacity) * slots as u64 * size_of::<UVec2>() as u64;
            if size > device.limits().max_storage_buffer_binding_size {
                warn!(
                    size,
                    "City visibility scratch exceeds device binding capacity"
                );
                return false;
            }
            let visible = device.create_buffer(&BufferDescriptor {
                label: Some("city camera-group visibility"),
                size,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
            let indirect = device.create_buffer(&BufferDescriptor {
                label: Some("city camera-group draws"),
                size: slots as u64 * 16,
                usage: BufferUsages::STORAGE
                    | BufferUsages::INDIRECT
                    | BufferUsages::VERTEX
                    | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let words: Vec<u8> = (0..slots)
                .flat_map(|slot| {
                    [VERTICES_PER_CLUSTER, 0, 0, slot as u32 * batch.capacity]
                        .into_iter()
                        .flat_map(u32::to_le_bytes)
                })
                .collect();
            queue.write_buffer(&indirect, 0, &words);
            let geometry = device.create_bind_group(
                "city indexed geometry",
                &layout,
                &BindGroupEntries::sequential((
                    vertices.buffer.as_entire_binding(),
                    buildings.buffer.as_entire_binding(),
                    visible.as_entire_binding(),
                    ranges.buffer.as_entire_binding(),
                    indices.buffer.as_entire_binding(),
                    owners.buffer.as_entire_binding(),
                    frame.buffer.as_entire_binding(),
                )),
            );
            prepared.push(Batch {
                geometry,
                visible,
                indirect,
            });
            bytes += size + slots as u64 * 16;
        }
        self.batches = prepared;
        self.source = source;
        self.slots = slots;
        info!(slots, bytes, "GPU city camera-group scratch");
        true
    }
}
