# Post-dac7 evidence recovery navigation

This guide covers the observed post-dac7 follow-up staging state and is not a
claim of audit completion. It identifies how to recover and verify evidence;
it does not authorize repairs, product execution, builds, tests, rendering,
fixtures, probes or product validation. The pinned product source is
`95e63628f9d70b8dd6e6fede64334696c6995009`. Recoveries of earlier records keep
their original revisions, reviewers and qualifications; reconstructed records
remain separately labeled. None establishes the missing checkpoint 3821.

The observed initial nine-packet `PENDING-DELEGATES-INITIAL.json` has SHA-256
`153d5d4fd4dd4f3f1bd7a9bbdaa8087c346a9b60beccd76d1c7fcdbd5bd472e1`.
Its ten-packet successor `PENDING-DELEGATES.json` has SHA-256
`4b2c44c25ce05f96131aa823803c791006dec100c0e802e2f5e04d3e110d7759`.
The successor adds q2945, a separately attributed Lens 2 packet still pending
central review. The coordinator reports a separate full evidence replay for
it; this guide's driver does not repeat that replay. A later q2968 review is
outside this frozen checkpoint. Check the final authenticated publication index
and manifests: if they change, independently review and update the pinned
identities before using these tools. The driver deliberately stops on mismatch.

## Recover the authenticated collections

The intended published location is
`delegated/postdac7-followups/encrypted-originals/`. At this guide's inspection
it was still being assembled. Its final `VERIFY.py`, manifest, archive count,
command-line interface and ciphertext identities were not yet available for
independent review here. Verify them against the final published checkpoint,
read the final verifier and its instructions, and authenticate each collection
before restoring it. Do not invent verifier arguments or restore over existing
data. The intended restored layout is `<RESTORED>/<label>/evidence/` for each
named staged private-inputs directory. Confirm this against the final manifest.

The primary decryption key is separately controlled. The q2910, q2754 and
independent-four secondary keys are expected inside their respective
primary-authenticated private collections; the independent-four key also
unlocks the separately authenticated prepared/stream replay seal described
below. Keep all keys, restored data and replay logs private. Same-machine key
custody is a recovery dependency, not independent public recoverability.
Selected raw Git objects and historical records contain private metadata.
Preserve the exact files and modes, encrypted originals, attribution, failed
commands and recovery limitations. Do not publish local path values or key
bytes. Do not discard a failed attempt or overwrite the initial index.

The guide assumes an offline environment with unoptimized Python 3, the
`cryptography` package for AES-GCM, Git, their standard dependencies and a
private existing scratch parent. Those tools, dynamic libraries and the
operating system are external dependencies; identical binaries and complete
Git history are not captured. Verify the final package in an isolated offline
workspace before running any evidence scripts. The scripts here create only a
new private scratch directory and new files; retained logs include local paths.

## Recover and verify the q2812 anchors

The earlier dac7 q2812 anchor collection is separate from the q2812 pending
semantic record. The prior prepared/stream replay seal contains the exact
anchor files and modes, including the original 059 queue, the reconstructed
059 queue, commit objects, ledger and supporting identity data. Its public
verifier authenticates and checks the recorded evidence but does not itself
restore the anchors. The helper in this guide authenticates that exact seal,
checks every encrypted member against its private manifest, creates a new
private output directory, restores the anchor files with their recorded modes,
and runs the pinned anchor verifier. It does not run product code.

After authenticating and restoring the final primary collections, set the
following variables to locations under your own private recovery workspace.
`SCRATCH` must already exist with private access and `SCRATCH/q2812-anchors`
must be absent. The restored anchor files retain their original modes inside a
mode-0700 parent directory.

```sh
python3 -B RESTORE-Q2812-ANCHORS.py \
  --seal "$RESTORED/independent-replay/evidence/public" \
  --key "$RESTORED/independent-four/evidence/private-key/KEY.bin" \
  --output "$SCRATCH/q2812-anchors"
```

The seal's verifier checks previously recorded results; it does not rerun the
original privacy scan. If the seal, its secondary key, the Python cryptography
implementation, or any anchor is unavailable, the strongest q2896, dialogue
and prepared replays cannot be claimed. Retain any failed restore and its
private logs; the helper never removes an output directory.

## Replay the selected evidence

The driver accepts `--layout restored` for the intended authenticated layout,
or `--layout stage` for the original staging `private-inputs` layout. `PUBLIC`
is the directory containing the two pending indexes, `transport/`,
`review-transport/`, `review-extra-transport/`, `checkpoint-dac7/` and
`q2910-packet/`. Use `--public` for those verified published bytes and
`--private` for the authenticated collection root. Choose an existing private
scratch parent; the driver creates a new uniquely named mode-0700 child and
writes each command, stdout and stderr there with exclusive creation. It never
prints key bytes or local path values. Its reported basename locates the
private receipts under the chosen parent.

```sh
python3 -B REPLAY-FOLLOWUPS.py \
  --public "$PUBLIC" \
  --private "$RESTORED" --layout restored \
  --anchors "$SCRATCH/q2812-anchors" \
  --scratch-parent "$SCRATCH"
```

The driver checks pinned script and control-manifest identities before use,
restores the three public transport groups to new scratch directories, and
runs evidence-only checks for:

- the inherited dac7 checkpoint receipt, selected v6 Git store, exact q2812
  anchors and q2904 standalone store;
- q2748, q2754, q2896 and q2904 transported packets, plus the later and
  separately attributed q2910 packet with its own secondary key;
- the four-record and dialogue reviews, prepared and stream reviews, stream
  restricted input binding, and stored stream scan receipt binding.

A clean run confirms only those selected checks and the statements made by
their verifiers. It does not establish complete source history, feature
activation, native behavior, independent custody, a fresh corpus privacy scan,
or a queue or lens closure. The separately reported q2945 replay requires its
own published packet, authenticated q2945 collection, restricted ledger and
historical values and q2812 anchors; inspect its own verifier and receipt. It
is not included in the four transported packet checks above. The older pending
q2786, q2799, q2812 and q2884 records retain their earlier attribution and
status. The q2773 stored privacy scans are receipts of earlier bounded scans,
not fresh scans of the present corpus.

The `shared-replay-v6` store validates 163 selected loose Git objects and 90
path chains in the observed state; its own verifier does not prove closure for
all commands. It is reconstructed selected object storage, with no complete
history, parents, refs or original pack bytes. In an independent replay the
q2904 verifier failed with v6 because a tree needed by its second Git
name-status comparison was missing. The exact failure is retained privately.
The separately authenticated `q2904/evidence/standalone-replay/` collection
contains a different selected store and its own verifier; both that verifier
and the transported q2904 verifier passed using this store. The driver uses
that store for q2904 rather than silently enlarging v6 or claiming v6 is
complete. Preserve both stores and the failure receipt.

The recorded `checkpoint-dac7` and extra replay verifiers have bounded scopes:
verifying stored receipts is not rerunning all prior checks or fresh scans.
Previously observed staging replay helpers contain machine-specific paths and
may write into the staging area; do not run those helpers for portable recovery.
The driver does not modify any staged evidence or publication tree.
