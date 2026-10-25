# Incoming material cause receiver Code Review

Reviewed `99d70e7c3f0bcf8db563c602f39869b91a9f23bf`, tree
`f9764699a468d5bf8ebc56075df4138ddbc87a01`, against parent
`12a14b478506044f85a062f07aa05855cae0c8c7` and Root
`3e74eea62f4138fb95e001bf93e9438966d12612`. No actionable finding
was identified in this bounded four-file correction.

All four complete source files match the manifest, exported package and
immutable Git bytes. Five former `Error::msg` adapters are removed from
studio material/trim, armory material/trim and clothing export receivers.
Direct propagation and retained context preserve concrete errors through
actual preview/export callers. Clothing returns a typed ClothingError; Bevy
PNG decoding returns TextureError retaining its ImageError. The unchanged
initialization wrapper retains the first error through shared ownership.

`MailImages::new` decodes color, normal and optional occlusion before any
asset insertion. A failed decode therefore publishes no handle and leaves
the mail cache empty. Successful insertion order, sRGB flags, sampler, asset
usage and material alpha/culling behavior are unchanged. The moved whole
spawn-equipment implementation differs only by its two removed adapters.
The existing armory fixture suffix and entire main file are byte-identical
to the parent.

The static atlas selector provides one mail family, which supports the
existing single cache. The separate draped MailWeave material owner has a
different generated source and repeating sampler; it is not a recipient of
this changed EquipmentMaps API. No broad material consolidation is implied.
Whole scene rebuilds and exports retain their existing partial-publication
policy; this change only makes the mail-image admission atomic.

Root receiving awaits the separate complete thirteen-lens inventory and
canonical API/error/producer composition. This package does not establish
whole migration, compile, render, hosted-check or final PR clearance. Final
full reviews and required checks must use the same final pushed revision.

No source was copied or edited. No project commands, tests, builds, scanners,
generators or runtime were executed. Source identities, read boundaries and
proofs are in `material-cause-receivers-99d-code-review.json`.
