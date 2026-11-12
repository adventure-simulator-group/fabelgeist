# Bounded Lens 2 owner-fit review: original 059 q2355 and q2356

This report supersedes `REPORT-v2.md` and `REPORT.md`: the first correction
expands an abbreviated source path, and this version identifies the usable
verifier. Both earlier reports remain as preserved drafts.

## Scope and provenance

This is a new source-only review by the delegated `verifier_scope` reader at
`95e63628f9d70b8dd6e6fede64334696c6995009` (95e), not a new historical
receipt, global closure, repair verdict or checkpoint 3821 recovery. The
original 059 queue is from `56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e`;
the public reconstruction is from `26e360920b7b2ec32d5c76238623eaf6d6617d7e`.
Whole streams differ, while the exact selected q2355 and q2356 byte slices
match. Six other related exact slices and three reserialized selected ledger
objects are separately compared in the packet. A reserialized object is not
represented as an original byte slice. See `HISTORY-IDENTITIES.json`,
`RELATED-SELECTION-IDENTITIES.json` and `LEDGER-COMPARISON.json`.

Both records historically have status
`open-connected-owner-current-alias-qualification` and their implementation
gate is false. The first asks about BakedMap/BakedRecipe wire admission and
native Image owner fit; the second asks whether a retained camera-derived
focus fits the admitted scene-local point owner. Original reader and adoption
records are preserved in `HISTORICAL-*.dat` and the bounded focus summaries.
The frozen Lens 2 source receipt used 16 rows, 1,638 fresh own-Lens-2 body
lines, 25 root personal bounded reread lines and 215 inherited same-lens
lines; no new unique count. Its first `dialogue_bindings.rs` row retains the
original Lens 2 `camera` reader's 1–215 scope, while root personally reread
216–240; that name is historical attribution, not a subsystem assumption.
The later e275 adoption qualified 1,878 selected lines and zero fresh personal
reads. It did not close current sources, native chains or all aliases. Its
verification was metadata-only. All 15 unique frozen files match the
historical hashes at `6533e444fa37552927070cc8d8190907fd3b413e`; 13 of 15
whole files match between e275 and 95e. The two differences are in texture
runtime and tactical materials. Camera rig `camera.rs` and the texture crate
manifest, additional support sources, also differ; registered complete diffs
are retained. Matching bytes do not grant new personal reading credit.

Nine directly hash-cited historical records, two selected nested records and
two related Lens 3 records were recovered and hash checked. Same-machine
transport metadata matches those 13 identities; this is not independent
custody. See the registration, recovery and transport files. This packet does
not assert that all historical dependencies of the related records were
recovered. `SOURCE-READ-RECEIPT.json` records 42 captured 95e source files,
45 prospective displays and 3,763 unique personally displayed lines; 9,863
lines of those captured files remain unread. Capture, search and hash checks
are not body-read credit. Local timestamp ordering is evidence of recorded
sequence, not proof of comprehension. `DISPLAY-QUALIFICATIONS.json` retains
locator limits, a truncated navigation output, one failed structural attempt,
one absent guessed path, test-text-only reading and the failed `python`
command. None supplies source-body credit.

## q2355: texture map and physical scale

`crates/adventuresim-procedural-textures/src/bake/wire.rs:60–123` admits a
nonempty bounded, unique-channel map list with finite positive tile metres,
finite nonnegative height range, valid square power-of-two sizes and mips,
exact payload lengths and no trailing payload. The compressed wrapper bounds
decompression and rejects trailing compressed bytes (`bake/compressed.rs:19–34`).
The native loader reads this codec and checks the recipe filename before
`From<BakedMap> for Image` constructs the native image (`runtime.rs:117–174`).
`BakedMap` and `BakedRecipe` expose fields (`bake.rs:79–95`), and the `From`
adapter has no independent fallible admission. The decoder proof therefore
applies to its decoded path; it does not by itself establish all alternate
public constructors, producers, or malformed reachable inputs. `BakeResolution`
is a sampling cap (`parameters.rs:54–69`), not an owner for every actual
pixel extent. Bounded owner searches do not prove a general pixel owner absent.

