# Evidence added after the frozen application

Read `CURRENT-AUDIT-STATUS.md` for the ledger and queue counts. The separate
`delegated/postf2ec-followups/late-evidence/INDEX.md` describes the additional
q0 behavior review and its historical recovery limits. It remains pending
central reconciliation. Evidence sealed after this cutoff belongs to a later
checkpoint and does not change these counts.

Run `python3 -B verify_postf2ec_checkpoint.py` for the published evidence
checks. The full evidence driver was run on 173 earlier files; separately
captured additions have their own checks and bounded scans. Ciphertext is
opaque to those scans. Full recovery requires the separately held same-machine
key, historical queue anchors and documented nested keys; custody is not
independent. The original checkpoint 3821 queue and ledger remain missing.
No product repair, build, test or runtime probe has been performed.
