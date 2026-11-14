# WeatherSnapshot scene admission: iterative audit checkpoint

This additive checkpoint follows the [party-enrollment validation update](../party-enrollment-20261114/VALIDATION-UPDATE.md)
at `7e921df3d730b09fce4c741296ce37fc72550e85`. The weather slice is based on
product commit `d276139b62e62bcbd1ed766433146c686765aab7`, the head of
[PR #1005](https://github.com/adventure-simulator-group/fabelgeist/pull/1005).
The selected weather and scene files individually match the earlier audited
`95e63628f9d70b8dd6e6fede64334696c6995009` blobs; that is not whole-tree equality.
The repair is commit `9591ab3d79485390fd079aefd7e98c5cc088ceca` in
[draft PR #1007](https://github.com/adventure-simulator-group/fabelgeist/pull/1007),
stacked on #1005. Its patch SHA-256 is
`f3914e0ad025419b26d7d2442b868b659f54e36a872f389456266d1c453133a8`.

The [slice ledger](LEDGER-v6.json) records thirteen bounded pre-repair lenses,
individual readers, post-repair rechecks, connected questions, validation and
remaining scope. It is a frozen pre-publication record; references to a draft PR
and pending publication describe that cutoff. The change repairs the existing
ledger 346 scene-temperature comparison on this branch; it is not a new unique
defect count or a claim of merged closure.

The original comparison added five to an `i32` temperature before comparing
its dew point. A scene-file integration regression reproduced an overflow
panic before the change and passed afterward. Widening both operands preserves
the existing mathematical comparison without imposing a physical range. The
integration test covers ordinary thresholds and both integer extremes through
file loading, JSON decoding and scene validation. The existing scene-catalog
test, `just fmt-check` and `just lint` also passed. An earlier command used an
incorrect draft path and Cargo selected no test target; that failed attempt is
preserved separately and is not the regression result. These tests do not run
the server or establish that a realistic weather producer reaches the extreme.

## Historical and open boundaries

The earlier [f2c assessment](https://github.com/adventure-simulator-group/fabelgeist/blob/f2c6d28a48bfa72c0801e9f33387eb6e448f9a18/ASSESSMENT-SCENE-WEATHER-ADMISSION-95E.json)
links ledger 346 to q1579 and the separately attributed Segovb and Adler scene
reads. q1579's historical wording said `i16`; the selected historical and
current owner is `i32`. The old wording and reader credits remain unchanged.
The selected original and reconstructed q1579 rows match as parsed; their full
queue streams differ. No earlier evidence is promoted to checkpoint 3821.

The physical temperature range, extreme direct strategic-survival arithmetic,
weather-clock and raw-coordinate boundaries, route precipitation consistency,
and tactical weather provenance remain separate open questions. A read-only
metadata scan selected 34 weather objects among 44 tracked scene JSONs: 32
stored unaligned intervals. This is not a validation run and does not establish
which records are accepted. It prevents assuming a blanket alignment rule
without resolving authored-fixture policy. The file metadata/read-size race
also remains distinct from the repaired temperature operation.

Original checkpoint 3821's ledger and queue remain missing; SOURCE004,
nontext/no-positive-EOF obligations and the other recovery limits remain open.
The earlier f2c and 7e checkpoints, original 059, protected extraction,
historical Git objects and previous keys are separate dependencies. No
independent backup, full Git closure, historical queue closure or
repository-wide lens closure is claimed. Third-slice candidate selection is
outside this checkpoint.

## Recoverable evidence

`custody/` contains an authenticated encrypted archive of the bounded slice,
review packets, validation logs, corrections and selected prior checkpoint
post-cutoff controls. The key is held separately on the same machine. A fresh
key was used for this one AES-256-GCM object. Same-machine authenticated
restoration and an independent source-byte, directory, mode, root-entry and
metadata comparison succeeded. The keyed restore receipt and private index
hash remain restricted. Owners, ACLs, timestamps, hardlink topology, crash
durability and independent custody are not claimed.

From the custody directory, `python3 -I -B VERIFY.py` checks only ciphertext
identity against the supplied manifest. With the separately held key and a
fresh disjoint destination, `VERIFY.py --key KEY --restore DESTINATION`
authenticates and restores the indexed files. The procedure requires POSIX,
Python and `cryptography`, and trusted nonconcurrent source and destination
parents; it is not hardened against concurrent same-user path replacement.
The no-key check alone does not authenticate the archive or prove its origin.
Later PR/CI events and this envelope's own publication controls are outside the
cutoff and must be retained with a later checkpoint.

- Custody manifest SHA-256: `e4d86cf92d257f5a86de04c6d12616078167f6dc2d57fe06c87df6554edbba2e`
- Public verifier SHA-256: `9c762d52b0f3e4325e10a3dfe82298222e5e55137bfa7645299bda5325c45784`
