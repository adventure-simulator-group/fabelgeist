# Binary FBX decoding errors

FBX is a format for 3D model files. `parse` reads a binary file into its
node/property tree, and `Scene::parse` builds scene queries from that tree.
Both return `fabelgeist_fbx::Result<T>` with the bespoke `FbxDecodeError`.
`Scene::from_roots` accepts decoded nodes and returns `Scene` directly.

## Error classifications

The error names the decoder's rejection paths:

- `AsciiInput`: recognized text FBX, with or without a byte-order mark.
- `NotBinaryInput`: an incomplete or unrecognized binary header.
- `ReadOverflow`: native cursor plus requested byte length overflows `usize`.
- `TruncatedRead`: a native read extends beyond the input buffer.
- `Inflate`: Zlib array decompression failed, retaining its native input/output
  (`std::io::Error`) cause.
- `UnsupportedArrayEncoding`: the canonical checked encoding rejects a code
  other than 0 or 1, retaining `UnsupportedFbxArrayEncoding` as its cause.
- `ArrayShort`: decoded bytes cannot supply the declared array count and
  selected decoder width.
- `PropertyTag`: a native byte does not identify a supported property tag.

Ordinary `Display` keeps each message. Alternate `Display` (`{error:#}`)
keeps the inflation context followed by every native cause, separated by a
colon and a space. `Debug` exposes the variant and context. The error derives
`std::error::Error`; `Inflate` retains its native cause through `Error::source`.
A focused formatter preserves that variant's alternate presentation. The
encoding variant derives its conversion and source from the same canonical
error returned by the encoding enum, including its public `reported_code`.

`ArrayShort` carries the existing public `FbxArrayCount` owner and the native
byte width supplied by the decoder's fixed-size input. `PropertyTag` carries
the rejected native byte; the checked tag enum remains private and admits
only the thirteen supported tags. Failed-read offsets and lengths, observed
buffer lengths, native widths, rejected bytes and input/output errors are
serialization diagnostics, rather than parallel metadata owners.

## Caller composition

The Momentum Human Rig (MHR) character loader keeps its existing
`Scene::parse(data)?` composition into `anyhow::Result`. It does not stringify
the decoder error. Callers can downcast to `FbxDecodeError` and follow its
source chain to the native inflation or canonical unsupported-encoding error.
Existing model-load contexts retain that chain. File-opening errors arise
before decoding and keep their existing reading context. A wider bespoke MHR
loader-error API remains separate; absent joints, assets and tensor
construction are not classified by this decoder.

## Parsing policy and limits

Header layout uses the existing version owner and the cutoff at 7500. Every
version word remains admitted. Encoding 0 copies raw bytes, encoding 1 selects
Zlib, and every other code is unsupported before payload decoding. Counts and
stored payload lengths retain their existing unsigned admission. Boolean bytes,
exact floating-point bits, extra decoded bytes, lossy names, raw string bytes,
ignored header fields, connection order, duplicate object lookup and trailing
input retain their parent behavior. Scalar and element copies use fixed-size
arrays, with widths determined by the selected native decoder.

The five existing parser tests retain 794 golden records. The checked parent
supplies the unsupported-code expectations in the 95-row encoding and 282-row
layout fixtures. The other fixture records and all test sources remain
unchanged. Existing records cover values and rejection presentation, including
alternate inflation diagnostics. They do not add direct variant, downcast or
source-chain assertions.

This error API adds no framing, extent, allocator or excessive-count policy.
Native casts, arithmetic and allocations remain, and malformed offsets can
still panic. The existing suite is not an arbitrary-input safety guarantee or
a production-asset, 32-bit host, renderer or full character-creator verdict.
