# Public resolver receiving Rule Review

Reviewed clean commit `8d65dd9e24c506f485cec31a30ad998351038c5b`, tree
`cc6bbf93c533253762f44c6b69de81e8dd192f23`, against parent
`a3a1b01a4cd24dbda2555023decd07b33a47b199`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`.

The complete changed entry_point.rs and exact diff were read. Sixty-one other
receiving source paths, seven canonical peers, governing rule sources and
pinned SDK source receipts independently match their prior hashes. Original
source coverage granularity is preserved: identity is not fresh unread body
credit. Current entry-to-reflection-to-native creation connections were
reconciled against source, not an automatic earlier verdict.

**No new owned receiving finding was established.** The whole canonical
`ComputeEntryPoint::resolve` function is now public (`entry_point.rs:15–39`),
with an accurate scope: resolve actual parsed native identity without claiming
module validation or compilation. Its complete executable body is
byte-identical. First selects the first actual compute-stage entry; Named
selects only the exact requested name; absence retains typed requested-name
context and never falls back. The returned reference belongs to the supplied
Naga module, so this helper creates no detached cached identity.

The owner remains the shared pipeline module, not a new Kernel-specific
selector. Exposing the existing whole method permits #780's Kernel and
ResourcePipeline to inspect actual native workgroup shape before checked shape
admission/native pipeline creation, preserving their required error order
without another `.find` implementation. It does not admit an unvalidated
ComputeShader or ComputePipeline: those constructors still parse/validate,
check actual Device identity, reflect the selected name, retain it in private
pipeline state and pass it explicitly at the native descriptor boundary
(`pipeline/mod.rs:63–77,101–149,173–180`). The public parsed Naga input/output
is an explicit native adapter, not an internal naked primitive path.

This satisfies canonical shared ownership/producer propagation and explicit
native-boundary rules (`crates/AGENTS.md:28–57,69–71`), receiver/method rules
(`128–139`), owning Result/error distinctions (`118–122`), and cohesive module
and item ordering (`90–110`). No compatibility alias, selector duplicate,
constructor fallback, broad visibility change beyond this agreed function,
serialization bypass, raw-string error classification or new wrapper appears.
The docs distinguish parsed identity from validation rather than implying
checked workgroup guarantees on every native module.

Receiving contracts remain unchanged: native cache entry identity and rejection
state; shared resource/sampling union; uniform native byte metadata and packing
policy; logical Buffer bytes and admitted aligned mapping extent; host
allocation/initialization and padding exclusion; release/callback/error state;
explicit completion marker/index and browser delivery limits; and Armor/Metal/
Staging actual native causes and session/prefix membership. Exact rule/SDK
identities preserve cfg-specific native std/thread scope and supported
non-atomic Wasm policies; they do not establish platform execution or global
Device health. The whole HairRenderer family remains unedited and outside
this receiving verdict.

The forty-line file and unchanged resolver body remain below no-growth
ceilings. No new production unsafe, unwrap/expect, suppression or baseline
ceiling was introduced. The sole delta is visibility plus documentation:
all shader bodies, tests, cases, fixtures, helpers, assertions, examples and
doctests are parent-identical. No new population was added.

Completeness remains **incomplete**. #780 must consume this canonical resolver,
full ComputePipeline in FastPath, the explicit constructor selection and new
ComputeReflection owner throughout Kernel/ResourcePipeline/shape callers.
Existing descriptor recipe/string/count families, three completion callers and
publication transitions, canonical lower sources/callers/fixtures/error-state
chains and lock integration remain required. Those dependencies are not an
accepted silent deferral and compilation is not inferred.

Only source and ignored report accounting was performed. No source edit,
project check/scanner/build/test/generator/runtime, push or CI occurred.
Required hosted baseline reductions/checks and final independent reviews/CI on
one final pushed SHA remain pending. This is not combined thirteen-lens,
whole-PR, compilation, hosted or final clearance.
