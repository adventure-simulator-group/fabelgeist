# fabelgeist-numpy-storage

NumPy `.npy` and `.npz` reading for Burn, in pure Rust — no NumPy, no C zlib.

```rust
use burn::tensor::Device;
use fabelgeist_numpy_storage::{Npz, NpzArrayName};

let archive = Npz::open("weights.npz")?;
for name in archive.keys() {
    println!("{name}: {:?}", archive.array(&name)?.shape());
}
let weights = archive.array(&NpzArrayName::new("layer0"))?.to_tensor::<2>(&Device::default())?;
```

- Archives are memory-mapped; members may be stored or deflated, and zip64 is
  handled (NumPy switches to it past 2 GB).
- `uncompressed_size` returns a bespoke `DeclaredUncompressedByteLength` from
  archive metadata. It does not decode or verify the member's actual length.
- Element types: `f4`, `f8`, `i4`, `i8`, `u1`, `b1`.
  Decoded numbers use `NpyFloatValues` and `NpyIntegerValues`; `NpyByteStates`
  classifies every payload byte as zero or nonzero. Tensor upload uses
  `to_tensor` / `to_int_tensor`, so a `.npz` written as `float64` still loads
  into an f32 tensor.
- Fortran-ordered arrays are rejected rather than silently transposed.

Used by `fabelgeist-mhr` for the MHR pose-corrective tensors.

## ZIP directory cardinality

The reader retains the declared member count through selection and iteration.
See the [count contract](member-count.md) for native ports and format limits.

## ZIP byte quantities

Compressed lengths, declared decoded lengths and archive-origin byte positions
have distinct bespoke types. See the [extent contract](zip-extents.md) for the
public size API and unchanged native addressing/allocation policy.

## Archive compression

The reader retains a bespoke compression choice from central-directory admission
through member decoding. Stored members borrow the original bytes; Deflate, a
ZIP
compression format, produces owned decoded output. Unsupported method codes
remain available as metadata for name and size queries, then fail when bytes are
requested. Local-header corruption and payload bounds are checked before that
unsupported-method error, preserving diagnostic precedence.

The central directory selects the method; a different method in the local header
does not change that policy. The two-byte wire code is admitted at its decoding
boundary and retained as a bespoke diagnostic code for unsupported methods.
Checksums, protection flags, counts, offsets, member identity and wider archive
errors remain separate concepts. This contract preserves their existing
behavior.

PKWARE APPNOTE section 4.4.5 defines the two-byte compression method field: code
0
stores bytes without compression, and code 8 uses Deflate. The reader classifies
these codes at central-directory admission and retains every other `u16` as an
unsupported code for the original decimal diagnostic. `ZipCompressionCode`
derives decimal display without changing that message.

[PKWARE ZIP format specification, section 4.4.5](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT)

## ZIP record kinds

The reader retains named record kinds after native signature decoding. See the
[record-kind contract](record-kinds.md) for interpretation and framing limits.

Decode a serialized array with `NpyArray::from_bytes`, or read a standalone
file with `NpyArray::read`.

## Array metadata

The decoder checks the shape, element type, and payload length. A decoded
`NpyArray` keeps that metadata immutable. Use `shape`, `dtype`, `element_count`,
and `occupancy` to inspect it. Public mutation cannot reinterpret one decoded
element as several differently sized values or
replace the dimensions behind its checked element count.

`NpyShape` owns a sequence of `NpyDimension` values and admits their product.
`NpyRank` counts axes; `NpyElementCount` counts stored elements. A scalar has
rank zero and one element. Any zero axis makes an array empty before other
dimensions
can overflow. `NpyElementWidth` describes the supported one-, four-, and
eight-byte storage words. `NpyPayloadLength` owns the checked serialized payload
length separately from element cardinality.

`NpyLayoutError` retains the failed dimensions, element count and width,
expected and available payload lengths, or expected/actual ranks and the native
rank
conversion cause. Constructing a tensor with the wrong rank returns this
concrete error.
Serialized array admission returns `NpyDecodeError`; standalone file reads
return `NpyReadError`. Their ordinary and alternate diagnostics preserve the
native cause and path context. Archive errors remain separate migration work.

Native representations stay at explicit adapters: header dimension admission,
little-endian word decoding, payload slicing, the immutable `bytes` encoding
view, and Burn tensor construction. MHR corrective-basis comparisons retain
`NpyDimension` until the direct tensor adapter. Shape queries expose nominal
dimensions, not mutable integer vectors. Numeric value collections retain
decoded storage numbers; general
storage/transport types remain separate bespoke type families.

Supported dtype spellings, header admission order, scalar/empty meanings,
stored value casts, row order, and tensor dimensions are preserved. Oversized
shape and payload products return layout errors rather than panicking or
wrapping. The focused tests cover overflow context, rank causes, zero axes,
scalar shape, header failure priority, stored-width decoding, and compile-time
metadata immutability.

## Decoded values

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

## Archive filenames and array keys

`ArchiveMemberName` retains an exact parsed archive filename. Serialized names
use lossy UTF-8 decoding: two invalid byte spellings can identify the same
parsed text. Case, separators, empty text, NUL and path-like spelling remain
unchanged; the name validates neither a filesystem path nor membership.

`NpzArrayName` carries a NumPy array query. A member matches its full filename
or
that filename with one trailing `.npy` removed. The first directory match wins,
even if a later member is an exact filename match. `keys` returns typed keys in
member order, removing one suffix without changing the stored filename.

`ZipArchive::bytes` and size queries take `ArchiveMemberName`; `Npz::array` and
its size query take `NpzArrayName`. Both `contains` methods return the shared
`ArchiveMemberPresence::Present` or `Absent` state. Text and Boolean conversions
are explicit presentation and native-boundary adapters. The distinct types
prevent an array key from silently becoming an exact archive lookup.

## Array decoding and file-read errors

`NpyArray::from_bytes` returns `DecodeResult<NpyArray>`;
`NpyArray::read` returns `ReadResult<NpyArray>`. Header rejections carry
`NpyDecodeError`, while file access or decoding failures retain the native path
in `NpyReadError`. A decoder layout rejection carries the existing
`NpyLayoutError` in `NpyDecodeError::Layout`; tensor rank errors still return
`NpyLayoutError` directly.

See the [error contract](npy-errors.md) for the native diagnostic boundaries,
read order, short-header guard and remaining format limits.
