//! 30-bit Morton codes: three 10-bit coordinates interleaved.
//!
//! Sorting primitives by the Morton code of their centroid puts spatial
//! neighbours next to each other in memory, which is the whole basis of the
//! linear BVH build in [`crate::gpu`]. The CPU version here and the WGSL in
//! `crate::gpu::shaders` must agree bit for bit -- a test pins that down.

/// Bits per axis. Three of these fit in the 30 bits a radix sort can cover in
/// four passes.
pub const BITS_PER_AXIS: u32 = 10;
/// Total significant bits in a code.
pub const BITS: u32 = BITS_PER_AXIS * 3;

/// Spread the low 10 bits of `value` out to every third bit.
pub fn expand_bits(value: u32) -> u32 {
    let mut v = value & 0x3FF;
    v = (v | (v << 16)) & 0x030000FF;
    v = (v | (v << 8)) & 0x0300F00F;
    v = (v | (v << 4)) & 0x030C30C3;
    v = (v | (v << 2)) & 0x09249249;
    v
}

/// Morton code of a point whose coordinates are already normalised to `[0, 1]`.
pub fn encode_unit(x: f32, y: f32, z: f32) -> u32 {
    let quantize = |value: f32| {
        // `1024` rather than `1023` matches the usual formulation; the clamp
        // is what keeps a coordinate of exactly 1.0 inside 10 bits.
        (value * 1024.0).clamp(0.0, 1023.0) as u32
    };
    (expand_bits(quantize(x)) << 2) | (expand_bits(quantize(y)) << 1) | expand_bits(quantize(z))
}

/// Length of the common prefix of two codes, used by the Karras hierarchy
/// build to decide where a node's range splits.
///
/// Equal codes are broken by index, as Karras prescribes: without it, a mesh
/// with duplicate centroids -- and a garment panel grid has plenty -- builds a
/// degenerate tree.
pub fn common_prefix(codes: &[u32], i: usize, j: isize) -> i32 {
    if j < 0 || j as usize >= codes.len() {
        return -1;
    }
    let j = j as usize;
    if codes[i] == codes[j] {
        // Tie-break on the indices, offset past the 32 code bits so that any
        // shared code always beats any differing one.
        return 32 + (i as u32 ^ j as u32).leading_zeros() as i32;
    }
    (codes[i] ^ codes[j]).leading_zeros() as i32
}
