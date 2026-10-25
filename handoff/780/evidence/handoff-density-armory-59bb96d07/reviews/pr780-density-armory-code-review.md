# PR #780 scoped independent Code Review: density and Armory

Snapshot HEAD: f2c7422fd6a0f995eb90b72cc9c2a5bd12de4d35, with the eight reviewed source files uncommitted.
Verdict: no actionable correctness defect found in this connected tranche.
This is a source-only verdict, not final combined same-SHA clearance, a
project-check result, or clearance for broader material/PBR/physical work.
No source edits, project checks, execution probes, tests, new fixtures,
helpers, cases or assertions were made. Report files only were written.

## Read scope and contracts

Read the complete new material/density.rs, complete new armory/exhibit.rs,
current armory.rs, armory_scene.rs including its existing fixture, armory_ui.rs,
armor_metal.rs including its existing fixtures, and their complete deltas.
Read fitted_catalog_item and downstream fit/selected callers, the retained
StudioCache admission/publication path, actual GeneratedArmor field owner,
Metal density constant, Ratio unit owner, OrbitCamera payload and catalog
placement admission/retention. Parent all-thirteen preinventory was read as
a contract map, not as clearance. Unrelated protected generated-program dirty
sources were not reviewed in this tranche.

## Density reasoning

The inherent method belongs to actual mutable GeneratedArmor, and the old
Creator free implementation is removed. Public material module exports the
error/result owners; no old/new execution path or forwarding adapter remains.
The existing Armor thiserror/Math dependencies support the new definitions.
Actual Creator fitted_catalog_item uses ? only inside the existing
is_plate_steel gate. The fixture calls the method directly. The real error
implements std::error::Error through thiserror and converts to anyhow via ?;
VertexAddress retains TryFromIntError as its source rather than formatting it.

DensityInput::new performs all required admission before indexing:
complete triangle triples, equal position/UV lengths, finite values in both
columns (including unused rows), host representability and membership of
all index words. It admits empty geometry with matching empty rows and no
indices. It deliberately does not certify normals, skin, components, trim,
face tags or morphs, which the operation does not read.

The exact arithmetic order of supported ordinary input is retained: visit
all three edges of each triangle, including duplicate shared edges; subtract
in f32, widen to f64 before square, sum in original component/edge order,
sqrt in f64, accumulate f64 surface and chart lengths, divide in f64, narrow
to f32, multiply by Metal::TILES_PER_METRE (currently 4). There is no epsilon,
clamp, new ceiling, averaging or unit substitution in this tranche.

Nonfinite subtraction results/edge sums are rejected before use. A chart
length sum of zero returns unchanged, including empty triangles. Positive
chart plus zero surface retains the original zero scale policy. A ratio
that cannot be represented after f32 narrowing/density multiplication is
rejected. All actual replacement coordinates, including unused UV rows,
are checked for finite products before mutation. Only the admitted Ratio
escapes the immutable borrow; the caller's exclusive &mut self prevents
source drift between admission and the second multiplication pass. No
fallible operation remains inside that pass, so an error leaves all UVs
unchanged. Underflow and signed zero are not silently repaired or clamped.

## Armory integration reasoning

ExhibitFit carries Pending, Fitted(GeneratedArmor), or Failed(anyhow::Error).
The fake initial failure string is gone. mesh() admits only Fitted into
bounds, triangles and scene construction. errors() returns real failed-fit
causes and the independent optional preview cause, excluding pending.
The new error is not Clone and no caller needs it to be; anyhow's retained
error meets the thread/Bevy Send+Sync requirements of the existing worker
and resource structure. The worker returns actual anyhow::Result values,
adds fitting-placement context, and joins in the existing chunk/input order.
The parent maps each result once into its payload-owning retained state.

A failed spawn now sets preview_error rather than replacing the first
successful fit with a diagnostic. Successful fitted geometry survives for
future finish changes. Every actual respawn assigns .err(), so a later
successful spawn clears the previous preview error. Existing finish/retry
trigger semantics and partial scene publication limits are unchanged;
this tranche does not promise atomic spawning of an entire exhibit.
Formatting occurs at terminal hover/editor/status output, using alternate
anyhow formatting to preserve the causal chain. Labels and problem counts
consume the same errors iterator.

The extracted module's public-to-crate Exhibit/ExhibitFit names are reexported
through the parent Armory module; private finished/stale/preview_error fields
are visible to their owning parent, and the existing scene child consumes
mesh() through the parent import. No source import/visibility cycle or new
feature/manifest dependency is introduced by this extraction.

The saved-camera match moves only a Copy payload: OrbitCamera and OrbitGoal
actually derive Copy. Entering retains the current camera exactly once;
leaving restores the captured value then sets the slot None, preserving the
old transition without expect. The stale-exhibit/missing-body branch now
publishes terminal UI status and returns before fitting rather than panic.
It does not claim a universal admission certificate for arbitrary manual
mutation of every public Armory vector/selected index.

