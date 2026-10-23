//! Native inclusive-scan programs over an authored storage-array value.

use super::SCAN_WORKGROUP_WIDTH;
use fabelgeist_gpu::data::gpu::signature::ResourceBaseType;

/// Scan each workgroup and emit one total.
pub(super) fn scan_blocks_code(operator: &str, value_type: &ResourceBaseType) -> String {
    let ty = value_type.as_str();
    let block_size = SCAN_WORKGROUP_WIDTH;
    format!(
        r#"@group(0) @binding(0) var<storage, read> input: array<{ty}>;
@group(0) @binding(1) var<storage, read_write> output: array<{ty}>;
@group(0) @binding(2) var<storage, read_write> aux: array<{ty}>;

var<workgroup> shared_data: array<{ty}, {block_size}>;

{code}

@compute @workgroup_size({block_size})
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>
) {{
    let input_len = arrayLength(&input);
    let index = global_id.x;

    // Load into shared memory
    if (index < input_len) {{
        shared_data[local_id.x] = input[index];
    }}
    workgroupBarrier();

    // Hillis-Steele inclusive scan within workgroup
    for (var offset = 1u; offset < {block_size}u; offset *= 2u) {{
        var t: {ty};
        if (local_id.x >= offset) {{
            t = scan(shared_data[local_id.x - offset], shared_data[local_id.x]);
        }}
        workgroupBarrier();
        if (local_id.x >= offset) {{
            shared_data[local_id.x] = t;
        }}
        workgroupBarrier();
    }}

    // Write to output and aux
    if (index < input_len) {{
        output[index] = shared_data[local_id.x];
    }}
    if (local_id.x == {block_size}u - 1u && group_id.x < arrayLength(&aux)) {{
        aux[group_id.x] = shared_data[local_id.x];
    }}

    // Handle the case where the last block is partially full
    if (index == input_len - 1u && local_id.x != {block_size}u - 1u && group_id.x < arrayLength(&aux)) {{
        aux[group_id.x] = shared_data[local_id.x];
    }}
}}
"#,
        ty = ty,
        code = operator,
        block_size = block_size
    )
}

/// Add the preceding scanned total to every later workgroup.
pub(super) fn add_offsets_code(operator: &str, value_type: &ResourceBaseType) -> String {
    let ty = value_type.as_str();
    let block_size = SCAN_WORKGROUP_WIDTH;
    format!(
        r#"@group(0) @binding(0) var<storage, read_write> output: array<{ty}>;
@group(0) @binding(1) var<storage, read> aux: array<{ty}>;

{code}

@compute @workgroup_size({block_size})
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>
) {{
    if (group_id.x > 0u && global_id.x < arrayLength(&output)) {{
        let block_sum = aux[group_id.x - 1u];
        output[global_id.x] = scan(block_sum, output[global_id.x]);
    }}
}}
"#,
        ty = ty,
        code = operator,
        block_size = block_size
    )
}
