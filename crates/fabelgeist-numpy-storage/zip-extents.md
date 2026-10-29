# Declared ZIP decoded lengths

`DeclaredUncompressedByteLength` describes the archive's declared decoded size,
including an NPY header for NumPy array members. It retains zero and the complete
`u64` width. It does not certify that a member exists, fits an allocation budget
or decodes to that length.

`ZipArchive::uncompressed_size` returns
`Option<DeclaredUncompressedByteLength>`. NPZ selects an optional member with
`Npz::get`; that handle's `uncompressed_size` returns the declared length
without repeating lookup. NPZ queries retain optional `.npy` suffix matching
and first-member selection. A stored or deflated member may
decode successfully even when the declaration disagrees.

```rust
use fabelgeist_numpy_storage::{DeclaredUncompressedByteLength, Npz, NpzArrayName};

fn declared_size(archive: &Npz) -> Option<DeclaredUncompressedByteLength> {
    archive.get(&NpzArrayName::new("weights"))
        .map(|member| member.uncompressed_size())
}
```

At an external byte-count boundary, `length.bytes()` projects the full declared
quantity into `u64`. Literal and decoding boundaries admit quantities through
`from_bytes`. Display and Debug delegate to the native integer, including its
formatter options. No serialization schema is introduced.

Rawzip owns compressed ranges and archive-origin positions through private
entry handles. The application no longer defines `CompressedByteLength` or
`ArchiveByteOffset`: those parser-only APIs have no application consumer.
ZIP64 per-entry fields are interpreted by rawzip before the decoded declaration
is admitted. Partial extra fields may retain unreplaced saturation values;
these declarations still do not measure actual bytes.

Rawzip checks local framing, range overflow and complete compressed payload
before handing back a borrowed slice. An out-of-archive local offset returns
an incomplete-member error rather than overflowing native arithmetic. Stored
members borrow archive bytes; deflated members return owned bytes. Deflate's
initial reservation still casts the decoded declaration to `usize`; no new
allocation budget, checksum, encryption or decoded-size verification is implied.
NPY payload sizes and tensor dimensions remain separate bespoke quantities.
