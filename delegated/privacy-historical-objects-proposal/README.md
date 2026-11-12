# Privacy-recovery proposal

This additive proposal binds three original recovered JSON objects to
privacy-reconstructed copies without presenting those copies as original
evidence. Two copies reuse previously published safe identities; one is new.
`CROSSWALK.json` records exact hashes and string-token byte offsets.
`INTEGRATION-PLAN.json` proposes qualified changes to original-hash-named
paths and verifier assertions; no changes are made here.
`DEPENDENCY-INVENTORY.json` records the bounded, read-only search and original
hash dependencies; `HISTORICAL-QUALIFICATIONS.json` preserves selected old
revision and reader qualifications. `EXPOSURE-QUALIFICATION.json` records two related raw
objects outside this proposal. Existing publication history is unchanged.

Run `python3 verify-proposal.py` to check only this stage. Add
`--originals-root DIRECTORY` to verify each substitution byte-for-byte against
the separately retained private original objects. Add `--publication-root
DIRECTORY` to recheck observed dependency hashes and original Git tree
bindings. The latter checks can report drift if the live working tree changes.
The original-prefix check defaults to the current user home; a different
private original prefix may be supplied with `--original-home-prefix`.
The verifier writes nothing and executes no old verifier or product code.
The private originals are not included in the public stage.
