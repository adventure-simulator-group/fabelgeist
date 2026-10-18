# Binary FBX reader

## Property tags

The binary FBX model-file reader admits each one-byte property tag into a
bespoke FbxPropertyTag before dispatch. The same owner names all thirteen
supported tag codes and distinguishes scalar numbers and Booleans, numeric and
Boolean arrays, strings, and raw byte payloads. Tag identity remains typed
through both property selection and array selection.

Tag admission is open. Unknown native bytes reach the existing dispatch
rejection and retain the same character diagnostic. The type does not admit
decoded payloads or change scalar widths, array decoding, Boolean truth rules,
string encoding, read ordering or parser errors. Version/header layouts, array
encoding, counts, offsets and structured errors remain separate migration
responsibilities.

A regression fixture records 284 observations from the unchanged main public
parser. Independently encoded properties cover all 256 native tag bytes, all
thirteen known payload representations, missing and short payloads, and mixed
forward/reverse property sequences. Comparisons retain exact floating-point bit
patterns, invalid string bytes and the distinct string/raw property variants.

## Array encoding

The binary FBX model-file reader admits the four-byte array encoding field into
`FbxArrayEncoding` before passing it into the shared array decoder. This bespoke
enum keeps encoding identity distinct from counts, byte lengths and property
tags through all five array kinds: 32-bit and 64-bit floats, 32-bit and 64-bit
integers, and Boolean bytes.

This reader supports exactly two encoding codes: 0 selects `Uncompressed`, and 1
selects `Zlib`. Every other code returns `UnsupportedFbxArrayEncoding` before
payload decoding. Its public `reported_code` field retains the raw wire value;
callers can inspect this bespoke error through the parser's `anyhow::Error`.
This deliberately replaces the previous policy that treated every nonzero code
as Zlib. The shared decoder matches the two supported variants exhaustively.

The existing regression fixture records 95 observations through the public
parser. Independently encoded wire properties cover all five array kinds with
codes 0, 1, 7 and 4294967295; empty, ordinary, short and extra payloads; and
corrupt compressed streams. The 45 observations for supported encodings retain
their exact output, including floating-point bit patterns, Boolean bytes and
decoder errors. The other 50 observations now report the unsupported encoding
code, including empty arrays and corrupt payloads.

The native `Reader::u32` call reads the serialized encoding word: FBX array
metadata stores element count, encoding and compressed byte length as three
little-endian 32-bit words. The enum declares codes 0 and 1 as explicit `u32`
discriminants and derives `TryFrom<u32>` with `num_enum::TryFromPrimitive`.
The enum's custom-error configuration makes the standard `TryFrom` associated
error `UnsupportedFbxArrayEncoding` for every caller. Its derived `From<u32>`
constructor retains the rejected wire value. `read_props` checks the wire word
once and propagates that error directly with `?`; the enum remains typed through
array decoding. Blender's published binary format description documents code 0
for raw contents and code 1 for Zlib; it reports that other values had not been
observed. Those values are unsupported by this reader, without asserting that
Autodesk formally declares them invalid.

[Blender FBX binary format description](https://code.blender.org/2013/08/fbx-binary-file-format-specification/)
