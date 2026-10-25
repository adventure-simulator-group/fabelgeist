# Density and Armory Comment Review

**Scoped CLEAR on the final eight current files below.** The two comment
findings raised during this review were corrected and independently re-read.
This is not complete material, all-thirteen, native scalp, whole-PR or CI
clearance. Existing broader material/schema/PBR/physical findings remain open.

HEAD: `f2c7422fd6a0f995eb90b72cc9c2a5bd12de4d35`, with the fingerprinted dirty
tranche. Worktree:
`/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator`.

Applied the Comment Review skill and governing root/crates/Armor instructions.
Freshly read all eight files in full, plus complete current GeneratedArmor design
source and the relevant canonical Ratio contract. Read the actual whole-source
diff and compared against the saved source at
`target/pr780-audit/density-armory-before-f2c7422fd/source`. The all-thirteen
preinventory ledger is context; it supplies no review clearance by itself.

No edits, new tests/cases/helpers/assertions/doctests, project checks, builds,
scanners, probes, local executions or hosted checks were performed. Existing
fixture code was inspected as source, not executed.

## Findings and final dispositions

- **DENSITY-C1, P3, density.rs:40-42 — resolved.** Initial prose said a
  zero-length surface with a nonconstant chart retains zero scaling. The actual
  branch depends on positive accumulated chart triangle-edge length; unused UV
  rows can vary without contributing to it. Final prose now states that exact
  accumulated-edge condition. `DensityInput::scale` returns None when total
  texture edge length is zero; otherwise finite zero surface produces scale0.
- **DENSITY-C2, P3, armory/exhibit.rs:22 — resolved.** Initial moved wording
  called `finished` the finish its preview meshes show. `refit` records it after
  both successful and failed preview attempts. Final doc says the finish used by
  the last preview attempt, including failure. That accurately explains the
  state compared to the next requested finish for respawn decisions.

Reversing only those two prose corrections reproduces the initial review hashes
`a7aee6566dea5834c762c8d5ffc722575525241f720306ae843eee4baf7eda90`
and `8efe614f555f895997d3f9744cd555ba0896ebd3ba08a0827d677287058f9c08`
respectively. No executable source changed during that correction/recheck.

## Supported current contracts

`material/density.rs:37-56` describes a method on GeneratedArmor operating on
model-space metre positions and dimensionless UVs. Its admission is deliberately
scoped: complete triangle triples, position/UV row agreement, finite values in
all those rows, host-representable native indices and membership in those rows.
It does not certify normals, skin, components or morphs; the doc explicitly
states that boundary. `GeneratedArmor` retains publicly mutable primitive arrays,
so admission at the operation is necessary and the docs do not pretend prior
construction already proved these relationships.

`DensityInput::scale` accumulates all three edges of every triangle, including
shared-edge repetitions, preserving the previous weighting. `edge_length`
subtracts f32 components before f64 square/sum/sqrt. The ratio narrows to f32
before multiplication by Metal's existing four-tiles/metre constant. Comments
state that operation order, and it is the actual source. This review does not
infer universal physical texel size from the global weighted scale.

After finite accumulated lengths and finite scale are established, every UV
replacement product is checked under an immutable borrow. Only then does the
exclusive method perform its replacement pass with the same scalar. Thus the
"before any coordinate changes" and exclusive-access comments match the actual
failure/mutation order. All input rows are checked, including unused rows.
Empty admitted triangle sets and constant finite charts leave UVs unchanged;
finite zero surface and positive accumulated chart-edge length retain the prior
zero scaling. Malformed/nonfinite/unrepresentable inputs now return typed errors
instead of reaching unchecked indexing or publishing nonfinite coordinates.

Creator `fitted_catalog_item` retains its exact plate-steel condition and calls
the new inherent method with `?`; there is no old forwarding helper. The existing
square fixture alone changes its density API call to the direct method plus
unwrap. Its coordinates, thresholds and assertions are retained. This source
comparison is not a runtime validation claim for the fixture or whole fitter.

