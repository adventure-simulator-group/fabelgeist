# Privacy and replay limits

The exact original packet and five anchors remain separately held. This public
successor preserves whole-file identities and original manifest, but not the
removed bytes; exact replay requires the original. Matched local filesystem and
audit-root paths are redacted, as is one incidental code-token pattern match.
The reconstructed evidence copy of the original verifier is not the original
verifier.

Pattern and path-fingerprint scans cover only their stated representations and
bounds; unknown encodings, undiscovered secrets, Git history and external
private anchors remain outside scope. Whole-file hashes do not demonstrate
independent custody.
