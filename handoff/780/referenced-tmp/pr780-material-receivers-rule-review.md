# Independent material-receiver Rule Review

Root worktree
`/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator`,
HEAD `12a14b478506044f85a062f07aa05855cae0c8c7` plus the four named Creator
files. No actionable finding remains in the five error adapters, fallible mail
record admission and adjacent PreviewScene item-order delta. This narrow verdict
does not clear the wider material/native/physical or generated-program families.

Read all four complete current files and diff, all thirteen entries of
material_cause_receivers_preinventory, complete underlayer material/ClothingError/
MetalImages owners, relevant actual generate_clothing_shells body, equipment
material selector and CharacterMorphs receiving source. Inspected actual pinned
anyhow1.0.104 MessageError construction/source and Bevy Image0.19.1 from_buffer /
TextureError code. Native initializer/cache causes were traced in the earlier
separate cache family; their implementation is unchanged by this four-file delta.
Source inspection and direct comparisons establish this verdict, not historical
review approval or searches alone.

## Causal error receiving contracts

Studio PreviewScene::spawn_equipment's material and trim-metal calls already
return anyhow errors through EquipmentMaps. Removing Error::msg before existing
with_context preserves their original structured source chain. Armory
spawn_exhibit's material and optional trim transpose likewise propagate the
existing anyhow error directly with ?. EquipmentExporter::garment receives
Result<ClothedMesh,ClothingError> from generate_clothing_shells; ? now converts
that actual typed error through anyhow's standard-error boundary, preserving
ClothingError's nested RigJointLookupError/GeometryError sources.

Pinned anyhow Error::msg constructs MessageError<M> whose standard Error impl
has no source method. This hides the original Error::source traversal even if
its stored message object can still be downcast; it is not necessarily a
literal string allocation or complete loss of every stored object identity.
The five removed wrappers were inappropriate for already causal errors.
New direct propagation reuses existing owners; no extra universal error wrapper,
cache, text classification or compatibility adapter is added. Native GPU causes
remain through metal_gpu -> MetalImages::new -> EquipmentMaps -> receiver.
Studio's error:# status receives the chain at its actual presentation boundary.

This does not justify a blanket Error::msg removal. CharacterMorphs' unchanged
proportions.set returns &'static str, LibraryName::try_from returns String, and
recipe.validate returns String. Such message owners have no original standard
error chain to preserve. Their unrelated migrations and callers are not silently
changed or globally cleared by this review.

## Mail image aggregate admission

The authoritative underlayer_material::textures producer selects fixed embedded
mail base-color/normal/occlusion PNGs for a mail Underlayer design, with Mask/Both/
Tile surface policy. Other designs return none. SurfaceTextures retains optional
base color as its wider schema; the local mail constructor must handle absence.

MailImages::new now admits the required base-color presence, then decodes color,
normal and optional occlusion into local Image values before images.add or
self.mail.insert occurs. Missing base color returns a boundary diagnostic;
decoding errors retain Bevy TextureError and its image::ImageError source through
Context. All fallible steps precede asset/cache publication. If admission fails,
the mail cache stays absent and no partial mail handles have been added by this
constructor. Asset handle creation after successful decoding is the same
infallible framework handoff; this is not a promise to recover allocation panics
or make all scene mutations transactional.

Actual Image::from_buffer signature returns Result<Image,TextureError>; pinned
Bevy0.19.1 TextureError derives Error and ImageError uses #[from]. The new
constructor consumes that native owner rather than replacing it with prose.
The existing PNG extension, no-compression flag, default sampler/asset usages,
sRGB=true for color and false for normal/occlusion are unchanged direct framework
parameters. Those boolean flags are actual Bevy adapter policy, not newly
introduced application state enums. Successful material texture selection,
white base color, Mask0.5, sidedness, sampling and metal cache behavior remain
unchanged. Existing Option<MailImages> remains the sole cached-record authority.

There were two original production expect call sites: the shared decoder
(called for multiple images) and required base-color presence. Both are removed.
No third source expect is claimed. No unwrap/expect remains in the changed mail
owner. Failed admission now returns Err instead of panicking; this is the
intentional failure behavior change. GPU initialize-once policies are untouched;
retrying an absent mail record is its preexisting separate cache lifecycle.

MailImages::new is an inherent production constructor for meaningful multi-image
admission, not a new test helper or a pass-through service. Its existing anyhow
Result spans native decode and missing-record errors; no new error type or
dependency is required. The short constructor/file remain below size ceilings.

