# Late-frontier source trace

The trace records 12 source observations from checkpoints 3818, 3820 and
3821. They cover 755 printed lines across seven pinned source files. The final
checkpoint itself accounts for four observations and 387 printed lines; the
other eight observations provide earlier connected evidence. Overlapping ranges
are counted as observations, not unique lines or audit credit.

Every recorded line was compared with the pinned Git source. The manifest
records the source hash, Git blob, ranges and historical command ordinals.
Original registration bytes and timestamps, dependent authority, contract
closure and runtime behavior remain unverified.

Run `python restore_sources.py REPOSITORY DESTINATION` from this directory to
restore the seven source blobs from a Git repository containing the pinned
revision. The destination must not exist; hashes are checked before completion.

The seven distinct selected-revision source objects are included under
`sources/`. `verify_portable_frontier.py` uses the preserved Git commit and
complete tree objects from the draft change archive to verify each path,
then checks all 755 displayed lines without a separate Git repository. Both
checkers require and isolate file headings for multi-file command outputs.
Historical registration bytes and audit authority remain separate questions.
