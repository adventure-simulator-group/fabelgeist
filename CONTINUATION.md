# Audit continuation

This continuation uses `BASELINE-20261110.json` as its explicitly incomplete
working authority at source revision
`c100638b720a7281a5fc96f6a3c5c23d5dda85da`. It is not a restoration of the
missing checkpoint 3821. All thirteen lenses and the connected repair ledger
remain open; no implementation or product validation is authorized here.

`CONTINUATION-PROGRESS.json` gives the present counts and qualifications.
`COVERAGE-L01-ROLES.json` is the Lens 1 worklist of distinct original roles;
`LOCKFILE-DELTA.json` preserves separate historical and current comparisons.
`RECONCILIATION-059.json` indexes the historical queue without turning its
entries into present findings. The `ASSESSMENT-*.json` records distinguish
bounded current source observations from historical proposals and name the
remaining work. An observation or matching source hash does not by itself
close a source scope, lens, finding, alias or repair.

`baseline-059/` contains the recovered checkpoint 059 ledger, queue, selector
and selected-state bytes at their original checkpoint. The reconstructed
snapshot 440 and the comparison in `SELECTED-STATE-COMPARISON.json` remain
separate from the final selector, whose bytes are missing. The recovery
package's `README.md`, `AUDIT-RESUME.md`, manifests and individual evidence
qualifications continue to control their respective historical records.
`LOCAL-BASE-REUSE.json` records the read-only manifest verification and bounded
search of older collections; an unsuccessful search is not proof of absence.

For the recoverable evidence on this branch, verify from its root:

```sh
python3 verify_recovered.py
python3 verify_selected_states.py
python3 verify_lock_inventory.py
python3 verify_role_worklist.py
python3 native-evidence/bevy-replicon-0.41.1/verify.py
python3 native-evidence/rust-nightly-2026-07-13/verify.py
python3 native-evidence/serde-saphyr-1.2.0/verify.py
python3 native-evidence/serde-derive-1.0.229/verify.py
```

These commands verify evidence bytes, not product behavior. The original
36,487-file transport is available at its separately pinned historical
revision and can be restored using the recovery `README.md` to a **new**,
case-sensitive destination. An already extracted copy can instead be checked
read-only with `verify_local_checkpoint_059.py EVIDENCE_ROOT`. Never restore
onto the user's existing evidence directory. A same-machine branch fetch and
verification does not establish independent-machine recovery.
