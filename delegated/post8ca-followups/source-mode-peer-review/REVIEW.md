# Independent review of the source-custody mode wrapper

Reviewer: `final_guard_peer`. This is a static, bounded review of the additive
source-custody wrapper and saved mode challenge. The wrapper manifest SHA-256
is `e772e571c1ae6cdc18d987433db0f52246245d4495142303ad64a73943929f96`;
the original receipt manifest is
`cc7a61edda16c75fbfdc2ec58e39d75ff1e5cc9409d5df19e60196c395f4af2d`.
The captured records and their dependencies are same-machine. I did not modify
the stage, execute the wrapper or original verifier, repeat authenticated
restoration, run product code, or validate product behavior.

The original receipt and its original verifier remain separate from the new
wrapper. The captured stage receipt matches the separately captured 0644
challenge copy byte-for-byte for all four files, including the original
verifier and manifest. Its three manifest-listed members match their recorded
hashes and 0664 modes. The source-delivery control record binds the same
receipt manifest and encrypted manifest hashes. These comparisons establish
local byte identity at the captured cutoff, not independent provenance or
uninterrupted custody.

The saved challenge records a 0644 copy with unchanged bytes. Its original
full-mode verifier exited 1 at the mode comparison; the additive checker exited
0. The saved stdout and stderr hashes match the captured logs, and the runner
shows the copied files were changed to 0644. I checked those records, not a
new execution. The original verifier enforces the full modes recorded for its
three manifest members; it does not itself establish the original root or
manifest-file mode. The wrapper compares bytes and executable-bit presence for
those same three members. It hashes the receipt manifest, four ciphertexts and
final scan, but does not compare modes of the receipt manifest, ciphertexts,
encrypted verifier or README, wrapper files, or directories. The encrypted
verifier and README are checked for expected names by the wrapper, not hashed
by that wrapper. Other separately sealed stage controls may have their own
scope.

One precision correction is needed for the wrapper's broad Git-mode wording.
Its expression treats *any* of the three execute bits (`0o111`) as executable.
For regular-file mode-change detection with executable-bit trust enabled,
[Git 2.43.0 checks the owner execute bit](https://github.com/git/git/blob/v2.43.0/read-cache.c#L259-L273).
For example, 0654 has no owner execute bit but does have a group execute bit:
the wrapper would reject it against a recorded 0664 even though that Git
mode-change comparison would not distinguish the owner execute state. The
actual 0664-to-0644 challenge has no execute bits in either mode and remains
valid. Either describe the wrapper as a stricter any-execute check for these
non-executable receipt members, or revise it additively to match the intended
Git owner-bit rule and document configuration and checkout assumptions. No
mode change or wrapper repair was made in this review.

I separately compared all four ciphertext bytes and sizes to the captured
manifest. Their four 12-byte nonce prefixes were distinct and disjoint from
the separately supplied 646 distinct 12-byte inventory values. The inventory
matched the captured control manifest, whose hash was in the delivery result;
this establishes that bounded comparison, not completeness or provenance of
the prior inventory. The wrapper itself makes that comparison only when an
inventory is supplied, and its default output explicitly says the prior check
was not repeated. Its verification of restoration and scanning checks the
bound recorded results; it neither authenticates ciphertext with the key nor
repeats the restoration. The separately held key, original evidence and prior
object inventories remain dependencies. A finite metadata privacy scan is not
a general privacy proof.

This check makes no product finding, implementation, lens closure, repair,
independent-backup or missing-checkpoint recovery claim. The wrapper and its
self-consistent hashes need an independently trusted outer binding for any
external authenticity claim.
