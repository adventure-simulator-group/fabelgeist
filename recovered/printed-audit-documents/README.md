# Historical printed audit documents

This collection preserves normalized JSON documents printed by successful
historical commands, excluding public progress-publication records and any
values containing nonessential personal local paths. `INDEX.json` maps values
to transcript output ordinals and command filename context. A filename in a
command is not proof that the printed value came from that file. `VALUES.jsonl`
contains the indexed values keyed by SHA-256 of their normalized JSON form.

These are historical observations; printing may have used field projections.
They are not automatically original file bytes, valid receipts, a complete
ledger, or the final checkpoint. For separately hash-matched original bytes,
see `../transcript-hash-matched/`. The raw transcript remains private. The
batch was screened for common credentials, private paths and email addresses;
individual audit claims and dependencies have not been independently reviewed.

`REDACTED-VALUES.jsonl` additionally preserves eight printed documents with
machine-specific paths removed or converted to relative project paths. Their
original normalized hashes identify the private versions; the published
redacted hashes cover changed content.
