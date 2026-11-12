# Independent source review of NPY q2306

Reader: `/root/verifier_scope`. This is a new, bounded assessment of the
prepared report by `/root/wrapped_scope`; it does not adopt a queue
disposition or assert a new finding. Current Rust source was displayed at
`95e63628f9d70b8dd6e6fede64334696c6995009`. The historical and current
crate manifests were separately displayed at e275 and 95e. Historical e275
Rust bodies were compared by Git identity but were not personally redisplayed
for read credit. Registrations, actual display ranges, unread complements,
source bytes, and limitations are retained in this packet.

## Source reasoning

- **Framing and dictionary.** `npy/header/frame.rs:7-40,51-67` checks the
  magic and minimum prefix, accepts 1.0 with a little-endian 16-bit header
  length and 2.0/3.0 with a little-endian 32-bit length, checks the length
  field and dictionary/payload split, and parses dictionary bytes as UTF-8.
  `header.rs:27-88,91-114,140-160` requires exactly one `descr`,
  `fortran_order`, and `shape`; rejects duplicate or unknown fields, bad
  separators, trailing non-whitespace text, unsupported dtype or Fortran
  order; and only then admits the shape and payload. The quoted-field parser
  rejects backslash escapes. The temporary optional lexical fields do not
  become a retained partially admitted array. `npy.rs:118-130` copies the
  admitted payload into an array with private shape, dtype and bytes.
- **Shape and payload.** `header.rs:117-137` parses an empty tuple as rank
  zero. `metadata.rs:83-99,113-143` gives that scalar one element; any zero
  axis gives zero elements before other dimension products are multiplied.
  Nonzero products and element-width multiplication use checked arithmetic.
  Admission requires at least the computed byte count and retains only that
  prefix: extra trailing bytes are not retained, including when the expected
  count is zero. These are the shown storage rules, not proof of conformance
  to every NPY version or agreement with every producer's policy.
- **Native layout is a separate boundary.** `tensor.rs:28-75` checks the
  ordered native dimension prefixes for an empty storage shape. Thus, for
  example, a storage shape with a zero axis can be admitted while a native
  layout with overflowing positive dimensions before that zero is rejected;
  placing the zero earlier changes which prefixes are multiplied. This is a
  source-control-flow observation, not an executed example. The float upload
  separately checks value count against layout before calling Burn
  (`tensor.rs:92-108`). These checks do not establish device, allocation,
  reshape, or native zero-rank/zero-axis behavior.
- **Value semantics remain separate.** `values.rs:96-99,108-140,152-166`
  shows fixed-width little-endian decode, native numeric casts, and a
  zero/nonzero classification of each payload byte. This does not itself
  establish finite values, exact numerical conversion, units or model/rig
  association, and byte classification is not generally an elementwise
  semantic boolean projection. These are open consumer-policy questions,
  not an assertion that the parser is defective.

The staged report's bounded source reasoning is corroborated. It correctly
limits the two historical/current identity claims and does not assert a new
header defect or runtime pass. Trailing-byte retention, tuple/shape support,
resource limits, NPZ/ZIP paths and broader format conformance remain policy or
separate source questions.

## Revision and recovery distinctions

The entire `crates/fabelgeist-numpy-storage/src` Git tree and the seven
individually checked Rust blob identities match e275 and 95e. The root and
crate tree IDs differ. An independent crate-tree name/status comparison shows
only `Cargo.toml` differs within that crate. The separately registered full
manifest views show the textual change adds `[profile.dev]` with
`debug = "none"`, `incremental = false`, and `strip = "debuginfo"`; their
shown dependency declarations are unchanged. This is a narrower observation
than a claim that dependencies changed. Equal source and declarations still do
not certify lockfiles, resolved transitive dependencies, compiler settings,
targets, native behavior or whole-tree equivalence; those were not verified.

The exact 16,343-byte q2306 entry slice in `Q2306-EXACT-SLICE.json` was
independently matched both to the original 059 queue stream at transport
commit `56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e` and to its separately
path-redacted public reconstruction at
`26e360920b7b2ec32d5c76238623eaf6d6617d7e`. Those full streams have different
hashes; their equal entry slice is not an original standalone queue file. The
entry records `rejected-suspicion-source-qualified`, but explicitly has
`audit_discovery_closed_within_exact_named_scope: false`, zero body/population
credit and no runtime pass. Preserve that historical incomplete scope.

The e275 `READ-170` records the Lens 11 agent's own lib/NPY/header/frame
reads; `READ-011` attributes metadata/tensor/values to the parallel
`lens11_reconcile` agent. Their original wording says root adoption pending.
Separate root `READ-L11-706` and `READ-L11-547` mappings record adoption of
the corresponding registrations without a root reread; these are different
historical acts and do not transfer read credit to this reviewer. The latter
also contains a separately attributed corrective-extent disposition, related
but not proven an alias of q2306. The nine historical captures matched their
recorded digests and published transport-inventory size/path identities; the
older private extraction itself was not independently repeated. The prepared
packet's unregistered/truncated locator history remains separately qualified.

No source identity, historical summary, entry slice, or reconstructed record
is promoted to the missing checkpoint 3821 ledger or queue.

## Remaining questions and limits

Identify actual producers and consumers of NPY/NPZ data, supported framing and
trailing-byte policy, resource and allocation limits, and model-specific
scalar finiteness, precision, units and rig/model-instance requirements.
Resolve the relationship to existing value and corrective ledger records with
exact source evidence. Native layout/device behavior and policy require
separate authorization for any future product validation.

The earlier unregistered metadata-only locator display is preserved in
`LOCATOR-QUALIFICATION.json` and is not credited as a historical read. The
prepared report itself was registered before display and its observed
publication HEAD is not proof of committed membership. Evidence verification
checks identity and recorded displays, not personal comprehension or semantic
truth. Product source, macros, fixtures, parsers, builds, tests, shaders and
runtime probes were not executed; no repairs or publication changes were made.
