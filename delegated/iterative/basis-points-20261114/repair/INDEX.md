# Basis-point repair and weather-review checkpoint

This checkpoint follows the
[basis-point discovery](https://github.com/adventure-simulator-group/fabelgeist/blob/ab44f8e201d374dab55ed7410b8e73c97a252bde/delegated/iterative/basis-points-20261114/discovery/INDEX.md).
The bounded discovery revision was
`5f84afc713601804ca8f9f096d3781018c59a63d`. The initial repair commit
`89061a8623cca1f46fe6de4d7103f9945005db70` was rebased without changing its
patch to `29ecf3ca12e3dfbfacb36b72a56ae130d4fc48fd`. The
[stack record](STACK-REBASE.json) retains the original, intermediate and final
revisions; prior reviewers keep their recorded revisions and attribution.
The basis candidate had not yet been published as a PR at this cutoff.

## Basis-point admission

The two bounded fraction owners previously allowed Serde and feature-gated
SpacetimeDB decoding to construct values outside the ranges enforced by their
constructors. The candidate routes both decoding paths through those
constructors, retaining the existing scalar Serde serialization and the native
named-product registration. The [ledger](LEDGER-v8.json) records the thirteen
bounded pre- and post-repair lens reviews and their separate open questions.
The ledger retains its discovery-era wording: the admission lens trace says
macro expansion remained open, and the finding says no invalid payload or
runtime decode had been observed. Those statements describe the earlier
assessment, not the later evidence. The feature-enabled expansion was then
inspected, and the unsigned Serde pipeline regression was run before and after
the repair. Native malformed decoding remains untested.

A real investigation pipeline observation with invalid nested confidence was
accepted before the repair and rejected afterward; its valid record
round-tripped. The feature-enabled world-schema suite passed 84 tests, an
existing disease behavior test passed, and the existing three-test SATS
integration harness passed. That harness does not exercise these exact owners.
Feature-off checking, feature-on compilation, formatting and full lint passed.

The saved feature-enabled expansions and independent source review support
unchanged generated serializers and native visitor shape, owner TypeIds,
names and primitive fields. They are static evidence, not an executed
typespace or native decoding comparison. Signed Serde negative and endpoint
cases, and native decoding, BSATN validation and schema behavior for these
owners remain untested. Actual supported malformed ingress, deployed feature
selection and previously stored invalid values remain unproved. Rejecting
out-of-range values is the intended clean contract; if such values exist in
stored data, they could fail to load.

## Weather review and stack

The user's review of [PR #1007](https://github.com/adventure-simulator-group/fabelgeist/pull/1007)
asked about widening temperatures to `i64` and flagged the temporary scene
filename. The updated comparison uses the named existing tolerance and
saturating addition: for the strict comparison, a threshold beyond
`i32::MAX` cannot be exceeded by an `i32` dew point. The test derives its
temporary filename from the selected fixture and records the cleanup path only
after exclusive creation succeeds. Earlier candidate versions, an identified
failed-create cleanup issue, and its correction are retained in the archive.

The final scene-file boundary regression, formatting and full lint passed on
the combined stack. The [weather validation record](WEATHER-VALIDATION.json)
states the checkout and test-time provenance limits. At the recorded remote
observation, #1007 was open at
`6432a845ccc3690f1cbe99dcc11f762754b73ddb`, and the rebased
[PR #1008](https://github.com/adventure-simulator-group/fabelgeist/pull/1008)
was open at `f38f1188e85c4ed23b578534a537129eb07ea34b`. Their changed-file
lists were checked. No review thread was replied to or resolved.

## Historical and recovery boundaries

The existing 95e Serde assessment, f2c ledger entry 27 and recovered 059 q131 /
ledger 443 are related records of the same selected admission family, not new
unique findings. The selected q131 source row retains its null recorded
revision, Git path and reader attribution; its separately identified Adler337
receipt is not a substitute for those fields. The native q2332 relationship
remains qualified. Matching file bytes do not transfer reader credit or prove
whole-tree identity.

The original checkpoint 3821 ledger and queue remain missing. SOURCE004, the
73 historical nontext/no-positive-EOF contracts, and other unresolved recovery
obligations remain open. No earlier record is promoted into checkpoint 3821.
The f2c baseline, previous checkpoints, original receipts, dependency archives,
selected Git objects and earlier keys remain dependencies. This is not full Git
closure or an independent backup.

## Recoverable evidence

`custody/` contains an AES-256-GCM archive of the post-discovery basis repair,
the prior discovery checkpoint's post-cutoff controls, and the weather review
and rebase evidence. A fresh key is held separately on the same machine.
Authenticated restoration succeeded; an independent comparator matched 216
files and 28 directories, their bytes, modes and root membership. A separate
check matched all 243 pre-capture inventory entries in the restored copy. The
inventory excluded itself, which the full comparator checked separately.
Owners, ACLs, timestamps, hardlink topology, crash durability and independent
custody are not established.

From `custody/`, `python3 -I -B VERIFY.py` checks ciphertext identity only.
With the separately held key and a fresh disjoint destination,
`VERIFY.py --key KEY --restore DESTINATION` authenticates and restores the
indexed archive. POSIX, Python, `cryptography`, and trusted nonconcurrent
parent directories are required. The current build, keyed receipt, key locator,
publication controls and work after the cutoff are outside this envelope and
must be preserved with a later checkpoint.

- Ciphertext SHA-256: `6fee5124a8ac745c75faeb34694826dfc059bc26b6f35b8339be445ad35dfb25`
- Public verifier SHA-256: `9eed4782526631597b1a09a099312950f7ef298a30855950afc9af1c6a39a964`
- Ledger SHA-256: `0044fbe18df8db2b413a6e44d45fd91b68255c7a464f1b7f1d34fed6b4e4d7cb`
