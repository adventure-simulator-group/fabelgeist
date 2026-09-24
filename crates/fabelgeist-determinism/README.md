# Deterministic draws

`Seed::derive` length-frames the root, purpose name, and each context field
with an unsigned 64-bit little-endian byte length. Numeric context fields
use explicit fixed-width little-endian encoding. BLAKE3's derive-key context
is `fabelgeist.determinism.seed.v1`; the first eight digest bytes initialize
`rand_xoshiro::SplitMix64::from_seed` directly. No warm-up draw is discarded.

`StreamId` names belong to their domain owners. Entity identity, semantic
event ordinal, and spatial coordinates are context, not numeric stream salts.
Stable ordinals describe authored slots or events, never incidental iteration
positions. Unordered candidates must be sorted by stable domain identity before
sampling. Authored order is meaningful and remains intact.

Each stream owns its draw order. Drawing more values from another purpose or
visiting unrelated entities in another order cannot perturb it. Changing the
eligible candidate set can change selection; stream separation does not make
selection independent of its inputs.

## Dependency audit

The pinned `rand_xoshiro` 0.7.0 implementation reads its eight seed bytes as a
little-endian `u64`. Its `next_u64` implements the reference SplitMix64 step.
Its separate `next_u32` mixer is deliberately not part of this contract.

Rand 0.9.5 `Uniform<u64>::new(...).sample(...)` uses a `u64` word, a `u128`
widening product, and rejection sampling. It does not sample a native pointer
width. `WeightedIndex<u64>` checks the cumulative integer sum for overflow,
samples this same uniform distribution, and finds the cumulative interval.
Its returned `usize` is an index, not an entropy source. Re-audit these source
paths on dependency changes; golden vectors detect changes in draws consumed
as well as selected values.

The shared descending Fisher–Yates shuffle calls the same bounded `u64`
sampler for every swap. Rand's slice helpers, `Uniform<usize>`, and inferred
integer ranges are not substitutes for this contract.

## Floating-point boundary

Unit conversions keep 24 bits for `f32` and 53 for `f64`, matching their
significand precision. Dividing by two to that power gives `[0, 1)`; dividing
by one less gives `[0, 1]`. The shifts discard only the unused low bits.
These are representation constants, not gameplay tuning.

For identical canonical inputs and draw sequences, native and wasm32 agree
on derived seeds, raw words, booleans, bounded integers, integer-weighted
choices, shuffles, and unit-float bit patterns. Scaled floats and downstream
terrain interpolation, physics, and rendering have no bitwise guarantee.
Shader-only noise remains a separately owned rendering algorithm.

Run `just test-determinism` with Node, the wasm32 Rust target, and
wasm-bindgen-cli 0.2.108 installed. No browser bundle is added to the product.

Before migrating a caller, classify its calculation as sampling, seed
derivation, spatial noise, or a domain transformation. Preserve spatial and
domain semantics rather than replacing similarly shaped arithmetic blindly.

## Identity boundary

Settlement businesses use stable local `(usage, ordinal)` coordinates from the
canonical demand plan. Population planning finalizes household demographics
before dependent character facts, then tactical scenes carry the globally
scoped business and resident operator identities. Stream isolation supports
that ordering but does not replace those explicit relationships.
