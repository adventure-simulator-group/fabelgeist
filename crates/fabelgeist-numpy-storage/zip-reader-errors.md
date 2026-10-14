# ZIP reader errors

`ZipArchive::open`, `ZipArchive::from_bytes` and member decoding return
`zip::Result<T>`, whose error is the bespoke `ZipReadError`. Callers can match
existing failure classes without interpreting diagnostic prose:

```rust
use fabelgeist_numpy_storage::{ZipArchive, zip::ZipReadError};

match ZipArchive::from_bytes(Vec::new()) {
    Err(ZipReadError::MissingEndRecord) => println!("no ZIP directory"),
    Err(error) => eprintln!("{error}"),
    Ok(archive) => println!("{} members", archive.names().count()),
}
```

The classifications cover a missing end record, a missing queried member,
a corrupt local header, a payload extending past the archive, a deflate
failure and an unsupported compression method. Native file failures also
identify `ZipFileOperation::Open` or `ZipFileOperation::Map`. Existing
`Display` messages and the order of validation remain unchanged.

Paths, queried names, central-directory names and rejected method words in
errors are diagnostic provenance captured at an OS or serialization boundary.
They retain the failed input spelling; they do not validate successful archive
metadata or establish a file, member, compression, offset or count owner.

Native open, map and deflate failures retain the original `std::io::Error`
through `std::error::Error::source`. `Npz` keeps its `anyhow::Result` boundary;
its error can be downcast to `ZipReadError`. The native I/O error is reached
through the source chain, rather than a direct `anyhow` downcast. `Debug` now
shows the structured class and its rejected input or native cause. MHR loader
contexts continue to surround this error, including corrective-basis and
sparse-activation diagnostics.

The archive constructor owns directory scanning and archive admission. Count
and offset words remain native at the existing capped `Vec` allocation,
`u64` iteration and slice-address ports. The little-endian scalar decoders
copy fixed-width slices into native arrays; they add no bounds policy.
Stored payloads still borrow archive bytes, while deflated payloads are owned.

This error contract preserves the reader's limited ZIP policy: scanning and
ZIP64 fallback, duplicate selection, lossy name decoding and declared sizes
are unchanged. It adds no checksum, encryption, decoded-size or framing
validation. Malformed central names or overflowing offsets can still panic
at inherited unchecked slices or arithmetic. Those framing responsibilities
remain separate from error classification.
