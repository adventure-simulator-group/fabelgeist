# fabelgeist-fbx

Reads binary FBX 3D asset files: node/property records, object and connection
lists, and animation takes. Transform composition, units, and rig membership
belong to the caller. The binary reader uses a pure-Rust inflater and does not
require a tensor runtime or an FBX SDK.

## Scene identities and connection roles

`Scene::parse` admits native file bytes and returns the existing binary decoding
error. `Scene::from_roots` assembles decoded records without a failure result;
malformed object identities and connection endpoints are skipped.

Object and connection fields use distinct semantic slot enums at scene admission.
Private `Node` lookups select native property positions only at the checked
container access, so scene assembly passes field roles rather than raw offsets.

`FbxObjectId` keeps an object-table identity distinct from an array index or an
animation time tick. Signed integers of all previously admitted widths and
boolean properties remain valid identity encodings. Negative and zero IDs are
allowed. An ID proves neither existence nor uniqueness nor membership in a
particular scene. Duplicate objects retain encounter order, while lookup picks
the last duplicate. Native wire construction uses `FbxObjectId::from(i64)`;
encoding an admitted ID uses `Prop::from(id)`.

`FbxObjectId::SCENE_ROOT` identifies the destination for root connections.
Source-zero connections are ignored. Unresolved source links stay in the graph,
but child traversal skips them. Connections keep file order across blocks,
including duplicate links and uninterpreted connection record/type tokens.

`FbxConnectionProperty` admits string or raw connection bytes. It preserves the
lossy UTF-8 spelling, including unknown names, whitespace and suffixes. The
immutable classification exposes `TransformProperty` for model translation,
rotation or scaling, and `CurveAxis` for scalar X, Y or Z channels. Unknown and
near-matching names have neither role. `Display` provides diagnostic spelling;
internal animation selection uses the roles.

```rust
use fabelgeist_fbx::{FbxObjectId, Scene};

fn main() -> anyhow::Result<()> {
    let file_bytes = std::fs::read("rig.fbx")?;
    let scene = Scene::parse(&file_bytes)?;
    for object in scene.children(FbxObjectId::SCENE_ROOT) {
        for (child, property) in scene.children_with_property(object.id) {
            if let Some(transform) = property.and_then(|name| name.transform()) {
                println!("{:?} targets {:?}", child.id, transform);
            }
        }
    }
    Ok(())
}
```

Object names retain their existing namespace stripping and class-suffix policy;
connection property names deliberately do not use that normalization. Animation
keeps the first connected layer, drops empty takes, assigns the last matching
axis connection, truncates mismatched key/value arrays, and preserves time
conversion, interpolation and numeric casts. These owners add no finite-value,
positive-ID, uniqueness or object-existence restriction.

Record/class selectors, object-name roles, numeric arrays and animation time or
value quantities still have primitive interfaces. Binary framing/read errors
remain a separate migration family.

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
