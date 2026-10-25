# Validation-scope checkpoint Comment Review

Reviewed exact `8cdcfa08f4e22804c384955c749c5640154f94c7`, tree
`934eb5dd2adbbbd85faa7a1cdb9b3679db71ace6`, against parent
`6bf03a8d04bf246c996bbbfa17c1925ebd150c38`. Main remains
`990430b77a9f63548ffcb642074185f61ddb3322`.

Freshly read all 22 changed files in full, including owning comments, errors,
texture guide and affected receiver functions. SDK guard/poll/pop/drop and
unchanged submission/completion boundaries were read with bounded coverage
recorded in JSON. That report includes exact source hashes and retained fixture
identities. Truncated combined output was followed by bounded full source reads.

One finding: **8CDC-C1 (P3)** at `compute/pipeline/mod.rs:115`, also lines 145-146
and `pipeline/error.rs:16`. Broad promises that rejected/complete outcomes are
cached and available to every clone/lookup now exclude an important case:
Progress, possibly retaining a native rejection, escapes uncached and removes
the earlier device-handled entry. Qualify these docs: completed native
validation outcomes are shared, including rejection; progress failures remain
uncached and retryable. The guide already states this correctly. Retrying does
not guarantee success.

Other reviewed scope/error documentation matches actual source. Finish attempts
one wait and always reads the scope after a returned poll error. Both concrete
poll and native causes remain available. Explicit finish observes results;
implicit SDK guard drop pops/discards them. Scopes remain on one thread without
crossing awaits. No timeout is explicitly requested, but backend failure remains
possible. Ordinary native rejection messages and direct SDK causes are retained.
The operation/observation resolver preserves both values and the primary
operation cause. Prior binding 6 ownership wording is corrected.

Cached/missing-view branch priority, Arc sharing, device/requested-format
semantics and native view descriptors remain unchanged. Progress does not
publish a successful pipeline/cache outcome. The guide correctly excludes
rollback of recorded commands, consumed slots or submitted converter work, and
keeps submission separate from completion and global health. Browser/borrowed
paths retain configured device handling without an observed native verdict.

All five peer wait sites remain required: KernelBatch dispatch, FastPath
prepare, cached layout construction, uniform arena allocation and ranged copy.
Canonical producers, Sim/rest/device/source/copy/manual/card and caller/lock
families also remain open. Root retains ownership of the complete all13 wave.
No source edits, project checks/builds/tests/scanners/generators/runtime, push
or CI occurred, and no tests/cases/helpers/assertions/doctests were added.
This is source-only evidence, not family, integrated, compiled, hosted or final
PR clearance. Final independent reviews and required hosted gates must pass on
one final pushed revision.
