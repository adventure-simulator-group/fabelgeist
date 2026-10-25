# Root baseline pre-repair inventory

Pinned Root `8c600f727d1eecf21dd241f02739b825edef5000`, tree
`1c141f5f1a28cb532cb51b472ba7e8fcc01486ae`, against main
`990430b77a9f63548ffcb642074185f61ddb3322`. All thirteen lenses have
separate source-backed ledger cells. Discovery of this bounded deletion
proposal is complete; repairs and final combined validation remain pending.
The baseline is byte-identical to main at this pin.

## Exact proposed removals

Remove these seven complete TOML records, without adding or raising debt:

| Table | Path under `crates/fabelgeist-gpu/src/` | Item | Proof |
| --- | --- | --- | --- |
| files | data/gpu/texture/texture_2d/mod.rs | — | Entire file 461 raw lines, including tests/comments, below 500. |
| files | data/gpu/texture/texture_cube/mod.rs | — | Entire file 315 raw lines, below 500. |
| functions | data/gpu/bind_group.rs | create_bind_groups | Old 18-line module only reexports the function. Actual body in assembly.rs:17–42 is below 100 raw lines. |
| functions | data/gpu/compute/pass/mod.rs | ComputePass::record | Entire owning module 76 raw lines, below 100; whole method read. |
| functions | data/gpu/texture/texture_cube/mod.rs | TextureCube::blit_texture_to_face | No old-module body; actual blit.rs:102–118 body below 100. |
| functions | data/gpu/texture/texture_cube/mod.rs | TextureCube::render_face_shader_raw | No old-module body; shader.rs:31–55 body below 100. |
| functions | data/gpu/texture/texture_cube/mod.rs | TextureCube::render_shader_raw | No old-module body; shader.rs:67–90 body below 100. |

`validate_debt` in `analyze.rs:227–319` rejects missing and below-limit
records. `scan.rs:425–449` counts only meaningful, non-test source lines.
An entire raw file/whole-function span below the configured bound is therefore
a sufficient upper-bound proof without running or reproducing the scanner.
Do not copy the old ceilings onto their extracted owners.

## Every remaining matched record

The complete TOML/Git intersection covers 138 changed paths and 82 baseline
records across 22 source paths: 6 file records, 29 function records and 47
raw-string finding records. Every record has its own disposition in the JSON.
After the seven removals, 75 matched records remain.

The renderer still has actual oversized production source and its three
baselined functions. In particular, `resize` retains 103 meaningful
signature/body lines by manual source reading. Its 103 ceiling must remain;
`new` and `render` also remain well above100. Their exact reduced ceilings
should be reconciled from the final combined hosted baseline export.

All other matched file/function records belong to existing or not-yet-received
canonical compute, extraction, native groom, host-card or mesh families. Their
current bodies/items remain present at this Root pin. They are not authorized
for deletion based on a future peer implementation. The 47 raw-string branch
records still belong to the current Gather, Map, Reduce, Scatter, Stencil and
Stream parsing/building functions. Main-to-Root diffs establish that no Root
native extraction removes those branches; their future canonical migration
requires a separate final-source reduction. This does not certify occurrence
counts or peer-family no-growth acceptance.

The affected unchanged `Blitter::new`188-line record is explicitly retained: its
entire source blob is identical to main, and new cached-view callers do not
remove its body. Other unrelated baseline entries remain outside this bounded
source audit and must keep their exact bytes.

## Dependencies and limits

Root owns the baseline mutation. Preserve the canonical 500/100 limits,
existing schema/derives and surviving record spelling. The broad GPU
`boundary_adapter` scope suppresses eligible literal findings; it does not
exempt structural file/function debt. No registry or validator weakening is
needed. No new types, API aliases, exceptions or placeholder records are
justified by this deletion.

Canonical producer/caller receiving work remains required. Inspect the final
combined hosted baseline export and apply only relevant reductions; no larger
ceilings or new debt entries. Required hosted project checks and independent
final reviews must still use the same final pushed revision.

No source or baseline was edited. No project scanner, baseline generator,
compilation, build, check, lint, test, generator or runtime was executed. No
tests, cases, helpers or assertions were added. Source identities, complete
matched-record dispositions, thirteen contracts and acceptance criteria are
in `root-baseline-8c-pre-repair-all13.json`. This is not final clearance.
