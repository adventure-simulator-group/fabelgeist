# Independent Code Review: material/error receivers

Verdict: **no actionable defect found in this scoped repair** by source review.
This does not clear the whole cache/migration family, all thirteen lenses, other
independent reviews, required hosted checks, or the final PR revision.

Base and current HEAD: `12a14b478506044f85a062f07aa05855cae0c8c7`, plus the four
working source files below. The ledger's
`material_cause_receivers_preinventory` was read in isolation. Applicable root,
crates and Creator instructions and the shared armor guide were considered.
This change repairs error handling and source order; no shape, geometry or
artistic acceptance is asserted.

## Review evidence

Full reads: current `studio_generation.rs`, `armory_scene.rs` (including its
existing fixture), `equipment_export.rs`, `underlayer_preview.rs`, and their
complete diffs. Connected owners read: `underlayer_material.rs`,
`metal_preview.rs`, `clothing/error.rs`, `export/textures.rs`, `metal_gpu.rs`,
`gpu_initialization.rs`. Targeted source: the actual parent `main.rs` import
scope, `generate_clothing_shells` signature/admission, and pinned SDK source.

- **Both studio receiver adapters:** `PreviewScene::spawn_equipment` retains
  `EquipmentMaps::material/metal`'s `anyhow::Error` through `with_context`.
  Pinned anyhow 1.0.104 `context.rs` has an explicit implementation for its
  `Error`, retaining the underlying error as the context source. Human
  presentation remains the actual `format!("{error:#}")` at the studio status
  boundary. The moved impl has exactly its original body with the two
  `map_err(anyhow::Error::msg)` lines removed; the rest of the studio file is
  byte-identical after accounting for that item move.
- **Both armory receivers:** `spawn_exhibit` uses direct `?` for the material
  and `transpose()?` for optional trim. They already return the same anyhow
  error type. No new error trait bound or semantic conversion is required.
  Whole-file reverse comparison confirms exactly the two adapter removals.
- **Garment receiver:** `EquipmentExporter::garment` receives
  `Result<ClothedMesh, ClothingError>` from the actual imported
  `generate_clothing_shells`. The concrete thiserror enum retains its
  geometry/rig causes and contains owned Send+Sync data; anyhow's pinned
  blanket `From<E>` admits `StdError+Send+Sync+'static`. Direct `?` retains the
  structured garment error and its source chain, rather than constructing a
  MessageError with no source. Whole-file comparison confirms exactly this
  one adapter removal.
- **Mail admission:** `MailImages::new` requires base color, decodes it as
  sRGB, decodes normal and optional occlusion as linear, with the same PNG,
  compressed-format, sampler and asset-usage arguments as before. Pinned
  Bevy 0.19.1 `Image::from_buffer` returns `TextureError`; anyhow Context
  accepts that real error and retains its cause, including `ImageError`.
  Missing base color creates an ordinary absence diagnostic because that
  Option carries no underlying error. It returns `Err` rather than panicking.
- **Asset/cache publication:** the three decode operations precede every
  `images.add`. Any missing image or decode error exits before changing the
  asset store or `self.mail`. Optional occlusion still uses one decode only
  when present. Assets are inserted in the original color/normal/occlusion
  order after admission. As with the original Bevy Assets API, no guarantee
  against process allocation failure/panic is claimed.
- **Borrow and inference:** `match self.mail` does not move a populated
  `MailImages`: `Some(ref maps)` borrows the payload, while `None` binds no
  payload and permits `self.mail.insert(...)` in that mutually exclusive arm.
  The `Some` arm yields `&MailImages`; `insert`'s `&mut MailImages` coerces to
  the shared reference used by the following handle clones. The constructor
  borrows independent `images` and `textures`, not `self.mail`; `?` completes
  before insertion. No self borrow is captured across a closure. The decode
  closure infers the SDK Boolean flag and error result; optional decoding's
  transpose produces `Result<Option<Image>, anyhow::Error>` as required.
- **Imports and successful behavior:** `use super::*` imports the parent's
  actual `anyhow::{Context, Result}`. `SurfaceTextures` is the existing public
  library type. New private `MailImages::new` has no API or feature exposure.
  `underlayer_material::textures` still selects the same embedded single mail
  atlas. Material colors, alpha/cull policy, shading and cache reuse remain
  unchanged. The existing successful mail Option remains cached once;
  failed decoding leaves it empty, while the native GPU once-cache failure
  policy remains independently unchanged.

