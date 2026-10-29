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

- File-backed archives use read-only mappings; in-memory archives own their
  input bytes. Members may be stored or deflated, and ZIP64 is handled.
- `NpzMember::uncompressed_size` returns the archive metadata as a bespoke
  `DeclaredUncompressedByteLength`. It does not decode or verify actual length.
- Element types: `f4`, `f8`, `i4`, `i8`, `u1`, `b1`.
  Decoded numbers use `NpyFloat32Values` and `NpyIntegerValues`; `NpyByteStates`
  classifies every payload byte as zero or nonzero. Tensor upload uses
  `to_tensor` / `to_int_tensor`, so a `.npz` written as `float64` still loads
  into an f32 tensor.
- Fortran-ordered arrays are rejected rather than silently transposed.

Used by `fabelgeist-mhr` for the MHR pose-corrective tensors.

## ZIP structure and byte quantities

[`rawzip` 0.5.2](https://docs.rs/rawzip/0.5.2/rawzip/) owns end records,
ZIP64 metadata, central-directory framing and bounded member access. Our adapter
owns the 66,000-byte end-record search window, declared member-count bound,
lookup names and decompression policy. Parser errors reject admission rather
than publishing a partially parsed directory. The [count contract](member-count.md)
describes the retained application bound and structural-policy differences.

The public `DeclaredUncompressedByteLength` remains an unverified archive
declaration. Structural positions and compressed ranges stay inside rawzip's
private entry handles; application APIs do not expose those parser details.
See the [size contract](zip-extents.md).

## Archive compression

The reader classifies rawzip's central-directory method into its supported
Stored or Deflated state, retaining any other method as a bespoke
`ZipCompressionCode` for decimal diagnostics. Unsupported methods remain
queryable metadata and fail when member bytes are requested. Local framing and
compressed bounds are checked before the method error. The central directory
selects the method even when the local header declares a different one.

Stored members borrow the original archive bytes. `flate2` decodes Deflate into
owned bytes. Checksums, encryption and agreement between declared and decoded
sizes are not verified. A declared size is used for the initial Deflate
reservation, retaining the existing native cast and allocation policy; it is
not a resource budget. `rawzip` is an unmodified registry dependency, not
vendored source. There is one structural parser.

Decode a serialized array with `NpyArray::from_bytes`, or read a standalone file
with `NpyArray::read`.

## Array metadata

The decoder checks the shape, element type, and payload length. A decoded
`NpyArray` keeps that metadata immutable. Use `shape`, `dtype`, `element_count`,
and `is_empty` to inspect it. Public mutation cannot reinterpret one decoded
element as several differently sized values or replace the dimensions behind its
checked element count.

`NpyShape` owns a sequence of `NpyDimension` values and admits their product.
`NpyRank` counts axes; `NpyElementCount` counts stored elements. A scalar has
rank zero and one element. Any zero axis makes an array empty before other
dimensions can overflow. `NpyElementWidth` describes the supported one-, four-,
and eight-byte storage words. `NpyPayloadLength` owns the checked serialized
payload length separately from element cardinality.

`NpyLayoutError` retains the failed dimensions, element count and width,
expected and available payload lengths, or expected/actual ranks and the native
rank conversion cause. Tensor layouts also reject dimension orders whose native
framework multiplication would overflow before a later zero axis. Storage shapes
still admit any zero axis as empty. Constructing a tensor with the wrong rank
returns this concrete error. Header and standalone-file errors remain
format-adapter responsibilities. ZIP failures use the concrete classes described
below.

Native representations stay at explicit adapters: header dimension admission,
little-endian word decoding, payload slicing, the immutable `bytes` encoding
view, and Burn tensor construction. MHR corrective-basis admission retains its
mesh vertex count, converting to `NpyDimension` at the numerical layout boundary.
Shape queries expose nominal
dimensions, not mutable integer vectors. Numeric value collections retain
decoded storage numbers; general storage/transport types remain separate bespoke
type families.

The reader supports NPY versions 1.0, 2.0 and 3.0 with checked header lengths.
One fallible `NpyHeader` admission checks framing and the dictionary, then
retains the dtype, shape and exact borrowed payload together. Consumers do not
reconstruct a range against another buffer.
The scalar header dictionary accepts either quote style and any key order.
It requires exactly `descr`, `fortran_order` and a shape tuple; duplicate or
unknown fields, malformed separators and escaped strings are rejected.
Structured descriptors, pickled objects and Fortran ordering are unsupported.
This is a parser for the supported scalar format, not a general Python parser.

Supported scalar and ordering admission precedes shape-product and payload
checks. Any zero axis gives zero elements; this flat reader computes no strides.
Oversized shape and payload products return layout errors. Numeric words are
little-endian; native-endian dtype spellings are admitted only on little-endian
hosts. Trailing payload bytes are ignored. Tensor rank admission precedes value
collection and device upload. Existing observations cover scalar and empty
shapes, overflow context, rank causes, header failure priority, stored-width
decoding and metadata immutability.

## Decoded values

`NpyFloat32Values`, `NpyIntegerValues` and `NpyByteStates` retain ordered
decoded
NumPy payload values in bespoke types. The first two contain storage numbers at
f32 and i64 precision; the third classifies each individual payload byte as zero
or nonzero, independently of element width.

Construct a collection with `From<&NpyArray>` and inspect its typed slice.
Convert explicitly to a native vector at a tensor-data or other external
representation boundary. Decoding retains stored dtype semantics: float64-to-
integer conversion occurs before any float32 projection, floating-to-integer
casts saturate as Rust defines, float32 NaN bits and signed zero are preserved,
and numeric byte values are not normalized to booleans.

Native fixed-width word iterators preserve order and exclude an incomplete final
word, as the previous decoder did. Each iterator supplies its exact allocation
count to standard collection operations. Array shape, dtype admission, archive
lookup and parser error handling retain their existing contracts; the decoded-
value collection does not validate an array's metadata or classify a number as a
mesh index or activation coefficient.

The character importer admits corrective tensor extents together, then validates
all sparse coordinate roles against that layout before allocating the dense
buffer. Sparse contributions retain their checked offsets and a borrow of the
same layout through upload. Decoded values stay typed until native indexing and
tensor-buffer assignment. Sparse
transposition, duplicate replacement and native narrowing before bounds checks
are unchanged. Impossible joint subtraction and overflowing tensor extents now
return concrete corrective-import errors before allocation. Native tensor upload
may use a backend's integer representation, such as i32, separately from the
decoded i64 storage number.

`NpyFloat32Values` explicitly projects stored numbers to float32 precision;
float64 input bytes remain available on the admitted array. Integer decoding
reads the original stored number independently. `NpyTensorLayout` retains fixed
rank, typed dimensions and their checked element product. Conversion from an
admitted `NpyShape` reuses its product; construction from dimension literals is
fallible. Empty shapes additionally admit each native prefix product, so
`[usize::MAX, usize::MAX, 0]` remains valid storage but returns a layout error
before tensor construction. Float32 upload compares its value count with the
admitted layout before handing Burn its native vector. Burn still owns
source-cardinality and zero-dimension interpretation during reshape; the layout
does not certify a particular source tensor. Corrective imports carry rig joint
counts and mesh vertex counts through both native and streamed loading.

## Archive filenames and array keys

`ArchiveMemberName` retains an exact parsed archive filename. Serialized names
use lossy UTF-8 decoding: two invalid byte spellings can identify the same
parsed text. Case, separators, empty text, NUL and path-like spelling remain
unchanged; the name validates neither a filesystem path nor membership.

`NpzArrayName` carries a NumPy array query. A member matches its full filename
or that filename with one trailing `.npy` removed. The first directory match
wins, even if a later member is an exact filename match. `keys` returns typed
keys in member order, removing one suffix without changing the stored filename.

`ZipArchive::bytes` and size queries take `ArchiveMemberName`; `Npz::get` and
`Npz::array` take `NpzArrayName`. The distinct types prevent an array key from
silently becoming an exact archive lookup. ZIP `contains` is an ordinary
predicate.

`Npz::get` returns `Option<NpzMember<'_>>`. The handle borrows the selected
member and its owning archive, and provides its full `name`, declared
`uncompressed_size` and `array` decoder without another lookup. It retains the
original query spelling for parsing diagnostics. An absent optional array is
`None`; a present member with malformed bytes returns a decoding error.
`Npz::array` requires a match and reports absence as a lookup error.

## ZIP reader errors

ZIP archive and member operations return concrete errors. See the [error
contract](zip-reader-errors.md) for classifications and source chains.