The historical packet said no physical-scale leaf fit had yet been established.
This new reader found a concrete narrower candidate: public
`fabelgeist_geometry::PositiveLength` (`crates/fabelgeist-geometry/src/lib.rs:8–11`
and `length.rs:1–40`) admits exactly finite strictly positive f32 metres,
matching the wire predicate for `tile_metres`, including rejection of both
signed zero values. The role is tile scale, so numeric and unit fit alone do
not prove the complete recipe owner, public-construction contract or intended
policy. Texture Cargo has no direct geometry dependency at 95e; geometry's
direct manifest is captured, but the full dependency graph was not checked.
Error conversion and producer/consumer migration remain open. The decoder
admits a zero height range (`wire.rs:76–81`), so PositiveLength is not a fit
for that field. An animation-owned `NonnegativeLength`
(`crates/adventuresim-tactical-core/src/animation/measurement.rs:10–37`)
has the same finite nonnegative numeric admission including signed zero, but
its animation ownership, semantics and dependency direction are unresolved;
it is not a recommendation to depend on tactical-core.

The separate historical Lens 3 q2701 expressly maps its recipe/channel
admission concern to q2934 `SEGOVB-L3-TEXTURE-RECIPE-CHANNEL-ADMISSION`.
The wire checks uniqueness but no recipe/channel association; installation
uses a reserved destination `expect` (`runtime.rs:84–105`,
`runtime/handles.rs:1–110`). If a mismatched decoded map reaches that
installation after reservation, this is a conditional source consequence,
not an observed runtime incident or proof of supported malformed producers.
Required versus allowed sparse-channel policy remains open. This explicit
Lens 3 relation is not an automatic alias or additional count for q2355.

## q2356: camera focus and scene owner

`crates/adventuresim-tactical-client/src/presentation/buildings/streaming.rs:17–22,62–77,234–251`
stores an `Option<Vec2>`, samples the first gameplay camera's global x/z,
reuses its previous priority within ten metres and sorts by squared distance
to the building's typed centre. `ScenePlanPoint`
(`crates/adventuresim-tactical-core/src/scene_coordinates.rs:1–64,93–104`)
is a finite signed scene-local east/north point in metres. Building input
centres use that type, and the selected compiled layout retains the centre;
the inspected root spawn places x/z from it (`streaming.rs:178–207`).

The selected third-person rig computes an observation with a
`Position<Scene>` final endpoint and writes its metres to the camera transform
(`camera.rs:48–56,89–110`, `camera/rig.rs:19–97`,
`camera/rig/observation.rs:17–120`, `camera/collision.rs:66–81` and
`crates/adventuresim-building-generator/src/spatial_geometry/coordinates.rs:9–29,62–76`).
That strengthens finite scene-frame fit for this selected producer. It does
not prove every camera mode, hierarchy/global transform or admission of the
actual sampled snapshot is scene-local and finite. The inspected orbit demo
also writes raw camera transforms; invalid reachability was not established.
Streaming's `present` is scheduled in Update while the selected rig is in
PostUpdate before transform propagation; this is a source scheduling fact,
not proof of a stale frame or runtime effect. Resolve actual supported
camera modes, parent/global conversion and owner policy before a universal
fit or alias decision.

Related q67 `ADLER-L11-CITY-STREAMING-FAILURE-OWNERSHIP` concerns retained
failure context and processed versus resident counts, a distinct first
operation. Related q87 `ADLER-L11-VISTA-TREE-PLACEMENT-FRAME` concerns a
scenery support/renderer frame relationship. These remain separately owned
open-reconciliation records with a frame dependency; no automatic alias or
count is asserted here.

## Remaining boundaries

This packet provides no exhaustive producer inventory, no dependency-graph
proof, no global alias disposition, no other-lens or historical body credit,
and no implementation or product validation. Test, macro and doctest text
was inspected only. No product code, tests, builds, fixtures, shaders,
renderers or runtime probes ran. The coordinator owns the connected ledger,
publication and any later classification. Use `VERIFY-EVIDENCE-v2.py`; the retained predecessor is superseded as
explained in `VERIFIER-NOTE.md`. The portable verifier checks
integrity and optional local Git/transport membership, not semantic truth or
independent custody. The bounded privacy scan has the limits stated by its
separately sealed receipt.
