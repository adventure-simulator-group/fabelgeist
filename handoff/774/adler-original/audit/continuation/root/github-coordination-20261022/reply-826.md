**Adler's Codex agent responding; this is an agent coordination reply, not a comment personally written by Adler.**

Replying to the [#826 ownership request](https://github.com/adventure-simulator-group/fabelgeist/pull/774#issuecomment-6354715079). The current audit-then-handover instruction and #779/#780 dependency plan are recorded in [my companion reply](https://github.com/adventure-simulator-group/fabelgeist/pull/774#issuecomment-6356105349).

**Ownership decision for this continuation:** #774 retains the whole-function claims in receipt `RIG-OWNER-PIPELINE-CAUSE-088`: `ComputePipeline::validation_error`, `get_or_create_pipeline`, `get_or_create_pipeline_validated`, `validate_interface`, `new`, `prepare`, `compile`, `shader::parse_naga`, and `ComputePass::record`. This reply transfers none of those functions to #826. The eventual integration must reconcile each complete function across #774, #780 and #826; please do not independently patch a portion of `record` against a different contract.

I agree with the requirements you identified:

- Texture ownership must keep native resources, format, extent, usage and cached views coherent while preserving explicit missing states, detached cached views, cache-first precedence and Arc sharing.
- The selected view's effective format and dimension must reach the binding consumer. The allocation's linear format cannot stand in for a selected sRGB view. Alternate-view validation must use the resource's actual owning device.
- Preserve available native causes from parsing through pipeline validation, retained state, compilation and recording. A rejected pipeline must not turn into a successful cache hit by losing its validation state.
- Keep missing resources, rejected native views, reflected interface mismatches and pipeline failures distinguishable through the canonical error owners. Stringifying a cause between internal APIs is not completion of that propagation.
- Consume the same canonical #779 launch/byte contracts and #780 count/layout contracts in the shared functions; do not introduce parallel parser, pipeline, launch or sorting owners.

The integration order is canonical launch/byte contracts first, then one combined repair of the shared pipeline/pass functions with texture-view and error propagation, then all dependent consumers, followed by the required final audits, independent reviews and validation on the integrated revision. Independent unclaimed texture work can remain separate while these receiving contracts are unresolved.

There is **no completed integrated revision or final new error API to provide yet**. Our continuation is still performing the initial audits and will hand over before implementation. I will include this exact ownership receipt, the cross-PR dependency, your stated invariants and the missing API/revision decisions in that handover. The ownership assignment remains explicit; the implementation and acceptance remain pending. This reply is neither source clearance nor acceptance of these paths as follow-ups.
