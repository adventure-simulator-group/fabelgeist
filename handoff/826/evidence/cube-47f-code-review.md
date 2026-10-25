# Cube native-program Code Review

Head `47f37a07687bc47347cedb9e274c2f273f9deae6`; parent `9b9d422e011ac23ee1e453483eb4356d0dd14b9f`.

No new actionable Cube implementation finding in bounded receiving source; complete storage/caller graph remains unintegrated and not compilable/accepted as a whole.

SP-CUBE-NATIVE is source-remedied across both raw shader methods, blit and
fast copy. All six changed files were freshly read in full; unchanged lower
view/context/cause owners were independently identity-revalidated.

All native phases and actual causes: Both raw methods admit array view first,
then canonical effective storage spelling. record_shader observes
shader/uniform/layout/group/pipeline/encoder/record through one infallible
context.observe_validation closure; scope drain occurs before Result mapping.
Both observe and submit map the actual error into
ComputePassError::Native(Arc<actual wgpu::Error>); Error::source exposes it. No
early fallible return is inside this observation closure, so no
OperationAndObservation wrapper is necessary or missing. First captured SDK
Validation cause is retained; SDK scopes do not promise to aggregate every
native error.

Connected blit/copy: blit selects source view, then target face view, then
observes all raw shader/sampler/layout/group/render-pipeline/encoder/pass
commands and calls submit afterwards. Fast copy retains destination
presence/device first; exact size/logical format/layer/no-layer eligibility;
then source device; its native encoder/copy phase is immediately observed and
submit still owns finish/deferred transfer validation. Fallback branch and all
copy origins/extents preserved. Native encoder-state errors can be immediate;
actual transfer resource validation may remain deferred to finish, so neither is
assumed exclusively.

Missing/device/cache/effective-format: Existing array/face/view selectors and
selected-view record are byte-identical. Default/no allocation returns
Uninitialized before format lookup; wrong context returns WrongDevice before
native creation; same logical cached-view behavior on Texture2d/3d/Cube and
missing-view state remains unchanged. Raw generated storage spelling and native
StorageTexture format both use effective_format(self.format), consistent with
selected array view; unsupported exact format returns StorageFormatError, never
rgba32float fallback. Blit render target uses that same effective view policy.
No successful native allocation/view or cache entry is published on failed
observed rendering.

Borrowed/browser/provider semantics: observe_validation is unchanged and only
pushes/pop scopes when native blocking_validation is enabled. It has no poll or
completion wait; browser and borrowed paths retain configured device handling
and do not provide a synchronous validation verdict. Existing view
ValidationScope may retain its prior wait; no additional rendering wait added.
Context.submit remains unchanged with separate finish/queue scopes, actual cause
retention, no completion promise and no rollback claim.

Native lifetimes, modules and Rust traits: Cube child modules can access
existing pub(super) fields and sibling private ancestor methods under normal
descendant visibility. Native bindings keep the same layout/group relation; SDK
bind groups retain resource ownership after local uniform/sampler handles drop.
wgpu error native/wasm trait policy and Cargo dependencies unchanged. Public
inherent method APIs remain at TextureCube; no compatibility alias/duplicated
compute-pipeline owner added. New source and native resource responsibilities
are cohesive; files35/89/213/266/312/421 lines, all below500.

Successful numerics and exact source: All3 raw WGSL blocks independently byte-
matched exactly once. Face uniform remains f32 time + native u32 CubeFace.index
+ two f32 zero pads; AllFaces remains four native f32 values. Workgroup math
div_ceil8 and dispatch depth1 or existing CubeFace::ALL.len()==6 preserved. Blit
filterable sampled2D layout, clamp/linear sampler, full-screen triangle
geometry, REPLACE, Load/Store and source/target admission order unchanged.
Native resource diagnostic labels are consolidated by the shared render helper;
semantic error causes remain actual native objects.

Required incomplete dependencies:

D-STORAGE-GRAPH: Current ResourceDescriptor::to_wgsl_storage_format still calls
f.to_wgsl_storage_format().to_string() at compute/mod.rs50 even though canonical
TextureFormat now returns Result. Current protected TextureBinaryOp builder
already expects canonical descriptor Result/typed base_type. Complete agreed des
criptor/signature/addressing/ResourcePipeline/recipe/BoundaryMode/stride/caller
family must be composed before source/compile acceptance. This is a known
unintegrated receiving dependency, not a second independent patch or source-
clear exception.

D-REDUCE: Canonical Min/Max empty-result consumers and exact owner receipt
remain required as identified in preinventory.

D-EXTRACTION: Do not receive isolated extraction WGSL/lib exports; complete
producer/caller/count/mesh/creator/manifest-lock and generated-shader families
remain held.

D-FINAL: No local checks authorized here; all13 root postwave, three independent
final reviews and required hosted checks must agree on same final pushed SHA, no
merge.

Read-only source/Git/provenance review only. No source edits, local
builds/checks/lints/tests/scanners/generators/runtime or new
tests/cases/helpers/assertions, push/CI/external posting.

Existing fixture/source identities are source evidence, not executed behavior.
Malformed shader handling is proven against SDK call/scope timing, not runtime
reproduced.

Known unresolved ResourceDescriptor/caller API and global producer/manifest
integration are explicitly not acceptance.

No source compile/whole family/global13lens/wholePR/final clearance.

Exact source hashes, three independently matched shader identities, canonical
format proof and all13 bounded reconciliation are recorded in the JSON.
