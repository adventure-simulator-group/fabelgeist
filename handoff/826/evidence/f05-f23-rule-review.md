# Independent F05/F23 Rule Review of PR #826

No owned source rule finding was identified in the reviewed F05/F23 repair.
The revised code preserves the actual pinned-main supplied-value protocol and
propagates the existing byte owners through the complete metadata family.
Previous c798 expected-shape/direct-name verdicts are withdrawn; they are not
evidence for this revision. Required #780/#774/F19 families and hosted baseline
and project checks remain incomplete. No final clearance is given.

## Exact snapshot and evidence

- HEAD: `3ff23fc9a47fe32fe514747a17585606ea001475`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- All 73 main-to-HEAD file contents independently matched their HEAD blobs;
  the working tree was clean.
- The `f05_f23_post_repair` revision and every source fingerprint matched HEAD.
- Independent digest of sorted changed-path/SHA-256 records:
  `341decd750b4533a5ccf4e28e157531dbaa9aabfc1b1e7032c9ea95be00a77a8`.

Evidence bookkeeping issue at initial verification: the older
`f05_f23_implementation` section names this revision but retains stale hashes
for `bind_group/uniforms.rs` and `texture-views.md`. The root was notified to
reconcile or mark those intermediate receipts historical before handoff. The
current post-repair wave and independent HEAD comparison are consistent; this
report does not use the stale section to establish source identity.

Read actual pinned-main static packing in `compute/pass/mod.rs:85-211`, fast
packing in `kernel/fast.rs:325-394` and uniform reflection in
`pipeline/reflection.rs:113-141`, then traced the current whole family and its
unchanged receiving graph. The governing Rule Review skill/reference and
root/crates AGENTS apply; no prior verdict replaced fresh source comparison.

This reviewer wrote only this ignored report. No source edits, project checks,
tests, cases, test helpers, assertions, doctests, builds, scanners, generators,
runtime, push, hosted CI or external posts were performed. Git reads/hashing
establish source identity only. Numerical quality gates and compilation remain
unverified.

## Actual behavior contract, compared directly with main

Pinned main selected the **provided PassParameter variant**, not a declared
uniform value shape. It wrote that variant at the immediate reflected member's
offset, subject to whole-block bounds. It did not require the supplied variant
to match the shader member type or limit every write to that member's span.
Array/nested members retained ordinary immediate-member name/offset/size
metadata. Nonstruct uniform blocks retained their size and an empty named
member dictionary, so those blocks stayed zero-filled regardless of an extra
global-name parameter.

The temporary expected-shape rejection and direct-global named mapping changed
that protocol. Their removal is a coherent final implementation, with no
compatibility shim, dual packing policy or deprecated forwarding API. The
current guide now explains the actual contract instead of promising stronger
declared-type admission. This respects root's no-compatibility-path rule while
avoiding an unrelated behavioral narrowing in a bespoke-type migration.

