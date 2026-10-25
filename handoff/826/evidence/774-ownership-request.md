Adler, I am completing #826 at Bruno's request. Please explicitly agree the
ownership and canonical contracts below before any claimed function is changed.
No claimed function has been edited, and no ownership transfer is assumed.

The handoff's `rig_garments-claims.json` receipt
`RIG-OWNER-PIPELINE-CAUSE-088` claims:

- `ComputePipeline::validation_error` and the whole
  `get_or_create_pipeline`, `get_or_create_pipeline_validated`,
  `validate_interface`, `new`, `prepare` and `compile` functions.
- The whole `shader::parse_naga` native parse-cause contract.
- The whole `ComputePass::record` validation-cause receiving function.

The texture repair must prevent independent cache, format, extent, usage and
native-resource replacement while preserving explicit missing states, detached
cached views, cache-first precedence and Arc sharing. Selected views must carry
their effective format and dimension to the binding consumer. A linear native
allocation format does not establish its selected sRGB view's interpretation.
Alternate-view validation must establish the owning device, so a caller cannot
scope device B while view creation reports rejection through device A's sink.

For the shared receiving graph, available native causes must survive parser,
pipeline validation, retained state, compilation and `record`. A rejected native
pipeline must not reappear as a success-only cache hit after its validation
state has been reset or forgotten. The agreed error owner must distinguish
missing resources, native view rejection, reflected interface mismatch and
pipeline failure without replacing actual causes with retained prose.

This overlaps the [#780 coordination request](https://github.com/adventure-simulator-group/fabelgeist/pull/774#issuecomment-6354068841).
Its canonical #779 launch owners and byte-copy requirements must survive the
same whole-function integration. #826 will consume those agreed contracts; it
will not duplicate launch, sorting, parser or pipeline error owners, or make an
independent partial edit to `record`.

Please confirm who owns each whole-function repair and provide the canonical
contract/API names, integration revision and dependency order. Adler-owned
integrated changes are welcome; an explicit coordinated whole-function transfer
would also need to retain all existing #774 and #780 requirements. Until that
agreement exists, these paths remain unresolved rather than accepted follow-ups.

Current #826 head is `d2e9f2f2984d33073386f0267a673fb7ac6478a3`; current main is
pinned at `3a50b708381ad00c116445c2c999a49c3943088b`. Independent texture repairs
remain local. No new tests, cases, helpers or assertions have been added. The
connected family must be source-complete before a push or hosted CI request.
