# Native review component extension: thirteen-lens discovery

Discovery is complete for this connected correction at HEAD
`d000e848f2f5bc28b0728c2cd664c727a41ecf50` plus the pinned material working
source. **PBR-CODE-1 is open; repair and validation are not started.** This
pre-repair inventory is not clearance of PBR or any broader family. The JSON
records exact before hashes and individual lens status. Twenty-two exact
sources are archived under
`/tmp/pr780-review-component-extension-before/source`.

The review inspected the whole shared example module, whole skirt and outfit
callers, whole binary review exporter, whole scalar projector, whole library
module wiring, whole Python armor receiver and the relevant main CLI call.
Canonical source component/color/PBR/metadata definitions and actual secondary
JSON receivers were read. Source accounts distinguish the complete files from
responsibility-only reads. Searches located candidates; their actual bodies
and callers provide the evidence below.

## Concrete final repair graph

Create `src/review_component.rs` in the Creator library as the single native
component serialization owner. It exposes `pub struct ReviewComponent<'a>`
with private fields, derives Serialize, and moves the existing explicit
`TryFrom<&'a ArmorComponent>` there. Fields retain the current native view:
role, borrowed vertices and indices ranges, copied hinge, borrowed optional
mount, and optional checked `LinearSurfaceMaterial`.

Keep `SurfaceProjectionError` and the RGB conversion in their existing
`surface_projection` owner. The new type certifies its scalar material
projection; it does not certify geometric ranges or invent a shape validator.
No public fields, Default, Deserialize, raw native constructor, second RGB
formula or second error taxonomy is needed.

Expose the new module unconditionally in library `lib.rs`. Both the binary
review exporter and the independently compiled example module import this
same library type. Remove the binary-local struct and conversion; do not leave
an alias or forwarding owner. Route every source component in the example
through `ReviewComponent::try_from`, retaining order and propagating its actual
typed error. Project the components before writing their review record; doing
so before `write_textile_maps` also avoids starting that new producer's PNG
publication when scalar projection already fails. This does not promise a
transaction for later image/file IO failures.

Planned changed paths are exactly `src/lib.rs`, `src/review_export.rs`,
`examples/textile_review/mod.rs`, and the new `src/review_component.rs`.
Existing source/color/error and metadata dependencies are already direct
Creator library dependencies; no manifest or lock change is needed. There is
no feature dependency on offline GLB export: the example should retain its
current non-offline body-model independence. No new test, case, helper,
assertion or doctest is required or authorized.

## Sequential lens inventory

1. **Producer propagation — actionable PBR-CODE-1.**
   `TextileColor::material` and `TextileAppearance::material` supply scalar
   appearance; `device_textile_materials::apply` retains Textile in the source
   enum; `assemble_recipe` retains those components in GeneratedArmor. The
   main `review_export.rs:33-47,135-139` projects them, but shared
   `examples/textile_review/mod.rs:63-78` serializes source components.
   `skirt_review.rs:22,116-121` establishes the real Python consumer, whose
   `render_armor_review.py:73-75` needs `material`. The outfit caller at
   `textile_outfit_review.rs:88-95` shares the same unfinished producer.
   **Acceptance:** both source-component artifact producers consume the single
   public native view, in source order, with no source-enum serde at either
   `components` assignment. Retained source API remains the clean required
   enum. Repair depends on the new common library owner and module wiring.

2. **Existing type owners — reuse, do not reconstruct.**
   Armor owns `ArmorComponentRole`, `ArmorHinge`, `PlateMount`,
   `ComponentAppearance` and `SurfaceMaterial`. Color owns `Srgba`; Math owns
   `Ratio`; Creator `surface_projection.rs:11-15,42-60` already owns the sealed
   native linear factors and checked conversion. Ranges are borrowed native
   geometry metadata, not GPU sorting or launch counts.
   **Acceptance:** the new view uses these actual owners and the existing
   native record. No parallel material record, primitive application PBR tuple,
   new color converter or semantic GPU count wrapper is introduced. Existing
   path, identity and mesh-array values stay at their actual artifact boundary.

3. **Admission — scalar proof only.**
   Source components are public mutable/serde records, while scalar projection
   rejects nonfinite decoded RGB, alpha and factors. `Inherit` has valid absent
   material. Source float leaves deliberately impose no range policy.
   **Acceptance:** native view fields stay private; its only construction is
   fallible `TryFrom<&ArmorComponent>`, calling the existing checked projector
   for a present material and preserving its real error. No Deserialize,
   unchecked public construction, normalization, unit clamp or range
   strengthening is added. Role/range/hinge/mount copying must not be advertised
   as full geometry admission. Empty components and zero-valued channels remain
   accepted where currently valid.

4. **Invariant operations — derive and publish coherently.**
   `ComponentAppearance::material` derives Textile scalar values from the
   actual current recipe. Finite encoded floats may overflow native decoding;
   the canonical projector checks the result. Borrowed range/mount fields pin
   the source through native serialization.
   **Acceptance:** collect every fallible component projection before the
   corresponding review-record write and preferably before that example's map
   writes. Preserve order; do not filter failed components, substitute white,
   swallow errors or publish a partially projected component list. Later IO
   failure remains ordinary fallibility, without an invented transaction.

5. **Aggregate states — same component and same index order.**
   Native metadata and material must come from the same source component.
   Example `write_textile_maps` uses component enumeration for its map index;
   source fields must not be reordered or filtered by the new projection.
   Main review adds real design/frame information separately.
   **Acceptance:** retain exact role, vertex/index ranges, hinge and mount from
   each component, and the same list length/order. Preserve separate top-level
   mesh, rig, morph, design and textile-map metadata. Do not invent an aggregate
   mesh validator or new index restriction in this correction; existing export
   admission is separately open.

