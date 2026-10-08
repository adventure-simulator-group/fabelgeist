# fabelgeist-numpy-storage

NumPy `.npy` and `.npz` reading for Burn, in pure Rust — no NumPy, no C zlib.

```rust
use burn::tensor::Device;
use fabelgeist_numpy_storage::Npz;

let archive = Npz::open("weights.npz")?;
for name in archive.keys() {
    println!("{name}: {:?}", archive.array(name)?.shape());
}
let weights = archive.array("layer0")?.to_tensor::<2>(&Device::default())?;
```

- Archives are memory-mapped; members may be stored or deflated, and zip64 is
  handled (NumPy switches to it past 2 GB).
- `uncompressed_size` reports a member's decoded size without decoding it.
- Element types: `f4`, `f8`, `i4`, `i8`, `u1`, `b1`.
  Values are converted on
  access with `to_f32`, `to_i64` or `to_bool`, or uploaded straight to a device
  with `to_tensor` / `to_int_tensor`, so a `.npz` written as `float64` still
  loads into an f32 tensor.
- Fortran-ordered arrays are rejected rather than silently transposed.

Used by `fabelgeist-mhr` for the MHR pose-corrective tensors.

Decode a serialized array with `NpyArray::from_bytes`, or read a standalone
file with `NpyArray::read`.

## Array metadata

The decoder checks the shape, element type, and payload length. A decoded
`NpyArray` keeps that metadata immutable. Use `shape`, `dtype`, `element_count`,
and `occupancy` to inspect it. Public mutation
cannot reinterpret one decoded element as several differently sized values or
replace the dimensions behind its checked element count.

`NpyShape` owns a sequence of `NpyDimension` values and admits their product.
`NpyRank` counts axes; `NpyElementCount` counts stored elements. A scalar has
rank
zero and one element. Any zero axis makes an array empty before other dimensions
can overflow. `NpyElementWidth` describes the supported one-, four-, and
eight-byte storage words. `NpyPayloadLength` owns the checked serialized payload
length separately from element cardinality.

`NpyLayoutError` retains the failed dimensions, element count and width,
expected
and available payload lengths, or expected/actual ranks and the native rank
conversion cause. Constructing a tensor with the wrong rank returns this
concrete error.
Existing file, header, and archive error paths are separate migration work.

Native representations stay at explicit adapters: header dimension admission,
little-endian word decoding, payload slicing, the immutable `bytes` encoding
view, and Burn tensor construction. MHR corrective-basis comparisons retain
`NpyDimension` until the direct tensor adapter. Shape queries expose nominal
dimensions, not mutable integer vectors. Numeric value collections and general
storage/transport types are separate bespoke type families.

Supported dtype spellings, header admission order, scalar/empty meanings,
stored value casts, row order, and tensor dimensions are preserved. Oversized
shape and payload products return layout errors rather than panicking or
wrapping. The focused tests cover overflow context, rank causes, zero axes,
scalar shape, header failure priority, stored-width decoding, and compile-time
metadata immutability.
