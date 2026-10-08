//! Entry selection stays consistent through reflection, pipeline cache and
//! cached versus ordinary native dispatch. Expected values were captured from
//! the implementation before replacing its entry-point strings.
#![cfg(not(target_arch = "wasm32"))]

use fabelgeist_compute::{Kernel, KernelBatch};
use fabelgeist_gpu::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

const BODY: &str = r#"
@group(0) @binding(0) var<storage, read_write> output: array<u32>;
@compute @workgroup_size(64)
fn first_pick(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x < arrayLength(&output)) { output[id.x] = 11u; }
}
@compute @workgroup_size(32)
fn second_pick(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x < arrayLength(&output)) { output[id.x] = 29u; }
}
"#;
const VERTEX: &str =
    "@vertex fn image() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }\n";

#[test]
fn selected_entry_reaches_both_dispatch_paths_and_cache() -> Result<()> {
    let context = pollster::block_on(WgpuContext::new())?;
    for (code, expected) in [
        (BODY.to_owned(), "first_pick"),
        (format!("{VERTEX}{BODY}"), "first_pick"),
        (BODY.replace("first_pick", "zebra_entry"), "zebra_entry"),
    ] {
        let shader = ComputeShader::new(&context, code.clone())?;
        let pipeline = ComputePipeline::new(&context, shader)?;
        let reflection = pipeline.reflection.as_ref().unwrap();
        let entry = &reflection.compute_entry_point;
        assert_eq!(<&str>::from(entry), expected);
        let first = pipeline.get_or_create_pipeline(&context.device, entry)?;
        assert!(Arc::ptr_eq(pipeline.pipeline.as_ref().unwrap(), &first));
        let repeat = pipeline.get_or_create_pipeline(&context.device, entry)?;
        assert!(Arc::ptr_eq(&first, &repeat));
        let other_entry = ShaderEntryPoint::from("second_pick");
        let second = pipeline.get_or_create_pipeline(&context.device, &other_entry)?;
        let again = pipeline.get_or_create_pipeline(&context.device, &other_entry)?;
        assert!(!Arc::ptr_eq(&first, &second));
        assert!(Arc::ptr_eq(&second, &again));
        let mut native_hash = DefaultHasher::new();
        expected.hash(&mut native_hash);
        let cache = pipeline.pipeline_cache.lock().unwrap();
        assert_eq!(cache.len(), 2);
        assert!(Arc::ptr_eq(&cache[&native_hash.finish()], &first));
        drop(cache);

        let kernel = Kernel::new(&context, code)?;
        assert_eq!(kernel.entry_point, *entry);
        assert_eq!(
            kernel
                .pipeline
                .reflection
                .as_ref()
                .unwrap()
                .compute_entry_point,
            *entry
        );
        assert_eq!(kernel.workgroup_size, [64, 1, 1]);
        assert!(kernel.is_cached());
        assert_eq!(
            format!("{kernel:?}"),
            format!(
                "Kernel {{ entry_point: {expected:?}, workgroup_size: [64, 1, 1], cached: true }}"
            )
        );
        let output = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&[0u32]),
            BufferDefinition::storage().with_usage(BufferUse::CopySource),
        )?;
        let mut parameters = PassParameters::new();
        parameters.insert("output", output.clone());
        let mut batch = KernelBatch::new(&context);
        batch.dispatch_items(&kernel, &parameters, 1)?;
        batch.submit();
        let fast: Vec<u32> = pollster::block_on(output.read(&context))?;
        assert_eq!(fast, [11]);
        kernel.run(&context, parameters.clone(), [1, 1, 1])?;
        let standard: Vec<u32> = pollster::block_on(output.read(&context))?;
        assert_eq!(standard, [11]);
        let mut alternate = pipeline.clone();
        alternate.pipeline = Some(second);
        ComputePass::dispatch(&context, alternate, parameters, 1, 1, 1)?;
        let read: Vec<u32> = pollster::block_on(output.read(&context))?;
        assert_eq!(read, [29]);
    }
    Ok(())
}

#[test]
fn entry_identity_keeps_native_validation_and_early_diagnostics() -> Result<()> {
    let context = pollster::block_on(WgpuContext::new())?;
    // Label admission preserves spelling without preempting module validation.
    for label in ["absent", "", "not a parsed identifier\n"] {
        let entry = ShaderEntryPoint::from(label.to_owned());
        assert_eq!(<&str>::from(&entry), label);
        assert_eq!(format!("{entry}"), label);
        assert_eq!(format!("{entry:?}"), format!("{label:?}"));
        let error = ComputePipeline::default()
            .get_or_create_pipeline_validated(&context.device, &entry, false)
            .unwrap_err();
        assert_eq!(error.to_string(), "ComputePipeline: Shader Module missing");
    }
    let shader = ComputeShader {
        code: VERTEX.to_owned(),
        ..Default::default()
    };
    assert_eq!(
        ComputePipeline::new(&context, shader.clone())
            .unwrap_err()
            .to_string(),
        "Compute Shader missing entry point"
    );
    assert_eq!(
        ComputePipeline {
            shader,
            ..Default::default()
        }
        .validate_interface()
        .unwrap_err()
        .to_string(),
        "Compute Shader missing entry point"
    );
    assert_eq!(
        Kernel::new(&context, VERTEX).unwrap_err().to_string(),
        "Kernel: no compute entry point"
    );
    Ok(())
}
