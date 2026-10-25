# Independent Comment Review: material cause receivers

HEAD: `12a14b478506044f85a062f07aa05855cae0c8c7`, with only the four
assigned Creator receiving files dirty.

Verdict: CLEAR for this receiver-cause and connected mail-image admission scope.
No actionable scoped comment finding. Final combined reviews remain open.

## Current SHA-256 fingerprints

- `crates/adventuresim-character-creator/src/studio_generation.rs`: `3031832a29165a676c81ee7b80a78ee5463dab02dbc69bdd959cd8bd3fd5d066`
- `crates/adventuresim-character-creator/src/armory_scene.rs`: `aad333ef6af184900a6ac5da03ab5dea1e6bea08a26dea86cd8da164b37b6fbc`
- `crates/adventuresim-character-creator/src/equipment_export.rs`: `2dfb462b1bfff461281e5bed831553766a392140c2772293f148f74f7d925390`
- `crates/adventuresim-character-creator/src/underlayer_preview.rs`: `48e7ef34b4f757817c54281efd85160d1920dcb9e61fd8536113fd0927e879ed`

## Source-backed coverage

- Read all four complete current files and their full diff, not just the five
  removed adapters. Read the material_cause_receivers_preinventory thirteen-lens
  entry for scope context; it is not clearance evidence.
- EquipmentMaps::material and metal already return anyhow::Result. Direct ?
  propagation and the Studio contextual wrappers now retain those causes.
  The equipment garment path receives Result<ClothedMesh, ClothingError> from
  generate_clothing_shells and converts that actual error through ?. No new
  source comment claims textual classification or loss of the original cause.
- MailImages::new decodes base color, normal and optional occlusion into local
  Images before the first images.add or publication into EquipmentMaps.mail.
  Missing required base color and Bevy TextureError values acquire ordinary
  anyhow context, replacing the two unconditional expect sites. The existing
  embedding/material comments remain accurate; no stronger whole-scene
  transaction or native resource guarantee has been added.
- Read the actual underlayer_material::textures producer: it supplies the same
  embedded PNG trio for mail construction, with Mask and Both-side policy.
  The new constructor preserves explicit PNG decoding, color is_srgb=true,
  normal/occlusion is_srgb=false, default sampler and RenderAssetUsages.
  Optional occlusion remains optional. Existing AlphaMode/culling/material
  assignment follows those texture policies as before.
- Read local Bevy Image::from_buffer: its actual image-type conversion and PNG
  reader decode return TextureError, and successful decoding constructs the
  host Image using the supplied color-space flag. The receiver does not claim
  that host image admission proves later native texture creation, submission,
  completion or global GPU health.
- Read MetalImages::new/material and relevant clothing producer/error signature
  to confirm these receiving comments against their actual source contracts.
  The cached baked-metal usage comments still match begin_generation and the
  mark-used path.
- PreviewScene documentation stays attached to its record and bounds field.
  Its impl now precedes free functions; the method context still names each
  failed piece/material rather than inventing a new classification. Armory and
  equipment-export existing comments are unaffected by cause-preserving ?
  propagation. No explanation of the removed string/error adapters is needed
  in durable source comments.

## Limits

No source edits, project checks, tests, cases, helpers, assertions, documentation
samples, external posts or hosted CI requests were made. Only this local report
was written. This is source-only scoped comment review, not observed decoding
failure injection, native rendering, all-lens clearance or final-pushed-revision
acceptance. Broader physical/Grid/AST/GPU-composition and final global review
remain open.
