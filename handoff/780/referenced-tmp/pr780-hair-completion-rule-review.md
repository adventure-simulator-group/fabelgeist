# Independent hair completion Rule Review

Reviewed Root worktree
`/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator` at clean
base `29fe6e75275dc722524af5791b2b6ccb38859a0c` plus exactly three changed
sources. No actionable finding remains in the completion and adjacent item-order
delta. This is a narrow verdict, not global migration clearance.

Read applicable root/crates rules, both thirteen-lens preinventories in
hair_completion_preinventory (including adjacent_order_preinventory), whole
current Creator HairGpu module, Hair GPU production declarations/implementation,
entire513-line existing render fixture, canonical completion/callback helpers,
and source diffs. Read the supplied14-comparison evidence, then independently
compared actual blocks and complete unchanged fixture bytes against Git HEAD.
The unchanged large Hair inline unit module is byte-equivalence evidence here,
not a fresh independent semantic audit of every simulation fixture or operation.

## Completion ownership and causal propagation

Creator HairGpu retains the same WgpuContext used to initialize all its kernels.
Its async finish now directly awaits that context's canonical submitted_work_done
and returns Result<()> using the existing heterogeneous anyhow boundary. The
real WorkCompletionError remains downcastable as the wrapped error and exposes
its actual Marker/Poll/Callback source. No formatted surrogate, bool outcome,
separate completion owner, raw queue/device adapter or compatibility fallback
appears. No groomer MutexGuard is acquired or held across await.

On native builds canonical completion submits an empty marker through the
context's validation observation policy, then waits for that exact submission
index with no requested timeout. Marker and polling failures propagate. A
borrowed context may leave native Validation reporting to device handling;
success does not retroactively certify producing submissions or global device
health. Browser builds register the canonical queue callback and await its
oneshot while yielding browser task turns through await_browser_callback.
Callback progress/cancellation errors propagate. The WebGPU backend can also
invoke its callback after a rejected promise; success establishes callback
delivery there, not an otherwise unavailable promise-success verdict.

The new finish docs accurately state the prior-queued-work completion point,
native causes, browser limitation and requirement that producing submissions
already succeeded. Future submissions and concurrent CPU work not yet queued
are outside this snapshot contract. Removing the obsolete GpuGroomer rawpoll
convenience eliminates its ignored PollError and parallel completion mechanism;
public context() already exposes the canonical owner for direct Groomer users.
No legacy forwarding method or temporary primitive adapter is retained.

Existing render fixture obtains the actual Bevy rendering context, constructs
HairGpu on it and uses finish at its existing post-geometry-build and post-app
boundaries. The same two calls now use pollster::block_on(gpu.finish())? inside
the existing synchronous fallible fixture. No case, helper, assertion, camera,
input/oracle, numerical policy or doctest is added. These synchronous fixture
calls do not introduce a production browser block_on path.

## Legal item and macro order

Ordinary imports and re-exports now precede module declarations in both touched
owners. Creator places its unchanged local params macro and immediate scoped
re-export after ordinary imports and before mods, followed by Params alias,
structs, impls and functions. This is a necessary lexical pair rather than a
reason to create a pass-through module or use macro_export.

The installed Rust Reference macros-by-example.html clauses
macro.decl.scope.textual.intro and macro.decl.scope.path.reexport explicitly
state that a macro_rules definition enters textual scope after its declaration
and the local re-export occurs within that textual scope. The resulting path
binding is item-scoped, but the re-export's unqualified source name still needs
the preceding definition. Keeping this pair together is sound. Params is an
ordinary named type item and its later declaration is available through the
macro's fully qualified $crate path at invocation. Macro body/visibility remain
unchanged; no broader crate-root macro export or hidden alternate implementation.

Hair's constant now follows modules, all native field structs precede impls,
head_shape follows impls with the other public functions, and cfg(test) remains
last. Docs and representation attributes stay attached. No semantic owner,
POD/ABI field, shader resource or numerical projection changes under item moves.
Existing oversized-file/physical producer obligations are not cleared by this
reordering, and no growth ceiling was raised here.

## Independent source confinement

Actual source comparison established all of these against the base:

- Hair's entire inline test module and module docs/doctest are byte-identical.
- Complete Kernels initialization, scratch/Groomer fields, head_shape and
  headless/required-limits blocks moved byte-identically.
- GpuGroomer implementation changes only by deleting obsolete finish.
- Creator params macro and re-export are byte-identical.
- Reversing exactly the two existing fixture API adaptations reconstructs the
  entire original fixture, including all helper/case/oracle bytes.
- Canonical native completion.rs and submission.rs remain byte-identical.

The source diff also retains all Creator construction, storage/upload, fields
and scratch access behavior beyond the admitted finish change and item moves.
Canonical native peer functions are not edited. Searches locate no additional
in-repository finish caller beyond the old forwarding connection and the two
fixture uses; that census supplements the receiving-body review, not clearance
of unrelated methods.

No source edits, project checks, probes, tests, new tests/cases/helpers/assertions,
builds, lints, scanners, generators, runtime, hosted CI or external posts occurred
in this review. Direct source comparisons are not executed acceptance tests.
Broader generated programs/defaults/ordered signatures, physical quantities/Grid,
completion-peer composition and all other PR families stay open. Final full
thirteen-lens waves, three independent reviews and required hosted checks must
pass on the same final pushed revision. This scoped verdict authorizes no
unfinished-family push or merge.

## Current SHA-256

- `crates/adventuresim-character-creator/src/hair/gpu/mod.rs`: `aea64b1f78e5def266d32a1ecf4ffce201b11292c344a40ad123fa28118f5644`

- `crates/fabelgeist-hair/src/gpu/mod.rs`: `f163ffb8a42b0ad714ea0d96f2c7ef1a191da8d085d01310e143f941a63caf22`

- `crates/adventuresim-character-creator/src/hair_preview/render_test.rs`: `af7733cf9f722b473e08584c2c35569cc0962376cadc4e73875ba71bef94e739`

- `crates/fabelgeist-gpu/src/globals/wgpu_context/completion.rs`: `0bf57e56179c36d0bf533b46daf1971b4037363b98505b14b55cf6c4ff2f4fdf`

- `crates/fabelgeist-gpu/src/globals/wgpu_context/submission.rs`: `70b94328d0cfb9388ff86b5cc7960a7bf2420867a0a0ad406002ead4dd1c5a27`

- `crates/fabelgeist-gpu/src/globals/callback_progress.rs`: `6edc84206e79466f1370de25ce9a19d46d9f94d6b209b1c77c42246d8c2ea961`

- `/home/brunosegovia/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/share/doc/rust/html/reference/macros-by-example.html`: `97bb4a5fc88b7ba916d6c7451ea4ff034fb5407f11c7987885725a51e012e288`
