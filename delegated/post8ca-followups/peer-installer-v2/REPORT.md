# Independent review of the prospective v2 stage builder

Reviewer: wrapped_scope. This static builder review is independent of the
coordinator's staging change, not all underlying evidence. The reviewer
authored q185, peer-reviewed q24 and q3100, and reviewed earlier application
and guard packets. Source-read credit does not transfer. Those packets remain
separately preserved.

The builder's 26 named pins exactly match the corrected guard's set. All six
hardcoded manifest and v3 root-capture hashes match the registered current
stage bytes. The builder reads and hashes every named pin before preparation,
writes five exclusive files inside a newly created prepared directory, repeats
the 26 input hashes, fsyncs the prepared directory and uses Linux `renameat2`
with `NOREPLACE` to move it to the final guard directory. The capture and
second hash pass were not an atomic snapshot. This review did not run the
builder or install a guard.

Two issues require separate treatment before staging:

1. The builder compares the predecessor guard, successor guard and old wrapper
   to identities in the adjacent derivation record. It does not independently
   hard-pin the reviewed successor guard hash. Mutually changed draft bytes and
   derivation could pass this check unless a separately anchored stage capture
   or successor builder pins the reviewed identities.
2. The builder writes its exclusive `STAGE-RESULT.json` only after the rename,
   without earlier reservation or an absence preflight. An already existing
   result, stage-directory fsync failure or later write/print error can leave a
   renamed final directory without a new durable success receipt. Earlier
   failures can leave a partial or complete prepared directory. There is no
   automatic rollback. Atomic no-replace rename concerns the namespace move;
   it does not establish full crash durability or an independently trusted
   stage anchor.

The two newly staged later-custody verifier interfaces support the corrected
guard's exact no-key invocations. Independent evidence-only replays of both
passed with empty stderr, as did this review's static comparison; 36 captured
metadata files and 16 external ciphertext identities rehashed unchanged. The
checks do not decrypt or authenticate the ciphertext, repeat the recorded
restoration, compare the 687-value prior nonce inventory, establish its
provenance, or establish independent custody. The 16 ciphertext bytes remain
separately held in the stage; this packet records their whole-file identities
and does not duplicate them.

The separately reviewed guard's directory and full POSIX-mode limitations,
optional predecessor replay, original and reconstructed history distinctions,
missing checkpoint 3821, SOURCE004, 73 nontext/no-positive-EOF obligations and
open thirteen lenses remain qualified. No installer, product code, build,
test, fixture, runtime probe, repair or product validation was executed.
Finite public privacy scans cannot exclude all encodings.
