//! Native prefix-sum programs and their uniform workgroup bounds guard.

use super::BLOCK;
use crate::kernel::FOLDED_INDEX_WGSL;

pub(super) fn scan_blocks_code() -> String {
    format!(
        r#"{FOLDED_INDEX_WGSL}
@group(0) @binding(0) var<storage, read_write> data: array<u32>;
@group(0) @binding(1) var<storage, read_write> sums: array<u32>;

struct Params {{
    count: u32,
    write_sums: u32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(2) var<uniform> params: Params;

var<workgroup> shared_values: array<u32, {BLOCK}u>;

@compute @workgroup_size({BLOCK}u)
fn main(
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>,
    @builtin(num_workgroups) groups: vec3<u32>,
) {{
    {block_index}

    var value = 0u;
    if (index < params.count) {{
        value = data[index];
    }}
    shared_values[local_id.x] = value;
    workgroupBarrier();

    // Hillis-Steele, inclusive. Every lane reads before any lane writes, so
    // the barrier between them is all the double-buffering it needs.
    for (var offset = 1u; offset < {BLOCK}u; offset = offset * 2u) {{
        var carried = 0u;
        if (local_id.x >= offset) {{
            carried = shared_values[local_id.x - offset];
        }}
        workgroupBarrier();
        shared_values[local_id.x] = shared_values[local_id.x] + carried;
        workgroupBarrier();
    }}

    let inclusive = shared_values[local_id.x];
    if (index < params.count) {{
        data[index] = inclusive - value;
    }}
    if (local_id.x == {BLOCK}u - 1u && params.write_sums != 0u) {{
        sums[block] = inclusive;
    }}
}}
"#,
        block_index = block_index_code(),
    )
}

pub(super) fn add_offsets_code() -> String {
    format!(
        r#"{FOLDED_INDEX_WGSL}
@group(0) @binding(0) var<storage, read_write> data: array<u32>;
@group(0) @binding(1) var<storage, read> sums: array<u32>;

struct Params {{
    count: u32,
    write_sums: u32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size({BLOCK}u)
fn main(
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>,
    @builtin(num_workgroups) groups: vec3<u32>,
) {{
    {block_index}
    if (index < params.count) {{
        data[index] = data[index] + sums[block];
    }}
}}
"#,
        block_index = block_index_code(),
    )
}

fn block_index_code() -> String {
    format!(
        r#"// Group ids, not invocations: each valid group owns one scan block.
    let block = folded_index(group_id, groups, 1u);
    let block_count = params.count / {BLOCK}u
        + select(0u, 1u, params.count % {BLOCK}u != 0u);
    // This workgroup-uniform guard precedes address products and barriers.
    if (block >= block_count) {{ return; }}
    let index = block * {BLOCK}u + local_id.x;"#
    )
}

