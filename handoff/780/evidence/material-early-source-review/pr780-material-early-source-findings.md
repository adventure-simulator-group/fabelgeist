# Early material source Code Review

This independent source review finds two actionable syntax defects in the
implemented owner/producer tranche. This is an early review, not clearance of
the unfinished receiving graph, the thirteen lenses, or the final PR revision.

Snapshot: HEAD `d000e848f2f5bc28b0728c2cd664c727a41ecf50` plus the 46
material paths recorded in the adjacent JSON report. Every captured SHA-256
matches `material_pbr_final_preinventory.current_files` at capture. Root is the
sole implementer. Baseline source is
`target/pr780-audit/material-pbr-before-d000e848f/source`.

## Findings

1. **EARLY-CODE-1 / P1: accidental fixture tail prevents parsing.**
   `crates/adventuresim-character-creator/src/skin_rules.rs:276-279` has a
   stray `t,` and an extra closing brace after the adapted `ArmorComponent`
   literal. The original existing helper has one literal closing brace and
   one function closing brace. Delete only the accidental tail. This is not
   an authorized fixture API adaptation and prevents the existing test module
   from parsing. The finding remains present in the captured bytes and was
   sent immediately to Root.

2. **EARLY-CODE-2 / P1: six struct fields end in semicolons.**
   `crates/fabelgeist-armor/src/material.rs:21-22`,
   `crates/fabelgeist-armor/src/construction.rs:254-255`, and
   `crates/adventuresim-character-creator/src/garment_material.rs:41,43`
   terminate the new `Srgb` and `Ratio` named fields with `;`. Rust named
   fields require commas. `Metal`, `Lacing`, and `MailWeave` therefore cannot
   parse, independently of the intentionally unfinished receivers. Replace
   these six separators with commas. The finding remains present in the
   captured bytes and was sent immediately to Root.

## Source reasoning and inspected scope

The complete new `encoded.rs`, `surface_material.rs`, and `components.rs`
owners were read. `Srgb`/`Srgba` preserve source float precision and transparent
array serialization without claiming finite or unit-range admission. Recipe
validators retain their numerical policies. The existing canonical sRGB
transfer is reused for linear projection; alpha is copied directly.
`ComponentAppearance` has the selected required `Inherit`, `Surface`, and
`Textile` payloads. It derives current textile material from the retained
recipe. Mutable textile operations consequently cannot leave a second stored
scalar material stale. No compatibility path or primitive adapter was found
in these inspected owners.

The complete material changes and relevant construction responsibilities in
the four fastener/eyelet files were inspected. The replacement enum preserves
the retained textile-versus-surface geometry decisions, component cloning,
and existing geometry math. The actual twelve `ArmorComponent` literal
adaptations were inspected, together with relief mutation, textile seam
selection, device textile assignment, example export and benchmark receivers.
No new fixture, case, helper or assertion was found in those changes. Existing
trim/textile/metal fixture values and benchmark assertion meanings are
preserved, subject to EARLY-CODE-1's accidental syntax corruption.

Typed metal, lacing, cloth, mail, UI and native metal-bake producer changes
were inspected. Existing arithmetic coefficients and accepted numerical
policies are retained in the changed responsibilities. The new picker adapter
passes linear channels to the native linear RGB picker and re-encodes only on
change; encoded byte picker behavior for mail remains unchanged. The new
terminal scalar projection and body appearance owner were also read, without
claiming closure of their still-changing consumers. The actual direct Color
serde dependency is present in all three manifests and relevant package
entries of both lockfiles.

The JSON report supplies all 46 exact current hashes, matching ledger hashes,
and per-file read extents. Full new-owner reads and changed-responsibility
reads are distinguished; unrelated algorithm bodies in large existing files
are not declared cleared. Existing `TransferFunction` source was also read.

## Limits

Final retained, Bevy, GLB, review-artifact and other receiving changes remain
in progress. Their known old API gaps are not new findings from this early
tranche. Unrelated generated-program changes were excluded. No local project
build, test, format, lint, scanner, parser probe or hosted CI was run. This
reviewer changed no source or fixture. A fresh complete receiving review and
same-revision independent final reviews remain required after repairs.