`armory/exhibit.rs` owns pending/fitted/failed states. Pending has no fabricated
error, mesh reads return only the Fitted payload, and error reads return only
actual Failed payloads. Bounds and triangle counts skip both Pending and Failed.
A preview failure is independently retained as `Option<anyhow::Error>` and does
not overwrite a successful fitted mesh. Its source is not converted to prose
while stored. The final doc correctly distinguishes fitting outcomes and later
preview outcomes.

The worker returns nested anyhow Results; lower `fitted_catalog_item` errors gain
placement context through `with_context` and remain causal errors. The absence
of a parametric design has an actual local absence diagnostic. The scene takes
only fitted meshes and propagates actual build/material/preview errors. Each
respawn replaces or clears `preview_error` with the latest spawn outcome, then
records the last attempted finish. UI list/hover/editor materializes the error
chain with `{error:#}`; no prose classification or fake not-fitted error remains.
Camera exit consumes the actually matched saved Option payload, and the missing
body path emits its local terminal status instead of an expectation panic.
These changes do not claim automatic preview retries or GPU completion.

## Source extraction and preservation evidence

`/tmp/pr780-density-armory-comment-source-comparisons.json` records the bounded
comparisons performed without running application code:

- Whole material.rs equals before after removing only density mod/export.
- Whole parametric_equipment.rs equals before after reversing its sole density
  call; its complete existing fixture module is therefore unchanged.
- Whole armory_scene.rs equals before after reversing its one mesh projection;
  its existing fixture module and all presentation/camera bodies are unchanged.
- Whole armory_ui.rs equals before after reversing the two terminal error-chain
  formatting changes.
- Whole armor_metal.rs equals before after restoring the removed helper and
  reversing the one existing fixture API line.
- Existing Armory select, inherent refit, ready, finish_for, arrange, target,
  Region label/of bodies are byte-exact despite item ordering/extraction.
- Extracted Exhibit item/shown bodies are byte-exact. Bounds/triangles replace
  Result flattening with the new Fitted projection; errors deliberately carry
  original fitting and independent preview errors.
- The edge-length arithmetic expression equals the prior lambda after variable
  naming/whitespace normalization; the native scale statement is verbatim.

These comparisons verify the named regions, not an unperformed numerical or
interactive run. No new fixture body or documentation example was added.

## Limits

No complete fitting geometry, skin/normal/morph validity, full public Armory
collection invariant, interactive rendering, concurrency scheduling or native
GPU behavior was established by this source-only comment review. Those are
separate audit/check obligations. Existing unchanged broader material comments,
including the previously inventoried waviness precision claim, are not silently
cleared by importing a density submodule. MAT-R1/R2/R3-R5 and physical/native
families remain pending. Final all-thirteen and independent review clearance,
with required hosted checks, must be tied to the final pushed revision.

## Final exact source snapshot

| Source | SHA-256 |
|---|---|
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/fabelgeist-armor/src/material/density.rs` | `94db413bd0eeefd9fc860abb8eee368ee1d11a55cb48cbb4cd7ba2113fd0d665` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/fabelgeist-armor/src/material.rs` | `12debec1a00cb2ff34e5ef09978a4a3c523bde4bfb1926a1dbef86b7decaf6ea` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/armor_metal.rs` | `00f657217f6dc1b7b5677176fad2153836a7aabc133a1c7b518bba391a607c16` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/parametric_equipment.rs` | `d71953e80a22f666f64d7733ae41279fe18700e1f6b0531ee1cfd404f4c95559` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/armory/exhibit.rs` | `b2f4f006cf78814970012ab94d85d178176813c53cd2fbd9768ff2b17349c221` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/armory.rs` | `74f95648715bc2456ec986093c649779f02d3c773607d24af7ff0ccfc4ca18ef` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/armory_scene.rs` | `ff67d80dde117a3e4dd23e394d0a5bc6efe5049ca032c0e5fc0a30eaaeec3429` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/armory_ui.rs` | `dea23f6efe94e9b5439739548675be1b0cfd109a3e153e2a083d7a816db43aeb` |