| Pinned-main path | Current source evidence and result |
| --- | --- |
| Struct reflection | `reflection/uniforms.rs:24-44` retains every validated immediate struct member's name, native offset and native type size, including nested/array members. There is no expected-shape filter. Naga validation still precedes `.size()` use. One block per shared group remains explicit; unsupported resource binding arrays remain distinct from uniform array values. |
| Direct nonstruct block | `reflection/uniforms.rs:37-40` retains its native size without adding a named destination. Both zero-initialized buffers have no members to visit. The former new global-name mapping is gone. This reproduces main's actual behavior rather than inventing a new required value. |
| Static scalar/vector input | `bind_group/uniforms.rs:16-51` visits members in retained order and dispatches solely on the supplied variant. Number→f32, Unsigned→32-bit word and Vec2/3/4 component encoding retain old bytes and whole-block guards. Missing or unsupported supplied variants remain skipped in zero-initialized data. Declared nested/array/different types do not trigger the withdrawn admission policy. |
| Integer words | Existing From<i32> still stores Unsigned(value as u32); From<u32> stores that same native word representation. The unchanged scalar writer emits those bits. No new signed interpretation, numerical conversion or parallel word wrapper is introduced. |
| Mat2/Mat4/Transform | Fixed spans/column strides remain 16/8 and 64/16 bytes; Transform still converts to Mat4. Column-then-row iteration matches main's `(i * width + j) * 4` component order. The old whole-block outer guard still skips a value whose fixed span does not fit. |
| Mat3 success and overlap | Main used the reflected member size divided by three for column stride. Current `write_value:42-45` uses the same span/3, and `write_matrix:95-117` visits the same column/row order. It checks actual block capacity, not member-span membership. Therefore prior successful overlap into later bytes is preserved, including subsequent member overwrite order. It does not impose the withdrawn shape restriction or a new member-local bound. |
| Mat3 former inner panic | Main's outer span check did not prove each inner 4-byte window was within the block. Current code retains that outer success/skip rule, then checks additions and each actual component window. Overflow or a formerly panicking window returns typed UniformPackingOverflow/OutOfBounds. The temporary host Vec is discarded on error before native allocation/upload, so no partial GPU state is committed. A previously successful component sequence is not converted into an error merely for exceeding the declared member span. |
| Fast supplied variants | `kernel/fast/uniforms.rs:14-61` retains old required-member order, Number/Unsigned/Vec*/Mat4 packing and narrower matrix support. Missing or unsupported provided values now retain canonical typed errors, as already authorized. No declared-shape test or new fallback is present. Direct nonstruct blocks still have an empty destination list and retain zeros. |

No new test population or fixtures were needed to establish these source-level
comparisons, and none was created. Existing hosted tests may cover only part of
this protocol; this review does not claim runtime coverage for the rest.

## Complete canonical byte family

`buffer/extent.rs` remains the sole owner of BufferByteOffset and
BufferByteLength. Offset admits zero; length admits zero; actual native binding
requirements separately use NonZeroU64. These are byte quantities/addresses,
not shader binding ordinals, component counts or workgroup counts.

The full current trace is:

    Naga member offset / type size / block span
      -> canonical byte leaves at reflection admission
      -> UniformMember and BindGroupReflection retained fields
      -> static checked allocation / packing and FastPath retained metadata
      -> checked host usize or actual native buffer binding/queue ports

| Port | Verified contract |
| --- | --- |
| Decoder/admission | `reflection/uniforms.rs:30-43` widens Naga's native u32 byte values and admits them once into the existing byte leaves. Group construction commits the correlated span/member dictionary only after construction. No independent public pipeline reflection replacement route is reintroduced. |
| Retained state | `shader.rs` stores offset as BufferByteOffset, member size and block extent as BufferByteLength. FastPath stores/clones the group's already-typed extent directly. Its stride remains a typed byte length, arena use remains a typed offset and prepared dynamic offset remains typed. No raw retained byte field or duplicated uniform-byte owner remains in this family. |
| Static allocation | `uniform_buffer::create` tests the canonical zero extent, extracts native bytes for SDK alignment/limit arithmetic, and keeps requested/limit payloads typed. It checks padded extent before packing, preserves unmapped allocation and native scope/readback policy, then uploads through BufferUpload. All new fallible packing/address results propagate before native state use. |
| Host slice boundary | Static and fast packers convert byte leaves through `usize::try_from`. UniformHostAddress retains the actual TryFromIntError through Error::source, rather than a formatted replacement. Buffer length/offset arithmetic remains checked where a general host range can fail. |
| Native binding/stride boundary | FastPath extracts u64 only for the actual native minimum binding size and alignment calculation; its retained size does not escape and get re-admitted between application operations. Canonical leaves reach uniform slot allocation, dynamic-offset narrowing and queue writes. |
| Packing errors | OutOfBounds retains byte offset, component byte size and host block capacity using the same canonical owners. Overflow is a separate closed outcome. The owning UniformPackingResult alias preserves ComputePassError rather than widening it. Standard generic conversions preserve these error identities and the actual narrowing cause. |

The fast writer's remaining native usize arithmetic is bounded by its admitted
256-byte block and validated member offsets; the largest provided fast value is
the existing Mat4. It cannot overflow a supported 32/64-bit host address under
those construction guarantees. Static matrix calculations now have explicit
checked inner windows because the general reflected-span rule did not supply
that proof. This distinction follows the earliest actual contract rather than
adding redundant hypothetical error paths.

