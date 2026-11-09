# Historical queue and lens fragments

These are complete JSON values printed by successful historical commands,
parsed and reserialized with normalized formatting. They are not original file
bytes, the complete queue, the latest checkpoint, or evidence of current
closure. The queue entry predates checkpoint 3821, and later changes must be
checked before relying on its status. The referenced evidence and state must
be independently restored and verified. Manifest ordinals identify the
recorded command outputs in the private transcript snapshot.

`CANDIDATE-INDEX.json` indexes 2,968 complete printed contract objects
covering 1,564 distinct contract IDs. The named queues are command context,
not proof of each object's origin. The index contains hashes of normalized values; corresponding values are in
`PRINTED-VALUES.jsonl`. The values were screened for common credential and
personal-data patterns, but audit claims and dependencies require individual
verification before use.

`EXTENDED-INDEX.json` expands the search to all command outputs and printed
array members: 3,422 observations, 3,146 distinct normalized values and
1,722 distinct IDs. These counts overlap, include field projections, and
do not represent complete or current queue entries. Verify the source
command, output and dependencies individually.

`PRINTED-VALUES.jsonl` preserves 3,146 normalized printed values keyed by
SHA-256. They are historical output or selected-field projections, not a
complete queue or original bytes. `EXTENDED-INDEX.json` connects each hash to
its transcript ordinal. The raw session remains private.
