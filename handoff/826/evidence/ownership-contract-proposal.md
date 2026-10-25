# Proposed #826 / #774 shared-function agreement

Status: proposal only. No agreement has been received. Nothing in this document
transfers ownership, authorizes editing claimed functions, or clears the audit.

PR #826 starts at `d2e9f2f2984d33073386f0267a673fb7ac6478a3`.
Pinned main is `3a50b708381ad00c116445c2c999a49c3943088b`.
The isolated integration head was `71a2d1ffc6203c251f87a4c4e759670058d620b3`.
Independent repairs are now committed locally at
`6bab9585a75c6e9fbec713f68aa968bf2e7e03e5`; `current-snapshot.json` records
their source fingerprints. This is not a source-complete or validated PR head.

## Exact claimed bodies

The supplied handoff's `rig_garments-claims.json:4949-5008` claims:

- The whole `ComputePipeline::validation_error` retained-cause record.
- `get_or_create_pipeline`, `get_or_create_pipeline_validated`,
  `validate_interface`, `new`, `prepare`, and `compile`.
- The whole `shader::parse_naga` native parse-cause/context contract.
- The whole `ComputePass::record` validation-cause receiving contract.

Claim receipt: `RIG-OWNER-PIPELINE-CAUSE-088`, with the connected
`RIG-OWNER-ARMOR-NATIVE-MATERIAL-087` receipt. Claim state is ownership only,
with the initial thirteen-lens source freeze. The handoff requires a single
whole-function repair for every overlapping identity/cause family.

No claimed body has been edited by #826. ZIP/NumPy and sorting owners are also
untouched. The public `create_bind_groups` helper has no matching exact claim
and no repository callers, but its public texture-binding contract is affected.

## Concrete texture contract offered for agreement

The texture owner will keep one canonical `TextureFormat` and native SDK
handles. It will restrict independent writes to cache/native/format/extent/usage
metadata and admit the correlated record only in its existing constructors.
Explicit operations may remove either optional cached view or optional native
allocation port; all four existing availability combinations remain supported.

A same-logical-format request still clones the exact cached Arc before requiring
an allocation handle. It returns `MissingView` if that cache is absent. A
changed-format request requires the allocation handle and returns
`Uninitialized` if it is absent. Alternate success does not replace the cache.
The existing four tests keep their inputs, order, assertions and oracle intent;
only existing helper bodies may be adapted to production state operations.

Logical requested format is distinct from linear native allocation format and
from the effective format of a selected view. Inherited storage usage maps an
sRGB request to its linear counterpart exactly as before. The error records the
original logical request and actual native cause. A selected binding view must
carry its native handle, effective format and dimension together, owned once by
the texture layer. It may use a crate-private named record; no duplicate
reflection enums or independent per-shape error owners are proposed.

Whole-view native return ports may still expose the SDK Arc directly. Internal
binding assembly must obtain the selected-view metadata from the texture owner
instead of reconstructing it from `Texture::format()`. Cube whole, face and
array views retain their Cube/D2/D2Array dimensions and six/one/six layers.

The admitted native resource must also retain or verify its owning SDK device.
A changed-format call can currently pass a different context, scope device B,
and create a view on device A: wgpu sends that failure to A's stored error sink.
The SDK source establishes this representable API weakness; no current caller
misuse has been observed. Cache-first returns need no device use and keep their
precedence. The final shared selection/error contract must account for this
provenance without inventing a parallel device identifier or context policy.

## Shared consumer and cause contract requested from Adler

The whole `record` repair must consume that selected-view contract for 2D, 3D,
cube and generic views. The existing `create_bind_groups` public adapter must
consume the same effective-view facts where the operations actually agree.
Missing view, absent allocation, view validation failure, reflected dimension or
format mismatch, and an invalid pipeline must remain distinguishable. A native
allocation's linear format is not proof of its selected sRGB view's format.

The canonical pipeline/parser error work must retain available concrete causes
through parse, pipeline validation, retained shared state, compile and record.
Neither `Option<String>` nor contextual formatting may replace the actual cause.
Clonable retained failures can share a typed cause through an Arc; the actual
error owner and API names must be supplied or agreed by the #774 owner.

A rejected native pipeline must not be returned from a success-only cache after
validation state has been reset or forgotten. Cache/readback/lifecycle changes
must be integrated once across every claimed function, preserving ordering and
existing fixtures. No partial competing patch by #826 is authorized.

## Agreement needed

Choose an owner for each whole-function repair and the canonical error/selected
view contracts. Either Adler provides an agreed source commit and contract for
#826 to consume, or explicitly transfers the exact claimed bodies and connected
requirements. Supply the integration revision and any cross-PR dependency/order
constraints. Silence or an elapsed timeout is not agreement.

Only after these dependencies and connected migration families are complete may
#826 push, request hosted project checks, repeat complete thirteen-lens waves,
and obtain final independent Code/Rule/Comment reviews at the final pushed SHA.
No new tests, cases, fixture helpers or assertions are permitted. Historical
software-GPU and Creator process limitations remain historical unless reproduced
by the permitted hosted checks at the relevant source revision. Do not merge.

## Published agreement request and launch dependency

[The direct request to Adler](https://github.com/adventure-simulator-group/fabelgeist/pull/774#issuecomment-6354715079)
now includes the existing #780 coordination request. No reply or agreement has
been received. #779 is pinned at
`71f9cab8a5f97760e2e61ca4c74876b715f8aebc`: its canonical `WorkgroupGrid`
receiver replaces the three primitive axes in `ComputePass::record/dispatch`.
An empty grid returns before batch parameter admission and does not increment
`RecordedDispatchCount`. #826 must preserve that agreed receiving contract
without replacing current-main preparation, folded dispatch or callbacks with
the older #779 function bodies. Integration belongs to the agreed shared owner.
