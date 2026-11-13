# q2952: enclosure binding, Lens 10 derivability

This bounded source assessment finds no established derivability repair in
`crates/adventuresim-tactical-core/src/city_layout/grounding/enclosure/binding.rs:1–209` at pinned 95e. The conclusion is limited to the file and
selected dependencies; it is not a closure of the lens, a defect count, or a
claim that all project derive options were exhaustively searched.

The file's single handwritten trait implementation is
`TryFrom<BindingWire> for BoundaryOwnerBinding` (`binding.rs:51–72`). It
constructs a candidate, invokes `validate`, and returns only after that check.
The local validation rejects a zero property ID, a member count other than two,
an empty element list, duplicate elements and reserved owner/gate-landing
elements (`binding.rs:109–124`). Replacing the checked conversion with direct
field construction would omit those local checks. Other scene and cell
relationships have their own checks (`binding.rs:134–172`); this observation
does not assert that the local conversion proves all those relationships.

Existing derives already cover the relevant mechanical traits:
`BoundaryOwnerBinding` derives `Clone`, `Debug`, `PartialEq`, `Serialize` and
`Deserialize`, with authored `#[serde(try_from = "BindingWire")]`; the wire
derives `Deserialize`. The digest derives `Clone`, `Copy`, `Debug`, `PartialEq`,
`Eq`, `Serialize` and `Deserialize` with transparent representation; the
admission error derives `Serialize` and `thiserror::Error`, and the projection
derives `Clone`, `Debug` and `PartialEq` (`binding.rs:4–15,51–58,175–209`).
The workspace declares serde with its derive feature, and the pinned lock
contains serde_derive 1.0.229. Its checksum-bound source template selects the
try-from path and shows a deserialize-then-`TryFrom` composition
(`serde_derive/src/de.rs:306–312,434–440`). This is source inspection, not a
project macro expansion, resolved-feature proof or runtime observation.

`BoundaryDescriptorDigest::of` (`binding.rs:16–49`) is handwritten semantic
mapping, not a handwritten serialization trait. It explicitly hashes the
wall count and ordered wall fields, f32 bit patterns in little-endian order,
then hinge and gate fields, using SHA-256. Any future alternative would need
to preserve the intended descriptor identity and representation; no available
mechanical derive is shown equivalent here. Whether the descriptor and retained
cells always correspond for supported producers remains the separate open
question in ledger 1082; it is not settled by this derivability review.

Historical 059 q2952 records 6533, required range 1–209 and EOF 209, with no
assessed Lens 10 range; its selected reconciliation at committed 8ca is
unassessed. The selected original/reconstructed q2368, q2945 and q2952 raw
records match, while their complete queue streams differ. The 6533 and e275
file identities match pinned 95e per file (Git blob
`05defe7da97ee80aecb4eb57602cbc4a7fba6541`, SHA-256
`9db0cfc4b4c3053884d835faaca49a404c9942f7f0599b3069b89729fd8d7ae3`), not as
whole trees. This reviewer personally displayed the registered current 95e
1–209 with EOF; 6533/e275 were identity-only in this Lens 10 assessment.

Recovered Root Lens 3 and Lens 2 receipts each record a fresh personal 6533
read and qualified same-lens e275 reuse without a fresh e275 personal read;
they retain their original scope and do not supply Lens 10 credit. The earlier
q2945 Lens 2 report was authored by this reviewer and independently displayed
all three revisions for that lens. Its corrected Git-object lookup, earlier
failed lookups, truncated metadata and adoption/attribution limits remain in
its original report. Historical q2368 and q1493 and ledger 1082 are related
owner/descriptor records, with their own revisions and first questions; no
alias, unique count or closure is inferred.

The initial registered q2945 and serde filename locators were tool-visible
without separately saved exact transcripts. A later q2945 locator observation
and the exact checksum-bound selected serde archive/member were captured. The
six bounded display events are reviewer-attested, without an independently
signed display witness. Exact source bytes, registrations and Git path proofs
are retained. No failed command was observed in this assessment; the prior
q2945 failures are separate.

The missing checkpoint 3821 is not reconstructed. The earlier 73 nontext or
no-positive-EOF obligations, SOURCE004, reconstructed 440 and the 774
normalized-versus-missing-pre-export-originals qualification remain open or
qualified as recorded. Strongest replay needs the separately held original
059 stream, a repository with the committed 8ca objects, and the read-only
historical extraction. The public report alone cannot recover those inputs or
prove independent custody. No product code, macro expansion, build, test,
fixture, repair or runtime probe was run.
