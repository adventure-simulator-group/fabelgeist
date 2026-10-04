# fabelgeist-numpy-storage

NumPy `.npy` and `.npz` reading for Burn, in pure Rust — no NumPy, no C zlib.

```rust
use burn::tensor::Device;
use fabelgeist_fs::NativeFile;
use fabelgeist_numpy_storage::{Npz, NpzArrayName};

let file = NativeFile::from(std::path::PathBuf::from("weights.npz"));
let archive = Npz::open(&file)?;
for name in archive.keys() {
    println!("{name}: {:?}", archive.array(&name)?.shape());
}
let weights = archive
    .array(&NpzArrayName::from("layer0"))?
    .to_tensor::<2>(&Device::default())?;
```

- Native archives are memory-mapped. Owned archives admit `FileContents` through
  `Npz::from_bytes`. Stored and deflated members and single-disk ZIP64
  directories are supported. Encrypted and unsupported compression methods are
  rejected.
- `uncompressed_size` reports a member's decoded size without decoding it.
  It returns `StorageByteLength`; missing members return `None`.
- Element types: `f4`, `f8`, `i4`, `i8`, `u1`, `b1`. `floating_values` and
  `integer_values` retain ordered nominal numbers, or arrays upload directly
  with `to_tensor` / `to_int_tensor`. A `.npz` written as `float64` still loads
  into an f32 tensor.
- Fortran-ordered arrays are rejected rather than silently transposed.

Used by `fabelgeist-mhr` for the MHR pose-corrective tensors.

## Admission and errors

`ArchiveMemberName` preserves the exact serialized filename bytes, including
non-UTF-8 names. Its display spelling is diagnostic only. `NpzArrayName` accepts
the exact member spelling with or without its `.npy` suffix; lookup keeps the
original member order. `ArchiveMemberPresence` expresses presence separately
from a decoded array or a reported byte length.

Storage views, byte positions, lengths, and spans are distinct types owned by
the dependency-free `fabelgeist_storage` crate, shared with FBX framing. Span
arithmetic and slicing reject overflow and truncation before accessing bytes.
An invalid central directory fails the whole archive admission. Member reads
check the local record, compression, decoded length, and CRC before exposing
contents. Inflate output is limited to the declared length plus one byte so
an oversized decoded member is detected without reading its complete output.

`ZipReadError` retains the native file and operation, directory ordinal, or
exact member name together with its concrete cause. `NpzArrayError`
distinguishes lookup, member-read, and NumPy-decode failures. `NpyReadError`
retains the file and provider or decoder cause. `NpyDecodeError` classifies
version, section, encoding, shape, layout, payload, and tensor-rank failures.
Format these errors at the presentation boundary; internal consumers should
retain their variants and causes.

NumPy versions 1.0, 2.0, and 3.0 use their respective header-length layouts.
Malformed headers and overflowing shapes return structured failures. Empty
arrays retain zero elements even when other axes would overflow their product.
The supported header dictionary and dtype subset remains limited; object,
structured, complex, big-endian, and Fortran-ordered data are unsupported.

`NpyShape` admits nominal `NpyDimension` values and checks their product before
exposing `NpyElementCount`. Rank uses `NpyRank`; `NpyOccupancy` distinguishes
empty arrays from nonempty ones, including rank-zero scalars. `NpyElementWidth`
separates byte, 32-bit, and 64-bit storage words, and the element-count owner
checks payload-size multiplication. A zero axis wins over otherwise overflowing
axes. Tensor upload checks rank before converting dimensions for the device SDK.

An admitted array exposes immutable metadata. Callers cannot replace its shape,
dtype, or payload and invalidate the layout that admission checked. Corrective
loading retains the nominal shape in diagnostics and converts dimensions only
when constructing its own admitted basis dimensions. Header error order remains
dtype, storage order, shape, then payload length; supported spellings and
element order are unchanged.

`NpyFloatValue` and `NpyIntegerValue` are decoded storage numbers, whose game or
model meaning is assigned by the consuming domain. Their ordered collections
retain nominal elements and counts. `NpyElementOrdinal` distinguishes a value's
position from an extent or count; offset and repeated-count arithmetic reject
overflow. Numeric casts preserve the original stored type: integer conversion
of float64 does not first round through float32. Float32 signed zero and NaN
payloads, saturating float-to-integer casts, and raw numeric values of accepted
boolean bytes are preserved.

`byte_states` returns `NpyByteStates` with `Zero` or `Nonzero` for each stored
payload byte. This retains the old byte-wise classification explicitly; it does
not collapse a multibyte element to one boolean. Its length uses storage-byte
units, separate from logical element counts. The old primitive-vector methods
and raw chunk callback helper are removed. Native scalar/vector conversions
belong at codec, tensor, native-index or presentation SDK boundaries; internal
consumers retain nominal values and assign their own semantic roles.

These contracts follow the [NumPy format specification](https://numpy.org/doc/stable/reference/generated/numpy.lib.format.html)
and [PKWARE ZIP specification](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT).
Previously tolerated corrupt directories, checksums, and inconsistent member
sizes now fail admission. Callers must use the final nominal API directly;
there are no legacy string/path overloads or compatibility readers.

The shared test encoder takes nominal dtype choices, shape dimensions or
explicit malformed-shape states, and `FileContents`. It preserves authored
header padding and payload bytes. A header too long for version 1 retains its
length and concrete conversion cause instead of truncating the length field.
Native codec values stay at direct byte-serialization calls; fixture helpers
do not accept primitive vectors or arbitrary header strings. Mathematical
consumers outside this crate remain recorded semantic migration debt.
