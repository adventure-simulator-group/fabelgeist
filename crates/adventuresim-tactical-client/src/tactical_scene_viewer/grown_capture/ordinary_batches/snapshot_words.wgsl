// Untimed diagnostic: copy exact bound storage words, never engine writes.
override COPY_GROUP_WORDS: u32;
@group(0) @binding(0) var<storage, read> source_words: array<u32>;
@group(0) @binding(1) var<storage, read_write> copied_words: array<u32>;

@compute @workgroup_size(COPY_GROUP_WORDS)
fn copy_words(@builtin(global_invocation_id) invocation: vec3<u32>) {
    if invocation.x < arrayLength(&source_words) {
        copied_words[invocation.x] = source_words[invocation.x];
    }
}
