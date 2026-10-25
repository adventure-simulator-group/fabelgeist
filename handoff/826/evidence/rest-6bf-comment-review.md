# Rest-prefix checkpoint Comment Review

Reviewed exact `6bf03a8d04bf246c996bbbfa17c1925ebd150c38`, tree
`daddbb27e119b442e4bc77b30de8d0c84b357fd8`, parent
`10cc1a69b1a9a5f4e54daa112477cc56b7487f45`. Main remains
`990430b77a9f63548ffcb642074185f61ddb3322`.

Fully read both changed files and the affected level receiver. Bounded layout
and SDK handle reads, exact source hashes and carried identity limits appear
in JSON. Renderer fixtures, POD declarations and all shaders retain exact bytes.

One finding: **6BF-C1 (P3)** at `render/deformation.rs:155` says all remaining
resources belong to the simulation source. Binding 6 is the new renderer-owned
per-level params buffer. Replace that sentence with “Other bindings retain
their own allocation ranges.” The DeformationBuffers documentation correctly
makes no simulation validation claim.

The no-work record description from 10CC-C1 is fixed. Canonical resident data
bytes now determine both capture and restoration copies; logical zero skips
the copy while positive placeholder binding extents remain in slots 0/1.
Cloning an old rest Buffer retains the same native allocation, not another copy.
Old logical dynamics remain borrowed until replacement/clear submission returns
normally. This is not synchronous native acceptance, GPU completion or rollback
of all external/native effects. Configured device handling remains unchanged.

Other simulation extents, Device/source identity, copy capabilities, retained
rest associations, canonical producers and native cards remain required. These
are not proved by native buffer packaging or this prefix repair. Final complete
all13 waves, independent reviews and hosted checks on the same final pushed
revision remain required.

No source edits, local project checks/builds/tests/scans/generators/runtime,
push or CI occurred. No tests/cases/helpers/assertions/doctests were added or
executed. This is bounded source evidence, not integrated, compiled,
whole-family, all13, hosted or final PR clearance.
