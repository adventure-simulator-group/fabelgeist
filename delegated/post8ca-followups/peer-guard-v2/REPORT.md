# Independent comparison of the prospective guard v2

Reviewer: wrapped_scope. This static successor review is independent of the
coordinator's guard change, not all underlying evidence. The reviewer authored
q185, peer-reviewed q24 and q3100, and reviewed the earlier applications and
guard. Source-read credit does not transfer. The prior guard review and its
findings remain separately preserved.

The captured v2 guard exactly matches the documented transformation from the
captured v1 guard; unrelated guard bytes are unchanged. It removes the special
allowance for four uncaptured late `closing/` files. The v2 root wrapper removes
its execution of a late closing verifier and has no other byte change. The
successor guard reports whether the nested predecessor guard was supplied or
skipped, and explicitly says that ciphertext checks are no-key checks and that
authentication and prior-inventory comparisons are not repeated.

All three prospective seven-file root captures link by predecessor SHA-256.
Only the assessment report row changes from v1 to v2, and only the root-wrapper
row changes from v2 to v3. The v3 row matches the captured wrapper's byte
length, SHA-256, Git blob identity and declared Git mode. All captured source
and parent identities agree. This comparison inherits the earlier review's
queue, ledger, archive and source-attribution limits; it does not perform a new
source assessment or independently verify eventual installed bytes.

The prior staging installer names 25 pins, targets root capture v2 and copies
the earlier guard. The successor guard requires 26 pins including root capture
v3. A matching successor installer, its exact pins and the later stage capture
need separate review before staging; this says nothing about any other installer
being prepared. The stage capture and its external anchor, complete POSIX
file/directory modes, parent Git objects, key custody and authenticated restore
remain separate. Later encrypted and receipt interfaces were not staged at the
initial capture and were not verified in this review.

The reviewer's first capture attempt created one exclusive scratch file then
failed because an output handle shadowed a source-path variable. The incomplete
file was retained. Its traceback was tool-visible only and is reconstructed in
the restricted record. A separately created capture directory and corrected
script passed; the input bytes were rehashed in a second pass, not atomically.
The static comparison was independently replayed without executing the guard,
wrapper, derivation script, installer or product code. No product build, test,
fixture, runtime probe, repair, validation, audit closure or recovery of the
missing original checkpoint 3821 follows. Finite privacy scans do not exclude
all encodings.
