# Canonical material Comment Review

Exact peer source: `c6659f6cfa4610f066c659411ade6636e21bd3b5`, tree
`12f2d808ab9d659999aaec627e02e594897c8beb`. Root context is
`cac5a7c33a83d1c08a5e55d43d912967edf06814`; pinned main is
`990430b77a9f63548ffcb642074185f61ddb3322`. This is reference review,
with no Root copy or whole-family acceptance.

## Actionable finding

**MATERIAL-C665-C1 (P2), one connected documentation repair.**
`crates/fabelgeist-armor/src/gpu/metal/textures.rs:12-13` promises that
fencing makes intermediates round the same on any device, and
`crates/fabelgeist-armor/src/gpu/metal/finish.rs:10-11` promises the same
result on any device. Actual `host_float/mod.rs:23-28,51-57` documents
rare transcendental rounding differences, fused-FMA requirements tested
on the executing device, subnormal flushing and bounded trigonometric
reduction. Material admission permits finite subnormal values; it does
not remove these limits. Seeded scheduling and fences do not establish
universal device bit identity.

Use consistent wording such as: “Intermediates use `host_float` fences to
retain the intended operation order, subject to that module’s documented
numerical and device limits.” Keep the seeded scheduling and atomic
maximum explanation. This requires prose repair, with no arithmetic,
admission, shader, fixture, or benchmark change. Owner: #780 material
shader wrappers; Root native helper bodies remain protected.

## Coverage and source conclusions

All 22 owning and six reference-only manifest files were freshly read in
full, and all 28 hashes match. This includes the complete 1,782-line
export module and all existing fixtures. Eight additional pinned
dependencies were fully read: shader plumbing, `host_float`, procedural
ornament WGSL, export shell/rigged/validation modules, and both owning
Cargo manifests. Eight earlier receiver accounts carry only verified
exact identity and their recorded full or bounded line coverage. JSON
lists every path, blob, SHA-256, line count, and coverage limit.

The checked tile/map extents, immutable validated-recipe borrow and
engraving/decoded-image association match this source. Actual image
admission checks signed wrap addition separately and checks computed
sampling coordinates at the four affine corner extrema, including floor
and floor-plus-one. It does more than admit a decoded u32 element count.
Concrete path IO/image/native causes remain represented through the
checked map and actual preview/export/material receivers.

Bake submission/readback and preview/export comments do not establish
global GPU health or transaction rollback. Export shell handling retains
actual UVs, map size and linear normal/metal-roughness data; this is no
new depth-parallax, finite-detail, or visual-quality guarantee. Existing
metal fixture diff against main contains API operand/getter/admission
adapters, preserving existing cases, values and oracles. Other existing
fixtures were read; universal main/#779 fixture provenance for the mixed
application/geometry graph is not newly cleared here.

## Required dependencies and limits

Resolve C1, then re-review the exact successor. Actual Root composition
of the full canonical producer/caller/error/cache/export/dependency/lock
graph remains required. Six reference-only files are not authorization
for native facade or mixed-export replacement. Preserve Root-owned
MetalGpu, Buffer and context helpers. Deeper garment/rig/socket geometry
and every arithmetic implementation were not wholly audited here; the
owning numerical contract and generated WGSL plumbing were inspected.

The Rule reviewer separately owns all thirteen preinventory lenses.
Final Code, Rule and Comment Reviews and required hosted checks must
pass on the same final pushed revision. This report supplies no compiled,
native/browser runtime, global, hosted, or final PR clearance.

Only source inspection and ignored report bookkeeping were performed.
No source edits, project execution/check/build/lint/test/scanner/
generator/runtime, new tests/cases/helpers/assertions/doctests, push/CI,
or external posts occurred.
