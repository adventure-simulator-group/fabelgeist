# ZIP reader errors

`ZipArchive::open`, `ZipArchive::from_bytes` and member decoding return
`zip::Result<T>`, whose error is the bespoke `ZipReadError`. Callers match typed
failure classes without interpreting diagnostic prose:

```rust
use fabelgeist_numpy_storage::{ZipArchive, zip::ZipReadError};

match ZipArchive::from_bytes(Vec::new()) {
    Err(ZipReadError::MissingEndRecord) => println!("no ZIP directory"),
    Err(error) => eprintln!("{error}"),
    Ok(archive) => println!("{} members", archive.names().count()),
}
```

`MissingEndRecord` represents rawzip's missing-end classification. Other
constructor failures use `Archive` and retain the concrete `rawzip::Error`.
Selected directory corruption rejects admission instead of exposing a partial
successful archive. Rawzip owns record framing and ZIP64 interpretation.

Member operations distinguish `MissingMember`, `CorruptMember`,
`MemberPastEnd`, `Inflate` and `UnsupportedCompression`. Names retain
`ArchiveMemberName`; unsupported methods retain `ZipCompressionCode` and its
decimal diagnostic. Rawzip's `Eof` becomes `MemberPastEnd`, covering incomplete
local framing as well as missing compressed bytes. Other member parser failures
become `CorruptMember`. Both retain the upstream error as their source. Rawzip
does not expose enough stage information to distinguish every former short
local-header failure from a truncated payload; no second parser recreates that
old distinction. Structural checks still precede unsupported-method errors.

Native file failures retain `ZipFileOperation::Open` or `Map`, the original OS
path and the concrete `std::io::Error`. Deflate failures retain their original
I/O cause. A rejected path or member name is diagnostic provenance, not proof
of path safety or successful file identity.

`Npz` keeps its `anyhow::Result` boundary, and errors can be downcast to
`ZipReadError`. Parser and native causes remain reachable through
`std::error::Error::source`; MHR's corrective-basis and sparse-activation
contexts surround the same chain. Default display prints the owning member or
file context. Alternate display also appends native I/O causes; structural
member causes remain in the source chain. NPZ's alternate `anyhow` display
includes those chained parser causes. Archive framing diagnostics include the
upstream explanation directly.

The adapter retains lossy names, duplicate selection, application count bounds,
central compression authority and declared-size queries. It does not enable
rawzip's optional CRC/decoded-size verifier or add encryption handling and
resource limits. Checked structural access does not certify arbitrary archives.
