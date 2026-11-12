# Scan and verification receipt

This seal binds the source-only packet manifest in `PACKET-ANCHOR.json`.
`VERIFY-STANDALONE` checked the flat packet manifest and recorded evidence.
`VERIFY-FULL` additionally used the local Git object store and the recovered
transport manifest. Both passed; same-machine metadata is not independent
custody, reading, or semantic proof.

`PRIVATE` used a restricted same-machine original GPU capture to derive a
digest corpus in memory; it reports only aggregate corpus sizes, counts and
sanitized locations, not individual path values or path-value digests. Its
private corpus and the local object store are separately held replay
dependencies, not independent public recoverability. `PATTERN` used the
separate bounded path and credential regex scan. Both exited zero, returned
zero candidate/digest matches and no exceptions on the packet. Scanners
omit Git administrative metadata/history and cannot clear unknown encodings,
formats, secrets, or unrecognized identifying data. No product code ran.

Run `python3 VERIFY-SCAN.py --packet PACKET_DIRECTORY` for the receipt and
anchor; rerunning private-input and Git/transport checks additionally requires
the original inputs. Read `REPORT-v3.md` in the packet for attribution,
source limits and the preserved earlier report/verifier drafts.
