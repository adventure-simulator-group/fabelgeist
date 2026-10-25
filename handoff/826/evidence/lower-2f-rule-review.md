# Lower and entry-selection checkpoint Rule Review

Reviewed exact commit `2f3880bfabd08674857bd47015e7c47287a691dc`, tree
`40d8b3798593f02ece6881718c1085e72a6aff78`, against parent
`f0a98e58d54b673e017cf5ae633e9b6c46b5b609`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`. The tracked tree was clean when
accounted. All seven changed files and their entire deltas were read, with
current lower receiving responsibilities revalidated, not only item movement.

The JSON records seven complete source receipts, 44 unchanged receipts,
seven exact canonical peers, affected GPU consumers and native source hashes.
All retained and peer identities independently match their prior receipts.
Fresh full reads cover current buffer/read/mapping/readback/error/completion,
Armor facade/Metal/staging, context initialization/observation/submission, shader
admission/error, image/uniform reflection, static bind assembly, static pass and
fast native pipeline compilation. Kernel/fast/mesh caller slices establish
pending receiver transitions; they do not grant full unrelated body coverage.

Root/crates/Armor AGENTS and Rule Review's bespoke reference govern this
review. Relevant rules are canonical ownership and propagation at
crates/AGENTS:28–57,69–71; explicit closed selection roles at 58–65; checked
access at 80–84; item/responsibility ordering at 90–110; cohesive Result aliases
at 118–122; no production unwrap/expect at 126–127; no-growth and test limits
at 166–192. Armor's unrelated-maintenance exclusion applies; no shape changed.

## Finding

**2F-ORDER-01 — P3, existing affected scoped rule gap.**
`crates/fabelgeist-gpu/src/data/gpu/compute/texture_ops.rs:123` declares
`TextureBinaryOp` after implementation blocks at 18–121. The required item
order is structs before impls (`crates/AGENTS.md:90–94`). Move the existing
unit struct next to `TextureBinaryOpDefinition` before the impl blocks,
preserving methods and population. This is existing rule debt in a changed
responsibility, not a new behavioral regression. No other new owned rule or
behavioral defect was established in the reviewed receiving increment.

F0-ORDER-01 is repaired: upload now orders constant, struct, enum, impl. Its
implementation body and attached fenced compile-fail block are byte-identical
to f0; the source movement adds no alternate API or behavior.

## Source-backed contracts

- **Selected entry owner and admission.** Pipeline-owned
  `ComputeEntryPoint::First/Named` (`entry_point.rs:5–36`) distinguishes the
  selection role from the open WGSL identifier. `resolve` searches only actual
  compute entries, returns the first compute declaration for First, and exact
  identity for Named. Missing named entries cannot fall back to another
  program; `MissingEntryPoint { requested }` retains the requested name
  (`error.rs:20–22,58–64`). First's missing name is represented by None, so
  diagnostics preserve the distinction without a second entry-name owner.
- **Producer, retained state and actual native consumer.** ComputeShader
  admits parsed/validated source, native shader module and creating Device
  together. `ComputePipeline::prepare` checks that actual Device before
  reflecting (`pipeline/mod.rs:63–77`). The chosen entry's name is cloned
  into pipeline-owned `ComputeReflection` (`reflection.rs:14–19,107–113`).
  Its Arc is private inside the pipeline; getters return shared references,
  and no public replacement path can detach this retained selection from
  admitted source/layout. A separately constructible reflection record does
  not provide an admission path back into the pipeline.
- **Compile and cache.** Compile, compiled access and interface validation
  consume that retained name (`mod.rs:101–109,139–149`). Native creation
  passes `Some(entry_point)` explicitly (`152–180`), with no SDK default-entry
  ambiguity. The existing public native entry lookup can also select another
  cached compute entry; its String key is an actual provider identifier and
  creation outcome remains keyed by that exact name. Cache locks retain
  poisoning as a typed failure, observed native rejection remains an error,
  and borrowed/device-handled cache entries are not mistaken for observed
  success when later used by an observing context (`114–135`). All native
  creation paths retain actual wgpu cause sources, rather than display text.
- **Reflection ownership and resource relationships.** `ComputeReflection`
  replaces shader-level `ReflectionData`; dummy fragment/vertex fields and
  the old type are removed directly, without a compatibility alias. Existing
  `BindGroupReflection` remains the shared native binding-record owner.
  Reflection intentionally unions used resources and sampling requirements
  across compute entries because one retained layout serves all entry-cache
  outcomes (`reflection.rs:35–73`). Named selection does not silently switch
  to a per-entry layout. Uniform member offsets/extents remain canonical byte
  quantities, populated from validated Naga native layout. Existing
  single-provided-variant static/fast packing policy is unchanged. Used
  Immediate inputs, binding arrays and unsupported handle/external shapes
  retain typed rejection; sampled/depth multisampling, sampler comparison,
  image view dimension and storage access retain actual SDK layout roles.
  Provider feature rejection remains a native provider decision.
- **Texture producer and static receiver.** Texture binary source generation
  authors `fn main` (`texture_ops.rs:59–60`) and explicitly selects
  `Named("main")` (`97–102`). Static pass asks the definition for its compiled
  pipeline, then static assembly consumes the same reflected bind groups and
  layouts. Removing unused vertex/fragment metadata changes no binding
  metadata consumed there. Existing authored shader grammar tokens remain
  native source boundaries, not mutable display prose. Existing descriptor
  classification/recipe/count migration obligations below remain open.
- **Lower unit/state/native graph.** Current Buffer logical bytes feed the
  private checked `MappingExtent`, canonical `Readback::new` and exact native
  map/copy ranges. Zero, alignment overflow, allocation capacity, partial/ZST
  host elements, preallocation before Wasm mapped view, complete typed
  initialization and padding exclusion retain their verified contracts.
  MappingRequest owns release before scope rejection; callback, polling,
  mapping, allocation and native causes remain distinct. Context completion
  owns its Result/error carrier, explicit native marker index and browser
  callback-delivery limit. Armor retains GenerateError sources; Metal uses
  canonical word count/stride and a cause-preserving contextual error;
  Staging retains session identity, checked exact prefix membership/order,
  allocation failures and distinct native/grouped-readback causes. Those
  responsibilities remain identical to f0 and were revalidated from source.

No duplicate unit owner, source-language wrapper, ordinal enum, raw constructor
fallback, new serialization bypass or compatibility API was introduced.
The entry selection enum's borrowed name is resolved and owned at admission;
the enum need not retain the caller's borrow. Existing derive choices preserve
admission and no new handwritten conversion or serialization boilerplate was
required. Public raw native binding ordinals and authored WGSL grammar remain
explicit native metadata; they are not silently granted application authority.

## Required integration and limits

Correct receiving code does not make this migration complete. FastPath still
names removed `shader::ReflectionData` (`kernel/fast.rs:85`), Kernel prepare
still has two arguments (`kernel/mod.rs:111`), and remaining compute and mesh
constructors still use the former two-argument API. #780 explicitly owns the
full constructor/FastPath/ResourcePipeline/shape caller integration. Consume the
new canonical owner and explicit selection directly; no restored alias,
forwarding constructor or default argument is acceptable.

The unchanged texture-operation descriptor tuple key (`texture_ops.rs:7–10,
112`), descriptor base-type string decisions (`67–92`) and dispatch workgroup
relationships (`179–192`) remain required #780 resource/producer contracts.
This constructor adaptation does not clear those retained paths. Likewise the
three completion callers, producing submit/publication results, canonical
count/stride/Metal-size/query/grouped-Readback integration, all lower caller and
fixture adaptations, retained material/application error-state chains, and
Armor manifest's canonical Cargo.lock integration remain required. These are
unresolved implementation dependencies, not accepted silent deferrals.

Native GPU wgpu std feature and its actual per-thread scope-key policy are
unchanged. Actual SDK Error sources retain their cfg-specific Send/Sync
constraints. Native marker completion can return actual marker/poll causes;
WebGPU SDK callback delivery still cannot expose an underlying promise
rejection. Borrowed and browser provider handling and unsupported browser
atomics limits are unchanged. No source identity claim implies platform
execution or global device-health certification.

All seven delta files are under 500 source lines and changed production
functions under 100 by direct source inspection. No suppression or baseline
ceiling increased. All existing tests, cases, fixtures, helpers, assertions and
doctests are preserved; this delta adds none. Upload's method body and doctest
are byte-identical despite movement, and all prior six literal test-operand
adaptations remain parent-identical. Required hosted baseline reductions and
project checks remain pending.

No source edit, project check, scanner, build, generator, test, runtime, push or
CI was performed. Migration completeness is **incomplete**; compilation is
unverified and current source intentionally awaits coordinated integration.
This checkpoint is not whole-PR, all-thirteen-lens, hosted or final clearance.
Independent final reviews and required CI must pass the same final pushed SHA.
