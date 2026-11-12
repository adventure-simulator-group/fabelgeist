# Scan and verification receipt

This seal binds the q2359 source-only packet manifest in `PACKET-ANCHOR.json`.
`VERIFY-STANDALONE` checked the flat packet and recorded evidence. `VERIFY-FULL`
additionally used the local Git object store and recovered transport manifest.
Both passed. The earlier preflight deliberately skipped the as-yet-uncreated
manifest; only these sealed ordinary runs assert its integrity. Same-machine
metadata is not independent custody, reading, or semantic proof.

`PRIVATE` derived a digest corpus in memory from a restricted same-machine
original GPU capture, reporting aggregate counts and sanitized locations,
not individual removed path values or their digests. `PATTERN` used a
separate bounded path and credential regex scan. Both exited zero, returned
zero candidate/digest matches and no exceptions on the packet. The private
corpus and local Git/transport inputs are separately held replay dependencies,
not independent public recoverability. The scans cannot clear unknown
encodings, formats, secrets or identifying data, and omit Git history. No
product code ran.

Run `python3 VERIFY-SCAN.py --packet PACKET_DIRECTORY` for this receipt and
anchor; rerunning private-input and Git/transport checks needs original inputs.
Read `REPORT.md` for attribution, original dispositions and source limits.