Actual catalog producer proof: ItemCatalog::wearable uses is_wearable, which
requires a material and at least one placement with nonempty surface. Exhibit
construction retains the actual immutable placement snapshots and filters by
that same nonempty-surface condition; therefore normal producer records have
at least one placement. fitted is initially built with exactly one Pending
per placement and replaced by one worker outcome per placement. item()[0]
and shown()/bounds slice remain supported by that producer relationship.
This is source-backed, rather than an assumed nonempty input contract.

## Existing source preservation and limits

Static exact source comparisons (not executable tests):
- Existing armor_metal fixture module is identical after replacing only the
  old free-function call by armor.scale_to_metal_density().unwrap().
- Existing Armory scene fixture module is byte-identical.
- Whole parametric_equipment.rs is identical after reversing the sole method
  call and ? propagation adaptation.

The companion JSON records the booleans and source identities. No fixtures,
cases, helpers or assertions were added. Broader PBR frontier, material schema,
validation-cause barriers, InventoryID, native scalp/Grid, physical and
protected generated-program changes remain outside this scoped verdict.
Hosted project gates and independent final combined reviews remain pending.

## Whole-source SHA256

The final prose successor was reread in full for density.rs and exhibit.rs.
The zero-scale doc now says positive accumulated chart-edge length, and the
finished field doc describes the last preview attempt, including failure.
These corrections match the implementation; the other sixteen fingerprinted
files did not drift. The table below pins the final current successor.

| Source | SHA256 |
|---|---|
| `crates/fabelgeist-armor/src/material/density.rs` | `94db413bd0eeefd9fc860abb8eee368ee1d11a55cb48cbb4cd7ba2113fd0d665` |
| `crates/fabelgeist-armor/src/material.rs` | `12debec1a00cb2ff34e5ef09978a4a3c523bde4bfb1926a1dbef86b7decaf6ea` |
| `crates/adventuresim-character-creator/src/armor_metal.rs` | `00f657217f6dc1b7b5677176fad2153836a7aabc133a1c7b518bba391a607c16` |
| `crates/adventuresim-character-creator/src/parametric_equipment.rs` | `d71953e80a22f666f64d7733ae41279fe18700e1f6b0531ee1cfd404f4c95559` |
| `crates/adventuresim-character-creator/src/armory.rs` | `74f95648715bc2456ec986093c649779f02d3c773607d24af7ff0ccfc4ca18ef` |
| `crates/adventuresim-character-creator/src/armory/exhibit.rs` | `b2f4f006cf78814970012ab94d85d178176813c53cd2fbd9768ff2b17349c221` |
| `crates/adventuresim-character-creator/src/armory_scene.rs` | `ff67d80dde117a3e4dd23e394d0a5bc6efe5049ca032c0e5fc0a30eaaeec3429` |
| `crates/adventuresim-character-creator/src/armory_ui.rs` | `dea23f6efe94e9b5439739548675be1b0cfd109a3e153e2a083d7a816db43aeb` |
| `crates/fabelgeist-armor/src/design.rs` | `86dac554a42b48cf1547e5656337fb718b4a4fc7e231a30009ce00ff7207dd00` |
| `crates/fabelgeist-armor/src/lib.rs` | `d4b6a4af8b3137b359c2d08f8ef03445e100edc2d1280ca74ec816212af31264` |
| `crates/fabelgeist-armor/Cargo.toml` | `5a51ad1fc7ac2e848b3d8fd8b016f5e5d12c4e433c5ff600353d0530d0b6eba1` |
| `crates/fabelgeist-math/src/units.rs` | `0c76a1a79a18e833d21fc2ffb011e2a351139ee7a299df13df12fdbcf3c27492` |
| `crates/adventuresim-character-creator/src/studio_scene.rs` | `e486c21e58ee4c7f689bceed15e636afc2b11d12fc0989b02ca4a5e128c85fcf` |
| `crates/adventuresim-character-creator/src/equipment_catalog.rs` | `da3139ef3625393db9041ad0680dc244b0f81235b86ea4519a87c9299763bdd6` |
| `crates/adventuresim-core/src/equipment_assets/selection.rs` | `162b898d78feeae3507d19844b4b11c999c50edd7609680e5ef7c2fac89486d6` |
| `crates/adventuresim-character-creator/src/studio_cache.rs` | `6d7378554d92fcefc39d39434744bdb50dd98358ce1596f571652a65ce4bf013` |
| `crates/adventuresim-character-creator/src/studio_generation.rs` | `3031832a29165a676c81ee7b80a78ee5463dab02dbc69bdd959cd8bd3fd5d066` |
| `crates/adventuresim-character-creator/src/main.rs` | `566a69fc6cd50bd7d7ea5c0347567c39f2bdc37e9f8004675dc5328e3751e12e` |
