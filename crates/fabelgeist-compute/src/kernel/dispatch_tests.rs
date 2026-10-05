use super::*;
use fabelgeist_gpu::prelude::{BufferDefinition, BufferUpload, WorkgroupShapeError};

const DOUBLE: &str = r#"
@group(0) @binding(0) var<storage, read_write> values: array<u32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x < arrayLength(&values)) { values[id.x] *= 2u; }
}
"#;

#[derive(Clone, Copy)]
enum RecordingPath {
    Cached,
    General,
}

#[tokio::test]
async fn recording_counts_only_successful_nonempty_dispatches_in_both_paths() -> Result<()> {
    let context = WgpuContext::new().await?;
    for path in [RecordingPath::Cached, RecordingPath::General] {
        let mut kernel = Kernel::new(&context, DOUBLE)?;
        assert!(kernel.fast.is_some());
        if matches!(path, RecordingPath::General) {
            kernel.fast = None;
        }
        let mut batch = KernelBatch::new(&context);
        let missing = PassParameters::new();
        for axes in [[0, 1, 1], [1, 0, 1], [1, 1, 0], [0, 0, 0]] {
            batch.dispatch(&kernel, &missing, WorkgroupGrid::from(axes))?;
        }
        batch.dispatch_items(&kernel, &missing, InvocationCount::from(0))?;
        assert_eq!(batch.dispatch_count(), RecordedDispatchCount::default());
        assert!(
            batch
                .dispatch(&kernel, &missing, WorkgroupGrid::from([1, 1, 1]))
                .is_err()
        );
        assert_eq!(batch.dispatch_count(), RecordedDispatchCount::default());
        assert!(
            kernel
                .run(
                    &context,
                    PassParameters::new(),
                    WorkgroupGrid::from([0, 1, 1])
                )
                .is_err()
        );

        let values = Buffer::new(&context, 12u64.into(), BufferDefinition::storage())?;
        let copied = Buffer::new(&context, 12u64.into(), BufferDefinition::storage())?;
        values.write(&context, BufferUpload::from_elements(&[1u32, 2, 3]))?;
        let mut parameters = PassParameters::new();
        parameters.insert("values", PassParameter::Buffer(values.clone()));
        batch.dispatch_items(&kernel, &parameters, InvocationCount::from(3))?;
        assert_eq!(batch.dispatch_count().to_string(), "1");
        batch.copy_buffer(&values, &copied, 12)?;
        batch.clear_buffer(&copied);
        assert_eq!(batch.dispatch_count().to_string(), "1");
        batch.dispatch(&kernel, &parameters, WorkgroupGrid::from([1, 1, 1]))?;
        assert_eq!(batch.dispatch_count().to_string(), "2");
        batch.submit();
        assert_eq!(values.read::<u32>(&context).await?, [4, 8, 12]);
        assert_eq!(copied.read::<u32>(&context).await?, [0, 0, 0]);
    }
    Ok(())
}

#[test]
fn declaration_error_retains_native_context_and_typed_cause() {
    let source = WorkgroupShape::try_from([64, 0, 1]).unwrap_err();
    let error: anyhow::Error = WorkgroupDeclarationError::at_entry_point("main", source).into();
    assert!(error.is::<WorkgroupDeclarationError>());
    assert!(
        error
            .chain()
            .any(|cause| cause.downcast_ref::<WorkgroupShapeError>().is_some())
    );
    assert_eq!(
        error.to_string(),
        "Kernel `main`: workgroup size [64, 0, 1] has a zero dimension"
    );
}
