# Binary FBX array encoding

The binary FBX model-file reader admits the four-byte array encoding field into
a bespoke FbxArrayEncodingCode before passing it into the shared array decoder.
This keeps encoding identity distinct from counts, byte lengths and property
tags through all five array kinds: 32-bit and 64-bit floats, 32-bit and 64-bit
integers, and Boolean bytes.

The encoding code is open. Zero selects an uncompressed payload; every nonzero
value uses the existing Zlib decompression path, including values other than 1.
This preserves the current decode policy rather than asserting which codes the
wider format supports. Decoder errors, empty counts, extra decoded bytes and the
original float bit patterns and Boolean byte values remain unchanged. Counts,
widths, tags, offsets and structured decode errors remain separate migration
responsibilities.

A regression fixture records 95 observations from the unchanged main public
parser. Independently encoded wire properties cover all five array kinds with
zero, one and other nonzero codes; empty, ordinary, short and extra payloads;
and corrupt compressed streams. The fixture retains exact floating-point bit
patterns and Boolean bytes when comparing decoded output.
