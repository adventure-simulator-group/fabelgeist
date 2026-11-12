# Privacy and custody qualification

This is an exact private packet and is not automatically suitable for public
release. It includes local paths in evidence locators and private incident
references. No local path values should be included in a public successor;
redaction must preserve identities, original attribution and recoverability.
The preliminary bounded scans found no match to the 40-digest restricted
corpus, no email-like or credential-like pattern, and 21 local-home path hits.
Those counts describe only that scan, not a general secrecy guarantee.

Scans have input-size, recursion and format limits and do not clear unknown
secrets, unknown encodings, unsupported formats, symlinks, Git history or
external anchors. The raw commit objects can contain personal metadata, while
the complete queues and the time-bound ledger snapshot may contain private
content; all are separately held and were not cleared by packet scans. A final
scan and evidence-verifier receipt are held externally to bind the manifest
without self-reference. The temporary-anchor incident record is also held
separately and must remain private; no pre-write existence check was made.
