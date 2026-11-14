# Party enrollment: iterative audit checkpoint

This is an additive checkpoint after the published f2c handoff. It covers
`PartyEnrollment` and the selected admission, configuration, timeout and
outcome interfaces, not the remaining repository-wide audit. Source reviews
were made against commit `95e63628f9d70b8dd6e6fede64334696c6995009` (tree
`1eec97de340d9b7debbb30ca0c0fb2240a73b93f`). The repair is commit
`d276139b62e62bcbd1ed766433146c686765aab7`, based on that revision, in
[draft PR #1005](https://github.com/adventure-simulator-group/fabelgeist/pull/1005).
The exact patch SHA-256 is
`c811f160152ec3b83b397e1fb3979a889ec7360a91269d85bafa69a0f94e35ef`.

The [slice ledger](LEDGER-v3.json) records all thirteen bounded pre-repair
reviews, their distinct readers, post-repair source reviews, the selected
finding, and remaining questions. It is the frozen local v3 record as of its
cutoff; its statement that publication was pending describes that earlier
moment. The repair addresses existing q3134 / ledger 406 on this branch; it is
not a new unique defect count or a merged closure. A party member who is loaded
or loading no longer triggers abandonment at zero grace; an actually empty
party still expires immediately once enrollment has begun.

The standalone system regression failed before the fix and passed afterward;
16 mission tests and the formatting check passed. The first exact test filter
selected zero tests and is not validation. The first full lint invocation
completed the main workspace checks, then failed because the offline cache
lacked `rawzip` for a separate workspace. A network-enabled retry downloaded
that dependency and was still running at this checkpoint cutoff. Later
validation results must be recorded separately. No connected database or
real-player path was exercised by the regression.

Open questions include the effect priority when sealing and abandonment occur
in one empty-party update (ledger 1097), the representable but unobserved
mismatch between retained and passed grace values, roster and duplicate entity
policy, other float inputs, and the distinct reconnect-snapshot timer. The
post-repair review does not close any whole-repository lens or historical queue.

## Historical and evidence boundaries

The immutable prerequisite is [f2c](https://github.com/adventure-simulator-group/fabelgeist/tree/f2c6d28a48bfa72c0801e9f33387eb6e448f9a18).
Its ledger and the mission enrollment and central follow-up assessments retain
their original records. Selected original and reconstructed 059 rows match;
the complete streams differ. The 6533 fresh Lens 3 read and e275 qualified
reuse remain separately attributed; matching selected files is not whole-tree
equality. Earlier q2982, q2983, and q2987 reports in `historical/` are
unadopted, separately attributed evidence; their examples describe pre-repair
source and are not promoted into checkpoint 3821.

Original checkpoint 3821's ledger and queue remain missing. Reconstructed
snapshot 440, normalized 774, SOURCE004, and the nontext/no-positive-EOF
obligations keep their prior qualifications. Original 059, the protected
extraction, prior keys and Git objects remain separately held same-machine
replay dependencies. This checkpoint does not establish independent custody or
full Git object closure. The unpublished 13-row candidate and other later
work remain outside this slice and are not counted as installed here.

The public review packets are bounded source evidence. Complete selected
restricted inputs, reviewer records, material corrections, and raw validation
logs through their respective cutoffs are in `custody/` and
`process-custody/`. Their keys are held separately on the same machine. The
second envelope contains the first custody verification and the offline lint
failure; the online retry and the second envelope's own subsequent verification
and publication process are outside its cutoff. They must be retained for a
later checkpoint.

Each envelope uses a fresh key for one AES-256-GCM object. From its directory,
`python3 -I -B VERIFY.py` checks the ciphertext against its supplied manifest
without authenticating it. With a separately held key and a fresh `--restore`
destination, `VERIFY.py --key KEY --restore DESTINATION` authenticates and
restores the indexed regular files and modes. The procedure requires POSIX,
trusted nonconcurrent destination and source parents, disjoint paths, Python
and `cryptography`; it is not a defense against concurrent same-user path
replacement. The no-key check alone does not prove origin or authentication.
Same-machine authenticated restoration and separate source-byte and mode
comparisons succeeded for both envelopes. Owners, ACLs, timestamps, hardlink
topology, crash durability and independent backup are not claimed.

## Fixed envelope identities

- Slice manifest SHA-256: `4ed710362d7f610b6914562cdcfc6eefa4212a67dd1a22065c9df61be2ca56ed`
- Slice verifier SHA-256: `fb38cbf18961daed2de7bc6d434753f22b632f5d89720fc04391c66a44c61e0c`
- Process manifest SHA-256: `454bad04bdfe06f899e4b62f5e2965d8c55b8cda2d0999a9047caefcc861a4c0`
- Process verifier SHA-256: `fb38cbf18961daed2de7bc6d434753f22b632f5d89720fc04391c66a44c61e0c`
