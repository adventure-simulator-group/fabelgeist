# Implementation and verification

Worktree: `.worktrees/weapon-historical-audit`, branch `codex/weapon-historical-audit`.
Base: fetched `origin/main` at `f2b3fb27`. The original checkout and its uncommitted work were left untouched.

## Completed remediation

The independent historical/artistic reviewer assessed all 44 Rust recipes, 42 browser presets, and 20 named composer combinations analytically and visually. [Final review](after-review.md) records acceptance and [historical assessment](historical-assessment.md) records the source evidence and family-specific qualifications. The CSV has all 106 entries. Images use actual generated triangles; hashes identify accepted exports and contact sheets.

The recipe changes correct pike length, polearm shaft/socket and heel dimensions, cutting-head thickness, sword/dagger hilt proportions, hammer/mace scale, hand-bow placement, and axe/glaive profiles. Comparative presets and unusual module combinations retain explicit study status. The crossbow provenance identifies the altered museum source rather than assigning the whole object an unsupported mid-century date.

Rust now integrates the same resolved component solids used by rendering. Mass, material mass, centre of mass, mean transverse rotational inertia, balance and reach derive from geometry and material densities. Flat/fullered blade thickness semantics now match actual forte depth. Tactical and strategic combat geometry use the generated grip-to-tip reach, removing the catalog reach adjustment. Generator version is 8. The subsequent contact consolidation removes obsolete weapon-stat and character-capability fields from the database schema; client bindings were regenerated through `just generate-db-client`.

Summed component solids include authored overlaps and omit some hidden manufacturing details. These are construction estimates rather than Boolean-unioned or complete manufactured replicas. Browser and Rust holding-point conventions differ. Generic slider values are not certified historical designs.

## Contact and handling consolidation

The follow-up design is implemented in the shared contact model. Generated melee precision replaces authored swing/stab precision, penetration, intrinsic accuracy, precise-critical flags and weapon damage-type flags. Physical length and imbalance determine handling accuracy. One continuous precision controls concentrated injury, armor resistance and access to actual openings, with a conserved energy budget. Smooth fist/haft/pommel contacts use zero concentration. The inventory presents one precision stat.

[Contact-model documentation](contact-model.md) describes the equations, the requested reference anchors and the remaining modeling limits. [Independent precision review](precision-implementation-review.md) and its 44-row CSV cover the calibration. The staff correction uses the same broad-contact footprint convention as clubs and maces. Ranged payloads and natural attacks use the same scalar but remain authored abstractions outside the generated melee grammar. Skill distributions and animation choices remain semantic inputs; no separate accuracy or damage bonuses derive from them.

The new physical/contact model exposed a short-weapon range-management issue: sword fighters could remain at haft-only distance. They now retreat until their generated striking surface can be used, while long weapons retain their preferred-measure behavior. The bounded 32-seed duel sweep has no timeouts after this correction.

## Historical-remediation verification (before contact consolidation)

- Rust: 935 shared-core tests and 33 weapon-model tests passed. They include actual-volume conservation, an independently analytic rod, translation invariance, taper effects, all section thicknesses, historical envelopes, attachment validity, icons and deterministic fuzz.
- Consumers: `cargo check -p adventuresim-stdb-module -p adventuresim-tactical-server --lib` passed.
- Scoped Clippy: `cargo clippy -p adventuresim-weapon-model -p adventuresim-core --all-targets -- -D warnings` passed.
- `just fmt-check` passed. `just lint` verified database bindings, then failed on pre-existing armor-model and character-creator Rust quality findings. Those source files are unchanged from the fetched main base. No quality baseline was raised; only resolved weapon-model debt was reduced.
- Browser: 150 bounded tests passed; all 62 named defaults/combinations passed production validation during export. The bounded suite includes geometry, manifold/winding checks, camera fit, malformed-input rejection, composer fuzz and pairwise endpoint tests, and new physical-property regressions. The two broad all-preset slider sweeps are excluded from the final bounded run; an earlier stale exhaustive run was stopped. This is not an exhaustive certification of arbitrary parameter combinations.

Exact test logs and rendered plates are under `output/weapon-audit/`; that generated evidence directory is intentionally untracked. Reproduce the accepted review with the exported definitions, commands in the modeler README, and `render-mesh-review.py`.

## Contact-consolidation verification

Final logs use the `consolidation-*` names under `output/weapon-audit/`. The inventory unit/DOM suite passes 29 tests. Tactical consumers compile with their tests. The final 938 shared-core and 33 weapon-model tests pass; they include the continuous contact calibration, handling monotonicity, energy conservation, condition scaling and the short-weapon range regression. `just fmt-check` passes. Strategic-web also compiles with its tests. Generated-client freshness verification and scoped Clippy pass. `just lint` verifies client freshness, then stops at the inherited armor-model/character-creator quality findings; those source directories are unchanged from `f2b3fb27`. The repository-wide quality gate still has unrelated armor-model/character-creator debt inherited from the fetched main base.
