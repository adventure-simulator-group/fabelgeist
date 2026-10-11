# fabelgeist-numpy-storage

NumPy `.npy` and `.npz` reading for Burn, in pure Rust — no NumPy, no C zlib.

```rust
use burn::tensor::Device;
use fabelgeist_numpy_storage::Npz;

let archive = Npz::open("weights.npz")?;
for name in archive.keys() {
    println!("{name}: {:?}", archive.array(name)?.shape);
}
let weights = archive.array("layer0")?.to_tensor::<2>(&Device::default())?;
```

- Archives are memory-mapped; members may be stored or deflated, and zip64 is
  handled (NumPy switches to it past 2 GB).
- `uncompressed_size` returns a bespoke `DeclaredUncompressedByteLength` from
  archive metadata. It does not decode or verify the member's actual length.
- Element types: `f4`, `f8`, `i4`, `i8`, `u1`, `b1`.
  Values are widened on
  access with `to_f32`, `to_i64` or `to_bool`, or uploaded straight to a device
  with `to_tensor` / `to_int_tensor`, so a `.npz` written as `float64` still
  loads into an f32 tensor.
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
