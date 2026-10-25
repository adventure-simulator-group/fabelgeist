The remaining texture-view contracts prevent completion of this draft.

At PR head `d2e9f2f2984d33073386f0267a673fb7ac6478a3`, the public texture
fields can relabel or replace a cached view independently of its native
resource. A same-format request can therefore return a cache with a different
actual interpretation. The binding consumers also check allocation format
rather than the selected view's effective format. Alternate-view validation
must establish that the supplied context scopes the device owning the texture;
otherwise native rejection can reach a different device's error sink.

The connected repair must preserve detached cached views, cache-first
precedence, explicit missing states, Arc sharing, and the logical/native format
distinction. Available native causes must survive construction, retained
pipeline state and the actual binding consumer.

The [#774 handoff](https://github.com/adventure-simulator-group/fabelgeist/blob/codex/774-handoff-20261021-records/handoff/774/HANDOFF.md)
claims the whole `ComputePass::record` receiving function, the pipeline
validation/cause lifecycle, and `shader::parse_naga`
(`RIG-OWNER-PIPELINE-CAUSE-088`). The [explicit ownership reply](https://github.com/adventure-simulator-group/fabelgeist/pull/774#issuecomment-6356111438)
retains all those claims with #774 and transfers no function to #826. The
invariant direction is agreed, but final receiving/error APIs and an integrated
implementation revision are explicitly unavailable. Its continuation will
finish audits and hand over before implementation. The shared dependency
therefore remains unresolved, rather than an accepted follow-up.

Current main is pinned at `990430b77a9f63548ffcb642074185f61ddb3322`, which
includes merged #774. The shared GPU pipeline/pass/parser bodies are unchanged
from the earlier pinned main; merging #774 has not completed these contracts
or transferred their ownership.
Independent texture-construction repairs remain isolated locally. No new source
has been pushed and no hosted CI has been requested while the connected
migration is incomplete. No tests, cases, helpers or assertions have been added.

Historical software-GPU numerical failures and the Creator process failure
remain historical validation limits; they have not been re-established on
current main. Final acceptance still requires complete audit waves, independent
Code/Rule/Comment reviews and required hosted checks on one final pushed
revision. This PR remains unmerged.