The concrete repairs are causal, not a new universal error facade. Existing
String/&str message adapters elsewhere are outside this exact four-file delta;
this review does not indiscriminately remove them or clear them by search.
The whole scene is not transactional: prior scene/material work can have
occurred before a later item fails. No new whole-scene atomicity claim is made.

## Source-only comparisons and limits

All five relevant whole-source/fixture comparisons in
`/tmp/pr780-material-receivers-source-proof.json` are true:

- studio impl equals its original minus the exact two adapters;
- all other studio source equals its original around the item move;
- armory equals its original minus its exact two adapters;
- the existing armory test module is byte-identical;
- equipment export equals its original minus its exact one adapter.

The underlayer diff was inspected in full: it removes the two original expect
call sites (the PNG decode call was exercised three times) and replaces the
closure with fallible aggregate admission. No tests, cases, fixture helpers or
assertions were added. `MailImages::new` is a production record constructor.
No local build, compiler probe, formatter, quality tool or test was run; no
hosted CI or external write was requested. Rust borrow/type conclusions here
are source reasoning, not a compiler success claim. Unread entire application
and lower renderer/physical/grid/AST families remain outside this verdict.

## Current fingerprints (SHA-256)

Paths are relative to the isolated audit-pr-780 worktree, except pinned SDK
paths. Targeted-file fingerprints identify the snapshot without claiming full
coverage of those files.

| Path | SHA-256 |
| --- | --- |
| `crates/adventuresim-character-creator/src/studio_generation.rs` | `3031832a29165a676c81ee7b80a78ee5463dab02dbc69bdd959cd8bd3fd5d066` |
| `crates/adventuresim-character-creator/src/armory_scene.rs` | `aad333ef6af184900a6ac5da03ab5dea1e6bea08a26dea86cd8da164b37b6fbc` |
| `crates/adventuresim-character-creator/src/equipment_export.rs` | `2dfb462b1bfff461281e5bed831553766a392140c2772293f148f74f7d925390` |
| `crates/adventuresim-character-creator/src/underlayer_preview.rs` | `48e7ef34b4f757817c54281efd85160d1920dcb9e61fd8536113fd0927e879ed` |
| `crates/adventuresim-character-creator/src/underlayer_material.rs` | `52ebd2ffe14baf52f9e8bdd18ecf23fb26b94b1cf52c655fbc58074cd18471b7` |
| `crates/adventuresim-character-creator/src/metal_preview.rs` | `89e313508ced73bcd3f9dfe8e60ba2026dbb11d0d19fb178654e60423afd26b0` |
| `crates/adventuresim-character-creator/src/clothing/error.rs` | `3a33f6fd8aaabd68b8be71ea21f472b142fd17f7648bd79551af19b2c8f24297` |
| `crates/adventuresim-character-creator/src/export/textures.rs` | `57efed82e178fb19ffd93c264126d2070ecabeff0f7a402ef54c797cd75782dc` |
| `crates/adventuresim-character-creator/src/metal_gpu.rs` | `3759935bfda14dd3ba57883c232764706caa464ab1be27716e8bf7b99e0c9c1f` |
| `crates/adventuresim-character-creator/src/gpu_initialization.rs` | `10bc50519c97e93024e55b870b20dac09dc6bee84958a36366e2b9eb07cc5e01` |
| `crates/adventuresim-character-creator/src/clothing.rs` | `d3bce4f4d9a53afc7ea141c5edf1163e85f55cb1b455b976b3bd5e329902ce33` |
| `crates/adventuresim-character-creator/src/main.rs` | `566a69fc6cd50bd7d7ea5c0347567c39f2bdc37e9f8004675dc5328e3751e12e` |
| `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_image-0.19.1/src/image.rs` | `92666f8786981b9bcb90be80e1a0caaf6e055af99fe71080a600f29c0c876d3f` |
| `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-1.0.104/src/context.rs` | `8b106e43df7a66fbc3e85b87a4d7f8a780d43d3ee0cd1ae86c571950ecf07d13` |
| `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-1.0.104/src/wrapper.rs` | `079218e1d8c9374f52311b1c7971f7fbdeba4d7af1b0749c0d535010f1232697` |
| `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-1.0.104/src/error.rs` | `2a78ee4e5f8d6b4476c761f77473362b9ed12f5402e4701001e7d04d628ba030` |
