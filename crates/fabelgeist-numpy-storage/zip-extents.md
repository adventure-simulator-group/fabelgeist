# ZIP byte quantities

ZIP members have a compressed payload length, a declared decoded length and a
local-header position. These values used to share `u64`, so a compressed span
could be confused with a decoded reservation or archive position. The reader
now retains distinct bespoke types from ordinary and ZIP64 wire admission to
its directory, entry storage and native consumers.

`CompressedByteLength` describes bytes before decompression.
`DeclaredUncompressedByteLength` describes the archive's declared decoded size,
including an NPY header for NumPy array members. `ArchiveByteOffset` measures
from the archive's first byte, whose origin is zero. Local headers, the central
directory and ZIP64 locator records use the same frame. All three retain zero
and the complete `u64` width without adding validation.

## Public size queries

Both `ZipArchive::uncompressed_size` and `Npz::uncompressed_size` return
`Option<DeclaredUncompressedByteLength>`. NPZ queries retain optional `.npy`
suffix matching and first-member selection. A declaration does not prove that
a member has decoded to that length: a stored or deflated member may decode
successfully even when the declaration disagrees.

```rust
use fabelgeist_numpy_storage::{DeclaredUncompressedByteLength, Npz};

fn declared_size(archive: &Npz) -> Option<DeclaredUncompressedByteLength> {
    archive.uncompressed_size("weights")
}
```

At an external byte-count boundary, `length.bytes()` projects the full declared
quantity into `u64`. Literal and decoding boundaries admit quantities through
`from_bytes`; positions use `from_archive_start`. Domain APIs carry the bespoke
types directly. Compressed lengths and archive positions cannot be assigned to
a declared decoded length. Display and Debug delegate to the native integer,
including its formatter options; no new serialization schema is introduced.

## Selection and native ports

ZIP64 replacement still visits saturated fields in uncompressed, compressed,
then local-header-offset order. An unsaturated field consumes no replacement.
Partial extra fields retain unreplaced values, malformed extents stop the extra
scan, and the first ZIP64 extra remains authoritative. Directory count and
offset selection retain the existing locator/signature/bounds conditions.
The canonical private `ArchiveMemberCount` still owns reservation and iteration.

Native slice addressing and decoded-vector reservation retain the existing
`usize` casts. Advancing directory positions retains unchecked `u64` addition;
local span arithmetic, framing panics, error order and allocation policy are
unchanged. These types do not certify bounds or make untrusted archives safe.
Checked conversions and new overflow/resource policies need a separate behavior
decision. NPY array payload sizes and GPU buffer lengths are separate meanings.

Stored members continue to borrow archive bytes; deflated members return owned
bytes. Duplicate names retain first-match behavior. Focused public tests cover
ZIP64 ordering and partial fields, full-width declarations, zero, mismatch
witnesses, NumPy values, native formatting and the inherited error/panic policy.
