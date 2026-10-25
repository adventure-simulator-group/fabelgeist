# Frozen policy 73b Comment Review

Reviewed immutable HEAD `73b6e3a2ddf319dec749e3b1ff4a25e99b9c6612`, tree
`c8f7763247eff6ae47b79973ef3be2051289b248`, against parent
`197eb8982c554203ac8652178cabe611d3652896`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`. Tracked source was clean at
review entry. All thirteen changed files were read completely through git
objects, including counts, quality, policy, presets and the full 2,210-line
renderer. JSON records exact hashes and affected-source coverage.

The three 197 findings are corrected: readback support is qualified by native
render acceptance; GPU groom prose distinguishes geometry reuse from atlas
upload; adapter limits no longer promise arbitrary full-groom fit. Public
renderer construction now documents exact device/queue and supplied-resource
provenance plus device-handled native failures.

Three wording findings remain:

- **73B-C1, P3, lod/policy.rs:60.** Empty says no groom geometry was prepared,
  but shadow() returns it for a card-only groom containing geometry. Say: “No
  geometry is selected for this draw.” Distinguish total absence from absence
  of a shadow strand representation.
- **73B-C2, P3, render/draws.rs:111.** The documented inclusive [0,1] random
  range and complete endpoint coverage claim conflict with half-open shader
  cuts. At amount 1, incoming excludes random 1 and outgoing retains it.
  Generated RNG values are [0,1), but public CardVertex values can contain 1.
  Document the exact keep.x <= random < keep.y cuts and preserved endpoint
  behavior, or restrict complete-coverage claims to generated [0,1) values.
  No shader change is implied.
- **73B-C3, P3, render/mod.rs:114.** FrameInfo.primitives says count drawn,
  while its implementation returns primary representation population. Blending
  can shorten that outgoing draw and add incoming geometry; shadows are also
  excluded. Say: “Population of the primary representation, excluding
  transition and shadow draws.” Keep this synchronized with the pending count
  family migration.

Named quantity documentation otherwise matches the inspected implementation.
PixelSpan retains signed/nonfinite numerical inputs; effective uniforms and
frame identity are checked for finiteness before publication. This does not
admit arbitrary policy geometry or resource relationships. Accumulation targets
require at least one sample, counts saturate, and the denominator widens before
addition. Shadow maps retain the sixteen-texel minimum with native upper limits.
Level ordinals require collection admission at use.

Empty bands, nonempty strands without levels and missing requested geometry
families have distinct typed failures. Entirely absent strand/card collections
select Empty; an unavailable incoming blend family can also fail selection.
NaN pixel comparisons choose the last band in a nonempty collection, while
frame finiteness admission remains separate. Canonical presets retain their
numeric catalog and shading choices. Dataset accumulation retains its minimum
through the new quantity.

Convergence reselects with the recorded pixel observation and compares policy
representation, blend, shading, minimum pixels, enabled shadow and resolution.
Scene/target replacement invalidates last/count/history. The actual moving pose
is validated before pose is excluded from the temporal identity. History toggles
between its two targets; a new mean starts at weight one, while movement retains
the deliberate temporal blend. These source contracts provide no completion or
performance guarantee.

The renderer's existing fixture module exactly hashes to
`44c66d149be12de0828289a8461e243c235facb1032230263538ebc471cfda5e`:
the same two tests, validate helper and five assertions. Dataset tests and all
three shader blobs are also byte-identical to 197. Matrix source was read fully;
card shader, Ratio, RNG and public card-input boundaries were inspected where
they govern the reviewed claims.

Required producer/count/head/native-layout work remains open. HairLods::counts
still constructs typed LodCounts using raw producers without adapters, and peer
GPU/policy/caller contracts have not been integrated. Owner #780's constructor,
reflection, Readback, completion, error/cache and lock migrations remain
required. The all-thirteen inventory and final integrated reviews are separate.

No source edits, project execution/checks/builds/tests/scanners/generators,
runtime, new cases/helpers/assertions/doctests, push, hosted CI or external posts
were performed. No compilation, browser/hardware/artistic, performance,
global-health or rollback verdict is supplied. Earlier detached-Wasm-view and
recorder-scope validation assurances remain withdrawn. Three wording findings
and required families prevent final clearance; all final independent reviews
and hosted gates remain required on the same final pushed revision.
