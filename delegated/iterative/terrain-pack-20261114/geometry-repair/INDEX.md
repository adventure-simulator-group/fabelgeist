# Terrain chunk geometry repair

This checkpoint extends the [geometry decision and rebased stack](https://github.com/adventure-simulator-group/fabelgeist/blob/04796c757f3aad2981397bb27a5c716329fc7e53/delegated/iterative/terrain-pack-20261114/geometry-assertions/INDEX.md).
The earlier bounded source assessment remains at product commit
`29ecf3ca12e3dfbfacb36b72a56ae130d4fc48fd`; its readers, cutoffs, null fields
and qualifications retain their original attribution. The later repair is
commit `d90045ad0cd5fb7c34722550b56d84a40427fe58`, based on the pinned stack
tip `e47535b6968bf86b39312ab0436e89d885201c46`. The
[connected ledger](LEDGER-v15.json) retains the earlier decision and records
the subsequent implementation, checks, sources and open relationships.

Terrain entries asserted their encoded width and height without checking that
the shape matched their native tile and chunk ordinals. An equal-area wrong
shape could be admitted and interpreted with an incorrect row stride. The
reader now checks each assertion against derived geometry, then uses that
geometry for decoded length and lookup. Schema 8 and its wire fields remain;
some malformed schema-8 inputs accepted by the former reader will now be
rejected. This is a deliberate tightening, with no alternate legacy path.

The scene regression failed on the unchanged stack when an equal-area wrong
shape was accepted; the final five-test feature suite passed. It covers valid
and invalid ordinal controls for all three supported native widths, selected
nonuniform interior and edge cells, selected corrupt payloads and missing
chunks, and one real raster-builder crop. The 31-test terrain suite, three
existing affected behavior tests, formatting and full lint passed. Named
source hashes were checked before and after all runs. HEAD was checked before
the feature run and before and after the terrain, existing-behavior and gate
runs; none were monitored atomically throughout. Initial lint failures and their corrections are retained.

The builder test checks selected cells of one 1,800-wide crop, not arbitrary
source-pixel fidelity. The road-identity fixture contains an opaque payload
and does not establish successful terrain decoding. The corruption tests check
the outer validation role, not precise multi-fault precedence. The release
validator checks schema, identities and hashes but does not currently check
chunk geometry: a self-consistent noncanonical package could pass that check
and then fail the stricter reader. An earlier observation of 483 declarations
does not establish decoded payloads, installation or release provenance. The
release input, conditional source-bundle water gap, object-store race, cache,
geographic arithmetic, Queue ordering candidate and raster/boolean design
question remain separately open.

## Recovery and custody

The original checkpoint 3821 queue and ledger remain missing. Selected q120
records are reconstructed with null recorded revision, Git path and named
reader; the separately attributed Adler and 95e records do not fill them.
SOURCE-004 proposal bytes were separately recovered, while native
qualification and receipt obligations remain open. Seventy-three historical
nontext/no-positive-EOF obligations also remain open. No earlier record or new
assessment is promoted into checkpoint 3821.

This checkpoint's encrypted evidence is intended to include the complete
repair collection, the prior checkpoint's post-cutoff controls and the later
ledger inputs. The earlier immutable archives, product Git objects and
separately held keys remain dependencies. The included product bundle requires
its named prerequisite and is not complete Git closure. Same-machine copies
are not an independent backup. The ledger maps the previous two collection
labels to their immutable v14 archive and preserves all older mappings.

Publication requires a fresh authenticated restore, comparison of source
bytes and modes, pre-capture inventory checks and finite selected-key and path
scans. Those checks do not establish owners, ACLs, timestamps, hardlink
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
`cryptography` for keyed authentication, and trusted nonconcurrent parent
directories are required. This checkpoint's keyed receipt, locator,
publication controls and later work are outside the envelope and require
later custody.