## Source confinement and item order

PreviewScene's impl now precedes free functions, with methods/docs/attributes
attached and only the two causal adapter lines removed. Independent comparisons
against HEAD establish:

- The whole PreviewScene impl differs only by the two removed message adapters.
- All scene/UI generation/free-function and body-mesh source is unchanged.
- Entire Armory source differs only by two adapter removals; its complete
  existing unit fixture is byte-identical.
- Entire EquipmentExporter source differs only by one adapter removal.
- EquipmentMaps imports/fields, entire metal cache method and successful mail
  material policy/return body are unchanged.

All other changed mail code is the explicit fallible constructor/cache admission
replacement. No source field quantity, renderer numerical value, native program,
image format, declaration owner, dependency, fixture/case/helper/oracle/assertion
was added or altered. Existing exporter index-zero use follows a one-element
specification input: the inspected generate_clothing_shells body pushes exactly
one shell per admitted specification or returns Err; CharacterMorphs.clothing
maps that same shell collection. No missing-element outcome is invented for
that proven cardinality in this scoped repair.

No source edits, project checks, probes, tests, builds, lints, scanners,
generators, runtime, visual acceptance, hosted CI or external posts were performed.
No tests/cases/helpers/assertions were authored. Direct source comparisons are
source evidence, not execution. Broader native/error-cache/physical/Grid and
private ordered shader family work remains open. Final full thirteen-lens waves,
three independent reviews and required hosted project checks must pass on the
same final pushed revision. This report supplies neither global clearance nor
permission to push an unfinished connected family or merge.

## Current SHA-256

- `crates/adventuresim-character-creator/src/studio_generation.rs`: `3031832a29165a676c81ee7b80a78ee5463dab02dbc69bdd959cd8bd3fd5d066`

- `crates/adventuresim-character-creator/src/armory_scene.rs`: `aad333ef6af184900a6ac5da03ab5dea1e6bea08a26dea86cd8da164b37b6fbc`

- `crates/adventuresim-character-creator/src/equipment_export.rs`: `2dfb462b1bfff461281e5bed831553766a392140c2772293f148f74f7d925390`

- `crates/adventuresim-character-creator/src/underlayer_preview.rs`: `48e7ef34b4f757817c54281efd85160d1920dcb9e61fd8536113fd0927e879ed`

- `crates/adventuresim-character-creator/src/underlayer_material.rs`: `52ebd2ffe14baf52f9e8bdd18ecf23fb26b94b1cf52c655fbc58074cd18471b7`

- `crates/adventuresim-character-creator/src/clothing/error.rs`: `3a33f6fd8aaabd68b8be71ea21f472b142fd17f7648bd79551af19b2c8f24297`

- `crates/adventuresim-character-creator/src/clothing.rs`: `d3bce4f4d9a53afc7ea141c5edf1163e85f55cb1b455b976b3bd5e329902ce33`

- `crates/adventuresim-character-creator/src/equipment_catalog.rs`: `da3139ef3625393db9041ad0680dc244b0f81235b86ea4519a87c9299763bdd6`

- `crates/adventuresim-character-creator/src/metal_preview.rs`: `89e313508ced73bcd3f9dfe8e60ba2026dbb11d0d19fb178654e60423afd26b0`

- `crates/adventuresim-character-creator/src/character_morphs.rs`: `8f76594ec3871a68df2f8e98740aa629e479b1e6fc7a0ad9a63e4e7f029e30a1`

- `crates/adventuresim-character-creator/src/library_name.rs`: `ecd978e94b850604d4345df1494e9536819e19d7a6d3b8921fde3d55bf54eb27`

- `crates/adventuresim-core/src/character_proportions.rs`: `7a3b0a5797dbb6d8c811aa2e2df9d831e7424367faf501cf6b3b9bffe1683433`

- `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-1.0.104/src/error.rs`: `2a78ee4e5f8d6b4476c761f77473362b9ed12f5402e4701001e7d04d628ba030`

- `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-1.0.104/src/wrapper.rs`: `079218e1d8c9374f52311b1c7971f7fbdeba4d7af1b0749c0d535010f1232697`

- `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_image-0.19.1/src/image.rs`: `92666f8786981b9bcb90be80e1a0caaf6e055af99fe71080a600f29c0c876d3f`
