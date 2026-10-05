# fabelgeist-fbx

Reads binary FBX 3D asset files: node/property records, object and connection
lists, and animation takes. Transform composition, units, and rig membership
belong to the caller. The binary reader uses a pure-Rust inflater and does not
require a tensor runtime or an FBX SDK.

## Scene identities and connection roles

`Scene::parse` admits native file bytes and returns the existing binary decoding
error. `Scene::from_roots` assembles decoded records without a failure result;
malformed object identities and connection endpoints are skipped.

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
