# Frozen lower 2f Comment Review

Reviewed clean HEAD `2f3880bfabd08674857bd47015e7c47287a691dc`, tree
`40d8b3798593f02ece6881718c1085e72a6aff78`, against parent
`f0a98e58d54b673e017cf5ae633e9b6c46b5b609`. Pinned main remains
`990430b77a9f63548ffcb642074185f61ddb3322`. All seven changed files and
all of the parent delta were read. The companion JSON gives exact source hashes
and distinguishes fresh reads from exact unchanged f0 coverage.

One small factual wording finding remains:

- **2F-C1, P3:** `pipeline/entry_point.rs:1` says entry identity is admitted
  independently of source declaration order. `First` intentionally resolves the
  first compute entry in source order; only `Named` selects independently of
  that order. Replace the introduction with: “Explicit compute entry selection,
  including source-order first selection.” The variant documentation already
  states both contracts accurately.

No additional owned comment mismatch was identified. The selected compute name
is retained in `ComputeReflection`; compile, compiled lookup and interface
validation use that identity. `Named` has no fallback, and errors preserve its
requested name. The texture binary builder authors and explicitly selects
`main`. Reflection continues to union resource usage and sampling requirements
over all compute entries; it does not promise a selected-entry-only layout.
The cache retains entry-name outcomes and native observation policy separately.
A device-handled successful return is not proof of observed native validation.

The upload reorder preserves its prose, example and behavior. Fresh full reads
of shader admission and source-cache responsibilities supported the entry and
cache review. Current completion and lower buffer-read responsibilities were
also read completely. Their f0 provenance, aligned covering-range, logical
prefix, cleanup and callback-delivery contracts remain accurate. Twenty-seven
other f0-accounted files were verified byte-identical; their previous coverage
is carried without claiming fresh full body reads. The complete f0 report is
preserved.

Required owner integration remains open. Kernel still calls the old two-argument
prepare API; FastPath still names removed `ReflectionData`. Other constructors,
ResourcePipeline, shape and producer families must consume the agreed canonical
selector/reflection contracts. The retained grouped Readback API and blanket
no-stall prose, ignored completion results and garment completion promises,
Metal/body/part callers and application error/cache chains remain required
#780 dependencies. None is cleared by this checkpoint review.

No source edits, project execution/checks/builds/tests/scanners/generators,
runtime, new cases/helpers/assertions, push, hosted CI or external posts were
performed. SDK evidence from the exact pinned f0 review is retained; no fresh
whole-SDK read or native/Wasm compilation is claimed. There is no whole-family,
combined all-thirteen, browser runtime, performance or final hosted clearance.
Earlier recorder-scope acceptance assurances remain withdrawn. Final integrated
reviews and required hosted gates must pass on the same final pushed revision.
