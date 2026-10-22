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

Queried and central-directory names retain `ArchiveMemberName` in the error.
Unsupported methods retain the canonical `ZipCompressionCode`; its native
integer conversion is an explicit diagnostic observation. Native OS paths
remain rejected-input provenance. These values do not certify member existence,
path safety or successful decoding.

Native open, map and deflate failures retain the original `std::io::Error`
through `std::error::Error::source`. `Npz` keeps its `anyhow::Result` boundary;
its error can be downcast to `ZipReadError`. The native I/O error is reached
through the source chain, rather than a direct `anyhow` downcast. `Debug` now
shows the structured class and its rejected input or native cause. MHR loader
contexts continue to surround this error, including corrective-basis and
sparse-activation diagnostics.

`CentralDirectory` owns directory selection and retains the canonical member
count and archive-origin offset. The archive constructor consumes that record
for capped reservation and full-width iteration. Error classification does not
replace those owners with raw words. The little-endian scalar decoders copy
fixed-width slices into native arrays; they add no bounds policy. Stored
payloads borrow archive bytes, while deflated payloads are owned.

Default `Display` prints the owning context; alternate `Display` also prints the
native cause, preserving direct ZIP diagnostics. The cause remains available
through `Error::source`, so NPZ and MHR contexts retain the full cause chain.

This error contract preserves the reader's limited ZIP policy: scanning and
ZIP64 fallback, duplicate selection, lossy name decoding and declared sizes
are unchanged. It adds no checksum, encryption, decoded-size or framing
validation. Malformed central names or overflowing offsets can still panic
at inherited unchecked slices or arithmetic. Those framing responsibilities
remain separate from error classification.
