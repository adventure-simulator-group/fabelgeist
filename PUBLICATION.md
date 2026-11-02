# Bespoke type progress reports

This branch contains progress reports, separate from product source.
[PROGRESS.md](PROGRESS.md) is the latest report. Numbered reports and
checkpoint summaries preserve each published checkpoint from 488 onward.

Each new local checkpoint requires a report export, a commit, a push,
and verification that the remote branch points to that commit. A local
checkpoint awaiting its push is not fully published. Between-checkpoint
report updates may also be pushed. Earlier snapshots remain unchanged.

Checkpoint JSON files here are public summaries, not the full local
audit evidence graph. They include original evidence hashes, completion
limits, and a hash of the published report. Machine paths and private
recovery details are excluded.