6. **Type ownership — move the existing native owner down one layer.**
   The current private binary `ReviewComponent` cannot be imported by the
   example. Library `lib.rs` already exposes `surface_projection` independent
   of offline export; both consumers depend on the Creator library.
   **Acceptance:** one focused public library `review_component` module owns
   the actual struct and conversion. Remove the binary definitions. Avoid a
   re-export-only compatibility module, binary dependency, duplicate schema or
   a dependency on `export` behind `offline-creator`. Existing Cargo edges
   suffice. Root owns these planned files; no protected native function is
   touched.

7. **Redundant authority — native view is a terminal snapshot.**
   The enum intentionally removed independently retained scalar/textile fields.
   Main binary native material is already computed by the checked projector.
   **Acceptance:** do not restore source `material` beside `appearance`, retain
   two component-view definitions, or cache native scalar material across
   recipe mutation. The serialized view may contain copied derived factors
   because it is an immediate terminal artifact, borrowing authoritative
   metadata until serialization. Source serde and native review serde remain
   explicitly different contracts.

8. **Behavior preservation — close an observed protocol mismatch.**
   Original component serde included material derived by
   `device_textile_materials::apply`. The source enum removes that key;
   Python's fallback is gray. Main binary now emits checked linear material,
   the required native Blender interpretation.
   **Acceptance:** the example emits that same canonical linear record and
   exact native keys, including null material for Inherit. Preserve metadata,
   filenames, image bytes/order on success, rig and morph data. Source schema
   stays the required enum with no fallback. Added projection errors propagate
   honestly. No 8-bit quantization or additional transfer is introduced. No
   existing case, helper or assertion is added or behaviorally changed.

9. **Enum shape — source choice versus actual native optional factor.**
   `ComponentAppearance` owns its Surface/Textile payload; the native protocol
   intentionally needs only `Option<LinearSurfaceMaterial>`.
   **Acceptance:** retain the source enum unchanged. `None` must mean Inherit,
   not a swallowed projection failure. Do not leak source textile payloads or
   add a second native tag with correlated optional fields. Hinge and mount
   are independent metadata and retain their existing optional shapes.

10. **Derivability — derive mechanical serialization, keep checked mapping.**
    The current view derives Serialize; manual TryFrom both derives scalar
    material and propagates conversion failure. That semantic conversion is
    not equivalent to a fieldwise derive.
    **Acceptance:** move the Serialize derive and explicit checked TryFrom
    intact. Do not handwrite serialization, derive an unchecked From or add
    Deserialize solely for boilerplate reduction. No new Clone/Copy/default
    traits are needed for the transient borrowed view.

11. **Type granularity — one narrow native view earns its distinction.**
    ArmorComponent is an authored mutable source schema with encoded material
    or recipe payload; ReviewComponent is a borrowed native wire view with
    checked linear scalar factors. Their different contracts justify separate
    types. LinearSurfaceMaterial already owns the only scalar proof needed.
    **Acceptance:** keep that distinction, without a universal review schema,
    new scalar wrappers, source-shaped intermediate adapters or duplicate
    finite validators. Borrow existing ranges and mount identity rather than
    clone unrelated geometry into an admission abstraction.

12. **Magic strings — preserve the actual native protocol.**
    Native fields role/vertices/indices/hinge/mount/material come from the
    existing derived view; Python reads those exact keys and material's
    base_color. Source appearance tags are a different clean schema. Example
    paths, IDs, placement labels, names and top-level JSON tokens are actual
    existing artifact/CLI boundaries. Python's right-placement branch,
    camera/render strings and authored variant names are unchanged and outside
    this protocol correction.
    **Acceptance:** preserve native keys and role serde names; do not branch
    on error prose, add a Python compatibility parser or decode source enum
    tags in the native receiver. No new string identity or closed catalog is
    required. The copied type's diagnostic cause remains its typed owner.

13. **Magic numbers — native factors and geometry metadata have owners.**
    The new view needs no numeric policy literal: factors arrive from the
    canonical native projector, and ranges/hinge/mount retain actual source
    metadata. Existing skirt Permille profile literals are authored catalog
    inputs; Python triangle width, coordinate axes, fallback color and camera
    numbers are existing native rendering math, unchanged by this correction.
    The panel examples directly author native material factors rather than
    derive encoded source colors; they are not broken by enum serialization.
    **Acceptance:** no new tolerance, cap, RGB coefficient, tuple, phantom
    component or GPU count conversion. Empty list serializes as empty; alpha
    and scalar zero stay unmodified; the same canonical transfer handles each
    present material exactly once.

## Other producers and explicit dispositions

The actual `components` JSON producers in `cloth_panel_review.rs:48` and
`prepared_panel_review.rs:109,143` were read in full. They already author native
material records directly and do not serialize source ArmorComponent; the enum
change therefore does not remove their `material` field. They remain unchanged
native examples, not a second source-to-linear implementation. No source owner
is fabricated solely to make those native literals pass through this converter.
The role/range-only bake receiver is unaffected. The museum renderer's real
Material.parse/import_review/override_material path consumes the same native
material record, with no Python RGB transfer added.

All thirteen cells are complete discovery with explicit repair criteria. The
single actionable defect remains open. New module implementation, independent
repair reviews, actual compilation/hosted project gates and final global audit
waves have not occurred. UV-R1/R2 and broader material/Grid/GPU families remain
open. No source edits, local checks, tests or probes were performed.
