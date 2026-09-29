//! Kernels dispatched from many threads at once, as a pool of workers each
//! building its own result does.

use fabelgeist_compute::{KernelBatch, KernelCache};
use fabelgeist_gpu::prelude::*;

/// Writes its uniform into every output slot, after a busy loop over an
/// uploaded input that keeps each submission in flight long enough for the
/// other threads' dispatches to overlap it.
const FILL: &str = r#"
@group(0) @binding(0) var<storage, read_write> output: array<u32>;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var<storage, read> input: array<u32>;
struct Params {
    count: u32,
    value: u32,
    pad0: u32,
    pad1: u32,
};

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x < params.count) {
        var sum = 0u;
        for (var i = 0u; i < arrayLength(&input); i = i + 1u) {
            sum = sum + input[i] * (id.x + 1u);
        }
        output[id.x] = params.value + sum * 0u;
    }
}
"#;

/// Every thread's first dispatches land in ring slots nothing has written
/// yet. Their uniforms must survive the device initialising those slots.
#[test]
fn every_thread_sees_its_own_uniforms() -> Result<()> {
    let context = pollster::block_on(WgpuContext::new())?;
    let kernel = KernelCache::new().get(&context, FILL)?;
    let threads = 32u32;
    let barrier = std::sync::Barrier::new(threads as usize);
    let wrong = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for thread in 0..threads {
            let (context, kernel, barrier, wrong) = (&context, &kernel, &barrier, &wrong);
            scope.spawn(move || {
                for round in 0..8u32 {
                    let count = 10_000 + thread * 13;
                    let value = thread * 1_000 + round + 1;
                    let output = Buffer::new(
                        context,
                        count as u64 * 4,
                        BufferDefinition::storage().with_copy_src(),
                    )
                    .unwrap();
                    let input: Vec<u32> = (0..20_000u32).map(|i| i ^ thread).collect();
                    let input =
                        Buffer::from_slice(context, &input, BufferDefinition::storage()).unwrap();
                    let mut parameters = PassParameters::new();
                    parameters.insert("output", output.clone());
                    parameters.insert("input", input);
                    parameters.insert("count", count);
                    parameters.insert("value", value);
                    parameters.insert("pad0", 0u32);
                    parameters.insert("pad1", 0u32);
                    barrier.wait();
                    let mut batch = KernelBatch::new(context);
                    batch.dispatch_items(kernel, &parameters, count).unwrap();
                    batch.submit();
                    let read: Vec<u32> = pollster::block_on(output.read(context)).unwrap();
                    let bad = read.iter().filter(|v| **v != value).count();
                    wrong.fetch_add(bad, std::sync::atomic::Ordering::Relaxed);
                }
            });
        }
    });
    assert_eq!(
        wrong.into_inner(),
        0,
        "outputs written with another dispatch's uniforms"
    );
    Ok(())
}

/// A batch that stays open while other threads dispatch the same kernel
/// thousands of times keeps the uniforms it recorded.
#[test]
fn a_long_open_batch_keeps_its_uniforms() -> Result<()> {
    let context = pollster::block_on(WgpuContext::new())?;
    let kernel = KernelCache::new().get(&context, FILL)?;
    let input = Buffer::from_slice(&context, &[1u32], BufferDefinition::storage())?;
    let parameters = |output: &Buffer, value: u32| {
        let mut parameters = PassParameters::new();
        parameters.insert("output", output.clone());
        parameters.insert("input", input.clone());
        parameters.insert("count", 64u32);
        parameters.insert("value", value);
        parameters.insert("pad0", 0u32);
        parameters.insert("pad1", 0u32);
        parameters
    };
    let storage = BufferDefinition::storage().with_copy_src();
    let held = Buffer::new(&context, 256, storage.clone())?;
    let mut open = KernelBatch::new(&context);
    open.dispatch_items(&kernel, &parameters(&held, 7), 64)?;
    std::thread::scope(|scope| {
        scope.spawn(|| {
            let other = Buffer::new(&context, 256, storage.clone()).unwrap();
            for round in 0..3_000u32 {
                let mut batch = KernelBatch::new(&context);
                batch
                    .dispatch_items(&kernel, &parameters(&other, round), 64)
                    .unwrap();
                batch.submit();
            }
        });
    });
    open.submit();
    let read: Vec<u32> = pollster::block_on(held.read(&context))?;
    assert!(read[..64].iter().all(|v| *v == 7), "{:?}", &read[..4]);
    Ok(())
}
