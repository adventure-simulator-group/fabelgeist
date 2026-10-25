# Scoped independent Rule Review: Kernel ordering and documentation

Verdict: no actionable rule finding in the two-file working delta against
`a6a37a8f5a99398789f512a7b4edec4ba9ab2a25`. This is a narrow source review;
combined Rule Review, global audit clearance and hosted validation remain
pending.

Reviewed snapshot:

- HEAD: `a6a37a8f5a99398789f512a7b4edec4ba9ab2a25`.
- `crates/fabelgeist-compute/src/kernel/mod.rs` SHA-256:
  `47b3faf12d1fdffc6251a9dcd840b28f8ce347bbf9e377e29671b4b171f9ff74`.
- `crates/fabelgeist-compute/src/kernel/fast/uniforms.rs` SHA-256:
  `c1e2f58f4f5a2bdb5a220a2e633075fc0fedc981e036958586b411ab050685d5`.

Read the current `crates/AGENTS.md`, the ledger's
`kernel_order_comment_packing_preinventory` entry, both complete changed files,
the bounded private FastPath admission/caller context, and
`target/pr780-audit/kernel-order-comment-source-review.json`. Directly inspected
the complete two-file diff against the pinned base. Its changes are the two
existing struct-definition moves and three documentation replacements. The
source-comparison report covers the preceding documentation wording; the final
Kernel.fast wording was independently rechecked in the final whole-file diff.
The two current fingerprints above identify this review's exact snapshot.

The module now places the existing KernelCache and KernelBatch structs after
Kernel and before the first impl. Their documentation, attributes, fields,
types and visibility move intact. Imports/re-exports precede module
declarations, constants precede structs, and the existing test modules remain
last. This satisfies the governing file-level item-order rule without creating
pass-through ownership or changing responsibility.

The units and canonical owners remain unchanged: WorkgroupShape,
WorkgroupGrid, InvocationCount, RecordedDispatchCount, BufferByteLength and
BufferByteOffset retain their current roles and boundaries. The delta adds no
quantity, conversion, invariant, primitive adapter or error policy. Kernel
methods, trait impls, cache behavior, batching, callbacks and submission bodies
are unchanged.

The corrected fast-field prose accurately describes per-batch uniform slots,
fresh bind groups, and interfaces without a uniform block requiring no slot.
The
module prose removes the unsupported assertion that every other compute
primitive submits separately. The packing prose distinguishes missing or
unsupported parameters from the existing successful whole-value block skip;
it explicitly states that the supplied variant selects representation and
that reflected member type/span is not validated.

The private FastPath admits reflected blocks at most 256 bytes and calls the
packer with that exact block slice. Provided supported variants occupy at most
64 bytes; the admitted Naga offsets and those spans do not demonstrate a
reachable host-address addition overflow on supported 32/64-bit hosts. The
existing packer writes a supported provided value only when its whole span
fits the output block; it otherwise leaves those bytes unchanged. The delta
preserves this policy, every supported variant, typed errors, primitive packing
arithmetic and native representation. The withdrawn strict-packing proposal
was not reintroduced.

No test, case, helper, assertion, fixture, shader, item body or function body
was added or changed in this scoped delta. No source edits, project checks,
local tests, CI or external posts were performed by this reviewer. This review
does not validate unrelated working changes, the complete migration, the
static packer owner, compiler acceptance or runtime behavior. The two source
fingerprints and HEAD were reconfirmed at the end of inspection.
