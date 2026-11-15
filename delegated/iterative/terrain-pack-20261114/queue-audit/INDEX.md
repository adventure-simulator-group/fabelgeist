# Routing queue: bounded audit and next repair candidate

This checkpoint extends the [terrain geometry repair evidence](https://github.com/adventure-simulator-group/fabelgeist/blob/6378812e93224a0e392fec5a02b03964c099dcfb/delegated/iterative/terrain-pack-20261114/geometry-repair/INDEX.md).
The new source assessment is pinned to product commit
`d90045ad0cd5fb7c34722550b56d84a40427fe58`, the head observed after opening
[PR 1010](https://github.com/adventure-simulator-group/fabelgeist/pull/1010).
The [connected ledger](LEDGER-v16.json) records each reader, source and
revision, thirteen bounded lens assessments, open candidates and their
relationships. No lens is claimed clear beyond its named scope.

The private routing queue orders estimated cost, accumulated cost, then
row-major index. Its manual comparison is source-equivalent to standard
derives for the current fields; that cleanup is the same candidate already
recorded at the previous checkpoint. It is separate from the grid and cost
contracts that feed the queue.

The heuristic can exceed the cost of a feasible path because it rounds
diagonal distance differently from individual steps. A retained source-derived
model and independent model review give a 103-by-107 grid where the modeled
first-goal path costs 4,141 seconds and another feasible path costs 4,140.
The source's minute conversion predicts 70 versus 69 minutes. This is a model
result, not an observed product failure or a claim about a shipped route. The
original model's point-retention prediction was corrected after review of
floating-point `signum`; both reports and the reviewer's corrected check are
preserved. The proposed next step is to reproduce or refute the case at the
real routing boundary, then assess a lower-bound repair against the exact
per-step rounding contract.

Other separate source-derived questions remain open: public-grid row-major
index and neighbor arithmetic, cost and heuristic intermediate overflow,
endpoint and grid-stability policy, cost/index ownership, typed planner errors,
and route-point frames and turn retention. An existing latitude-degree scale
is a reuse candidate; the nearby longitude clamp is a distinct policy. A
conditional water-crossing coarsening question requires at least one passable
sample; zero passable samples take an earlier Water fallback. These observations
do not establish actual incidents or historical unique finding counts.

## Recovery and custody

The original checkpoint 3821 queue and ledger remain missing. Selected q120
records are reconstructed with null recorded revision, Git path and named
reader; the separately attributed Adler and 95e records do not fill them.
SOURCE-004 proposal bytes were separately recovered, while native
qualification and receipt obligations remain open. Seventy-three historical
nontext/no-positive-EOF obligations also remain open. The historical queue is
distinct from the Rust routing queue; no earlier record or new assessment is
promoted into checkpoint 3821.

This checkpoint's encrypted evidence is intended to include the complete
selected Queue collections, independent reviews and corrections, and the prior
checkpoint's post-cutoff controls. Earlier immutable archives, product Git
objects and separately held keys remain dependencies. The prior product bundle
requires its named prerequisite and is not complete Git closure. Same-machine
copies are not an independent backup. The ledger maps the previous checkpoint's
three labels to its immutable ciphertext and preserves older mappings.

Publication requires a fresh authenticated restore, source-byte and mode
comparison, pre-capture inventory checks and finite selected-key and path
scans. These checks do not establish owners, ACLs, timestamps, hardlink
topology, crash durability or independent custody. `python3 -I -B VERIFY.py`
in `custody/` checks ciphertext identity without a key. To authenticate and
restore with the separately held key and a fresh disjoint destination, set
`RECEIPT` to a fresh path outside the public directory in a restricted parent,
then run:

```sh
(umask 077; set -C; python3 -I -B VERIFY.py --key KEY \
  --restore DESTINATION > "$RECEIPT")
```

The keyed receipt contains a private-index digest; keep it restricted and do
not display it. The restored private index is also restricted. POSIX, Python,
`cryptography` for keyed authentication and trusted nonconcurrent parent
directories are required. This checkpoint's keyed receipt, locator, later
publication controls and subsequent work are outside the envelope and require
later custody.
