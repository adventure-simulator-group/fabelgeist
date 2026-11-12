# Post-dac7 prepared and stream evidence replay

REPORT.json records the evidence-only results and limits. The encrypted collection contains exact commands, complete stdout/stderr, staged prepared and stream packet files, and the separate q2812 anchors; file and directory mode metadata are recorded. Restricted source material, raw commit objects, local paths and author metadata remain encrypted. The previously held same-machine key is required; this is not independent public recovery.

`python3 VERIFY.py` checks public identities. Add `--key KEY_PATH` to authenticate the exact private inputs and check staged manifests, anchor identities and recorded command results. This verifier checks recorded evidence, not a repeat of the original seven commands or any privacy scan. Fresh strongest stream replay needs the separately retained Git object store; GPU/q2332 private scan corpora remain separate and unconfirmed. No product code, builds, tests or runtime probes are run.
