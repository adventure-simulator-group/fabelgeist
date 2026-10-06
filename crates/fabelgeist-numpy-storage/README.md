# NumPy decoded values

`NpyFloatValues`, `NpyIntegerValues` and `NpyByteStates` retain ordered decoded
NumPy payload values in bespoke types. The first two contain storage numbers at
f32 and i64 precision; the third classifies each individual payload byte as zero
or nonzero, independently of element width.

Construct a collection with `From<&NpyArray>` and inspect its typed slice.
Convert explicitly to a native vector at a tensor-data or other external
representation boundary. Decoding retains stored dtype semantics:
float64-to-integer conversion occurs before any float32 projection,
floating-to-integer casts saturate as Rust defines, float32 NaN bits and signed
zero are preserved, and numeric byte values are not normalized to booleans.

Native fixed-width word iterators preserve order and exclude an incomplete final
word, as the previous decoder did. Each iterator supplies its exact allocation
count to standard collection operations. Array shape, dtype admission, archive
lookup and parser error handling retain their existing contracts; the
decoded-value collection does not validate an array's metadata or classify a
number as a mesh index or activation coefficient.

The character importer retains decoded values through sparse lookup and converts
only at native indexing and tensor-buffer assignment. Sparse transposition,
duplicate replacement, narrowing before bounds checks and diagnostic order are
unchanged. Native tensor upload may use a backend's integer representation, such
as i32, separately from the decoded i64 storage number.
