# q2962 independent Lens 8 peer review

The selected source supports the author's conditional before/after comparison.
This is independent of the author's assessment, but the reviewer previously
assessed the same assigned file under Lens 2 in q2957. That earlier work, other
readers' work and other lenses do not transfer read credit. This review does not
establish a supported input, a product regression, a defect count or closure.

At 58ea2ca the selected producer individually constructs finite framed
positions and checks head above base, then fills public cell fields
(`enclosure/projection.rs:166–219`, `spatial_geometry/coordinates.rs:42–53`).
At 95e the producer additionally passes those points through the aggregate
constructor (`projection.rs:177–229`, `enclosure.rs:24–57`,
`admission.rs:25–43`). It rejects a nonfinite sum, unmatched planar pairs,
reversed heights and a cell with no positive pair. The author's symbolic
finite-coordinate sum-overflow case follows from that ordering; the old
collider also summed points before the native call (`enclosure.rs` at 58ea:
122–143). Policy, supported inputs and old native outcome remain open.

A further conditional representability question follows from the selected
source. Head is compared to base before subtracting the gate datum; each
relative value is then formed separately (`projection.rs` at 58ea:184–216,
95e:195–229; `member_levels` at 58ea:228–262 and 95e:238–272). For example,
mathematically distinct f32 heights 2 and 1 against a gate near 100,000,000
can round to the same f32 relative height. If this happens to all three pairs
and the other checks pass, the old public fields could carry them and the new
aggregate returns `Height`. This is symbolic source reasoning only: consistent
site/gate construction, supported reach, desired policy and practical effect
are unproved, and no numerical or product probe was run. It is not a separate
count or automatic alias.

The old selected enclosure and foundation colliders pass fixed faces to native
`convex_mesh` and expect a shape (`enclosure.rs` at 58ea:122–143;
`foundations.rs` at 58ea:73–96). The new source skips exact-zero plan area,
reverses negative winding and retains positive winding (`admission.rs:46–58`),
and turns other native `None` outcomes into typed errors (`enclosure.rs:
227–252`, `foundations.rs:176–203`). If no solid parts remain,
`SupportCollision::ContactOnly` is returned (`collider.rs:35–60`); the selected
boundary and replacement consumers act on `Solid` (`tactical-server/src/
boundaries.rs:4–29`, `foundations/replacement.rs:87–120`). The old native
result, supported geometry and downstream product effects remain unknown.
Comments describe zero-area contact as valid; no compatibility requirement or
unrequested regression is inferred.

The historical 059 q2962 row records revision 6533, the assigned 1–58 range,
positive EOF 58, and no assessed Lens 8 range. At committed 8ca the selected
reconciliation is unassessed. Selected original and reconstructed q2368,
q2369, q2957 and q2962 raw records match, but the complete streams differ.
Root personally read 6533 in Lens 3 and Lens 2 respectively; their e275
records describe qualified same-lens reuse without an e275 fresh personal
read. Adler's separately recovered Lens 11 read and registration concern
source registry index 2708, which is not queue q2708. The original receipts
retain their own attribution and qualifications, do not establish the missing
checkpoint 3821, and grant no Lens 8 credit. The author's packet is by
mhr_character; this peer is wrapped_scope.

The assigned file at 6533, e275, 95e and introduction 5a is the same Git
blob `a65dc4432cb4dbea04fdd65f02e78d2d1ba81922` (SHA-256
`11bb44acefc12ab10604d09e33f8b1d26fc3958fa916255972ec6ea63e1f6d2f`). The
selected parent 58ea grounding tree lacks that path. These are per-file and
path observations, not whole-tree or whole-behavior equivalence. This reviewer
personally displayed the registered 95e and 5a assigned body plus selected
58ea/95e producer and consumer windows; 6533/e275 were identity-only in this
peer. The 15 display events are reviewer-attested; their registered bytes are
replayable but the display events have no independent signed witness.

Historical q2369 is a related native-representation record. Ledger 1083
concerns finite sum followed by nonfinite centered subtraction, a different
first operation from the sum overflow. q2957 is this reviewer's separate Lens 2
work. The author also related q2368. These are connections for central
reconciliation, without a global alias or unique-defect decision.

The author packet preserves its metadata-selection and caller-locator failures,
truncated metadata/filename displays, pre-registration history/existence-only
locators and corrected support capture preflights; no prior source credit was
transferred. During this peer a broad author selection metadata display occurred
before its specific registration and was truncated; its exact tool transcript
was not separately saved. It supplied no source credit. The selected status was
subsequently registered and captured. A history-capture invocation failed
because `python` was unavailable; a separately named `python3` retry succeeded.
Three newly named scripts and their shell stdout/stderr were created with shell
`>` in the fresh private scratch after a directory listing showed those names
absent, rather than exclusive creation. No per-file pre-write existence check
or backup was made, so earlier contents, if any, cannot be established beyond
the initial listing. No publication or product path was written by this peer.

The restricted evidence includes exact selected source bodies, registered
views, Git commit/tree/path proofs, nine history records and corrections.
Strongest replay additionally needs the separately held complete original 059
stream, a Git repository containing the 8ca objects and the read-only protected
extraction. A public packet alone cannot reconstruct those inputs, prove
independent custody or restore missing checkpoint 3821. Only evidence checks
were executed; no product code, build, test, fixture or runtime probe was run.
