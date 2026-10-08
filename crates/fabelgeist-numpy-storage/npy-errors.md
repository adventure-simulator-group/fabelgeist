# NPY decoder and file-reader errors

`NpyArray::from_bytes` admits a serialized NumPy array and returns
`npy::DecodeResult<T>` with a bespoke `NpyDecodeError`. `NpyArray::read`
returns `npy::ReadResult<T>`: `NpyReadError::Read` retains a native file-read
failure, while `NpyReadError::Decode` surrounds the decoder cause with its
original path context.

```rust
use fabelgeist_numpy_storage::{NpyArray, npy::NpyDecodeError};

match NpyArray::from_bytes(&[]) {
    Err(NpyDecodeError::Magic) => println!("not an NPY array"),
    Err(error) => eprintln!("{error}"),
    Ok(array) => println!("{} elements", array.len()),
}
```

The decoder classifies existing magic, header extent/encoding, missing-field,
descriptor, Fortran-order, shape-token and payload-extent failures. A closed
`NpyHeaderField` distinguishes missing descriptor and shape fields. Existing
`Display` text and rejection order remain unchanged. The free parse/read
entries and root read alias are removed; construction belongs to `NpyArray`.

Rejected descriptor/token strings are serialization diagnostic provenance.
Failed native paths are OS diagnostic provenance. They preserve rejected
spelling and provide no admitted metadata, file, rank or storage identity.
UTF-8 and shape-token failures retain their original standard-library causes;
file failures retain the original I/O or decoder cause through `Error::source`.
`Debug` now exposes structured error classes and their diagnostic fields.

NPZ still adds its array-name context around decoding. Its outer `anyhow` error
can be downcast to `NpyDecodeError`; native standard-library causes remain in
the source chain. A standalone read instead owns `NpyReadError`, followed by
its I/O or decoder source. Direct `anyhow` downcasts to nested native causes
are intentionally replaced by source traversal. MHR corrective and Model
loader contexts continue to surround these causes.

Successful metadata and values keep their current API. Header words are
native little-endian arrays at serialized slicing ports. Numeric callbacks
receive native arrays with the Rust scalar's byte arity; complete chunks prove
copy bounds and partial words still disappear after public reinterpretation.
No checked shape, rank, byte width, offset or framing owner is introduced.

Tensor methods keep their existing native `TryFromSliceError`/`anyhow` rank
boundary. Rank rejection occurs before tensor construction and remains
separate from decoder classification. Checked layout admission belongs to
the layout responsibility; this contract does not add a competing rank type.

Parsing preserves loose header spellings, version/minor handling, casts,
trailing bytes, public metadata mutation and current bounds/arithmetic policy.
Unsupported order/dtypes remain rejected; no stricter NumPy grammar, checked
size policy or new version rejection is added. Short non-v1 length words and
overflowing shape/payload arithmetic can still panic at inherited native
operations. A later zero axis still does not repair an earlier overflow.
