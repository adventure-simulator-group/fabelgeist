# Botanical API contracts

Climate accepts distinct temperature, precipitation and lapse-rate units.
Species retain those units through their adaptation parameters. Terrain planting
accepts a `GroundPoint` in world-space metres. Woody species, climbing species,
plants, vines, prototypes, stable modules and current collection locations have
separate identities.

Elapsed `Years` differ from `PhysiologicalAge` and `AgeRate`. Regulation stores
the latter two on modules; development consumes them without changing the
equations or their evaluation order. A `StepDuration` admits only a finite,
positive interval. Seeding, mortality and global shadowing are named policies.
Random generation accepts the shared `fabelgeist_determinism::Seed` while
retaining the existing SplitMix64 word stream.

Module parents and children use optional `ModuleIndex` values. Node layout is
`None` until geometry is built, then carries a `NodeOffset`. Compaction remaps
these references together. Environment and viewer packing explicitly produce
the native indices, sentinels, vectors and scalar units their numerical kernels
and GPU records require. Shader records and buffer layouts remain unchanged.

`BotanyError` distinguishes unknown species, blocked ground, missing attachments,
invalid regulation intervals and mismatched environment results. GPU failures
retain their underlying causes. Planting returns a typed collection index on
success. Catalog row admission returns an error for an unknown row; selecting
an admitted row is infallible.

Foliage and litter behavior belong to species roles. Display names are labels;
renaming a species cannot turn its needles into broad leaves or change its
ground-cover role. The viewer admits UI scalars into these shared contracts at
the controls and event boundaries.
