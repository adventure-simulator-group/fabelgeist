# Qualified 059 successor evidence verifiers

This packet contains separately named successors for the checkpoint 15
reconciliation verifier and checkpoint 16–29 link verifiers. The captured
original scripts remain unchanged. The deterministic builder checks each
original hash, makes only the listed root and 059 identity edits, and reverses
those edits in memory to the exact original script bytes. A second build is
retained for byte comparison.

The helper checks the two historical 059 tuples against the original transport
manifest and checks the proposed public files against their separate SHA-256,
length, and Git blob. It reports `QUALIFIED_ORIGINAL_TO_PUBLIC_CROSSWALK`; it
does not report a direct original/public byte match or prove private path
redaction from public data alone. All other original assertions remain strict.
The evidence tree and public directory are scanned for symlinks before reads;
no symlink overlay or full-tree copy is used. A preflight cannot protect
against an adversarial concurrent filesystem race.

Each successor accepts `--publication PATH`, and optionally `--baseline-dir
PATH` for separate reconstructed files. Fifteen source/evidence checks passed
against the captured publication working tree using the separate staged public
files; this is not a full driver, product validation, or an old-verifier pass.
`SEMANTIC-POINTERS.json` records the seven literal overlapping references.
`DRIVER-INTEGRATION.json` describes preserving the full driver's order while
substituting only these 15 scripts in a separately named qualified driver.

Run `python3 -B verify_stage.py` for a read-only integrity and preservation
check. Optional `--publication`, `--public-dir`, and `--private-dir` arguments
independently check external source identities, public blobs and the privately
retained original/public semantic comparison. The private directory must not be
published. No old verifier or product code is executed by the stage verifier.
