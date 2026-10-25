# Independent Code Review recheck: Creator receiving correction

No actionable correctness defect found in this four-file correction.
Reviewed current Root HEAD `fbab0a1c55b7e1421ec2dbe609b6df539833171c` plus the working source bytes below. This extends the prior scoped full
Creator receiving review; it is not final PR/global clearance or check evidence.

Read the four complete current files (hair/gpu/strips.rs, hair/geometry.rs,
hair_preview.rs, hair_preview/assets.rs) and both actual count/collection owners.
Read the corrective all-thirteen preinventory contracts. Comparing the 39 pinned
files from the preceding full review identifies exactly these four changed
paths; all other prior source fingerprints still match. No project checks,
source edits, tests, cases, helpers, assertions or external actions were made.

Card selection is sound: GpuLods::counts().cards is derived directly from
self.cards.len(), CardLevelCount::select returns None for zero or
min(requested, len - 1) for nonzero, and cards() borrows that same immutable
collection. The selected ordinal therefore proves the direct index's bounds.
The prior get(requested).or_else(last) selected exactly the same element for
empty, in-range and out-of-range requests. GpuLods' sealed producer always
creates a first strand level, which also supports counts()'s strand_count call.
No independent shape/count or compatibility implementation was introduced.

The preview move retains imports/reexports and module paths, places FormChoice
and StrandsDrawn before impls, and keeps cfg(test) attached to render_test as the
last item. These item moves do not change name resolution, source identity or
worker/publication behavior. All bodies in the four current files were reread;
no new correctness issue was identified. Comment edits describe estimate-based
bounded subdivision, minimum ribbon width before taper and actual card ordinal
clamping/absent-family behavior.

The previous PlacedHair.longest migration finding remains resolved: its current
Metres field fingerprint is unchanged since the corrected full review. Existing
render fixture and extracted WGSL fingerprints are also unchanged. Earlier
source proofs of fixture/shader preservation remain applicable to those exact
unchanged bytes; this recheck ran no fixture.

Known broader physical/grid and lower-layer repairs remain outside this narrow
verdict. Complete audit waves, final combined independent reviews and required
hosted checks are still pending on one final pushed revision.

```text
818294f1347dd1b4d5bcd5535dae5c0cfe30b75dd1d4380091bf98a06e906b6b  crates/adventuresim-character-creator/src/hair/gpu/strips.rs
ba9b0696088257a6f3d984edf137b59435823ff73ba1f176a3fa82d96a3915fe  crates/adventuresim-character-creator/src/hair/geometry.rs
83895f2f6e29bda031bc1764ed1d51cc133051198e963ead3eb0ee728138ffff  crates/adventuresim-character-creator/src/hair_preview.rs
436b82cac0058641b1c230f488b1ade01d293b23c6822584d39ed6846476e143  crates/adventuresim-character-creator/src/hair_preview/assets.rs
44ebb1a94560acc8633f707efbc477822c5c2f46fa4945b8ca640d41c7efc85e  crates/fabelgeist-hair/src/lod/counts.rs
7814ed1212fc58bea86aebf290862d5937f4e3e20d2b508050d20983948323a8  crates/fabelgeist-hair/src/gpu/lods/records.rs
d6c46f6ef0b1f037cc45648c2baac76b9e383aad331cf1c3757f1e1150702b43  crates/adventuresim-character-creator/src/hair/groom.rs
f141e4878ee8868ded50d8a481f8f1effb5c1ee4bb382fbcec321918c254c57e  crates/adventuresim-character-creator/src/hair_preview/render_test.rs
525528762142f6b0e781d6f29ee1b58f66055b6e41873a86c6da96ba14b15e10  crates/adventuresim-character-creator/src/hair/gpu/strips/wgsl.rs
```
