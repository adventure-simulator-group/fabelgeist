# Sole UV authority Comment Review

Revision: `d000e848f2f5bc28b0728c2cd664c727a41ecf50`.
Tree: `57cea419b31eac49981a90599b87e6990b407bcc`.
Base: `59bb96d07af08244582c6767d81e969b820ec3ed`.

All six owning files were freshly read completely, including the full export
fixture/helper/assertion source. All six packet bytes match manifest SHA-256
and exact pinned Git. Five unchanged relying files were freshly fully read:
compactor, texture writer, morph/shell producers, garment export and hair export.
JSON records every source path, hash, trace and limit. Prior reviewer verdicts
and eleven preservation comparisons are context only.

No actionable comment mismatch was established. `surface` now contains maps
only; `texcoords` owns the sole UV column. Either map payload requires finite
UVs with exactly one row per position at export validation. The unchanged
compactor remaps that column alongside the other channels, and the writer uses
only the resulting column. It has no stale surface-coordinate fallback.
Character and equipment map producers fit this contract; trim, garment and hair
producers retain their own chart selection. Their deeper source correspondence
obligations remain unchanged.

The complete packet diff preserves the existing image/PBR override order and
only adapts the existing textured fixture's map/UV operands. Fixture cases,
helpers, inputs, oracles and assertions are unchanged. No tests were executed.
Public structs are still admitted at export; documentation does not prove all
constructed records valid. Earlier fitting, baking or output activity also
prevents interpreting this UV boundary as whole-operation transactionality.

Root has not copied this packet. Later material/component/PBR changes must
preserve the sole-column contract during actual composition. The remaining
material, native/body/physical/generated and full caller/cache/lock families,
all final thirteen-lens waves, all three reviews and hosted gates remain open.
This is no compilation, native acceptance, runtime, family or final PR clearance.

Only read-only source/Git inspection and ignored report bookkeeping occurred;
no source edits, project execution, checks, tests, scanners, generators, runtime,
new fixtures, push, CI or external publication occurred.
