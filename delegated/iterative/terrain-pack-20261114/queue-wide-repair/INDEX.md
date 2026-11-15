# Routing cost: bounded repair and evidence

This checkpoint continues the [bounded Queue audit](https://github.com/adventure-simulator-group/fabelgeist/blob/e1d3542a493ac1111e190cfdd31a9be93168e118/delegated/iterative/terrain-pack-20261114/queue-audit/INDEX.md).
The later [connected ledger](LEDGER-v17.json) preserves all earlier values and
records the selected repair on product commit `cb06af32370bd32bde69fac004af34ab3ada877f`, based on
`d90045ad0cd5fb7c34722550b56d84a40427fe58`. Thirteen lenses are assessed only within
their recorded scope; broader discovery and the listed follow-ups remain open.

On a fixed public generic-grid case, the old routing API returned 70 minutes
although a separately checked feasible route costs 4,140 seconds (69 minutes).
The exact returned path and seconds, global optimum, real pack reachability and
a deployed incident were not observed. The repair aligns the heuristic with
per-edge distance truncation and both cost ceilings, and carries step, queue,
reconstruction and span-bucket intermediates in a wider representation. Public
minute and distance fields remain `u64`; previously overflowing or saturating
extreme cases may produce different results.

The all-features terrain suite passed 31 library tests, five normal-grid tests
and two wide-grid tests. `just fmt-check` and full `just lint` passed on the
final named inputs. The earlier lint failure and corrected tests are preserved.
The f-only saturated-priority counterfactual is a source/model prediction, not
an executed mutant. Named inputs were checked before and after the runs, not
monitored atomically. Public grid indexing, mutability, parent coherence,
window selection and downstream extreme-value consumers remain open.

The original checkpoint 3821 queue and ledger remain missing. Selected q120
rows are reconstructed with null recorded revision, Git path and named reader;
Adler, f2c and 95e records retain their separate original attributions.
SOURCE-004 proposal bytes were separately recovered, while native qualification
and receipt obligations remain open. Seventy-three historical
nontext/no-positive-EOF obligations remain open. No earlier record or new work
is promoted into checkpoint 3821 or assigned a historical unique count.

## Recovery and custody

The encrypted archive contains the registered later collections and the
separately pinned peer and protocol controls. The earlier immutable archives,
separately held keys and product Git prerequisites remain necessary. The
product bundle is prerequisite-dependent, not full Git closure. Same-machine
copies are not independent backup. The current capture, keyed receipt, key
locator, publication controls and later observations occur after the archive
cutoff and require later restricted custody.

The verifier can check ciphertext identity without a key. For an authenticated
restore, use the separately held key and a fresh disjoint destination on a POSIX
system with Python and `cryptography`:

```sh
(umask 077; set -C; python3 -I -B VERIFY.py --key KEY \
  --restore DESTINATION > "$RECEIPT")
```

Keep the keyed receipt and restored private index restricted; neither belongs
in public output. Fresh restoration and comparisons cover registered paths,
bytes and modes. Finite selected-key and path scans are not general secret
detection and do not prove owner, ACL, hardlink, crash-durability or
independent-custody properties.
