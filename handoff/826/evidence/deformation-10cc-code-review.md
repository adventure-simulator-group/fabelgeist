# Independent deformation checkpoint Code Review

Revision: `10cc1a69b1a9a5f4e54daa112477cc56b7487f45`.
Tree: `2f11b71144a48958f3ad726989f1e5e084b21071`.
Parent: `515815cdbc6e7f4da073185bdead65ac8d2abbc2`.
Comparison: `21118b983b27e00d2cc70f45ed345686e0262cc2`.
Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.

No new owned actionable correctness finding in this bounded source review.
All four changed files were freshly read completely, including the full
1,960-line renderer and 1,563-line GPU module. Additional source coverage and
exact unchanged identities are recorded individually in the JSON companion.

The deformation owner parses the exact composed source used to create its
native shader, resolves the exact compute entry and derives its declaration
from that entry. The module owns its parsed names, so moving source into the
native descriptor leaves the selected-entry borrow valid. Native layout,
access modes and pipeline descriptors preserve the previous behavior.
Canonical parse/entry/shape errors retain concrete causes through anyhow and
the existing hair facade error. Parsing supplies no semantic-validation
verdict; native validation retains the documented device-handled policy.

Dispatch consumes canonical admitted sample invocations, checks every padded
folded lane, returns early for empty geometry and checks the actual receiving
device before parameter writes or native pass recording. The unchanged shader
uses the same width64 declaration and folded-index stride. Successful native
coordinates are preserved; unrepresentable padded launches now fail admission.

Recording still deduplicates level obligations. Errors propagate before any
Pending obligation or temporal mean is published. A preceding level's commands
may already be encoded and uniforms uploaded, but the failed encoder is never
submitted. This is staged logical publication, not rollback of all queue
writes. On success the frame order, history formulas and moving-state behavior
are unchanged, with both publication methods after normal queue submission.
No GPU-completion or observed device-handled validation result is implied.

Every existing renderer/GPU fixture module, shader source and native POD body
is byte-identical to211. Export/import composition changes preserve identity;
the buffer-prefix documentation now correctly identifies the particular
binding adapter that rejects empty logical bytes.

Canonical producer getters and sealed source/device associations remain
unintegrated. Host/GPU simulation source and array admission, whole-physical
rest copies and bindings, native card admissions and producer/caller/lock
integration remain required connected work. The dispatch repair does not
accept or silently defer those paths. #780 cached receiver integration remains
required under its agreed ownership.

No source was edited and no project execution, build, check, test, scanner,
generator, runtime, push or CI ran. This is no compilation, full renderer or
native-family, all13-lens, whole-PR or final clearance. Final independent
reviews and hosted checks remain required on the same final pushed revision.
