# Independent Code Review: native publication/closest/docs tranche

No actionable defect found in this scoped eight-file delta.

The review began at HEAD `4ddddf5856d269d36cf898d59f34af7a30186479` and the
final fingerprint read observed HEAD
`47c23647be6d0e764076237b8a0a692906663d0a` after the separate material commit.
All eight reviewed current files still exactly match the ledger's
native_scalp_current_discovery.first_tranche_files identities. Comparison base
is the saved pre-edit source tree at
`target/pr780-audit/native-scalp-preinventory-4ddddf585/source`, not another
review's verdict. Exact before/current hashes and preservation comparisons are
in `/tmp/pr780-native-scalp-first-code-review-fingerprints.json`.

No source edits or project checks, tests, probes, formatters, quality scanners,
hosted checks or external posts were performed. This review does not clear the
remaining Grid/count/Surface/root/guide migrations, numerical proof obligations
or PR #780 as a whole.

## Findings and dispositions

S5, previously reported discarded publication failure: repaired at
`gpu/scalp.rs:395`. The final boundary batch now uses `batch.submit()?` before
constructing GpuScalp. The actual KernelBatch::submit consumes its encoder,
returns Result<SubmissionIndex>, propagates the context/native cause before
running application callbacks, and documents that success is host submission,
not device completion. HairGpuError already has the required From<anyhow::Error>
edge. Discarding only the successful index is appropriate here: the boundary
buffers are submitted on the same retained queue before later consumers. Any
submission error now prevents publication without inventing a new error or
changing retry policy. The preceding two metadata submissions/readbacks and
all output ownership are unchanged.

S8, previously reported invalid closest-point pruning: repaired at
`gpu/wgsl/guides.rs:181-190`. Every triangle is now considered by the existing
closest_on_triangle calculation. A boundary point near a far endpoint of a
valid coarse triangle can reach its actual zero-distance projection instead
of being excluded solely because its first vertex is more than 4 cm away.
Triangle traversal is still ascending t; the existing `d <= best` tie update,
maximum distance, best_point/best_triangle, failure marker, barycentric
projection and normal/UV interpolation remain exact. The change removes an
unsupported size assumption without narrowing uploaded Surface admission.
The shader bindings, structs and loop syntax remain coherent. More triangles
are evaluated by design; no speculative replacement acceleration is required
for this correctness repair. Existing degenerate/nonfinite/source-admission
issues remain separately open and are not solved or hidden by this delta.

The orphan guide-map Rustdoc has moved from the empty end of the grow impl to
the actual record_set_map method. Its owning body sets set.guide_of through
record_map and then marks the set mapped. Documentation now attaches to its
actual receiver without a new item, wrapper or helper. All related bodies are
unchanged.

The revised documentation matches current source: grow submits roots/readback
before the remaining groom batch; cut performs two grouped metadata reads;
scalp reuse depends on head geometry, hairline and sampling recipe; Scratch
retains capacity while requests fit and grows capacity with its existing
checked policy; guide_comb uses one eight-lane workgroup per guide; Head's
ellipsoid/smooth-union field and finite-difference normal are approximate.
Surface::closest's doc now attaches to closest instead of boundary. These
wording changes do not certify unrelated numerical claims elsewhere in those
modules or full CPU/native equivalence under malformed input.

## Preservation evidence

All eight complete sources were compared against their saved originals.
After reversing just the final submit error propagation and removed prune,
non-comment source is identical in every file. This comparison removes
whole-line comments and whitespace; source reading separately confirms the
two exact meaningful edits and doc attachments.

The complete CLOSEST/barycentric helper block is byte-identical, as is the
entire guides shader source after edge_closest. Existing fixture tails in
head.rs, surface.rs and gpu/mod.rs are byte-identical. No tests, cases, fixture
helpers or assertions were added or changed. No native buffers, counts,
allocation formulas, ABI fields or launch projections changed in this tranche.

## Current source identities

| File under crates/fabelgeist-hair/src/ | SHA-256 |
| --- | --- |
| gpu/scalp.rs | 40486b2fcb0f2a2742a0a43ac588c71ba46f47495236def99dd6f0b27c821447 |
| gpu/grow.rs | b4907574e60ff614aef176c62a138331eb3bc047cb799db0fe6f82d5f92b6c26 |
| gpu/grow/maps.rs | 7b8b6157448397c687e7da311bb8ca6a933455934080093196854e20c1c0c0bc |
| surface.rs | 0cf3b1a2d8fa1e50f2494291f44e803aff7501f7d5f92763c87ef54a5db0ac0d |
| head.rs | c95b6fcaaec095658c85f5c325781ecd1c349ddefb7b7eecd018ab61b500f709 |
| gpu/buffers.rs | e82d568414119b0fd6077ab911224dc9eee4eefb2d6a5cf7ff38c98dea9a78e1 |
| gpu/mod.rs | f5696d4762b8a0414cd89406639a49edeabbd1acbd330df25eeee220291cac9a |
| gpu/wgsl/guides.rs | 7eb85b1be589db745f6411247e2b22ee076a63ebf49f97dcec29fab7caf14043 |

Coverage includes actual submit/error owners, the complete changed cut and
closest paths and their preserved downstream boundary/guide protocols,
record_set_map, actual grow batching, Scratch allocation, Head's distance and
gradient, and Surface boundary/closest source. The earlier source discovery
reports identify the remaining native family and unread broader consumers;
no historical pass or search is used to grant them clearance here.

No local compilation or runtime/native/browser evidence is claimed. Final
combined independent reviews, all thirteen lenses and required hosted checks
must still pass on one final pushed revision after the connected migrations.
