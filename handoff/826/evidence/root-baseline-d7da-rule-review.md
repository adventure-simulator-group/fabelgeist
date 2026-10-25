# Root baseline deletion Rule Review

Pinned `d7da77bb9d21200170b522b8e6a1f7aa4aef2bab`, tree
`868959fcaec6f7b624c0c462c6bea4a6a70528d4`, against
`8c600f727d1eecf21dd241f02739b825edef5000`.

No actionable finding in this bounded baseline-only change. The diff removes
exactly two file records and five function records. Independently removing
those seven exact TOML blocks from parent bytes reconstructs the successor
byte for byte. Every surviving record, ceiling, order and other byte remains
unchanged. File records fall from 209 to 207, function records from 484 to 479;
all 1,369 raw-string finding records remain unchanged. Only the baseline path
changes; checker, source, configuration, fixtures and shaders are identical.

The removal proof follows actual owners, not only old record names:

| Removed record | Current source proof |
| --- | --- |
| Texture2d file | Whole module 461 raw lines, including tests and comments, below 500. |
| TextureCube file | Whole module 315 raw lines, below 500. |
| create_bind_groups | Old module is an 18-line re-export; actual assembly.rs:17–42 is 26 raw lines. |
| ComputePass::record | Complete owning module is 76 raw lines; method starts at line 42. |
| TextureCube::blit_texture_to_face | Actual blit.rs:102–118 is 17 raw lines; no body remains in old module. |
| TextureCube::render_face_shader_raw | Actual shader.rs:31–55 is 25 raw lines; no body remains in old module. |
| TextureCube::render_shader_raw | Actual shader.rs:67–90 is 24 raw lines; no body remains in old module. |

All seven source targets and their extracted owners were read completely.
The checker owner is unchanged: `validate_debt` at analyze.rs:225–319 rejects
missing and below-limit retained entries; scan.rs:425–449 counts a subset of
raw lines. The raw upper bounds therefore suffice for these removals without
executing or reproducing the scanner. Actual configured limits remain 500/100
at rust-quality.toml:1–2. This follows crates/AGENTS.md:166–178: retain the
no-growth ceilings and apply relevant reductions without raising debt. No
exception, suppression, alternate owner, compatibility API or checker change
was introduced.

Correctness is established only for the seven-deletion mutation. Completeness
of the wider canonical graph, remaining canonical baseline records and exact
final reductions remains open. The final combined hosted baseline export and
required hosted checks must still run on the final pushed revision, followed
by all required final reviews. This report does not certify remaining debt
counts, string occurrence counts, compilation, native behavior or whole-PR
clearance.

No project scanner, generator, baseline command, build, check, lint, test or
runtime executed. No checker function, production source, fixture, assertion
or helper was edited. Only ignored review reports were written. Exact source
hashes, coverage distinctions, removed records and complete baseline comparison
are recorded in the adjacent JSON report.
