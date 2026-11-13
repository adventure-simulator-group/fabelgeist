# Independent comparison of the prospective v3 stage builder

Reviewer: wrapped_scope. This static review is independent of the coordinator's
v3 change, not all underlying evidence. The reviewer authored q185,
peer-reviewed q24 and q3100, and reviewed the earlier applications, guards and
v2 builder. Source-read credit does not transfer. All earlier review packets
remain separately preserved.

The v3 builder retains the corrected guard's 26 pin names and the six fixed
input identities. It hard-pins the reviewed v2 guard, new root wrapper and
v2 guard-peer manifest, and checks the v3 root capture's wrapper identity. The
captured staged peer package's seven files match the separately sealed public
review and its manifest. The correction record identifies the exact v2 and v3
builder bytes. The current predecessor guard, predecessor wrapper and adjacent
derivation also match the separately sealed v2 review byte-for-byte.

The builder creates seven files exclusively in a new prepared directory,
including the installer correction and a staging receipt explicitly described
as pre-rename. It repeats the 26 input hashes before a Linux `renameat2`
`NOREPLACE` move and then fsyncs the stage directory. It removes v2's required
post-rename result write. These are static source observations; the builder was
not executed here.

Two separate later checks remain necessary:

1. The staged guard-peer manifest is checked once before preparation, outside
   the 26-name second hash pass. A change between that check and rename is not
   detected by the builder itself. The coordinator proposes a separate
   independently anchored capture that rechecks the manifest before and after;
   that future check is not a property of v3 or established here.
2. The predecessor guard and wrapper and the correction's descriptive fields
   are checked against the adjacent derivation, rather than independently
   hard-pinned by v3. They match the sealed prior review at this capture. A
   later anchored stage check should compare the staged predecessor files and
   correction against the reviewed predecessor and derivation identities.

Pre-rename failures can leave a partial or complete prepared directory.
A rename failure preserves the prepared directory and does not replace an
existing destination. A stage-directory fsync or output failure after rename
can leave the final directory present. The pre-rename receipt alone does not
prove that rename or fsync succeeded; action logs and a later exact capture
must establish final placement and any observed failure. Namespace atomicity
alone does not prove crash durability or independent custody.

The prior v2 builder review independently replayed both later custody verifiers
without keys, not the authenticated restore or the 687-value prior inventory
comparison. This v3 comparison does not repeat those calls. The original 059
and reconstructed records, normalized 774 attribution, missing checkpoint 3821,
SOURCE004, 73 nontext/no-positive-EOF obligations, point-in-time main and all
thirteen incomplete lenses remain separately qualified. Capture files were
rehashed, not atomically locked. No staging installer, product code, build,
test, fixture, runtime probe, repair or product validation was executed. Finite
privacy scans cannot exclude all encodings.
