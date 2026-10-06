# Binary FBX decoding failures

`parse` reads a binary FBX file into its top-level node list. `Scene::parse`
uses the same decoder and builds scene queries from those roots. Both return
`fabelgeist_fbx::Result<T>`, whose concrete error is `FbxDecodeError`.
`Scene::from_roots` accepts already decoded nodes and returns `Scene` directly;
it has no failure path.

The bespoke error classifies the decoder's seven existing rejection cases:

- `AsciiInput`: recognized ASCII FBX input, including a byte-order mark.
- `NotBinaryInput`: an incomplete or unrecognized binary header.
- `ReadOverflow`: native cursor plus requested byte length overflows `usize`.
- `TruncatedRead`: the requested read extends beyond the input buffer.
- `Inflate`: zlib array inflation fails, retaining the original native I/O
  error.
- `ArrayShort`: the decoded buffer cannot supply the declared scalar array.
- `PropertyTag`: a property type byte is unrecognized.

The public error's `Display` preserves the decoder's existing messages.
`Inflate` displays the FBX context separately from its native I/O cause, exposed
through `std::error::Error::source`. `Debug` exposes the new variant and its
rejected provenance, allowing callers to match the variant instead of prose.

The MHR character and model loaders retain their existing `anyhow` contexts.
Their directly downcastable FBX failure is now `FbxDecodeError`. For an
inflation failure, follow `Error::source` or the `anyhow` error chain to
downcast the native
I/O cause. File-opening errors remain native I/O failures; they arise before
FBX decoding and keep the loader's reading context.

## Native representation ports

Offsets, lengths, counts and tag bytes in error fields are rejected serialized
input or observed native buffer sizes. They are diagnostic provenance at the
serialization boundary, not admitted storage or FBX metadata values. The error
adds no primitive arithmetic, getter or conversion API for them. Their broader
storage and metadata ownership remains separate.

The private reader retains native positions and wire words directly at byte
slicing, allocation, range iteration and little-endian scalar conversion. Its
checked read supplies an exact slice length before copying into a fixed array;
scalar conversion no longer requires a fallible conversion followed by unwrap.
The module-wide root-list parser remains the serialized file entry point;
reader, property and node admission belong to their respective constructors.

## Preserved wire policy and limits

Every nonzero array encoding word still selects zlib, and extra decoded array
bytes remain ignored. Versions below 7500 use narrow node headers; all other
version words use wide headers without unsupported-version rejection. Ignored
header fields, lossy names, raw string bytes, Boolean decoding, ordering,
duplicate object lookup and trailing data retain their existing behavior.

This change does not introduce stricter framing or extent validation. Native
integer casts, unchecked node-end arithmetic and allocation behavior remain.
Malformed offsets or excessive property counts can still panic. The error
classes describe existing rejection paths rather than guaranteeing safe
admission of every malformed file.
