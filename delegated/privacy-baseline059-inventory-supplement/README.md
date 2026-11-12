# Additive 059 reference inventory and verifier plan

This supplement leaves the earlier frozen reconstruction proposal unchanged.
The tail-aware deeper scan resolves its 315 size or depth omissions and records
one remaining damaged nested checkpoint 7 tar: the parser reaches five entries
and stops at a bad header. Its separately published tar-aware successor, with
186 entries, was scanned independently as a distinct reconstruction.

`INTEGRATION-PLAN.json` identifies all 15 statically affected current checks,
the exact checkpoint link records, and the seven literal semantic references
that overlap redacted fields. It proposes separate successor evidence
verifiers that preserve historical manifests and report the replacement as a
qualified original-to-public crosswalk. It does not claim any old exact-byte
check passed, or that product behavior was validated.

`TAILCHECK-INVENTORY.json` and `NONDELEGATED-TAILCHECK.json` contain the
superseding scans. The earlier `DEEP-INVENTORY.json` and
`NONDELEGATED-INVENTORY.json` are retained as provisional evidence: they did
not detect a tar parser silently stopping at an invalid header. Scans are
literal, bounded to the listed formats and snapshot, and do not prove the
absence of dynamic or external references.

Run `python3 verify.py` for a read-only check of this supplement. Supply
`--publication PATH` to compare its recorded script identities, transport
manifest and checkpoint 7 archive against a publication tree, read-only. It
will not run any existing verifier or product code.