Governing rules: `crates/AGENTS.md:28-57,69-75` require canonical owner reuse,
producer propagation and actual boundary extraction. The new leaf fields meet
those requirements. Native group/binding indices remain SDK ordinals, not byte
lengths. Mat3's three columns, f32 component widths and fixed matrix byte layout
are the existing native encoding grammar; substituting policy constants or
count/physical-unit wrappers would misstate their role.

## Type ownership, granularity and removal

Source search found no production UniformValueShape, `shape` admission helper,
member shape field or UnsupportedUniformType left behind. Those distinctions
did not describe the actual supplied-value protocol and imposed behavioral
narrowing. They were removed completely, not left as forwarding aliases or an
optional compatibility branch. There is one static supplied-variant policy and
the existing narrower fast writer; their distinct error/default behavior is
intentional and documented.

The existing byte owners express the relevant retained unit distinctions.
PassParameter remains the native supplied-value tag at this encoding boundary;
shader identifiers remain open strings. No primitive classification was moved
into mutable display prose, no duplicate equivalent type was added, and no
unrelated stricter numeric range was introduced.

New production functions/modules remain focused on packing, reflection or
allocation and follow item order. The matrix helper/Result alias add real
range/error policy, rather than a pass-through service. Source size does not
establish the actual numerical quality gate; applicable hosted baseline
reductions/no-growth checks remain required and unrun. No registry scope,
baseline ceiling, Cargo lint inheritance or manifest was broadened.

## Revalidated unaffected and receiving graph

All 73 main-diff paths and affected relationships remain accounted for. The ten
F05/F23 paths were reread in full at the changed responsibility; remaining
source identities and their current owner/recipient contracts were revalidated.
Uniform metadata source search reaches the sole reflector, static allocation
and writers, FastPath and its writer; there is no unadapted raw-byte receiving
field in that family.

Texture constructors/cache/device/view selection, effective selected binding
facts, converter/copy/read/write/clear recipients, the one immutable-pipeline
assembler, actual sampler/image native roles, usage/sampling unions, typed
unsupported resource/Immediate handling and concrete native/Naga cause caches
are unchanged by this repair. They continue to use their canonical owners and
actual SDK device identity. Unsupported binding arrays remain resource
collections, not the accepted array values inside a uniform block.

Arena/Prepared/batch retain byte bounds, actual native offset narrowing, empty
batch precedence/counting, folded launches and callbacks. The standalone/static
versus batched/fast policies remain explicit. Available native causes and the
configured native/fragile non-atomic-Wasm trait contract remain intact; no
format/reparse or unsafe trait adapter was introduced. Actual native/Wasm
compilation and browser atomics configurations are not verified.

The current guide explicitly describes the supplied-variant, whole-block,
direct-zero and Mat3 contracts instead of the withdrawn expected-shape promise.
Its prose remains in the owning GPU guide and follows root wrapping rules.
Root README/wiki prose, existing test/fixture population and documented
historical software-GPU/browser/hardware/performance/rendering limits remain.

## Required incomplete families and acceptance limits

All #780 source/cache/checked producer integration remains required, including
Kernel raw workgroup/count fields, grid/byte producer adapters, definition and
mesh source/device identity, lock unwraps and the delegated Kernel/module/field
documentation. These are not repaired by typed uniform receiving metadata.

All retained #774 hair/fitting/body/armor/metal/engraving and connected F19
runtime equipment cause/recipient work remain required. Restored claimed hair
array recipients still need their exact agreed WorkgroupGrid lineage. The
later core ownership transfer does not authorize a duplicate repair inside
retained whole functions. None of these paths is accepted as a silent deferral.

Hosted quality baseline reductions and all required fmt/lint/native/Wasm/project
checks remain pending. No local execution was substituted. Final full audit
waves and independent Code/Rule/Comment reviews must agree with required hosted
CI on the same final pushed SHA. This is a local source review only; do not
merge.
