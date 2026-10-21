//! Native GPU upload and readback boundary shared by rendering integration tests.

use bevy::render::render_resource::*;
use fabelgeist_gpu::globals::WgpuContext;

pub fn buffer(context: &WgpuContext, bytes: &[u8], usage: BufferUsages) -> Buffer {
    let buffer = context.device.create_buffer(&BufferDescriptor {
        label: None,
        size: bytes.len() as u64,
        usage: usage | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    context.queue.write_buffer(&buffer, 0, bytes);
    buffer.into()
}

pub fn read<T: bytemuck::Pod>(context: &WgpuContext, source: &Buffer) -> Vec<T> {
    let staging = context.device.create_buffer(&BufferDescriptor {
        label: None,
        size: source.size(),
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = context
        .device
        .create_command_encoder(&CommandEncoderDescriptor::default());
    encoder.copy_buffer_to_buffer(source, 0, &staging, 0, source.size());
    context.queue.submit([encoder.finish()]);
    let (send, receive) = std::sync::mpsc::channel();
    staging
        .slice(..)
        .map_async(MapMode::Read, move |result| send.send(result).unwrap());
    context.device.poll(PollType::wait_indefinitely()).unwrap();
    receive
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap()
        .unwrap();
    let result = staging
        .slice(..)
        .get_mapped_range()
        .chunks_exact(size_of::<T>())
        .map(bytemuck::pod_read_unaligned)
        .collect();
    staging.unmap();
    result
}
