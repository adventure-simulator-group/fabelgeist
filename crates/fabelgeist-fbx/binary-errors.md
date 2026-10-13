# Binary FBX decoding errors

FBX is a format for 3D model files. `parse` reads a binary file into its
node/property tree, and `Scene::parse` builds scene queries from that tree.
Both return `fabelgeist_fbx::Result<T>` with the bespoke `FbxDecodeError`.
`Scene::from_roots` accepts decoded nodes and returns `Scene` directly.

## Error classifications

The error names the decoder's existing rejection paths:

- `AsciiInput`: recognized text FBX, with or without a byte-order mark.
- `NotBinaryInput`: an incomplete or unrecognized binary header.
- `ReadOverflow`: native cursor plus requested byte length overflows `usize`.
- `TruncatedRead`: a native read extends beyond the input buffer.
- `Inflate`: Zlib array decompression failed, retaining its native input/output
  (`std::io::Error`) cause.
- `ArrayShort`: decoded bytes cannot supply the declared array count/layout.
- `PropertyTag`: the admitted property tag is unknown at dispatch.

Ordinary `Display` keeps each existing message. Alternate `Display`
(`{error:#}`) keeps the inflation context followed by every native cause,
separated by a colon and a space. `Debug` exposes the variant and its context.
The error derives `std::error::Error`; `Inflate` retains its original native
cause through `Error::source`. A focused formatter preserves that variant's
alternate presentation while the other messages use derived formatting.

`ArrayShort` carries the existing `FbxArrayCount` and `FbxArrayKind` owners;
`PropertyTag` carries the existing `FbxPropertyTag`. These context types are
publicly exported for matching the error. Their admission remains open and
unchanged. Failed-read offsets and lengths, observed decoded-buffer lengths and
native input/output errors are serialization diagnostics, rather than new
shared storage or metadata owners.

## Caller composition

The Momentum Human Rig (MHR) character loader keeps its existing
`Scene::parse(data)?` composition into `anyhow::Result`. It does not stringify
the decoder error. Callers can downcast to `FbxDecodeError` and follow its
source chain to the native inflation error. Existing model-load contexts also
retain that chain. File-opening errors arise before decoding and keep their
existing reading context. A wider bespoke MHR loader-error API remains separate;
absent joints, assets and tensor construction are not classified by this
decoder.

## Preserved parsing policy and limits

Header layout still uses the existing version owner and the cutoff at 7500.
Every version word remains admitted. Zero array encoding copies raw bytes;
every nonzero encoding uses Zlib. Boolean byte values, exact floating-point
bits, extra decoded bytes, lossy names, raw string bytes, ignored header fields,
connection order, duplicate object lookup and trailing input keep their current
behavior. Scalar copies use the same exact bytes through fixed-size arrays.

The five existing parser tests and their 794 golden records remain unchanged.
They cover parsing values and rejection presentation, including alternate
inflation diagnostics. They do not add direct variant, downcast or source-chain
assertions. No tests, cases, fixtures or Rust documentation tests are added.

This error API adds no framing, extent, allocator or excessive-count policy.
Native casts, arithmetic and allocations remain, and malformed offsets can
still panic. The existing suite is not an arbitrary-input safety guarantee or
a production-asset, 32-bit host, renderer or full character-creator verdict.
